use std::env;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use borsh::{BorshDeserialize, BorshSerialize};
use rollup_config::{ROLLUP_NAMESPACE_RAW, SEQUENCER_DA_ADDRESS};
use demo_stf::app::{App, DefaultContext, DefaultPrivateKey};
use demo_stf::genesis_config::create_demo_genesis_config;
use demo_stf::runtime::{get_rpc_methods, GenesisConfig};
use jupiter::da_service::CelestiaService;
use jupiter::types::NamespaceId;
use jupiter::verifier::address::CelestiaAddress;
use jupiter::verifier::{ChainValidityCondition, RollupParams};
use risc0_adapter::host::Risc0Verifier;
use sov_db::ledger_db::LedgerDB;
use demo_stf::{SequencerOutcome, TxEffect};
use sov_rollup_interface::da::DaSpec;
use sov_rollup_interface::services::da::DaService;
use sov_rollup_interface::zk::ValidityConditionChecker;
use sov_sequencer::get_sequencer_rpc;

use sov_stf_runner::{from_toml_path, get_ledger_rpc, RollupConfig, StateTransitionRunner};
use tracing::{debug, Level};



// The rollup stores its data in the namespace b"sov-test" on Celestia
// You can change this constant to point your rollup at a different namespace
const ROLLUP_NAMESPACE: NamespaceId = NamespaceId(ROLLUP_NAMESPACE_RAW);

/// Initializes a [`LedgerDB`] using the provided `path`.
pub fn initialize_ledger(path: impl AsRef<std::path::Path>) -> LedgerDB {
    LedgerDB::with_path(path).expect("Ledger DB failed to open")
}

// TODO: Remove this when sov-cli is in its own crate.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct HexKey {
    hex_priv_key: String,
    address: String,
}

/// Configure our rollup with a centralized sequencer using the SEQUENCER_DA_ADDRESS
/// address constant. Since the centralize sequencer's address is consensus critical,
/// it has to be hardcoded as a constant, rather than read from the config at runtime.
///
/// If you want to customize the rollup to accept transactions from your own celestia
/// address, simply change the value of the SEQUENCER_DA_ADDRESS to your own address.
/// For example:
/// ```rust,no_run
/// const SEQUENCER_DA_ADDRESS: [u8;47] = *b"celestia1qp09ysygcx6npted5yc0au6k9lner05yvs9208"
/// ```
pub fn get_genesis_config() -> GenesisConfig<DefaultContext> {
    let hex_key: HexKey = serde_json::from_slice(include_bytes!(
        "../../test-data/keys/token_deployer_private_key.json"
    ))
    .expect("Broken key data file");
    let sequencer_private_key = DefaultPrivateKey::from_hex(&hex_key.hex_priv_key).unwrap();
    assert_eq!(
        sequencer_private_key.default_address().to_string(),
        hex_key.address,
        "Inconsistent key data",
    );
    let sequencer_da_address = CelestiaAddress::from_str(SEQUENCER_DA_ADDRESS).unwrap();
    create_demo_genesis_config(
        100000000,
        sequencer_private_key.default_address(),
        sequencer_da_address.as_ref().to_vec(),
        &sequencer_private_key,
        &sequencer_private_key,
    )
}

/// Main demo runner. Initialize a DA chain, and starts a demo-rollup using the config provided
/// (or a default config if not provided). Then start checking the blocks sent to the DA layer in
/// the main event loop.
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct CelestiaChainChecker {
    current_block_hash: [u8; 32],
}

impl ValidityConditionChecker<ChainValidityCondition> for CelestiaChainChecker {
    type Error = anyhow::Error;

    fn check(&mut self, condition: &ChainValidityCondition) -> Result<(), anyhow::Error> {
        anyhow::ensure!(
            condition.block_hash == self.current_block_hash,
            "Invalid block hash"
        );
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let rollup_config_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "rollup_config.toml".to_string());

    debug!("Starting demo rollup with config {}", rollup_config_path);
    let rollup_config: RollupConfig =
        from_toml_path(&rollup_config_path).context("Failed to read rollup configuration")?;

    // Initializing logging
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_err| eprintln!("Unable to set global default subscriber"))
        .expect("Cannot fail to set subscriber");

    let ledger_db = initialize_ledger(&rollup_config.runner.storage.path);

    let da_service = CelestiaService::new(
        rollup_config.da.clone(),
        RollupParams {
            namespace: ROLLUP_NAMESPACE,
        },
    )
    .await;

    let mut app: App<Risc0Verifier, ChainValidityCondition, jupiter::BlobWithSender> =
        App::new(rollup_config.runner.storage.clone());

    let storage = app.get_storage();
    let mut methods = get_rpc_methods::<DefaultContext>(storage);

    // register rpc methods
    {
        register_ledger(ledger_db.clone(), &mut methods)?;
        register_sequencer(da_service.clone(), &mut app, &mut methods)?;
    }

    // let storage = app.get_storage();
    // storage.isEmty()
    let genesis_config = get_genesis_config();

    let mut runner = StateTransitionRunner::new(
        rollup_config,
        da_service,
        ledger_db,
        app.stf,
        true,
        genesis_config,
    )?;

    runner.start_rpc_server(methods).await;
    runner.run().await?;

    Ok(())
}

fn register_sequencer<DA>(
    da_service: DA,
    demo_runner: &mut App<
        Risc0Verifier,
        <DA::Spec as DaSpec>::ValidityCondition,
        jupiter::BlobWithSender,
    >,
    methods: &mut jsonrpsee::RpcModule<()>,
) -> Result<(), anyhow::Error>
where
    DA: DaService<Error = anyhow::Error> + Send + Sync + 'static,
{   
    
    let batch_builder = demo_runner.batch_builder.take().unwrap();
    let sequencer_rpc = get_sequencer_rpc(batch_builder, Arc::new(da_service));
    methods
        .merge(sequencer_rpc)
        .context("Failed to merge Txs RPC modules")
}

fn register_ledger(
    ledger_db: LedgerDB,
    methods: &mut jsonrpsee::RpcModule<()>,
) -> Result<(), anyhow::Error> {
    let ledger_rpc = get_ledger_rpc::<SequencerOutcome<CelestiaAddress>, TxEffect>(ledger_db);
    methods
        .merge(ledger_rpc)
        .context("Failed to merge ledger RPC modules")
}


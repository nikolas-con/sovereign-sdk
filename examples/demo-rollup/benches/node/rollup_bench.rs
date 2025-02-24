use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use criterion::{criterion_group, criterion_main, Criterion};
use demo_stf::genesis_config::{get_genesis_config, GenesisPaths};
use demo_stf::runtime::Runtime;
use sov_db::ledger_db::{LedgerDB, SlotCommit};
use sov_mock_da::{MockBlock, MockBlockHeader};
use sov_modules_api::default_context::DefaultContext;
use sov_modules_stf_blueprint::kernels::basic::BasicKernel;
use sov_modules_stf_blueprint::StfBlueprint;
use sov_risc0_adapter::host::Risc0Verifier;
use sov_rng_da_service::{RngDaService, RngDaSpec};
use sov_rollup_interface::services::da::DaService;
use sov_rollup_interface::stf::StateTransitionFunction;
use sov_rollup_interface::storage::StorageManager;
use sov_stf_runner::{from_toml_path, RollupConfig};
use tempfile::TempDir;

fn rollup_bench(_bench: &mut Criterion) {
    let start_height: u64 = 0u64;
    let mut end_height: u64 = 100u64;
    if let Ok(val) = env::var("BLOCKS") {
        end_height = val.parse().expect("BLOCKS var should be a +ve number");
    }

    let mut c = Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(20));
    let rollup_config_path = "benches/node/rollup_config.toml".to_string();
    let mut rollup_config: RollupConfig<sov_celestia_adapter::CelestiaConfig> =
        from_toml_path(&rollup_config_path)
            .context("Failed to read rollup configuration")
            .unwrap();

    let temp_dir = TempDir::new().expect("Unable to create temporary directory");
    rollup_config.storage.path = PathBuf::from(temp_dir.path());
    let ledger_db =
        LedgerDB::with_path(&rollup_config.storage.path).expect("Ledger DB failed to open");

    let da_service = Arc::new(RngDaService::new());

    let storage_config = sov_state::config::Config {
        path: rollup_config.storage.path,
    };
    let storage_manager = sov_state::storage_manager::ProverStorageManager::new(storage_config)
        .expect("Failed to initialize prover storage manager");
    let stf = StfBlueprint::<
        DefaultContext,
        RngDaSpec,
        Risc0Verifier,
        Runtime<DefaultContext, RngDaSpec>,
        BasicKernel<DefaultContext>,
    >::new();

    let demo_genesis_config = get_genesis_config(&GenesisPaths::from_dir(
        "../test-data/genesis/integration-tests",
    ))
    .unwrap();

    let (mut current_root, _) =
        stf.init_chain(storage_manager.get_native_storage(), demo_genesis_config);

    // data generation
    let mut blobs = vec![];
    let mut blocks = vec![];
    for height in start_height..end_height {
        let num_bytes = height.to_le_bytes();
        let mut barray = [0u8; 32];
        barray[..num_bytes.len()].copy_from_slice(&num_bytes);
        let filtered_block = MockBlock {
            header: MockBlockHeader {
                hash: barray.into(),
                prev_hash: [0u8; 32].into(),
                height,
            },
            validity_cond: Default::default(),
            blobs: Default::default(),
        };
        blocks.push(filtered_block.clone());

        let blob_txs = da_service.extract_relevant_blobs(&filtered_block);
        blobs.push(blob_txs.clone());
    }

    let mut height = 0u64;
    c.bench_function("rollup main loop", |b| {
        b.iter(|| {
            let filtered_block = &blocks[height as usize];

            let mut data_to_commit = SlotCommit::new(filtered_block.clone());
            let apply_block_result = stf.apply_slot(
                &current_root,
                storage_manager.get_native_storage(),
                Default::default(),
                &filtered_block.header,
                &filtered_block.validity_cond,
                &mut blobs[height as usize],
            );
            current_root = apply_block_result.state_root;
            for receipts in apply_block_result.batch_receipts {
                data_to_commit.add_batch(receipts);
            }

            ledger_db.commit_slot(data_to_commit).unwrap();
            height += 1;
        })
    });
}

criterion_group!(benches, rollup_bench);
criterion_main!(benches);

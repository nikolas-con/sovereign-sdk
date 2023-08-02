mod config;
pub mod runner_config;
use std::net::SocketAddr;
pub mod ledger_rpc;

pub use config::RollupConfig;
use jsonrpsee::RpcModule;
use sov_db::ledger_db::{LedgerDB, SlotCommit};
use sov_rollup_interface::da::DaSpec;
use sov_rollup_interface::services::da::DaService;
use sov_rollup_interface::stf::StateTransitionFunction;
use sov_rollup_interface::zk::Zkvm;
use tracing::{debug, info};

pub struct RollupRunner<ST, DA, Vm: Zkvm>
where
    DA: DaService,
    ST: StateTransitionFunction<Vm, <<DA as DaService>::Spec as DaSpec>::BlobTransaction>,
{
    start_height: u64,
    da_service: DA,
    app: ST,
    ledger_db: LedgerDB,
    state_root: <ST as StateTransitionFunction<
        Vm,
        <<DA as DaService>::Spec as DaSpec>::BlobTransaction,
    >>::StateRoot,
    socket_address: SocketAddr,
}

impl<ST, DA, Vm: Zkvm> RollupRunner<ST, DA, Vm>
where
    DA: DaService<Error = anyhow::Error> + Clone + Send + Sync + 'static,
    ST: StateTransitionFunction<Vm, <<DA as DaService>::Spec as DaSpec>::BlobTransaction>,
{
    pub fn new(
        rollup_config: RollupConfig,
        da_service: DA,
        ledger_db: LedgerDB,
        mut app: ST,
        is_storage_empty: bool,
        genesis_config: <ST as StateTransitionFunction<
            Vm,
            <<DA as DaService>::Spec as DaSpec>::BlobTransaction,
        >>::InitialState,
    ) -> Result<Self, anyhow::Error> {
        let rpc_config = rollup_config.rpc_config;

        let prev_state_root = {
            // Check if the rollup has previously been initialized
            if is_storage_empty {
                info!("No history detected. Initializing chain...");
                app.init_chain(genesis_config);
                info!("Chain initialization is done.");
            } else {
                debug!("Chain is already initialized. Skipping initialization.");
            }

            let res = app.apply_slot(Default::default(), []);
            // HACK: Tell the rollup that you're running an empty DA layer block so that it will return the latest state root.
            // This will be removed shortly.
            res.state_root
        };

        let socket_address = SocketAddr::new(rpc_config.bind_host.parse()?, rpc_config.bind_port);

        // Start the main rollup loop
        let item_numbers = ledger_db.get_next_items_numbers();
        let last_slot_processed_before_shutdown = item_numbers.slot_number - 1;
        let start_height = rollup_config.start_height + last_slot_processed_before_shutdown;

        Ok(Self {
            start_height,
            da_service,
            app,
            ledger_db,
            state_root: prev_state_root,
            socket_address,
        })
    }

    pub async fn start_rpc_server(&self, methods: RpcModule<()>) {
        let socket_address = self.socket_address;
        let _handle = tokio::spawn(async move {
            let server = jsonrpsee::server::ServerBuilder::default()
                .build([socket_address].as_ref())
                .await
                .unwrap();

            info!("Starting RPC server at {} ", server.local_addr().unwrap());
            let _server_handle = server.start(methods).unwrap();
            futures::future::pending::<()>().await;
        });
    }

    pub async fn run(&mut self) -> Result<(), anyhow::Error> {
        for height in self.start_height.. {
            info!("Requesting data for height {}", height,);

            // Fetch the relevant subset of the next Celestia block
            let filtered_block = self.da_service.get_finalized_at(height).await?;

            let mut blobs = self.da_service.extract_relevant_txs(&filtered_block);

            info!(
                "Extracted {} relevant blobs at height {}",
                blobs.len(),
                height
            );

            let mut data_to_commit = SlotCommit::new(filtered_block.clone());

            let slot_result = self.app.apply_slot(Default::default(), &mut blobs);
            for receipt in slot_result.batch_receipts {
                data_to_commit.add_batch(receipt);
            }
            let next_state_root = slot_result.state_root;

            self.ledger_db.commit_slot(data_to_commit)?;
            self.state_root = next_state_root;
        }

        Ok(())
    }
}

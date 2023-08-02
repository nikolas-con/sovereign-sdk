#[cfg(feature = "native")]
pub use sov_modules_api::default_context::DefaultContext;
pub use sov_modules_api::default_context::ZkDefaultContext;
#[cfg(feature = "native")]
pub use sov_modules_api::default_signature::private_key::DefaultPrivateKey;
use sov_modules_api::hooks::{ApplyBlobHooks, TxHooks};
#[cfg(feature = "native")]
use sov_modules_api::Spec;
use sov_modules_api::{Context, DispatchCall, Genesis};
pub use sov_modules_stf_template::Batch;
use sov_modules_stf_template::{AppTemplate, SequencerOutcome};
use sov_rollup_interface::da::BlobReaderTrait;
#[cfg(feature = "native")]
use sov_rollup_interface::zk::Zkvm;
#[cfg(feature = "native")]
use sov_state::ProverStorage;
use sov_state::Storage;
#[cfg(feature = "native")]
use sov_stf_runner::runner_config::StorageConfig;

use crate::batch_builder::FiFoStrictBatchBuilder;

#[cfg(feature = "native")]
pub struct App<RT, Vm: Zkvm, B: BlobReaderTrait> {
    pub stf: AppTemplate<DefaultContext, RT, Vm, B>,
    pub batch_builder: Option<FiFoStrictBatchBuilder<RT, DefaultContext>>,
}

#[cfg(feature = "native")]
impl<RT, Vm: Zkvm, B: BlobReaderTrait> App<RT, Vm, B>
where
    RT: DispatchCall<Context = DefaultContext>
        + TxHooks<Context = DefaultContext>
        + Genesis<Context = DefaultContext>
        + ApplyBlobHooks<Context = DefaultContext, BlobResult = SequencerOutcome>
        + Default,
{
    pub fn new(storage_config: StorageConfig) -> Self {
        let storage =
            ProverStorage::with_config(storage_config).expect("Failed to open prover storage");
        let app = AppTemplate::new(storage.clone(), RT::default());
        let batch_size_bytes = 1024 * 100; // 100 KB
        let batch_builder = FiFoStrictBatchBuilder::new(
            batch_size_bytes,
            u32::MAX as usize,
            RT::default(),
            storage,
        );
        Self {
            stf: app,
            batch_builder: Some(batch_builder),
        }
    }

    pub fn get_storage(&self) -> <DefaultContext as Spec>::Storage {
        self.stf.current_storage.clone()
    }
}

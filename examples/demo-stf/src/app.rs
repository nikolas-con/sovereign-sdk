#[cfg(feature = "native")]
pub use sov_modules_api::default_context::DefaultContext;
pub use sov_modules_api::default_context::ZkDefaultContext;
#[cfg(feature = "native")]
pub use sov_modules_api::default_signature::private_key::DefaultPrivateKey;
use sov_modules_api::hooks::{ApplyBlobHooks, TxHooks};
#[cfg(feature = "native")]
use sov_modules_api::RpcRunner;
#[cfg(feature = "native")]
use sov_modules_api::Spec;
use sov_modules_api::{Context, DispatchCall, Genesis};
pub use sov_modules_stf_template::Batch;
use sov_modules_stf_template::{AppTemplate, SequencerOutcome, TxEffect};
use sov_rollup_interface::da::BlobReaderTrait;
use sov_rollup_interface::services::stf_runner::StateTransitionRunner;
#[cfg(feature = "native")]
use sov_rollup_interface::stf::ProverConfig;
use sov_rollup_interface::stf::ZkConfig;
use sov_rollup_interface::zk::Zkvm;
#[cfg(feature = "native")]
use sov_state::ProverStorage;
use sov_state::{Storage, ZkStorage};

use crate::batch_builder::FiFoStrictBatchBuilder;
#[cfg(feature = "native")]
use crate::runner_config::StorageConfig;

pub struct DemoAppRunner<RT, C: Context, Vm: Zkvm, B: BlobReaderTrait> {
    pub stf: DemoApp<RT, C, Vm, B>,
    pub batch_builder: Option<FiFoStrictBatchBuilder<RT, C>>,
}

pub type ZkAppRunner<RT, Vm, B> = DemoAppRunner<RT, ZkDefaultContext, Vm, B>;

#[cfg(feature = "native")]
pub type NativeAppRunner<RT, Vm, B> = DemoAppRunner<RT, DefaultContext, Vm, B>;

pub type DemoApp<RT, C, Vm, B> = AppTemplate<C, RT, Vm, B>;

/// Batch receipt type used by the demo app. We export this type so that it's easily accessible to the full node.
pub type DemoBatchReceipt = SequencerOutcome;
/// Tx receipt type used by the demo app. We export this type so that it's easily accessible to the full node.
pub type DemoTxReceipt = TxEffect;

#[cfg(feature = "native")]
impl<RT, Vm: Zkvm, B: BlobReaderTrait> DemoAppRunner<RT, DefaultContext, Vm, B>
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
}

#[cfg(feature = "native")]
impl<RT, Vm: Zkvm, B: BlobReaderTrait> StateTransitionRunner<ProverConfig, Vm, B>
    for DemoAppRunner<RT, DefaultContext, Vm, B>
where
    RT: DispatchCall<Context = DefaultContext>
        + TxHooks<Context = DefaultContext>
        + Genesis<Context = DefaultContext>
        + ApplyBlobHooks<Context = DefaultContext, BlobResult = SequencerOutcome>,
{
    type Inner = DemoApp<RT, DefaultContext, Vm, B>;
    type BatchBuilder = FiFoStrictBatchBuilder<RT, DefaultContext>;

    fn inner(&self) -> &Self::Inner {
        &self.stf
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.stf
    }

    fn take_batch_builder(&mut self) -> Option<Self::BatchBuilder> {
        self.batch_builder.take()
    }
}

#[cfg(feature = "native")]
impl<RT, Vm: Zkvm, B: BlobReaderTrait> RpcRunner for DemoAppRunner<RT, DefaultContext, Vm, B>
where
    RT: DispatchCall<Context = DefaultContext>
        + TxHooks<Context = DefaultContext>
        + Genesis<Context = DefaultContext>
        + ApplyBlobHooks<Context = DefaultContext, BlobResult = SequencerOutcome>,
{
    type Context = DefaultContext;
    fn get_storage(&self) -> <Self::Context as Spec>::Storage {
        self.inner().current_storage.clone()
    }
}

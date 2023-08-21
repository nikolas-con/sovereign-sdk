use sov_accounts::{AccountsRpcImpl, AccountsRpcServer};
use sov_bank::{BankRpcImpl, BankRpcServer};
use demo_module::{DemoModuleRpcImpl, DemoModuleRpcServer};
use sov_blob_storage::{BlobStorageRpcImpl, BlobStorageRpcServer};
use sov_election::{ElectionRpcImpl, ElectionRpcServer};
use sov_modules_api::capabilities::{BlobRefOrOwned, BlobSelector};
pub use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::hooks::SlotHooks;
use sov_modules_api::macros::DefaultRuntime;
use sov_modules_api::macros::{expose_rpc, CliWallet};
use sov_modules_api::{Context, DispatchCall, Genesis, MessageCodec, Spec};
use sov_modules_api::hooks::{ApplyBlobHooks, TxHooks};
use sov_modules_api::transaction::Transaction;
use sov_rollup_interface::da::BlobReaderTrait;
use sov_rollup_interface::zk::ValidityCondition;
use sov_sequencer_registry::{SequencerRegistryRpcImpl, SequencerRegistryRpcServer};
use sov_sequencer_registry::SequencerRegistry;
use sov_state::WorkingSet;
use sov_value_setter::{ValueSetterRpcImpl, ValueSetterRpcServer};
use sov_modules_stf_template::SequencerOutcome;
use tracing::info;

/// The Rollup entrypoint.
///
/// On a high level, the rollup node receives serialized call messages from the DA layer and executes them as atomic transactions.
/// Upon reception, the message has to be deserialized and forwarded to an appropriate module.
///
/// The module-specific logic is implemented by module creators, but all the glue code responsible for message
/// deserialization/forwarding is handled by a rollup `runtime`.
///
/// In order to define the runtime we need to specify all the modules supported by our rollup (see the `Runtime` struct bellow)
///
/// The `Runtime` together with associated interfaces (`Genesis`, `DispatchCall`, `MessageCodec`)
/// and derive macros defines:
/// - how the rollup modules are wired up together.
/// - how the state of the rollup is initialized.
/// - how messages are dispatched to appropriate modules.
///
/// Runtime lifecycle:
///
/// 1. Initialization:
///     When a rollup is deployed for the first time, it needs to set its genesis state.
///     The `#[derive(Genesis)` macro will generate `Runtime::genesis(config)` method which returns
///     `Storage` with the initialized state.
///
/// 2. Calls:      
///     The `Module` interface defines a `call` method which accepts a module-defined type and triggers the specific `module logic.`
///     In general, the point of a call is to change the module state, but if the call throws an error,
///     no state is updated (the transaction is reverted).
///
/// `#[derive(MessageCodec)` adds deserialization capabilities to the `Runtime` (implements `decode_call` method).
/// `Runtime::decode_call` accepts serialized call message and returns a type that implements the `DispatchCall` trait.
///  The `DispatchCall` implementation (derived by a macro) forwards the message to the appropriate module and executes its `call` method.
///
/// Similar mechanism works for queries with the difference that queries are submitted by users directly to the rollup node
/// instead of going through the DA layer.

#[derive(CliWallet)]
#[expose_rpc(DefaultContext)]
#[derive(Genesis, DispatchCall, MessageCodec, DefaultRuntime)]
#[serialization(borsh::BorshDeserialize, borsh::BorshSerialize)]
#[serialization(serde::Serialize, serde::Deserialize)]
pub struct Runtime<C: Context> {
    pub bank: sov_bank::Bank<C>,
    pub demo_module: demo_module::DemoModule<C>,
    pub sequencer_registry: sov_sequencer_registry::SequencerRegistry<C>,
    #[cli_skip]
    pub blob_storage: sov_blob_storage::BlobStorage<C>,
    pub election: sov_election::Election<C>,
    pub value_setter: sov_value_setter::ValueSetter<C>,
    pub accounts: sov_accounts::Accounts<C>,
}

impl<C: Context, Cond: ValidityCondition> SlotHooks<Cond> for Runtime<C> {
    type Context = C;

    fn begin_slot_hook(
        &self,
        _slot_data: &impl sov_rollup_interface::services::da::SlotData,
        _working_set: &mut sov_state::WorkingSet<<Self::Context as sov_modules_api::Spec>::Storage>,
    ) {
    }

    fn end_slot_hook(
        &self,
        _working_set: &mut sov_state::WorkingSet<<Self::Context as sov_modules_api::Spec>::Storage>,
    ) {
    }
}

impl<C, Cond, B> sov_modules_stf_template::Runtime<C, Cond, B> for Runtime<C>
where
    C: Context,
    Cond: ValidityCondition,
    B: BlobReaderTrait,
{
}

impl<C: Context> BlobSelector for Runtime<C> {
    type Context = C;

    fn get_blobs_for_this_slot<'a, I, B>(
        &self,
        current_blobs: I,
        working_set: &mut WorkingSet<<Self::Context as Spec>::Storage>,
    ) -> anyhow::Result<Vec<BlobRefOrOwned<'a, B>>>
    where
        B: BlobReaderTrait,
        I: IntoIterator<Item = &'a mut B>,
    {
        self.blob_storage
            .get_blobs_for_this_slot(current_blobs, working_set)
    }
}

impl<C: Context> TxHooks for Runtime<C> {
    type Context = C;

    fn pre_dispatch_tx_hook(
        &self,
        tx: &Transaction<Self::Context>,
        working_set: &mut WorkingSet<<Self::Context as Spec>::Storage>,
    ) -> anyhow::Result<<Self::Context as Spec>::Address> {
        self.accounts.pre_dispatch_tx_hook(tx, working_set)
    }

    fn post_dispatch_tx_hook(
        &self,
        tx: &Transaction<Self::Context>,
        working_set: &mut WorkingSet<<Self::Context as Spec>::Storage>,
    ) -> anyhow::Result<()> {
        self.accounts.post_dispatch_tx_hook(tx, working_set)
    }
}

impl<C: Context, B: BlobReaderTrait> ApplyBlobHooks<B> for Runtime<C> {
    type Context = C;
    type BlobResult = SequencerOutcome<B::Address>;

    fn begin_blob_hook(
        &self,
        blob: &mut B,
        working_set: &mut WorkingSet<<Self::Context as Spec>::Storage>,
    ) -> anyhow::Result<()> {
        self.sequencer_registry.begin_blob_hook(blob, working_set)
    }

    fn end_blob_hook(
        &self,
        result: Self::BlobResult,
        working_set: &mut WorkingSet<<Self::Context as Spec>::Storage>,
    ) -> anyhow::Result<()> {
        match result {
            SequencerOutcome::Rewarded(_reward) => {
                // TODO: Process reward here or above.
                <SequencerRegistry<C> as ApplyBlobHooks<B>>::end_blob_hook(
                    &self.sequencer_registry,
                    sov_sequencer_registry::SequencerOutcome::Completed,
                    working_set,
                )
            }
            SequencerOutcome::Ignored => Ok(()),
            SequencerOutcome::Slashed {
                reason,
                sequencer_da_address,
            } => {
                info!("Sequencer {} slashed: {:?}", sequencer_da_address, reason);
                <SequencerRegistry<C> as ApplyBlobHooks<B>>::end_blob_hook(
                    &self.sequencer_registry,
                    sov_sequencer_registry::SequencerOutcome::Slashed {
                        sequencer: sequencer_da_address.as_ref().to_vec(),
                    },
                    working_set,
                )
            }
        }
    }
}

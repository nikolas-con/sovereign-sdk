mod call;
mod query;

/// Specifies the call methods using in that module.
pub use call::CallMessage;

/// Specifies the different queries used in that module.
pub use query::{DemoModuleRpcImpl, DemoModuleRpcServer};
use sov_modules_api::{Error, ModuleInfo};
use sov_state::WorkingSet;

/// Initial configuration for sov-bank module.
pub struct ModuleConfig {}

/// The sov-bank module manages user balances. It provides functionality for:
/// - Token creation.
#[derive(sov_modules_api::ModuleCallJsonSchema)]
#[derive(ModuleInfo, Clone)]
pub struct DemoModule<C: sov_modules_api::Context> {
    /// The address of the sov-bank module.
    #[address]
    pub(crate) address: C::Address,

    #[state]
    pub(crate) name: sov_state::StateValue<String>,
}

impl<C: sov_modules_api::Context> sov_modules_api::Module for DemoModule<C> {
    type Context = C;

    type Config = ModuleConfig;

    type CallMessage = call::CallMessage<C>;

    fn genesis(
        &self,
        _config: &Self::Config,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<(), Error> {

        self.name.set(&"test_str".to_owned(), working_set);
        Ok(())
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Self::Context,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<sov_modules_api::CallResponse, Error> {
        match msg {
            call::CallMessage::UpdateAddress { address: _ } => { unimplemented!() },
            call::CallMessage::UpdateName { name } => Ok(self.update_name(name, context, working_set)?)
        }
    }
}

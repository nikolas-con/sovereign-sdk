use anyhow::Result;
use sov_state::WorkingSet;

use crate::BankA;

impl<C: sov_modules_api::Context> BankA<C> {
    /// Init an instance of the bank module from the configuration `config`.
    /// For each token in the `config`, calls the [`Token::create`] function to create
    /// the token. Upon success, updates the token set if the token address doesn't already exist.
    pub(crate) fn init_module(
        &self,
        _config: &<Self as sov_modules_api::Module>::Config,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<()> {
        self.name.set(&"test_str".to_owned(), working_set);

        Ok(())
    }
}

use anyhow::Result;

use sov_modules_api::macros::CliWalletArg;
use sov_modules_api::CallResponse;
use sov_state::WorkingSet;

use crate::BankA;

/// This enumeration represents the available call messages for interacting with the sov-bank module.
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(CliWalletArg),
    derive(schemars::JsonSchema),
    schemars(bound = "C::Address: ::schemars::JsonSchema", rename = "CallMessage")
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub enum CallMessage<C: sov_modules_api::Context> {
    UpdateAddress {
        address: C::Address,
    },
    UpdateName {
        name: String
    }
}

impl<C: sov_modules_api::Context> BankA<C> {

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_name(
        &self,
        name: String,
        _context: &C,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<CallResponse> {
        self.name.set(&name, working_set);
        Ok(CallResponse::default())
    }

}

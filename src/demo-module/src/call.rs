use anyhow::{bail, Result};
#[cfg(feature = "native")]
use sov_modules_api::macros::CliWalletArg;
use sov_modules_api::CallResponse;
use sov_state::WorkingSet;

use crate::{Amount, BankA, Token};

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
    /// Creates a new token with the specified name and initial balance.
    CreateToken {
        /// Random value use to create a unique token address.
        salt: u64,
        /// The name of the new token.
        token_name: String,
        /// The initial balance of the new token.
        initial_balance: Amount,
        /// The address of the account that the new tokens are minted to.
        minter_address: C::Address,
        /// Authorized minter list.
        authorized_minters: Vec<C::Address>,
    },
    UpdateName {
        name: String
    }
}

impl<C: sov_modules_api::Context> BankA<C> {
    /// Creates a token from a set of configuration parameters.
    /// Checks if a token already exists at that address. If so return an error.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_token(
        &self,
        token_name: String,
        salt: u64,
        initial_balance: Amount,
        minter_address: C::Address,
        authorized_minters: Vec<C::Address>,
        context: &C,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<CallResponse> {
        let (token_address, token) = Token::<C>::create(
            &token_name,
            &[(minter_address, initial_balance)],
            &authorized_minters,
            context.sender().as_ref(),
            salt,
            self.tokens.prefix(),
            working_set,
        )?;

        if self.tokens.get(&token_address, working_set).is_some() {
            bail!(
                "Token {} at {} address already exists",
                token_name,
                token_address
            );
        }

        self.tokens.set(&token_address, &token, working_set);
        Ok(CallResponse::default())
    }

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

/// Creates a new prefix from an already existing prefix `parent_prefix` and a `token_address`
/// by extending the parent prefix.
pub(crate) fn prefix_from_address_with_parent<C: sov_modules_api::Context>(
    parent_prefix: &sov_state::Prefix,
    token_address: &C::Address,
) -> sov_state::Prefix {
    let mut prefix = parent_prefix.as_aligned_vec().clone().into_inner();
    prefix.extend_from_slice(format!("{}", token_address).as_bytes());
    sov_state::Prefix::new(prefix)
}

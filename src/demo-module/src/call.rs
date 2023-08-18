use anyhow::{bail, Context, Result};
#[cfg(feature = "native")]
use sov_modules_api::macros::CliWalletArg;
use sov_modules_api::CallResponse;
use sov_state::WorkingSet;

use crate::{Amount, BankA, Coins, Token};

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

    /// Transfers a specified amount of tokens to the specified address.
    Transfer {
        /// The address to which the tokens will be transferred.
        to: C::Address,
        /// The amount of tokens to transfer.
        coins: Coins<C>,
    },
    
    /// Mints a specified amount of tokens.
    Mint {
        /// The amount of tokens to mint.
        coins: Coins<C>,
        /// Address to mint tokens to
        minter_address: C::Address,
    },
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

    /// Transfers the set of `coins` to the address specified by `to`.
    /// Helper function that calls the [`transfer_from`] method from the bank module
    pub fn transfer(
        &self,
        to: C::Address,
        coins: Coins<C>,
        context: &C,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<CallResponse> {
        self.transfer_from(context.sender(), &to, coins, working_set)
    }

    /// Mints the `coins` set by the address `minter_address`. If the token address doesn't exist return an error.
    /// Calls the [`Token::mint`] function and update the `self.tokens` set to store the new minted address.
    pub(crate) fn mint(
        &self,
        coins: Coins<C>,
        minter_address: C::Address,
        context: &C,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<CallResponse> {
        let context_logger = || {
            format!(
                "Failed mint coins({}) to {} by minter {}",
                coins,
                minter_address,
                context.sender()
            )
        };
        let mut token = self
            .tokens
            .get_or_err(&coins.token_address, working_set)
            .with_context(context_logger)?;
        token
            .mint(context.sender(), &minter_address, coins.amount, working_set)
            .with_context(context_logger)?;
        self.tokens.set(&coins.token_address, &token, working_set);

        Ok(CallResponse::default())
    }

}

impl<C: sov_modules_api::Context> BankA<C> {
    /// Transfers the set of `coins` from the address `from` to the address `to`.
    /// Returns an error if the token address doesn't exist. Otherwise, call the [`Token::transfer`] function.
    pub fn transfer_from(
        &self,
        from: &C::Address,
        to: &C::Address,
        coins: Coins<C>,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<CallResponse> {
        let context_logger = || {
            format!(
                "Failed transfer from={} to={} of coins({})",
                from, to, coins
            )
        };
        let token = self
            .tokens
            .get_or_err(&coins.token_address, working_set)
            .with_context(context_logger)?;
        token
            .transfer(from, to, coins.amount, working_set)
            .with_context(context_logger)?;
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

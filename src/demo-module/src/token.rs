#[cfg(feature = "native")]
use core::str::FromStr;
use std::collections::HashSet;
use std::fmt::Formatter;
#[cfg(feature = "native")]
use std::num::ParseIntError;

use anyhow::{bail, Result};
use sov_state::{Prefix, WorkingSet};
#[cfg(feature = "native")]
use thiserror::Error;

use crate::call::prefix_from_address_with_parent;

/// Type alias to store an amount of token.
pub type Amount = u64;

/// Structure that stores information specifying
/// a given `amount` (type [`Amount`]) of coins stored at a `token_address` (type [`Context::Address`]).
#[cfg_attr(
    feature = "native",
    derive(serde::Serialize),
    derive(serde::Deserialize),
    derive(clap::Parser),
    derive(schemars::JsonSchema),
    schemars(bound = "C::Address: ::schemars::JsonSchema", rename = "Coins")
)]
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub struct Coins<C: sov_modules_api::Context> {
    /// An `amount` of coins stored.
    pub amount: Amount,
    /// The address where the tokens are stored.
    pub token_address: C::Address,
}

/// The errors that might arise when parsing a `Coins` struct from a string.
#[cfg(feature = "native")]
#[derive(Debug, Error)]
pub enum CoinsFromStrError {
    /// The amount could not be parsed as a u64.
    #[error("Could not parse {input} as a valid amount: {err}")]
    InvalidAmount { input: String, err: ParseIntError },
    /// The input string was malformed, so the `amount` substring could not be extracted.
    #[error("No amount was provided. Make sure that your input is in the format: amount,token_address. Example: 100,sov15vspj48hpttzyvxu8kzq5klhvaczcpyxn6z6k0hwpwtzs4a6wkvqmlyjd6")]
    NoAmountProvided,
    /// The token address could not be parsed as a valid address.
    #[error("Could not parse {input} as a valid address: {err}")]
    InvalidTokenAddress { input: String, err: anyhow::Error },
    /// The input string was malformed, so the `token_address` substring could not be extracted.
    #[error("No token address was provided. Make sure that your input is in the format: amount,token_address. Example: 100,sov15vspj48hpttzyvxu8kzq5klhvaczcpyxn6z6k0hwpwtzs4a6wkvqmlyjd6")]
    NoTokenAddressProvided,
}

#[cfg(feature = "native")]
impl<C: sov_modules_api::Context> FromStr for Coins<C> {
    type Err = CoinsFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, ',');

        let amount_str = parts.next().ok_or(CoinsFromStrError::NoAmountProvided)?;
        let token_address_str = parts
            .next()
            .ok_or(CoinsFromStrError::NoTokenAddressProvided)?;

        let amount =
            amount_str
                .parse::<Amount>()
                .map_err(|err| CoinsFromStrError::InvalidAmount {
                    input: amount_str.into(),
                    err,
                })?;
        let token_address = C::Address::from_str(token_address_str).map_err(|err| {
            CoinsFromStrError::InvalidTokenAddress {
                input: token_address_str.into(),
                err,
            }
        })?;

        Ok(Self {
            amount,
            token_address,
        })
    }
}
impl<C: sov_modules_api::Context> std::fmt::Display for Coins<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // implement Display for Coins
        write!(
            f,
            "token_address={} amount={}",
            self.token_address, self.amount
        )
    }
}

/// This struct represents a token in the sov-bank module.
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq, Clone)]
pub(crate) struct Token<C: sov_modules_api::Context> {
    /// Name of the token.
    pub(crate) name: String,
    /// Total supply of the coins.
    pub(crate) total_supply: u64,
    /// Mapping from user address to user balance.
    pub(crate) balances: sov_state::StateMap<C::Address, Amount>,

    /// Vector containing the authorized minters
    /// Empty vector indicates that the token supply is frozen
    /// Non empty vector indicates members of the vector can mint.
    /// Freezing a token requires emptying the vector
    /// NOTE: This is explicit so if a creator doesn't add themselves, then they can't mint
    pub(crate) authorized_minters: Vec<C::Address>,
}

impl<C: sov_modules_api::Context> Token<C> {
    /// Creates a token from a given set of parameters.
    /// The `token_name`, `sender` address (as a `u8` slice), and the `salt` (`u64` number) are used as an input
    /// to an hash function that computes the token address. Then the initial accounts and balances are populated
    /// from the `address_and_balances` slice and the `total_supply` of tokens is updated each time.
    /// Returns a tuple containing the computed `token_address` and the created `token` object.
    pub(crate) fn create(
        token_name: &str,
        address_and_balances: &[(C::Address, u64)],
        authorized_minters: &[C::Address],
        sender: &[u8],
        salt: u64,
        parent_prefix: &Prefix,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<(C::Address, Self)> {
        let token_address = super::get_token_address::<C>(token_name, sender, salt);
        let token_prefix = prefix_from_address_with_parent::<C>(parent_prefix, &token_address);
        let balances = sov_state::StateMap::new(token_prefix);

        let mut total_supply: Option<u64> = Some(0);
        for (address, balance) in address_and_balances.iter() {
            balances.set(address, balance, working_set);
            total_supply = total_supply.and_then(|ts| ts.checked_add(*balance));
        }

        let total_supply = match total_supply {
            Some(total_supply) => total_supply,
            None => bail!("Total supply overflow"),
        };

        let mut indices = HashSet::new();
        let mut auth_minter_list = Vec::new();

        for (i, item) in authorized_minters.iter().enumerate() {
            if indices.insert(item.as_ref()) {
                auth_minter_list.push(authorized_minters[i].clone());
            }
        }

        let token = Token::<C> {
            name: token_name.to_owned(),
            total_supply,
            balances,
            authorized_minters: auth_minter_list,
        };

        Ok((token_address, token))
    }
}

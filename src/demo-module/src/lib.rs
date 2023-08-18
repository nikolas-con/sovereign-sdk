mod call;
mod genesis;
#[cfg(feature = "native")]
mod query;
mod token;
mod utils;

/// Specifies the call methods using in that module.
pub use call::CallMessage;
#[cfg(feature = "native")]
/// Specifies the different queries used in that module.
pub use query::{BalanceResponse, BankARpcImpl, BankARpcServer, TotalSupplyResponse};
use sov_modules_api::{Error, ModuleInfo};
use sov_state::WorkingSet;
use token::Token;
/// Specifies an interfact to interact with tokens.
pub use token::{Amount, Coins};
/// Methods to get a token address.
pub use utils::{get_genesis_token_address, get_token_address};

/// [`TokenConfig`] specifies a configuration used when generating a token for the bank
/// module.
pub struct TokenConfig<C: sov_modules_api::Context> {
    /// The name of the token.
    pub token_name: String,
    /// A vector of tuples containing the initial addresses and balances (as u64)
    pub address_and_balances: Vec<(C::Address, u64)>,
    /// The addresses that are authorized to mint the token.
    pub authorized_minters: Vec<C::Address>,
    /// A salt used to encrypt the token address.
    pub salt: u64,
}

/// Initial configuration for sov-bank module.
pub struct BankConfig<C: sov_modules_api::Context> {
    /// A list of configurations for the initial tokens.
    pub tokens: Vec<TokenConfig<C>>,
}

/// The sov-bank module manages user balances. It provides functionality for:
/// - Token creation.
#[cfg_attr(feature = "native", derive(sov_modules_api::ModuleCallJsonSchema))]
#[derive(ModuleInfo, Clone)]
pub struct BankA<C: sov_modules_api::Context> {
    /// The address of the sov-bank module.
    #[address]
    pub(crate) address: C::Address,

    /// A mapping of addresses to tokens in the sov-bank.
    #[state]
    pub(crate) tokens: sov_state::StateMap<C::Address, Token<C>>,
}

impl<C: sov_modules_api::Context> sov_modules_api::Module for BankA<C> {
    type Context = C;

    type Config = BankConfig<C>;

    type CallMessage = call::CallMessage<C>;

    fn genesis(
        &self,
        config: &Self::Config,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<(), Error> {
        Ok(self.init_module(config, working_set)?)
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Self::Context,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Result<sov_modules_api::CallResponse, Error> {
        match msg {
            call::CallMessage::CreateToken {
                salt,
                token_name,
                initial_balance,
                minter_address,
                authorized_minters,
            } => Ok(self.create_token(
                token_name,
                salt,
                initial_balance,
                minter_address,
                authorized_minters,
                context,
                working_set,
            )?)
        }
    }
}

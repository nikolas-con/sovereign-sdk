#![allow(missing_docs)]
use jsonrpsee::core::RpcResult;
use sov_modules_api::macros::rpc_gen;
use sov_state::WorkingSet;

use crate::DemoModule;

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct NameResponse {
    /// The amount of token supply for a given token address. Equivalent to u64.
    pub name: Option<String>
}

#[rpc_gen(client, server, namespace = "demo_module")]
impl<C: sov_modules_api::Context> DemoModule<C> {

    #[rpc_method(name = "getName")]
    /// Rpc method that returns the supply of token of the token stored at the address `get_name`.
    pub fn get_name(
        &self,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> RpcResult<NameResponse> {
        Ok(NameResponse {
            name: self.get_name_of(working_set)
        })
    }
}

impl<C: sov_modules_api::Context> DemoModule<C> {
    pub fn get_name_of(
        &self,
        working_set: &mut WorkingSet<C::Storage>,
    ) -> Option<String> {
        self.name.get(working_set)
    }
}

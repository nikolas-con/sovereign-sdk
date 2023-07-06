use ethereum_types::{Address, H256, U256, U64};
use ethers_core::types::{Block, BlockId, FeeHistory, Transaction, TransactionReceipt, TxHash};
use sov_modules_macros::rpc_gen;
use sov_state::WorkingSet;

use crate::Evm;

#[rpc_gen(client, server, namespace = "eth")]
impl<C: sov_modules_api::Context> Evm<C> {
    #[rpc_method(name = "chainId")]
    pub fn chain_id(&self, _working_set: &mut WorkingSet<C::Storage>) -> Option<U64> {
        println!("eth_chainId!");
        Some(U64::from(1u64))
    }

    #[rpc_method(name = "getBlockByNumber")]
    pub fn get_block_by_number(
        &self,
        b: Option<String>,
        l: Option<bool>,
        _working_set: &mut WorkingSet<C::Storage>,
    ) -> Option<Block<TxHash>> {
        println!("eth_getBlockByNumber!");
        println!("{:?} {:?}", b, l);

        let b = Block::<TxHash> {
            base_fee_per_gas: Some(100.into()),
            ..Default::default()
        };

        Some(b)
    }

    #[rpc_method(name = "feeHistory")]
    pub fn fee_history(&self, _working_set: &mut WorkingSet<C::Storage>) -> FeeHistory {
        FeeHistory {
            base_fee_per_gas: Default::default(),
            gas_used_ratio: Default::default(),
            oldest_block: Default::default(),
            reward: Default::default(),
        }
    }

    #[rpc_method(name = "blockNumber")]
    pub fn block_number(&self, _working_set: &mut WorkingSet<C::Storage>) -> U256 {
        unimplemented!()
    }

    #[rpc_method(name = "getTransactionByHash")]
    pub fn get_transaction_by_hash(
        &self,
        _hash: H256,
        _working_set: &mut WorkingSet<C::Storage>,
    ) -> Option<Transaction> {
        unimplemented!()
    }

    #[rpc_method(name = "getTransactionReceipt")]
    pub fn get_transaction_receipt(
        &self,
        _hash: H256,
        _working_set: &mut WorkingSet<C::Storage>,
    ) -> Option<TransactionReceipt> {
        unimplemented!()
    }

    #[rpc_method(name = "eth_getTransactionCount")]
    pub fn get_transaction_count(
        &self,
        _address: Address,
        _block_number: Option<BlockId>,
        _working_set: &mut WorkingSet<C::Storage>,
    ) -> Option<U256> {
        unimplemented!()
    }
}

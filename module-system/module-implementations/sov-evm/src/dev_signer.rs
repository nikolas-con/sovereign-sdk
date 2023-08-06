use reth_primitives::{
    sign_message, Address, Signature, Transaction as RethTransaction, TransactionSigned,
    TxEip1559 as RethTxEip1559, H256,
};
use reth_rpc::eth::error::SignError;
use secp256k1::SecretKey;

type Result<T> = std::result::Result<T, SignError>;

/// Holds developer keys
pub(crate) struct DevSigner {
    secret_key: SecretKey,
}

impl DevSigner {
    pub(crate) fn sign_hash(&self, hash: H256, account: Address) -> Result<Signature> {
        let signature = sign_message(H256::from_slice(self.secret_key.as_ref()), hash);
        signature.map_err(|_| SignError::CouldNotSign)
    }

    pub(crate) fn sign_transaction(
        &self,
        transaction: RethTxEip1559,
        address: &Address,
    ) -> Result<TransactionSigned> {
        let transaction = RethTransaction::Eip1559(transaction);

        let tx_signature_hash = transaction.signature_hash();
        let signature = self.sign_hash(tx_signature_hash, *address)?;

        Ok(TransactionSigned::from_transaction_and_signature(
            transaction,
            signature,
        ))
    }
}

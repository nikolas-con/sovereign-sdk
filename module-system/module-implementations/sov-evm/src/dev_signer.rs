use reth_primitives::{
    sign_message, Address, Bytes as RethBytes, Signature, Transaction as RethTransaction,
    TransactionKind, TransactionSigned, TxEip1559 as RethTxEip1559, H256,
};
use reth_rpc::eth::error::SignError;
use secp256k1::SecretKey;

use crate::evm::EvmTransaction;
//let public_key = PublicKey::from_secret_key(SECP256K1, &secret_key);
type Result<T> = std::result::Result<T, SignError>;

/// Holds developer keys
pub(crate) struct DevSigner {
    secret_key: SecretKey,
}

impl DevSigner {
    pub(crate) fn sign_hash(&self, hash: H256) -> Result<Signature> {
        let signature = sign_message(H256::from_slice(self.secret_key.as_ref()), hash);
        signature.map_err(|_| SignError::CouldNotSign)
    }

    pub(crate) fn sign_transaction(&self, transaction: RethTxEip1559) -> Result<TransactionSigned> {
        let transaction = RethTransaction::Eip1559(transaction);

        let tx_signature_hash = transaction.signature_hash();

        let signature = sign_message(
            H256::from_slice(self.secret_key.as_ref()),
            tx_signature_hash,
        )
        .map_err(|_| SignError::CouldNotSign)?;

        Ok(TransactionSigned::from_transaction_and_signature(
            transaction,
            signature,
        ))
    }

    pub(crate) fn sign_default_transaction(
        &self,
        to: TransactionKind,
        data: Vec<u8>,
        nonce: u64,
    ) -> Result<TransactionSigned> {
        let reth_tx = RethTxEip1559 {
            to,
            input: RethBytes::from(data),
            nonce,
            ..Default::default()
        };

        self.sign_transaction(reth_tx)
    }
}

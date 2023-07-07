use borsh::ser::BorshSerialize;
use demo_stf::app::DefaultPrivateKey;
use demo_stf::runtime::{DefaultContext, Runtime};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::{Bytes, TxHash};
use ethers::utils::rlp::Rlp;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::params::ArrayParams;
use jsonrpsee::http_client::{HeaderMap, HttpClient};
use jsonrpsee::RpcModule;
use sov_evm::call::CallMessage;
use sov_evm::evm::EvmTransaction;
use sov_modules_api::transaction::Transaction;

const GAS_PER_BYTE: usize = 120;
pub struct Ethereum {}

pub fn get_ethereum_rpc() -> RpcModule<Ethereum> {
    let e = Ethereum {};
    let mut rpc = RpcModule::new(e);
    register_rpc_methods(&mut rpc).expect("Failed to register sequencer RPC methods");

    rpc
}

fn default_rpc_addr() -> String {
    "http://localhost:11111/".into()
}

fn default_max_response_size() -> u32 {
    1024 * 1024 * 100 // 100 MB
}

fn celestia_rpc_auth_token() -> String {
    todo!()
}

fn client() -> HttpClient {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        format!("Bearer {}", celestia_rpc_auth_token())
            .parse()
            .unwrap(),
    );

    jsonrpsee::http_client::HttpClientBuilder::default()
        .set_headers(headers)
        .max_request_body_size(default_max_response_size()) // 100 MB
        .build(default_rpc_addr())
        .expect("Client initialization is valid")
}

fn register_rpc_methods(rpc: &mut RpcModule<Ethereum>) -> Result<(), jsonrpsee::core::Error> {
    rpc.register_method("eth_sendRawTransaction", |p, e| {
        println!("eth_sendRawTransaction");
        let data: Bytes = p.one().unwrap();
        let data = data.as_ref();

        if data[0] > 0x7f {
            panic!("lol")
        }

        let r = Rlp::new(data);

        let (decoded_tx, _decoded_sig) = TypedTransaction::decode_signed(&r).unwrap();
        println!("decoded_tx {:?}", decoded_tx);

        let h: TxHash = decoded_tx.sighash();

        /*
                let tx = EvmTransaction {
                    ..Default::default()
                };

                let tx = CallMessage { tx };

                let message = Runtime::<DefaultContext>::encode_evm_call(tx);

                let sender = DefaultPrivateKey::generate();
                let tx = Transaction::<DefaultContext>::new_signed_tx(&sender, message, 0);

                let raw = tx.try_to_vec().unwrap();

                let blob = vec![raw].try_to_vec().unwrap();

                let client = client();

                let fee: u64 = 2000;
                let namespace = todo!();
                let blob = todo!();
                // We factor extra share to be occupied for namespace, which is pessimistic
                let gas_limit = (blob.len() + 512) * GAS_PER_BYTE + 1060;

                let mut params = ArrayParams::new();
                params.insert(namespace)?;
                params.insert(blob)?;
                params.insert(fee.to_string())?;
                params.insert(gas_limit)?;
                let response = client
                    .request::<serde_json::Value, _>("state.SubmitPayForBlob", params)
                    .await?;
        */
        Ok(h)
    })?;

    Ok(())
}

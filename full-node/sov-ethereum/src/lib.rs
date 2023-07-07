use borsh::ser::BorshSerialize;
use const_rollup_config::ROLLUP_NAMESPACE_RAW;
use demo_stf::app::DefaultPrivateKey;
use demo_stf::runtime::{DefaultContext, Runtime};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::{Bytes, TxHash};
use ethers::utils::rlp::Rlp;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::params::ArrayParams;
use jsonrpsee::http_client::{HeaderMap, HttpClient};
use jsonrpsee::RpcModule;
use jupiter::da_service::DaServiceConfig;
//use jupiter::types::NamespaceId;
use sov_evm::call::CallMessage;
use sov_evm::evm::EvmTransaction;
use sov_modules_api::transaction::Transaction;

const GAS_PER_BYTE: usize = 120;

pub struct Ethereum {
    pub config: DaServiceConfig,
}

impl Ethereum {
    fn client(&self) -> HttpClient {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.config.celestia_rpc_auth_token.clone())
                .parse()
                .unwrap(),
        );

        jsonrpsee::http_client::HttpClientBuilder::default()
            .set_headers(headers)
            .max_request_body_size(default_max_response_size()) // 100 MB
            .build(self.config.celestia_rpc_address.clone())
            .expect("Client initialization is valid")
    }
}

pub fn get_ethereum_rpc(config: DaServiceConfig) -> RpcModule<Ethereum> {
    let e = Ethereum { config };
    let mut rpc = RpcModule::new(e);
    register_rpc_methods(&mut rpc).expect("Failed to register sequencer RPC methods");

    rpc
}

fn register_rpc_methods(rpc: &mut RpcModule<Ethereum>) -> Result<(), jsonrpsee::core::Error> {
    rpc.register_async_method("eth_sendRawTransaction", |p, e| async move {
        println!("eth_sendRawTransaction");
        let data: Bytes = p.one().unwrap();
        let data = data.as_ref();

        if data[0] > 0x7f {
            panic!("lol")
        }

        let r = Rlp::new(data);

        let (decoded_tx, _decoded_sig) = TypedTransaction::decode_signed(&r).unwrap();
        //println!("decoded_tx {:?}", decoded_tx);

        let h: TxHash = decoded_tx.sighash();

        let tx = EvmTransaction {
            ..Default::default()
        };

        let tx = CallMessage { tx };

        let message = Runtime::<DefaultContext>::encode_evm_call(tx);

        let sender = DefaultPrivateKey::generate();
        let tx = Transaction::<DefaultContext>::new_signed_tx(&sender, message, 0);

        let raw = tx.try_to_vec().unwrap();

        {
            let blob = vec![raw].try_to_vec().unwrap();

            let client = e.client();

            let fee: u64 = 2000;
            let namespace = ROLLUP_NAMESPACE_RAW.to_vec();

            // We factor extra share to be occupied for namespace, which is pessimistic
            let gas_limit = (blob.len() + 512) * GAS_PER_BYTE + 1060;

            let mut params = ArrayParams::new();
            params.insert(namespace)?;
            params.insert(blob)?;
            params.insert(fee.to_string())?;
            params.insert(gas_limit)?;
            let response = client
                .request::<serde_json::Value, _>("state.SubmitPayForBlob", params)
                .await
                .unwrap();

            println!("RESP {}", response);
        }

        Ok(h)
    })?;

    Ok(())
}

fn default_max_response_size() -> u32 {
    1024 * 1024 * 100 // 100 MB
}

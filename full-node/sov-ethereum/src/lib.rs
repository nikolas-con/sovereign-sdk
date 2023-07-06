use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::{Bytes, TxHash};
use ethers::utils::rlp::Rlp;
use jsonrpsee::RpcModule;

pub struct Ethereum {}

pub fn get_ethereum_rpc() -> RpcModule<Ethereum> {
    let e = Ethereum {};
    let mut rpc = RpcModule::new(e);
    register_rpc_methods(&mut rpc).expect("Failed to register sequencer RPC methods");

    rpc
}

fn register_rpc_methods(rpc: &mut RpcModule<Ethereum>) -> Result<(), jsonrpsee::core::Error> {
    rpc.register_method::<Result<(), ()>, _>("eth_sendTransaction", |p, e| {
        println!("eth_sendTransaction");
        unimplemented!()
    })?;

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
        Ok(h)
    })?;

    Ok(())
}

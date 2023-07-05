use ethers::types::U64;
use jsonrpsee::RpcModule;

pub struct Ethereum {}

pub fn get_ethereum_rpc() -> RpcModule<Ethereum> {
    let e = Ethereum {};
    let mut rpc = RpcModule::new(e);
    register_rpc_methods(&mut rpc).expect("Failed to register sequencer RPC methods");

    rpc
}

fn register_rpc_methods(rpc: &mut RpcModule<Ethereum>) -> Result<(), jsonrpsee::core::Error> {
    rpc.register_method("eth_sendRawTransaction", |p, e| {
        println!("eth_sendRawTransaction");
        println!("Params {:?}", p);

        Ok(())
    })?;

    rpc.register_method("eth_sendTransaction", |p, e| {
        println!("eth_sendTransaction");
        println!("Params {:?}", p);

        Ok(())
    })?;

    rpc.register_method("eth_getTransactionCount", |p, e| {
        println!("eth_getTransactionCount");
        println!("Params {:?}", p);

        Ok(())
    })?;

    rpc.register_method("eth_chainId", |p, e| {
        println!("eth_chainId");
        println!("Params {:?}", p);

        Ok(Some(U64::from(1)))
    })?;

    rpc.register_method("eth_getBlockByNumber", |p, e| {
        println!("eth_getBlockByNumber");
        println!("Params {:?}", p);

        Ok(Some(1))
    })?;

    rpc.register_method("eth_feeHistory", |p, e| {
        println!("eth_feeHistory");
        println!("Params {:?}", p);

        Ok(())
    })?;

    Ok(())
}




### Transaction

```sh
### submit tx
cargo run --bin demo-cli submit-transaction keys/token_deployer_key.json DemoModule "{ \"UpdateName\": { \"name\": \"gm\" } }" 0 http://127.0.0.1:12345
cargo run --bin demo-cli publish-batch http://127.0.0.1:12345

### query data
curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"demo_module_getName","params":[],"id":1}' http://127.0.0.1:12345
```

<details>
<summary>Submitting a transaction manually</summary>

```sh

### create batch
cargo run --bin demo-cli generate-transaction-from-json keys/token_deployer_key.json DemoModule "{ \"UpdateName\": { \"name\": \"gm\" } }" 0 > /tmp/update_name_tx.dat
cargo run --bin demo-cli make-batch /tmp/update_name_tx.dat > /tmp/test_blob_tx.dat

### submit batch
cargo run --bin demo-cli util print-namespace ### get namespace
docker exec sov-celestia-local celestia-appd tx blob PayForBlobs ${NAMESPACE} $(cat /tmp/test_blob_tx.dat | tr -d '\n') --from validator --chain-id=test --fees=300utia -y
```
</details>


### Setup

```sh

### start celestia
IMAGE_NAME=dubbelosix/sov-celestia-local:genesis-v0.7.1
docker run -d --name sov-celestia-local --platform linux/amd64 -p 26657:26657 -p 26659:26659 -p 26658:26658 ${IMAGE_NAME} # start new
docker start sov-celestia-local # resume existing

### fund validator
docker exec sov-celestia-local celestia-appd keys show validator # show wallets
docker exec sov-celestia-local celestia-appd keys add validator # create new wallet
docker exec sov-celestia-local celestia-appd tx bank send validator ${VALIDATOR_ADDRESS} 10000000utia --fees=300utia -y # fund address

### update config
docker exec sov-celestia-local /celestia bridge auth admin --node.store /bridge ### get auth token
sed -i '' 's/^\(celestia_rpc_auth_token = \)"[^"]*"/\1"${AUTH_TOKEN}"/' rollup_config.toml

### cleanup
docker stop sov-celestia-local # stop
docker rm sov-celestia-local # remove
rm -rf "../../data" # clean rollup

```


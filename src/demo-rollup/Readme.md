

### Create Batch
```sh
cargo run --bin sov-cli generate-transaction-from-json ../test-data/keys/token_deployer_private_key.json Bank ../test-data/requests/create_token.json 0
cargo run --bin sov-cli make-batch ../test-data/requests/create_token.dat > ../test-data/requests/test_blob.dat
```

### Submit Transaction
```sh
cargo run --bin sov-cli util print-namespace ### get namespace
docker exec sov-celestia-local celestia-appd tx blob PayForBlobs ${NAMESPACE} $(cat ../test-data/requests/test_blob.dat) --from validator --chain-id=test --fees=300utia -y
```




### Setup Celestia

```sh

### start celestia
IMAGE_NAME=dubbelosix/sov-celestia-local:genesis-v0.7.1
docker run -d --name sov-celestia-local --platform linux/amd64 -p 26657:26657 -p 26659:26659 -p 26658:26658 ${IMAGE_NAME} # start docker
docker start sov-celestia-local # resume docker

### fund validator
docker exec sov-celestia-local celestia-appd keys show validator # show wallets
docker exec sov-celestia-local celestia-appd keys add validator # create new wallet
docker exec sov-celestia-local celestia-appd tx bank send validator $(VALIDATOR_ADDRESS) 10000000utia --fees=300utia -y # fund docker

### update config
docker exec sov-celestia-local /celestia bridge auth admin --node.store /bridge ### get auth token
sed -i '' 's/^\(celestia_rpc_auth_token = \)"[^"]*"/\1"${AUTH_TOKEN}"/' rollup_config.toml
sed -i '' 's#^\(celestia_rpc_address = \)"[^"]*"#\1"http://127.0.0.1:26658"#' rollup_config.toml
sed -i '' 's#^\(start_height = \)[0-9]*#\11#' rollup_config.toml

```

### Cleanup

```sh
docker stop sov-celestia-local # stop docker
docker rm sov-celestia-local # rm docker
rm -rf "../../data" # clean rollup
```

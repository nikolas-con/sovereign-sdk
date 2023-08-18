

### Get Namespace
```sh
cargo run --bin sov-cli util print-namespace
```

### Create Batch
```sh
cargo run --bin sov-cli generate-transaction-from-json ../test-data/keys/token_deployer_private_key.json Bank ../test-data/requests/create_token.json 0
cargo run --bin sov-cli make-batch ../test-data/requests/create_token.dat > ../test-data/requests/test_blob.dat
```

### Submit Transaction
```sh
docker exec sov-celestia-local celestia-appd tx blob PayForBlobs ${NAMESPACE} $(cat ../test-data/requests/test_blob.dat) --from validator --chain-id=test --fees=300utia -y
```

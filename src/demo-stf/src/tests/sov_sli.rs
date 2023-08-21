
#[cfg(test)]
mod test {
    use borsh::BorshDeserialize;
    use demo_stf::app::App;
    use demo_stf::config::{create_demo_config, DEMO_SEQUENCER_DA_ADDRESS, LOCKED_AMOUNT};
    use demo_stf::runtime::{GenesisConfig, Runtime};
    use sov_modules_api::Address;
    use sov_modules_stf_template::{AppTemplate, Batch, RawTx, SequencerOutcome};
    use sov_rollup_interface::mocks::{MockValidityCond, MockZkvm, MockBlock};
    use sov_rollup_interface::stf::StateTransitionFunction;
    use sov_state::WorkingSet;
    use sov_stf_runner::Config;

    use borsh::BorshSerialize;
    use crate::*;

    type TestBlob = sov_rollup_interface::mocks::MockBlob<Address>;

    fn new_test_blob(batch: Batch, address: &[u8]) -> TestBlob {
        let address = Address::try_from(address).unwrap();
        let data = batch.try_to_vec().unwrap();
        TestBlob::new(data, address, [0; 32])
    }

    #[test]
    fn test_sov_cli() {
        // Tempdir is created here, so it will be deleted only after test is finished.
        let tempdir = tempfile::tempdir().unwrap();
        let mut test_demo = TestDemo::with_path(tempdir.path().to_path_buf());
        let test_data = read_test_data();

        execute_txs(&mut test_demo.demo, test_demo.config, test_data.data);

        // get minter balance
        let balance = get_balance(
            &mut test_demo.demo,
            &test_data.token_deployer_address,
            test_data.minter_address,
        );

        assert_eq!(balance, None)
    }

    #[test]
    fn test_update_name() {
        let tempdir = tempfile::tempdir().unwrap();
        let mut test_demo = TestDemo::with_path(tempdir.path().to_path_buf());
        let test_tx = serialize_call(&Commands::GenerateTransactionFromJson {
            sender_priv_key_path: make_test_path("token_deployer_key.json")
                .to_str()
                .unwrap()
                .into(),
            module_name: "DemoModule".into(),
            call_data_path: make_test_path("update_name_tx.json")
                .to_str()
                .unwrap()
                .into(),
            nonce: 0,
        })
        .unwrap();

        let mut test_data = read_test_data();
        test_data.data.pop();
        test_data.data.pop();

        let batch = Batch {
            txs: test_data.data.clone(),
        };

        println!("batch: {}", hex::encode(batch.try_to_vec().unwrap()));

        let blob = make_hex_blob(vec![test_tx].into_iter()).unwrap();
        println!("generated: {}", &blob);

        let blob = hex::decode(blob.as_bytes()).unwrap();

        let batch = Batch::deserialize(&mut &blob[..]).expect("must be valid blob");
        execute_txs(&mut test_demo.demo, test_demo.config, batch.txs);
    }

    // Test helpers
    struct TestDemo {
        config: GenesisConfig<C>,
        demo: AppTemplate<C, MockValidityCond, MockZkvm, Runtime<C>, TestBlob>,
    }

    impl TestDemo {
        fn with_path(path: PathBuf) -> Self {
            let value_setter_admin_private_key = DefaultPrivateKey::generate();
            let election_admin_private_key = DefaultPrivateKey::generate();

            let genesis_config = create_demo_config(
                LOCKED_AMOUNT + 1,
                &value_setter_admin_private_key,
                &election_admin_private_key,
            );

            let runner_config = Config {
                storage: sov_state::config::Config { path },
            };

            Self {
                config: genesis_config,
                demo: App::<MockZkvm, MockValidityCond, TestBlob>::new(runner_config.storage).stf,
            }
        }
    }

    struct TestData {
        token_deployer_address: Address,
        minter_address: Address,
        data: Vec<RawTx>,
    }

    fn make_test_path<P: AsRef<Path>>(path: P) -> PathBuf {
        let mut sender_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        sender_path.push("..");
        sender_path.push("..");
        sender_path.push("test-data");

        sender_path.push(path);

        sender_path
    }

    fn read_test_data() -> TestData {
        let serialized_tx = SerializedTx::new(
            make_test_path("token_deployer_key.json"),
            "DemoModule",
            make_test_path("update_name_tx.json"),
            0,
        )
        .unwrap();

        let data = vec![serialized_tx.raw];

        TestData {
            token_deployer_address: serialized_tx.sender,
            minter_address: serialized_tx.sender,
            data,
        }
    }

    fn execute_txs(
        demo: &mut AppTemplate<C, MockValidityCond, MockZkvm, Runtime<C>, TestBlob>,
        config: GenesisConfig<C>,
        txs: Vec<RawTx>,
    ) {
        StateTransitionFunction::<MockZkvm, TestBlob>::init_chain(demo, config);

        let data = MockBlock::default();
        let blob = new_test_blob(Batch { txs }, &DEMO_SEQUENCER_DA_ADDRESS);
        let mut blobs = [blob];

        let apply_block_result = StateTransitionFunction::<MockZkvm, TestBlob>::apply_slot(
            demo,
            Default::default(),
            &data,
            &mut blobs,
        );

        assert_eq!(1, apply_block_result.batch_receipts.len());
        let apply_blob_outcome = apply_block_result.batch_receipts[0].clone();

        assert_eq!(
            SequencerOutcome::Rewarded(0),
            apply_blob_outcome.inner,
            "Sequencer execution should have succeeded but failed",
        );
    }

    fn get_balance(
        demo: &mut AppTemplate<C, MockValidityCond, MockZkvm, Runtime<C>, TestBlob>,
        token_deployer_address: &Address,
        user_address: Address,
    ) -> Option<u64> {
        let token_address = create_token_address(token_deployer_address);

        let mut working_set = WorkingSet::new(demo.current_storage.clone());

        let balance = demo
            .runtime
            .bank
            .balance_of(user_address, token_address, &mut working_set)
            .unwrap();

        balance.amount
    }

    fn create_token_address(token_deployer_address: &Address) -> Address {
        sov_bank::get_token_address::<C>("sov-test-token", token_deployer_address.as_ref(), 11)
    }
}

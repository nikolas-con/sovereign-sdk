use hex;
use proptest::{array::uniform32, strategy::Strategy};
use proptest::{prop_compose, proptest};
use reqwest::header::CONTENT_TYPE;
use sov_db::ledger_db::{LedgerDB, SlotCommit};
use sov_rollup_interface::services::da::SlotData;
use std::net::SocketAddr;

use sov_rollup_interface::mocks::{TestBlock, TestBlockHeader};

use sov_rollup_interface::stf::{BatchReceipt, Event, TransactionReceipt};
use tendermint::crypto::Sha256;
use tokio::sync::oneshot;

use crate::{config::RpcConfig, ledger_rpc};

async fn query_test_helper(data: String, expected: &str, rpc_config: RpcConfig) {
    let (addr, port) = (rpc_config.bind_host, rpc_config.bind_port);
    let client = reqwest::Client::new();
    let url_str = format!("http://{addr}:{port}");

    let res = client
        .post(url_str)
        .header(CONTENT_TYPE, "application/json")
        .body(data)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().as_u16(), 200);
    let contents = res.text().await.unwrap();
    assert_eq!((&contents), expected);
}

fn populate_ledger(ledger_db: &mut LedgerDB, slots: Vec<SlotCommit<TestBlock, i32, i32>>) {
    for slot in slots {
        ledger_db.commit_slot(slot).unwrap();
    }
}

fn test_helper(data: String, expected: &str, slots: Vec<SlotCommit<TestBlock, i32, i32>>) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(async {
        let (tx_start, rx_start) = oneshot::channel();
        let (tx_end, rx_end) = oneshot::channel();

        let address = SocketAddr::new("127.0.0.1".parse().unwrap(), 0);

        // Initialize the ledger database, which stores blocks, transactions, events, etc.
        let tmpdir = tempfile::tempdir().unwrap();
        let mut ledger_db = LedgerDB::with_path(tmpdir.path()).unwrap();

        populate_ledger(&mut ledger_db, slots);

        let ledger_rpc_module = ledger_rpc::get_ledger_rpc::<i32, i32>(ledger_db.clone());

        rt.spawn(async move {
            let server = jsonrpsee::server::ServerBuilder::default()
                .build([address].as_ref())
                .await
                .unwrap();
            let actual_address = server.local_addr().unwrap();
            let _server_handle = server.start(ledger_rpc_module).unwrap();
            tx_start.send(actual_address.port()).unwrap();
            rx_end.await.unwrap();
        });

        let bind_port = rx_start.await.unwrap();
        let rpc_config = RpcConfig {
            bind_host: "127.0.0.1".to_string(),
            bind_port,
        };

        query_test_helper(data, expected, rpc_config).await;

        tx_end.send("drop server").unwrap();
    });
}

fn regular_test_helper(data: String, expected: &str) {
    let mut slots: Vec<SlotCommit<TestBlock, i32, i32>> = vec![SlotCommit::new(TestBlock {
        curr_hash: sha2::Sha256::digest(b"slot_data"),
        header: TestBlockHeader {
            prev_hash: sha2::Sha256::digest(b"prev_header"),
        },
    })];

    let batches = vec![
        BatchReceipt {
            batch_hash: ::sha2::Sha256::digest(b"batch_receipt"),
            tx_receipts: vec![
                TransactionReceipt::<i32> {
                    tx_hash: ::sha2::Sha256::digest(b"tx1"),
                    body_to_save: Some(b"tx1 body".to_vec()),
                    events: vec![],
                    receipt: 0,
                },
                TransactionReceipt::<i32> {
                    tx_hash: ::sha2::Sha256::digest(b"tx2"),
                    body_to_save: Some(b"tx2 body".to_vec()),
                    events: vec![
                        Event::new("event1_key", "event1_value"),
                        Event::new("event2_key", "event2_value"),
                    ],
                    receipt: 1,
                },
            ],
            inner: 0,
        },
        BatchReceipt {
            batch_hash: ::sha2::Sha256::digest(b"batch_receipt2"),
            tx_receipts: vec![TransactionReceipt::<i32> {
                tx_hash: ::sha2::Sha256::digest(b"tx1"),
                body_to_save: Some(b"tx1 body".to_vec()),
                events: vec![],
                receipt: 0,
            }],
            inner: 1,
        },
    ];

    for batch in batches {
        slots.get_mut(0).unwrap().add_batch(batch)
    }

    test_helper(data, expected, slots)
}

// These tests reproduce the README workflow for the ledger_rpc, ie:
// - It creates and populate a simple ledger with a few transactions
// - It initializes the rpc server
// - It successively calls the different rpc methods registered and tests the answer
// Side note: we need to change the port for each test to avoid concurrent access issues
#[test]
fn test_get_head() {
    let data = r#"{"jsonrpc":"2.0","method":"ledger_getHead","params":[],"id":1}"#.to_string();
    let expected = r#"{"jsonrpc":"2.0","result":{"number":1,"hash":"0xd1231a38586e68d0405dc55ae6775e219f29fff1f7e0c6410d0ac069201e550b","batch_range":{"start":1,"end":3}},"id":1}"#;

    regular_test_helper(data, expected);
}

#[test]
fn test_get_transactions() {
    // Tests for different types of argument
    let data = r#"{"jsonrpc":"2.0","method":"ledger_getTransactions","params":[[{ "batch_id": 1, "offset": 0}]],"id":1}"#.to_string();
    let expected = r#"{"jsonrpc":"2.0","result":[{"hash":"0x709b55bd3da0f5a838125bd0ee20c5bfdd7caba173912d4281cae816b79a201b","event_range":{"start":1,"end":1},"body":[116,120,49,32,98,111,100,121],"custom_receipt":0}],"id":1}"#;
    regular_test_helper(data, expected);

    // Tests for flattened args
    let data =
        r#"{"jsonrpc":"2.0","method":"ledger_getTransactions","params":[1],"id":1}"#.to_string();
    regular_test_helper(data, expected);

    let data =
        r#"{"jsonrpc":"2.0","method":"ledger_getTransactions","params":[[1]],"id":1}"#.to_string();
    regular_test_helper(data, expected);

    let data =
        r#"{"jsonrpc":"2.0","method":"ledger_getTransactions","params":[[1], "Standard"],"id":1}"#
            .to_string();
    regular_test_helper(data, expected);

    let data =
        r#"{"jsonrpc":"2.0","method":"ledger_getTransactions","params":[[1], "Compact"],"id":1}"#
            .to_string();
    regular_test_helper(data, expected);

    let data =
        r#"{"jsonrpc":"2.0","method":"ledger_getTransactions","params":[[1], "Full"],"id":1}"#
            .to_string();
    regular_test_helper(data, expected);

    let data = r#"{"jsonrpc":"2.0","method":"ledger_getTransactions","params":[[{ "batch_id": 1, "offset": 1}]],"id":1}"#
            .to_string();
    let expected = r#"{"jsonrpc":"2.0","result":[{"hash":"0x27ca64c092a959c7edc525ed45e845b1de6a7590d173fd2fad9133c8a779a1e3","event_range":{"start":1,"end":3},"body":[116,120,50,32,98,111,100,121],"custom_receipt":1}],"id":1}"#;
    regular_test_helper(data, expected);
}

#[test]
fn test_get_batches() {
    let data =
        r#"{"jsonrpc":"2.0","method":"ledger_getBatches","params":[[2], "Standard"],"id":1}"#
            .to_string();
    let expected = r#"{"jsonrpc":"2.0","result":[{"hash":"0xf85fe0cb36fdaeca571c896ed476b49bb3c8eff00d935293a8967e1e9a62071e","tx_range":{"start":3,"end":4},"txs":["0x709b55bd3da0f5a838125bd0ee20c5bfdd7caba173912d4281cae816b79a201b"],"custom_receipt":1}],"id":1}"#;
    regular_test_helper(data, expected);

    let data =
        r#"{"jsonrpc":"2.0","method":"ledger_getBatches","params":[[2]],"id":1}"#.to_string();
    regular_test_helper(data, expected);

    let data = r#"{"jsonrpc":"2.0","method":"ledger_getBatches","params":[2],"id":1}"#.to_string();
    regular_test_helper(data, expected);

    let data = r#"{"jsonrpc":"2.0","method":"ledger_getBatches","params":[[1], "Compact"],"id":1}"#
        .to_string();
    let expected = r#"{"jsonrpc":"2.0","result":[{"hash":"0xb5515a80204963f7db40e98af11aedb49a394b1c7e3d8b5b7a33346b8627444f","tx_range":{"start":1,"end":3},"custom_receipt":0}],"id":1}"#;
    regular_test_helper(data, expected);

    let data = r#"{"jsonrpc":"2.0","method":"ledger_getBatches","params":[[1], "Full"],"id":1}"#
        .to_string();
    let expected = r#"{"jsonrpc":"2.0","result":[{"hash":"0xb5515a80204963f7db40e98af11aedb49a394b1c7e3d8b5b7a33346b8627444f","tx_range":{"start":1,"end":3},"txs":[{"hash":"0x709b55bd3da0f5a838125bd0ee20c5bfdd7caba173912d4281cae816b79a201b","event_range":{"start":1,"end":1},"body":[116,120,49,32,98,111,100,121],"custom_receipt":0},{"hash":"0x27ca64c092a959c7edc525ed45e845b1de6a7590d173fd2fad9133c8a779a1e3","event_range":{"start":1,"end":3},"body":[116,120,50,32,98,111,100,121],"custom_receipt":1}],"custom_receipt":0}],"id":1}"#;
    regular_test_helper(data, expected);

    let data = r#"{"jsonrpc":"2.0","method":"ledger_getBatches","params":[[0], "Compact"],"id":1}"#
        .to_string();
    let expected = r#"{"jsonrpc":"2.0","result":[null],"id":1}"#;
    regular_test_helper(data, expected);
}

#[test]
fn test_get_events() {
    let data = r#"{"jsonrpc":"2.0","method":"ledger_getEvents","params":[1],"id":1}"#.to_string();
    let expected = r#"{"jsonrpc":"2.0","result":[{"key":[101,118,101,110,116,49,95,107,101,121],"value":[101,118,101,110,116,49,95,118,97,108,117,101]}],"id":1}"#;
    regular_test_helper(data, expected);

    let data = r#"{"jsonrpc":"2.0","method":"ledger_getEvents","params":[2],"id":1}"#.to_string();
    let expected = r#"{"jsonrpc":"2.0","result":[{"key":[101,118,101,110,116,50,95,107,101,121],"value":[101,118,101,110,116,50,95,118,97,108,117,101]}],"id":1}"#;
    regular_test_helper(data, expected);

    let data = r#"{"jsonrpc":"2.0","method":"ledger_getEvents","params":[3],"id":1}"#.to_string();
    let expected = r#"{"jsonrpc":"2.0","result":[null],"id":1}"#;
    regular_test_helper(data, expected);
}

proptest!(
    #[test]
    // Generates multiple slots with random headers and try to retrieve them
    fn proptest_get_head(hashs in proptest::collection::vec(proptest::array::uniform32(0_u8..), 1..100)){
        let mut slots = vec![];

        let mut prev_hash = [0;32];

        let num_hashes = hashs.len();

        for hash in hashs{
            slots.push(SlotCommit::new(TestBlock {
                curr_hash: hash,
                header: TestBlockHeader {
                    prev_hash,
                },
            }));

            prev_hash = hash;
        }

        let prev_hash_str = hex::encode(prev_hash);
        let data = r#"{"jsonrpc":"2.0","method":"ledger_getHead","params":[],"id":1}"#.to_string();
        let expected = format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"number\":{num_hashes},\"hash\":\"0x{prev_hash_str}\",\"batch_range\":{{\"start\":1,\"end\":1}}}},\"id\":1}}");
        test_helper(data, expected.as_str(), slots);
    }
);

prop_compose! {
    fn arb_event()(key in "\\w*", value in "\\w*") -> Event {
        Event::new(key.as_str(), value.as_str())
    }
}

prop_compose! {
    fn arb_txs(max_events : usize)(tx_hash in proptest::array::uniform32(0_u8..), body_to_save in "[\\w\\d]*", events in proptest::collection::vec(arb_event(), 0..max_events),
receipt in 0..3) -> TransactionReceipt::<i32> {
    TransactionReceipt{
        tx_hash,
        body_to_save: Some(body_to_save.into_bytes()),
        events,
        receipt
    }

}
}

prop_compose! {
    fn arb_batch(max_txs: usize, max_events: usize)(batch_hash in proptest::array::uniform32(0_u8..), tx_receipts in proptest::collection::vec(arb_txs(max_events), 0..max_txs), inner in 0..3) -> BatchReceipt<i32, i32>{
        BatchReceipt{
            batch_hash,
            tx_receipts,
            inner
        }

    }
}

prop_compose! {
    fn arb_batches_and_slot_hash(max_batch : usize, max_txs: usize, max_events: usize)
    (slot_hash in proptest::array::uniform32(0_u8..), batches in proptest::collection::vec(arb_batch(max_txs, max_events), 1..max_batch)) -> (Vec<BatchReceipt<i32, i32>>, [u8;32]){
        (batches, slot_hash)
    }
}

prop_compose! {
    fn arb_slots(max_slots : usize, max_batch: usize, max_txs: usize, max_events: usize)
    (batches_and_hashes in proptest::collection::vec(arb_batches_and_slot_hash(max_batch, max_txs, max_events), 1..max_slots)) -> (Vec<SlotCommit<TestBlock, i32, i32>>, usize)
    {
        let mut slots = std::vec::Vec::with_capacity(max_slots);

        let mut total_num_batches = 1;

        let mut prev_hash = [0;32];

        for (batches, hash) in batches_and_hashes{
            let mut new_slot = SlotCommit::new(TestBlock {
                curr_hash: hash,
                header: TestBlockHeader {
                    prev_hash,
                },
            });

            total_num_batches += batches.len();

            for batch in batches {
                    new_slot.add_batch(batch)
            }


            slots.push(new_slot);

            prev_hash = hash;
        }

        (slots, total_num_batches)
    }
}

proptest!(
    #[test]
    fn proptest_get_head_complete((slots, total_num_batches) in arb_slots(10, 10, 10, 10)){
        let num_slots = slots.len();
        let last_slot = slots.last().unwrap();

        let last_slot_hash = hex::encode(last_slot.slot_data().hash());
        let last_slot_num_batches = last_slot.batch_receipts().len();

        let last_slot_start_batch = total_num_batches - last_slot_num_batches;
        let last_slot_end_batch = total_num_batches;

        let data = r#"{"jsonrpc":"2.0","method":"ledger_getHead","params":[],"id":1}"#.to_string();
        let expected = format!("{{\"jsonrpc\":\"2.0\",\"result\":{{\"number\":{num_slots},\"hash\":\"0x{last_slot_hash}\",\"batch_range\":{{\"start\":{last_slot_start_batch},\"end\":{last_slot_end_batch}}}}},\"id\":1}}");
        test_helper(data, expected.as_str(), slots);
    }
);

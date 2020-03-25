mod common;
use crate::common::*;
use bee_bundle::*;

#[tokio::test]
async fn test_add_neighbors() {
    let client = client_init();
    let _ = client
        .add_neighbors(&["tcp://0.0.0.0:15600"])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_attach_to_tangle() {
    let client = client_init();
    let res = client
        .attach_to_tangle(
            &Hash::from_str(TEST_TRUNK_HASH),
            &Hash::from_str(TEST_BRANCH_HASH),
            9,
            &[tx()],
        )
        .await
        .unwrap();
    dbg!(&res);
    assert!(!res.trytes.is_empty());
}

#[tokio::test]
async fn test_broadcast_transactions() {
    let client = client_init();
    let _ = client.broadcast_transactions(&[tx()]).await.unwrap();
}

#[tokio::test]
async fn test_check_consistency() {
    let client = client_init();
    let res = client
        .check_consistency(&[Hash::from_str(TEST_BUNDLE_TX_0)])
        .await
        .unwrap();

    match res.state {
        true => assert!(res.info.is_none()),
        false => assert!(res.info.is_some()),
    }
}

#[tokio::test]
async fn test_find_tx_by_bundle() {
    let client = client_init();
    let res = client
        .find_transactions(
            Some(&[Hash::from_str(TEST_BUNDLE_HASH_0)]),
            None,
            None,
            None,
        )
        .await
        .unwrap();

    assert!(!res.hashes.is_empty());
}

#[tokio::test]
async fn test_find_tx_by_address() {
    let client = client_init();
    let res = client
        .find_transactions(None, Some(&[Address::from_str(TEST_ADDRESS_0)]), None, None)
        .await
        .unwrap();

    assert!(!res.hashes.is_empty());
}

#[tokio::test]
async fn test_find_tx_by_tag() {
    let client = client_init();
    let res = client
        .find_transactions(None, None, Some(&[Tag::from_str(TEST_TAG_0)]), None)
        .await
        .unwrap();

    assert!(!res.hashes.is_empty());
}

#[tokio::test]
async fn test_find_tx_by_approvee() {
    let client = client_init();
    let res = client
        .find_transactions(None, None, None, Some(&[Hash::from_str(TEST_BUNDLE_TX_1)]))
        .await
        .unwrap();

    assert!(!res.hashes.is_empty());
}

#[tokio::test]
async fn test_get_balances() {
    let client = client_init();
    let _ = client
        .get_balances(&[Address::from_str(TEST_ADDRESS_0)], None, None)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_get_inclusion_states() {
    let client = client_init();
    let _ = client
        .get_inclusion_states(&[tx()], None)
        .await
        .unwrap_err();
}

#[tokio::test]
async fn test_get_neighbors() {
    let client = client_init();

    match client.get_neighbors().await {
        Ok(res) => {
            assert!(res.neighbors.iter().all(|x| !x.address.is_empty()));
        }
        Err(e) => {
            let error = format!("{}", e);
            assert!(error.contains("COMMAND getNeighbors is not available on this node"));
        }
    }
}

#[tokio::test]
async fn testget_missing_transactions() {
    let _ = client_init().get_missing_transactions().await.unwrap();
}

#[tokio::test]
async fn test_get_node_api_configuration() {
    client_init().get_node_api_configuration().await.unwrap();
}

#[tokio::test]
async fn test_get_node_info() {
    client_init().get_node_info().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn test_get_tips() {
    let res = client_init().get_tips().await.unwrap();

    assert!(!res.hashes.is_empty());
}

#[tokio::test]
async fn test_get_transactions_to_approve() {
    let _ = client_init()
        .get_transactions_to_approve(3, None)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_get_trytes() {
    let res = client_init()
        .get_trytes(&[
            Hash::from_str(TEST_BUNDLE_TX_1),
            Hash::from_str(TEST_BUNDLE_TX_0),
        ])
        .await
        .unwrap();

    assert!(!res.trytes.is_empty());
}

#[tokio::test]
async fn test_remove_neighbors() {
    let res = client_init()
        .remove_neighbors(&["tcp://0.0.0.0:15600"])
        .await
        .unwrap();

    if let Some(neighbor) = res.removed_neighbors {
        assert_eq!(neighbor, 0);
    }
}

#[tokio::test]
async fn test_store_transactions() {
    client_init().store_transactions(&[tx()]).await.unwrap();
}

#[tokio::test]
async fn test_were_addresses_spent_from() {
    let res = client_init()
        .were_addresses_spent_from(&[Address::from_str(TEST_ADDRESS_0)])
        .await
        .unwrap();

    assert_eq!(res.states[0], true);
}

fn tx() -> Transaction {
    TransactionBuilder::new()
        .with_payload(Payload::zeros())
        .with_address(Address::zeros())
        .with_value(Value(0))
        .with_obsolete_tag(Tag::zeros())
        .with_timestamp(Timestamp(0))
        .with_index(Index(0))
        .with_last_index(Index(0))
        .with_tag(Tag::zeros())
        .with_attachment_ts(Timestamp(0))
        .with_bundle(Hash::zeros())
        .with_trunk(Hash::zeros())
        .with_branch(Hash::zeros())
        .with_attachment_lbts(Timestamp(0))
        .with_attachment_ubts(Timestamp(0))
        .with_nonce(Nonce::zeros())
        .build()
        .unwrap()
}

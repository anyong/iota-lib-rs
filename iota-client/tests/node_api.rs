// These are E2E test samples, so they are ignored by default.

use bee_message::prelude::*;
use bee_signing_ext::binary::Ed25519PrivateKey;
use iota_client::{hex_to_address, hex_to_message_id, hex_to_transaction_id};

#[ignore]
#[tokio::test]
async fn test_get_info() {
    iota_client::Client::get_info("http://0.0.0.0:14265")
        .await
        .unwrap();
}

#[ignore]
#[tokio::test]
async fn test_get_health() {
    iota_client::Client::get_health("http://0.0.0.0:14265")
        .await
        .unwrap();
}

#[ignore]
#[tokio::test]
async fn test_get_tips() {
    let r = iota_client::Client::new()
        .node("http://0.0.0.0:14265")
        .unwrap()
        .build()
        .unwrap()
        .get_tips()
        .await
        .unwrap();

    println!("{:#?}", r);
}

#[ignore]
#[tokio::test]
async fn test_post_message_with_indexation() {
    let index = Indexation::new(String::from("Hello"), Box::new([]));

    let client = iota_client::Client::new()
        .node("http://0.0.0.0:14265")
        .unwrap()
        .build()
        .unwrap();

    let tips = client.get_tips().await.unwrap();

    let message = Message::builder()
        .parent1(tips.0)
        .parent2(tips.1)
        .payload(Payload::Indexation(Box::new(index)))
        .build()
        .unwrap();

    let r = client.post_message(&message).await.unwrap();

    println!("{}", r);
}

#[ignore]
#[tokio::test]
async fn test_post_message_with_transaction() {
    let key = Ed25519PrivateKey::from_bytes(
        &hex::decode("256a818b2aac458941f7274985a410e57fb750f3a3a67969ece5bd9ae7eef5b2").unwrap(),
    )
    .unwrap();
    let pubkey = key.generate_public_key();
    println!("{:?}", hex::encode(pubkey.to_bytes()));
    // let index = Indexation::new(String::from("Hello"), Box::new([]));

    // let client = iota_client::Client::new()
    //     .node("http://0.0.0.0:14265")
    //     .unwrap()
    //     .build()
    //     .unwrap();

    // let tips = client.get_tips().await.unwrap();

    // let message = Message::builder()
    //     .parent1(tips.0)
    //     .parent2(tips.1)
    //     .payload(Payload::Indexation(Box::new(index)))
    //     .build()
    //     .unwrap();

    // let r = client.post_message(&message).await.unwrap();

    // println!("{}", r);
}

#[ignore]
#[tokio::test]
async fn test_get_message_by_index() {
    let r = iota_client::Client::new()
        .node("http://0.0.0.0:14265")
        .unwrap()
        .build()
        .unwrap()
        .get_message()
        .index("HORNET Spammer")
        .await
        .unwrap();

    println!("{:#?}", r);
}

#[ignore]
#[tokio::test]
async fn test_get_message_data() {
    let r = iota_client::Client::new()
        .node("http://0.0.0.0:14265")
        .unwrap()
        .build()
        .unwrap()
        .get_message()
        .data(
            &hex_to_message_id("2d7ef1e96f034ae002c6fba062503a842ab9d622b38040f8362a857f4f99c3c9")
                .unwrap(),
        )
        .await
        .unwrap();

    println!("{:#?}", r);
}

#[ignore]
#[tokio::test]
async fn test_get_message_metadata() {
    iota_client::Client::new()
        .node("http://0.0.0.0:14265")
        .unwrap()
        .build()
        .unwrap()
        .get_message()
        .metadata(
            &hex_to_message_id("a008ce3354591950232c0dacdfcb17c4f6457c5bf407eff1befaab5fa7b3b7b3")
                .unwrap(),
        )
        .await
        .unwrap();
}

#[ignore]
#[tokio::test]
async fn test_get_message_raw() {
    iota_client::Client::new()
        .node("http://0.0.0.0:14265")
        .unwrap()
        .build()
        .unwrap()
        .get_message()
        .raw(
            &hex_to_message_id("a008ce3354591950232c0dacdfcb17c4f6457c5bf407eff1befaab5fa7b3b7b3")
                .unwrap(),
        )
        .await
        .unwrap();
}

#[ignore]
#[tokio::test]
async fn test_get_message_children() {
    iota_client::Client::new()
        .node("http://0.0.0.0:14265")
        .unwrap()
        .build()
        .unwrap()
        .get_message()
        .children(
            &hex_to_message_id("a008ce3354591950232c0dacdfcb17c4f6457c5bf407eff1befaab5fa7b3b7b3")
                .unwrap(),
        )
        .await
        .unwrap();
}

#[ignore]
#[tokio::test]
async fn test_get_address_balance() {
    let r = iota_client::Client::new()
        .node("http://0.0.0.0:14265")
        .unwrap()
        .build()
        .unwrap()
        .get_address()
        .balance(
            &hex_to_address("6920b176f613ec7be59e68fc68f597eb3393af80f74c7c3db78198147d5f1f92")
                .unwrap(),
        )
        .await
        .unwrap();

    println!("{:#?}", r);
}

#[ignore]
#[tokio::test]
async fn test_get_address_outputs() {
    let r = iota_client::Client::new()
        .node("http://0.0.0.0:14265")
        .unwrap()
        .build()
        .unwrap()
        .get_address()
        .outputs(
            &hex_to_address("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
        )
        .await
        .unwrap();

    println!("{:#?}", r);
}

#[ignore]
#[tokio::test]
async fn test_get_output() {
    let r = iota_client::Client::new()
        .node("http://0.0.0.0:14265")
        .unwrap()
        .build()
        .unwrap()
        .get_output(
            &UTXOInput::new(
                hex_to_transaction_id(
                    "0000000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap(),
                0,
            )
            .unwrap(),
        )
        .await
        .unwrap();

    println!("{:#?}", r);
}

#[ignore]
#[tokio::test]
async fn test_get_milestone() {
    let r = iota_client::Client::new()
        .node("http://0.0.0.0:14265")
        .unwrap()
        .build()
        .unwrap()
        .get_milestone(2)
        .await
        .unwrap();

    println!("{:#?}", r);
}

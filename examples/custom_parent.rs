// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example custom_parent --release

use std::str::FromStr;

use iota_client::{bee_message::MessageId, Client};

/// In this example we will define a custom message parent which be used for promoting

#[tokio::main]
async fn main() {
    // Create a client instance
    let client = Client::builder()
        .with_node("http://localhost:14265") // Insert your node URL here
        .unwrap()
        .finish()
        .await
        .unwrap();

    let custom_parent =
        MessageId::from_str("b5634e05a7c665d7f87330a53633f001a5d1d96b346dc98dc225c4d6c204f23b").unwrap();

    let message = client
        .message()
        .with_parents(vec![custom_parent])
        .unwrap()
        .finish()
        .await
        .unwrap();

    println!(
        "Empty message sent: https://explorer.iota.org/devnet/message/{}",
        message.id()
    );
}

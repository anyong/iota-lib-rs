// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 01_get_info --release

use iota_client::Client;

/// In this example we will get information about the node

#[tokio::main]
async fn main() {
    // Create a client instance
    let client = Client::builder()
        .with_node("http://localhost:14265") // Insert your node URL here
        // Node with optional authentication
        // .with_node_auth(
        //     "https://somechrysalisiotanode.com",
        //     Some("Some JWT"),
        //     Some(("name", "password")),
        // )
        .unwrap()
        .with_node_sync_disabled()
        .finish()
        .await
        .unwrap();

    let info = client.get_info().await.unwrap();
    println!("Node Info: {:?}", info);
}

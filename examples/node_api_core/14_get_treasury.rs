// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! `cargo run --example node_api_core_get_treasury --release -- [NODE URL]`.

use iota_client::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Takes the node URL from command line argument or use localhost as default.
    let node = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "http://localhost:14265".to_string());
    // Creates a client instance with that node.
    let client = Client::builder()
        .with_node(&node)?
        .with_node_sync_disabled()
        .finish()
        .await?;

    // Sends the request.
    let treasury = client.get_treasury().await?;

    // Prints the response.
    println!("{:#?}", treasury);

    Ok(())
}

// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! `cargo run --example node_api_core_get_milestone_by_id --release -- [NODE URL]`.

use std::str::FromStr;

use iota_client::{bee_message::payload::milestone::MilestoneId, Client, Result};

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

    // Fetches the latest milestone ID from the node.
    let info = client.get_info().await?;
    let milestone_id = MilestoneId::from_str(&info.nodeinfo.status.latest_milestone.milestone_id)?;
    // Sends the request.
    let milestone = client.get_milestone_by_id(&milestone_id).await?;

    // Prints the response.
    println!("{:?}", milestone);

    Ok(())
}

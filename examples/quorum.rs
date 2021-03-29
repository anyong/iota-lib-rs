// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example quorum --release

use iota::{client::Result, Client, Seed};
extern crate dotenv;
use dotenv::dotenv;
use std::env;

/// In this example we will get the account balance of a known seed with quorum, which will compare the responses from
/// the nodes

#[tokio::main]
async fn main() -> Result<()> {
    let iota = Client::builder()
        .with_node("https://api.hornet-0.testnet.chrysalis2.com")?
        .with_node("https://api.hornet-1.testnet.chrysalis2.com")?
        .with_node("https://api.hornet-2.testnet.chrysalis2.com")?
        .with_quorum(true)
        .with_quorum_size(3)
        .with_quorum_threshold(66)
        .finish()
        .await?;

    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    let seed = Seed::from_bytes(&hex::decode(env::var("NONSECURE_USE_OF_DEVELOPMENT_SEED_1").unwrap()).unwrap());

    let seed_balance = iota.get_balance(&seed).finish().await.unwrap();
    println!("Account balance: {:?}i\n", seed_balance);

    Ok(())
}

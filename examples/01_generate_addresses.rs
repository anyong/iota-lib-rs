// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 01_generate_addresses --release

use std::env;

use dotenv::dotenv;
use iota_client::{
    api::GetAddressesBuilder,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    Client, Result,
};

/// In this example we will create addresses from a mnemonic defined in .env

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    let node_url = env::var("NODE_URL").unwrap();

    // Create a client instance
    let client = Client::builder()
        .with_node(&node_url)? // Insert your node URL here
        .with_node_sync_disabled()
        .finish()
        .await?;

    let secret_manager = SecretManager::Mnemonic(MnemonicSecretManager::try_from_mnemonic(
        &env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1").unwrap(),
    )?);

    // Generate addresses with default account index and range
    let addresses = client.get_addresses(&secret_manager).finish().await.unwrap();
    println!("List of generated public addresses:\n{:?}\n", addresses);

    // Generate addresses with custom account index and range
    let addresses = client
        .get_addresses(&secret_manager)
        .with_account_index(0)
        .with_range(0..4)
        .finish()
        .await?;

    println!("List of generated public addresses:\n{:?}\n", addresses);

    // Generating addresses with `client.get_addresses(&secret_manager)`, will by default get the bech32_hrp (Bech32
    // human readable part) from the nodeinfo, generating it "offline" requires setting it with
    // `with_bech32_hrp(bech32_hrp)`
    let addresses = GetAddressesBuilder::new(&secret_manager)
        .with_bech32_hrp(client.get_bech32_hrp().await?)
        .with_account_index(0)
        .with_range(0..4)
        .finish()
        .await?;

    println!("List of offline generated public addresses:\n{:?}\n", addresses);
    Ok(())
}

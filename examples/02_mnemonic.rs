// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 02_mnemonic --release

use iota_client::{
    api::GetAddressesBuilder,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    Client, Result,
};

/// In this example we will generate a mnemonic, convert it to a seed and generate the first address

#[tokio::main]
async fn main() -> Result<()> {
    let mnemonic = Client::generate_mnemonic()?;
    println!("Mnemonic: {}", mnemonic);

    let secret_manager = SecretManager::Mnemonic(MnemonicSecretManager::try_from_mnemonic(&mnemonic)?);

    // Generate addresses with custom account index and range
    let addresses = GetAddressesBuilder::new(&secret_manager)
        .with_account_index(0)
        .with_range(0..1)
        .finish()
        .await?;

    println!("First public address: {}", addresses[0]);
    Ok(())
}

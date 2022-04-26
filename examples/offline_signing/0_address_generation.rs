// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! In this example we generate addresses which will be used later to find inputs.
//! This example uses dotenv, which is not safe for use in production.
//! `cargo run --example 0_address_generation --release`.

use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use dotenv::dotenv;
use iota_client::{signing::mnemonic::MnemonicSigner, Client, Result};

const ADDRESS_FILE_NAME: &str = "examples/offline_signing/addresses.json";

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Creates a client instance.
    let offline_client = Client::builder().with_offline_mode().finish().await?;
    let signer = MnemonicSigner::new(&env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1").unwrap())?;

    // Generates addresses offline.
    let addresses = offline_client
        .get_addresses(&signer)
        .with_range(0..10)
        .with_bech32_hrp("rms")
        .finish()
        .await?;

    write_addresses_to_file(ADDRESS_FILE_NAME, addresses)
}

fn write_addresses_to_file<P: AsRef<Path>>(path: P, addresses: Vec<String>) -> Result<()> {
    let json = serde_json::to_string_pretty(&addresses)?;
    let mut file = BufWriter::new(File::create(path)?);

    println!("{}", json);

    file.write_all(json.as_bytes())?;

    Ok(())
}

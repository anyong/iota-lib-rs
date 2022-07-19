// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example tagged_data_to_utf8 --release

use iota_client::{block::payload::TaggedDataPayload, Client, Result};

/// In this example we will UTF-8 encode the tag and the data of an `TaggedDataPayload`.

#[tokio::main]
async fn main() -> Result<()> {
    // `hello` in hexadecimal.
    let tag = hex::decode("68656c6c6f")?;
    // `world` in hexadecimal.
    let data = hex::decode("776f726c64")?;

    let (tag_utf8, data_utf8) = Client::tagged_data_to_utf8(&TaggedDataPayload::new(tag, data)?)?;

    println!("tag: {}\ndata: {}", tag_utf8, data_utf8);

    Ok(())
}

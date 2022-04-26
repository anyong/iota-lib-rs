// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example 05_get_address_balance --release

use std::env;

use dotenv::dotenv;
use iota_client::{
    bee_message::output::Output,
    node_api::indexer::query_parameters::QueryParameter,
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    Client, Result,
};

/// In this example we will get the outputs of an address that have no additional unlock conditions and sum the amounts
/// and native tokens

#[tokio::main]
async fn main() -> Result<()> {
    // Create a client instance
    let client = Client::builder()
        .with_node("http://localhost:14265")? // Insert your node URL here
        .with_node_sync_disabled()
        .finish()
        .await?;

    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();
    let secret_manager =
        MnemonicSecretManager::try_from_mnemonic(&env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1").unwrap())?;

    // Generate the first address
    let addresses = client
        .get_addresses(&SecretManager::Mnemonic(secret_manager))
        .with_account_index(0)
        .with_range(0..1)
        .finish()
        .await?;

    // Get output ids of outputs that can be controlled by this address without further unlock constraints
    let output_ids = client
        .output_ids(vec![
            QueryParameter::Address(addresses[0].clone()),
            QueryParameter::HasExpirationCondition(false),
            QueryParameter::HasTimelockCondition(false),
            QueryParameter::HasStorageDepositReturnCondition(false),
        ])
        .await?;

    // Get the outputs by their id
    let outputs_responses = client.get_outputs(output_ids).await?;

    // Calculate the total amount and native tokens
    let mut total_amount = 0;
    let mut total_native_tokens = NativeTokensBuilder::new();
    for output_response in outputs_responses.into_iter() {
        let output = Output::try_from(&output_response.output)?;

        if let Some(native_tokens) = output.native_tokens() {
            total_native_tokens.add_native_tokens(native_tokens.clone())?;
        }
        total_amount += output.amount();
    }

    println!(
        "Outputs controlled by {} have: {:?}i and native tokens: {:?}",
        addresses[0],
        total_amount,
        total_native_tokens.finish()?
    );
    Ok(())
}

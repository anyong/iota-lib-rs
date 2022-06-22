// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example microtransaction --release

use std::{
    env,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use dotenv::dotenv;
use iota_client::{
    bee_block::output::{
        unlock_condition::{
            AddressUnlockCondition, ExpirationUnlockCondition, StorageDepositReturnUnlockCondition, UnlockCondition,
        },
        BasicOutputBuilder,
    },
    secret::{mnemonic::MnemonicSecretManager, SecretManager},
    Client, Result,
};

/// In this example we will do a microtransaction using unlock conditions to an output.
///
/// Due to the required storage deposit, it is not possible to send a small amount of tokens.
/// However, it is possible to send a large amount and ask a slightly smaller amount in return to
/// effectively transfer a small amount.

const NODE_URL: &'static str = "http://localhost:14265";

#[tokio::main]
async fn main() -> Result<()> {
    // Create a client instance.
    let client = Client::builder()
        .with_node(NODE_URL)?
        .with_node_sync_disabled()
        .finish()
        .await?;

    // This example uses dotenv, which is not safe for use in production!
    // Configure your own mnemonic in the ".env" file. Since the output amount cannot be zero, the seed must contain
    // non-zero balance.
    dotenv().ok();
    let secret_manager = SecretManager::Mnemonic(MnemonicSecretManager::try_from_mnemonic(
        &env::var("NON_SECURE_USE_OF_DEVELOPMENT_MNEMONIC_1").unwrap(),
    )?);

    let addresses = client.get_addresses(&secret_manager).with_range(0..1).get_raw().await?[0];

    let tomorrow = (SystemTime::now() + Duration::from_secs(24 * 3600))
        .duration_since(UNIX_EPOCH)
        .expect("clock went backwards")
        .as_secs()
        .try_into()
        .unwrap();
    let outputs = vec![
        // with storage deposit return
        BasicOutputBuilder::new_with_amount(255100)?
            .add_unlock_condition(UnlockCondition::Address(AddressUnlockCondition::new(address)))
            /// Return 100 less than the original amount.
            .add_unlock_condition(UnlockCondition::StorageDepositReturn(
                StorageDepositReturnUnlockCondition::new(address, 255000)?,
            ))
            /// If the receiver does not consume this output, we Unlock after a day to avoid
            /// locking our funds forever.
            .add_unlock_condition(UnlockCondition::Expiration(
                ExpirationUnlockCondition::new(address, tomorrow).unwrap(),
            ))
            .finish_output()?,
    ];

    let block = client
        .block()
        .with_secret_manager(&secret_manager)
        .with_outputs(outputs)?
        .finish()
        .await?;

    println!("Transaction sent: {NODE_URL}/api/core/v2/blocks/{}", block.id());
    println!("Block metadata: {NODE_URL}/api/core/v2/blocks/{}/metadata", block.id());
    let _ = client.retry_until_included(&block.id(), None, None).await?;
    Ok(())
}

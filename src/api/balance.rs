// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{signing::SignerHandle, Client, Result};

#[cfg(not(feature = "wasm"))]
use crate::constants::MAX_PARALLEL_API_REQUESTS;

/// Builder of get_balance API
pub struct GetBalanceBuilder<'a> {
    client: &'a Client,
    signer: &'a SignerHandle,
    account_index: u32,
    initial_address_index: u32,
    gap_limit: u32,
}

/// GetBalanceBuilder options
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBalanceBuilderOptions {
    /// Account index
    pub account_index: Option<u32>,
    /// Initial address index
    pub initial_address_index: Option<u32>,
    /// Gap limit
    pub gap_limit: Option<u32>,
}

impl<'a> GetBalanceBuilder<'a> {
    /// Create get_balance builder
    pub fn new(client: &'a Client, signer: &'a SignerHandle) -> Self {
        Self {
            client,
            signer,
            account_index: 0,
            initial_address_index: 0,
            gap_limit: super::ADDRESS_GAP_RANGE,
        }
    }

    /// Sets the account index.
    pub fn with_account_index(mut self, account_index: u32) -> Self {
        self.account_index = account_index;
        self
    }

    /// Sets the index of the address to start looking for balance.
    pub fn with_initial_address_index(mut self, initial_address_index: u32) -> Self {
        self.initial_address_index = initial_address_index;
        self
    }

    /// Sets the gap limit to specify how many addresses will be checked each round.
    /// If gap_limit amount of addresses in a row have no balance the function will return.
    pub fn with_gap_limit(mut self, gap_limit: u32) -> Self {
        self.gap_limit = gap_limit;
        self
    }

    /// Set multiple options from builder options type
    /// Useful for bindings
    pub fn set_options(mut self, options: GetBalanceBuilderOptions) -> Self {
        if let Some(account_index) = options.account_index {
            self = self.with_account_index(account_index);
        }

        if let Some(initial_address_index) = options.initial_address_index {
            self = self.with_initial_address_index(initial_address_index);
        }

        if let Some(gap_limit) = options.gap_limit {
            self = self.with_gap_limit(gap_limit);
        }

        self
    }

    /// Consume the builder and get the API result
    pub async fn finish(self) -> Result<u64> {
        let mut index = self.initial_address_index;

        // get account balance and check with value
        let mut balance = 0;
        // Count addresses with zero balances in a row
        let mut found_zero_balance = 0;
        loop {
            let addresses = self
                .client
                .get_addresses(self.signer)
                .with_account_index(self.account_index)
                .with_range(index..index + self.gap_limit)
                .get_all()
                .await?;

            #[cfg(feature = "wasm")]
            for address in addresses.public.iter().chain(addresses.internal.iter()) {
                let address_balance = self.client.get_address().balance(address).await?;
                match address_balance.balance {
                    0 => found_zero_balance += 1,
                    _ => {
                        balance += address_balance.balance;
                        // reset
                        found_zero_balance = 0;
                    }
                }
            }
            #[cfg(not(feature = "wasm"))]
            for addresses_chunk in addresses
                .public
                .into_iter()
                .chain(addresses.internal.into_iter())
                .collect::<Vec<String>>()
                .chunks(MAX_PARALLEL_API_REQUESTS)
                .map(|x: &[String]| x.to_vec())
            {
                let mut tasks = Vec::new();
                for address in addresses_chunk {
                    let client_ = self.client.clone();

                    tasks.push(async move {
                        tokio::spawn(async move {
                            let address_balance = client_.get_address().balance(&address).await?;
                            crate::Result::Ok(address_balance)
                        })
                        .await
                    });
                }
                for res in futures::future::try_join_all(tasks).await? {
                    let address_balance = res?;
                    match address_balance.balance {
                        0 => found_zero_balance += 1,
                        _ => {
                            balance += address_balance.balance;
                            // reset
                            found_zero_balance = 0;
                        }
                    }
                }
            }
            // The gap limit is 20 and use reference 40 here because there's public and internal addresses
            if found_zero_balance >= self.gap_limit * 2 {
                break;
            }
            index += self.gap_limit;
        }

        Ok(balance)
    }
}

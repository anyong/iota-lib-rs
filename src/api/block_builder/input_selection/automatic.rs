// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Automatic input selection for transactions

use bee_api_types::responses::OutputResponse;
use bee_block::{
    address::Address,
    output::{feature::Features, Output, RentStructure},
};
use crypto::keys::slip10::Chain;

use crate::{
    api::{
        address::search_address,
        block_builder::input_selection::{output_data::get_utxo_chains_inputs, types::SelectedTransactionData},
        input_selection::try_select_inputs,
        ClientBlockBuilder, ADDRESS_GAP_RANGE,
    },
    constants::HD_WALLET_TYPE,
    node_api::indexer::query_parameters::QueryParameter,
    secret::types::{InputSigningData, OutputMetadata},
    Error, Result,
};

async fn address_outputs(block_builder: &ClientBlockBuilder<'_>, address: String) -> Result<Vec<OutputResponse>> {
    let mut output_ids = Vec::new();

    // First request to get all basic outputs that can directly be unlocked by the address.
    output_ids.extend(
        block_builder
            .client
            .basic_output_ids(vec![
                QueryParameter::Address(address.clone()),
                QueryParameter::HasStorageReturnCondition(false),
            ])
            .await?,
    );

    // Second request to get all basic outputs that can be unlocked by the address through the expiration condition.
    output_ids.extend(
        block_builder
            .client
            .basic_output_ids(vec![
                QueryParameter::ExpirationReturnAddress(address),
                QueryParameter::HasExpirationCondition(true),
                QueryParameter::HasStorageReturnCondition(false),
            ])
            .await?,
    );

    block_builder.client.get_outputs(output_ids).await
}

fn is_output_address_unlockable(output: &Output, address: &Address, local_time: u32) -> bool {
    if let Some(unlock_conditions) = output.unlock_conditions() {
        if unlock_conditions.is_time_locked(local_time) {
            return false;
        }

        if let Some(expiration) = unlock_conditions.expiration() {
            if let Some(expiration_address) = expiration.return_address_expired(local_time) {
                return expiration_address == address;
            }
        }

        // PANIC: safe to unwrap as basic outputs always have an address.
        unlock_conditions.address().unwrap().address() == address
    } else {
        // Should not happen anyway as there should always be at least the address unlock condition.
        false
    }
}

impl<'a> ClientBlockBuilder<'a> {
    /// Searches inputs for provided outputs, by requesting the outputs from the account addresses or for
    /// alias/foundry/nft outputs get the latest state with their alias/nft id. Forwards to [try_select_inputs()]
    pub(crate) async fn get_inputs(&self, rent_structure: &RentStructure) -> Result<SelectedTransactionData> {
        log::debug!("[get_inputs]");
        let account_index = self.account_index;
        let mut gap_index = self.initial_address_index;
        let mut empty_address_count: u64 = 0;
        let mut cached_error = None;

        // First get inputs for utxo chains (alias, foundry, nft outputs)
        let mut available_inputs = get_utxo_chains_inputs(self, &self.outputs).await?;
        let (force_use_all_inputs, required_ed25519_inputs) = get_inputs_for_sender_and_issuer(self).await?;
        available_inputs.extend(required_ed25519_inputs.into_iter());

        let local_time = self.client.get_time_checked().await?;

        // Try to select inputs with required inputs for utxo chains alone before requesting more inputs from addresses
        if let Ok(selected_transaction_data) = try_select_inputs(
            available_inputs.clone(),
            self.outputs.clone(),
            force_use_all_inputs,
            self.custom_remainder_address,
            rent_structure,
            // Don't allow burning of native tokens during automatic input selection, because otherwise it
            // could lead to burned native tokens by accident
            false,
            local_time,
        ) {
            return Ok(selected_transaction_data);
        };

        log::debug!("[get_inputs from addresses]");
        // then select inputs with outputs from addresses
        let selected_transaction_data = 'input_selection: loop {
            // Get the addresses in the BIP path/index ~ path/index+20
            let addresses = self
                .client
                .get_addresses(
                    self.secret_manager
                        .ok_or(crate::Error::MissingParameter("secret manager"))?,
                )
                .with_account_index(account_index)
                .with_range(gap_index..gap_index + ADDRESS_GAP_RANGE)
                .get_all()
                .await?;
            // Have public and internal addresses with the index ascending ordered
            let mut public_and_internal_addresses = Vec::new();
            for index in 0..addresses.public.len() {
                public_and_internal_addresses.push((addresses.public[index].clone(), false));
                public_and_internal_addresses.push((addresses.internal[index].clone(), true));
            }

            // For each address, get the address outputs
            let mut address_index = gap_index;
            for (index, (str_address, internal)) in public_and_internal_addresses.iter().enumerate() {
                let address_outputs = address_outputs(self, str_address.to_string()).await?;

                // If there are more than 20 (ADDRESS_GAP_RANGE) consecutive empty addresses, then we stop
                // looking up the addresses belonging to the seed. Note that we don't
                // really count the exact 20 consecutive empty addresses, which is
                // unnecessary. We just need to check the address range,
                // (index * ADDRESS_GAP_RANGE, index * ADDRESS_GAP_RANGE + ADDRESS_GAP_RANGE), where index is
                // natural number, and to see if the outputs are all empty.
                if address_outputs.is_empty() {
                    // Accumulate the empty_address_count for each run of output address searching
                    empty_address_count += 1;
                } else {
                    // Reset counter if there is an output
                    empty_address_count = 0;

                    for output_response in address_outputs {
                        let output = Output::try_from(&output_response.output)?;
                        let address = Address::try_from_bech32(str_address)?.1;

                        if is_output_address_unlockable(&output, &address, local_time) {
                            available_inputs.push(InputSigningData {
                                output,
                                output_metadata: OutputMetadata::try_from(&output_response.metadata)?,
                                chain: Some(Chain::from_u32_hardened(vec![
                                    HD_WALLET_TYPE,
                                    self.coin_type,
                                    account_index,
                                    *internal as u32,
                                    address_index,
                                ])),
                                bech32_address: str_address.clone(),
                            });
                        }
                    }
                    let selected_transaction_data = match try_select_inputs(
                        available_inputs.clone(),
                        self.outputs.clone(),
                        force_use_all_inputs,
                        self.custom_remainder_address,
                        rent_structure,
                        // Don't allow burning of native tokens during automatic input selection, because otherwise it
                        // could lead to burned native tokens by accident
                        false,
                        local_time,
                    ) {
                        Ok(r) => r,
                        // for these errors ,just try again in the next round with more addresses which might have more
                        // outputs
                        Err(err @ crate::Error::NotEnoughBalance { .. }) => {
                            cached_error.replace(err);
                            continue;
                        }
                        Err(err @ crate::Error::NotEnoughNativeTokens { .. }) => {
                            cached_error.replace(err);
                            continue;
                        }
                        // Native tokens left, but no balance for the storage deposit for a remainder
                        Err(err @ crate::Error::NoBalanceForNativeTokenRemainder) => {
                            cached_error.replace(err);
                            continue;
                        }
                        // Currently too many inputs, by scanning for more inputs, we might find some with more amount
                        Err(err @ crate::Error::ConsolidationRequired { .. }) => {
                            cached_error.replace(err);
                            continue;
                        }
                        // Not enough balance for a remainder
                        Err(crate::Error::BlockError(block_error)) => match block_error {
                            bee_block::Error::InvalidStorageDepositAmount { .. } => {
                                cached_error.replace(crate::Error::BlockError(block_error));
                                continue;
                            }
                            _ => return Err(block_error.into()),
                        },
                        Err(e) => return Err(e),
                    };

                    break 'input_selection selected_transaction_data;
                }

                // if we just processed an even index, increase the address index
                // (because the list has public and internal addresses)
                if index % 2 == 1 {
                    address_index += 1;
                }
            }
            gap_index += ADDRESS_GAP_RANGE;
            // The gap limit is 20 and use reference 40 here because there's public and internal addresses
            if empty_address_count >= (ADDRESS_GAP_RANGE * 2) as u64 {
                // returned last cached error
                return Err(cached_error.unwrap_or(Error::NoInputs));
            }
        };

        Ok(selected_transaction_data)
    }
}

async fn get_inputs_for_sender_and_issuer(
    block_builder: &ClientBlockBuilder<'_>,
) -> Result<(bool, Vec<InputSigningData>)> {
    log::debug!("[get_inputs_for_sender_and_issuer]");
    let mut force_use_all_inputs = false;
    let mut required_ed25519_inputs = Vec::new();
    let bech32_hrp = block_builder.client.get_bech32_hrp().await?;

    // get Ed25519 address if there is a Sender or Issuer block, because we then need to unlock an output with this
    // address
    let mut required_ed25519_addresses = Vec::new();
    for output in &block_builder.outputs {
        if let Some(sender_feature) = output.features().and_then(Features::sender) {
            required_ed25519_addresses.push(sender_feature.address());
        }
        if let Some(issuer_feature) = output.immutable_features().and_then(Features::issuer) {
            required_ed25519_addresses.push(issuer_feature.address());
        }
    }

    required_ed25519_addresses.dedup();
    for address in required_ed25519_addresses {
        if let Address::Ed25519(address) = address {
            let (address_index, internal) = search_address(
                block_builder
                    .secret_manager
                    .ok_or(Error::MissingParameter("secret manager"))?,
                &bech32_hrp,
                block_builder.coin_type,
                block_builder.account_index,
                block_builder.input_range.clone(),
                &Address::Ed25519(*address),
            )
            .await?;
            // if we didn't return with an error, then the address was found

            let address = Address::Ed25519(*address);
            let address_outputs = address_outputs(block_builder, address.to_bech32(&bech32_hrp)).await?;
            let local_time = block_builder.client.get_time_checked().await?;

            let mut found_output = false;
            for output_response in address_outputs {
                let output = Output::try_from(&output_response.output)?;

                if is_output_address_unlockable(&output, &address, local_time) {
                    required_ed25519_inputs.push(InputSigningData {
                        output: Output::try_from(&output_response.output)?,
                        output_metadata: OutputMetadata::try_from(&output_response.metadata)?,
                        chain: Some(Chain::from_u32_hardened(vec![
                            HD_WALLET_TYPE,
                            block_builder.coin_type,
                            block_builder.account_index,
                            internal as u32,
                            address_index,
                        ])),
                        bech32_address: address.to_bech32(&bech32_hrp),
                    });
                    // we want to include all outputs, because another output might be better balance wise,
                    // but will not unlock the address we need
                    force_use_all_inputs = true;
                    found_output = true;
                    break;
                }
            }

            if !found_output {
                return Err(Error::MissingInputWithEd25519Address);
            }
        }
    }

    Ok((force_use_all_inputs, required_ed25519_inputs))
}

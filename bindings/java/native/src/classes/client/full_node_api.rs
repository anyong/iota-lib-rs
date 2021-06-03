// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};

use std::{
    convert::{From, Into},
    str::FromStr,
};

use iota_client::{
    bee_message::{input::UtxoInput as RustUtxoInput, payload::transaction::TransactionId, MessageId},
    client::Client as ClientRust,
};

use crate::{
    address::*,
    balance::GetBalanceBuilderApi,
    bee_types::*,
    client_builder::ClientBuilder,
    message::{ClientMessageBuilder, GetMessageBuilder, Message, MessageWrap},
    mqtt::MqttManager,
};

impl From<ClientRust> for Client {
    fn from(client: ClientRust) -> Self {
        Self(client)
    }
}

pub struct Client(ClientRust);

/// Full node API
impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn borrow<'a>(&'a self) -> &'a ClientRust {
        &self.0
    }

    pub fn borrow_mut<'a>(&'a mut self) -> &'a mut ClientRust {
        &mut self.0
    }

    pub fn get_health(&self) -> Result<bool> {
        Ok(crate::block_on(async { self.0.get_health().await })?)
    }

    pub fn get_node_health(&self, node: &str) -> Result<bool> {
        Ok(crate::block_on(async {
            iota_client::Client::get_node_health(node).await
        })?)
    }

    pub fn get_info(&self) -> Result<NodeInfoWrapper> {
        Ok(crate::block_on(async { self.0.get_info().await })?.into())
    }

    pub fn get_tips(&self) -> Result<Vec<String>> {
        let tips = crate::block_on(async { self.0.get_tips().await })?;
        Ok(tips.into_iter().map(|p| p.to_string()).collect())
    }

    pub fn get_peers(&self) -> Result<Vec<PeerDto>> {
        Ok(crate::block_on(async { self.0.get_peers().await })?
            .into_iter()
            .map(PeerDto::from)
            .collect())
    }

    pub fn post_message(&self, msg: Message) -> Result<MessageId> {
        let ret = crate::block_on(async { self.0.post_message(&msg.to_inner_clone()).await });

        match ret {
            Ok(s) => Ok(s),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub fn get_output(&self, output_id: String) -> Result<OutputResponse> {
        Ok(crate::block_on(async { self.0.get_output(&RustUtxoInput::from_str(&output_id)?).await })?.into())
    }

    /// GET /api/v1/addresses/{address} endpoint
    pub fn get_address(&self) -> GetAddressBuilder {
        GetAddressBuilder::new(self)
    }

    pub fn get_address_balance(&self, address: &str) -> Result<BalanceAddressResponse> {
        let mut result: BalanceAddressResponse =
            crate::block_on(async { self.0.get_address().balance(&String::from(address)).await })?.into();
        result.address = crate::block_on(async { self.0.hex_to_bech32(&result.address, None).await })?;
        Ok(result)
    }

    pub fn get_addresses_balances(&self, addresses: Vec<String>) -> Result<Vec<BalanceAddressResponse>> {
        let result: Vec<BalanceAddressResponse> =
            crate::block_on(async { self.0.get_address_balances(&addresses).await })?
                .into_iter()
                .map(|b| b.into())
                .collect();

        Ok(result)
    }

    pub fn find_outputs(
        &self,
        output_ids: Option<Vec<String>>,
        addresses: Option<Vec<String>>,
    ) -> Result<Vec<OutputResponse>> {
        let output_ids: Vec<RustUtxoInput> = output_ids
            .unwrap_or_default()
            .into_iter()
            .map(|input| RustUtxoInput::from_str(&input).unwrap_or_else(|_| panic!("invalid input: {}", input)))
            .collect();
        let output_metadata_vec = crate::block_on(async {
            self.0
                .find_outputs(&output_ids[..], &addresses.unwrap_or_default()[..])
                .await
        })?;
        Ok(output_metadata_vec
            .into_iter()
            .map(|metadata| metadata.into())
            .collect())
    }

    pub fn get_milestone(&self, index: u32) -> Result<MilestoneResponse> {
        Ok(crate::block_on(async { self.0.get_milestone(index).await })?.into())
    }

    pub fn get_milestone_utxo_changes(&self, index: u32) -> Result<MilestoneUtxoChangesResponse> {
        Ok(crate::block_on(async { self.0.get_milestone_utxo_changes(index).await })?.into())
    }

    pub fn get_receipts(&self) -> Result<Vec<ReceiptDto>> {
        let receipts: Vec<ReceiptDto> = crate::block_on(async { self.0.get_receipts().await })?
            .into_iter()
            .map(|r| r.into())
            .collect();
        Ok(receipts)
    }

    pub fn get_receipts_migrated_at(&self, index: u32) -> Result<Vec<ReceiptDto>> {
        let receipts: Vec<ReceiptDto> = crate::block_on(async { self.0.get_receipts_migrated_at(index).await })?
            .into_iter()
            .map(|r| r.into())
            .collect();
        Ok(receipts)
    }

    /// GET /api/v1/treasury endpoint
    /// Get the treasury output.
    pub fn get_treasury(&self) -> Result<TreasuryResponse> {
        let res = crate::block_on(async { self.0.get_treasury().await });
        match res {
            Ok(t) => Ok(t.into()),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub fn get_included_message(&self, transaction_id: TransactionId) -> Result<Message> {
        let res = crate::block_on(async { self.0.get_included_message(&transaction_id).await });
        match res {
            Ok(m) => Ok(m.into()),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    /// Reattaches messages for provided message id. Messages can be reattached only if they are valid and haven't been
    /// confirmed for a while.
    pub fn reattach(&self, message_id: MessageId) -> Result<MessageWrap> {
        let res = crate::block_on(async { self.0.reattach(&message_id).await });

        match res {
            Ok(w) => Ok(MessageWrap::new(w.0, w.1.into())),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    /// Reattach a message without checking if it should be reattached
    pub fn reattach_unchecked(&self, message_id: MessageId) -> Result<MessageWrap> {
        let res = crate::block_on(async { self.0.reattach_unchecked(&message_id).await });

        match res {
            Ok(w) => Ok(MessageWrap::new(w.0, w.1.into())),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    /// Promotes a message. The method should validate if a promotion is necessary through get_message. If not, the
    /// method should error out and should not allow unnecessary promotions.
    pub fn promote(&self, message_id: MessageId) -> Result<MessageWrap> {
        let res = crate::block_on(async { self.0.promote(&message_id).await });

        match res {
            Ok(w) => Ok(MessageWrap::new(w.0, w.1.into())),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    /// Promote a message without checking if it should be promoted
    pub fn promote_unchecked(&self, message_id: MessageId) -> Result<MessageWrap> {
        let res = crate::block_on(async { self.0.promote_unchecked(&message_id).await });

        match res {
            Ok(w) => Ok(MessageWrap::new(w.0, w.1.into())),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    // HIGH LEVEL API

    /// Return the balance for a provided seed and its wallet chain account index.
    /// Addresses with balance must be consecutive, so this method will return once it encounters a zero
    /// balance address.
    pub fn get_balance(&self, seed: &str) -> GetBalanceBuilderApi {
        GetBalanceBuilderApi::new(self, seed)
    }

    /// A generic send function for easily sending transaction or indexation messages.
    pub fn message(&self) -> ClientMessageBuilder {
        ClientMessageBuilder::new(self)
    }

    /// GET /api/v1/messages/{messageId} endpoint
    pub fn get_message(&self) -> GetMessageBuilder {
        GetMessageBuilder::new(self)
    }

    pub fn get_addresses(&self, seed: &str) -> GetAddressesBuilder {
        GetAddressesBuilder::new(seed).with_client(self)
    }

    pub fn retry_until_included(
        &self,
        message_id: MessageId,
        interval: usize,
        max_attempts: usize,
    ) -> Result<Vec<MessageWrap>> {
        let mut opt_int = None;
        if interval > 0 {
            opt_int = Some(interval as u64);
        }

        let mut opt_attempt = None;
        if max_attempts > 0 {
            opt_attempt = Some(max_attempts as u64);
        }

        let res = crate::block_on(async { self.0.retry_until_included(&message_id, opt_int, opt_attempt).await });

        match res {
            Ok(w) => {
                let mut output = vec![];
                for pair in w {
                    output.push(MessageWrap::new(pair.0, pair.1.into()))
                }

                Ok(output)
            }
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    // Mqtt

    pub fn subscriber(&mut self) -> MqttManager {
        MqttManager::new(self)
    }

    pub fn mqtt_event_receiver(&self) {}

    // UTIL BELOW

    /// Generates a new mnemonic.
    pub fn generate_mnemonic() -> Result<String> {
        let res = ClientRust::generate_mnemonic();

        match res {
            Ok(m) => Ok(m),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    /// Returns a hex encoded seed for a mnemonic.
    pub fn mnemonic_to_hex_seed(mnemonic: &str) -> Result<String> {
        let res = ClientRust::mnemonic_to_hex_seed(mnemonic);

        match res {
            Ok(m) => Ok(m),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub fn bech32_to_hex(bech32: &str) -> Result<String> {
        let res = iota_client::Client::bech32_to_hex(bech32);
        match res {
            Ok(s) => Ok(s),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    pub fn hex_to_bech32(&self, hex: &str, bech32_hrp: Option<&str>) -> Result<String> {
        let res = crate::block_on(async { self.0.hex_to_bech32(hex, bech32_hrp).await }).into();
        match res {
            Ok(s) => Ok(s),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    /// Returns a valid Address parsed from a String.
    pub fn parse_bech32_address(address: &str) -> Result<Address> {
        let res = iota_client::Client::parse_bech32_address(address);
        match res {
            Ok(s) => Ok(s.into()),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }

    /// Checks if a String address is valid.
    pub fn is_address_valid(address: &str) -> bool {
        iota_client::Client::is_address_valid(address)
    }
}

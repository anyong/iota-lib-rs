// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::ops::Range;

use bee_message::{
    output::{AliasId, FoundryId, NftId, OutputId},
    payload::{dto::PayloadDto, milestone::MilestoneId, transaction::TransactionId},
    MessageDto, MessageId,
};
use serde::Deserialize;

use crate::{
    api::{
        ClientMessageBuilderOptions as GenerateMessageOptions, GetAddressesBuilderOptions as GenerateAddressesOptions,
        PreparedTransactionDataDto,
    },
    node_api::indexer::query_parameters::QueryParameter,
    node_manager::node::NodeAuth,
    secret::SecretManagerDto,
};

/// Each public client method.
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "name", content = "data")]
pub enum ClientMethod {
    /// Generate a addresses.
    GenerateAddresses {
        /// Create secret manager from json
        #[serde(rename = "secretManager")]
        secret_manager: SecretManagerDto,
        /// Addresses generation options
        options: GenerateAddressesOptions,
    },
    /// Generate client message
    GenerateMessage {
        /// Secret manager
        #[serde(rename = "secretManager")]
        secret_manager: Option<SecretManagerDto>,
        /// Options
        options: Option<GenerateMessageOptions>,
    },
    /// Get a node candidate from the synced node pool.
    GetNode,
    // /// Gets the miner to use based on the PoW setting
    // GetPoWProvider,
    /// Gets the network related information such as network_id and min_pow_score
    GetNetworkInfo,
    /// Gets the network id of the node we're connecting to.
    GetNetworkId,
    /// Returns the bech32_hrp
    GetBech32Hrp,
    /// Returns the min pow score
    GetMinPoWScore,
    /// Returns the tips interval
    GetTipsInterval,
    /// Returns if local pow should be used or not
    GetLocalPoW,
    /// Get fallback to local proof of work timeout
    GetFallbackToLocalPoW,
    /// returns the unsynced nodes.
    #[cfg(not(target_family = "wasm"))]
    UnsyncedNodes,
    /// Prepare a transaction for signing
    PrepareTransaction {
        /// Secret manager
        #[serde(rename = "secretManager")]
        secret_manager: Option<SecretManagerDto>,
        /// Options
        options: Option<GenerateMessageOptions>,
    },
    /// Sign a transaction
    SignTransaction {
        /// Secret manager
        #[serde(rename = "secretManager")]
        secret_manager: SecretManagerDto,
        /// Prepared transaction data
        #[serde(rename = "preparedTransactionData")]
        prepared_transaction_data: PreparedTransactionDataDto,
    },
    /// Store a mnemonic in the Stronghold vault
    #[cfg(feature = "stronghold")]
    StoreMnemonic {
        /// Stronghold secret manager
        #[serde(rename = "secretManager")]
        secret_manager: SecretManagerDto,
        /// Mnemonic
        mnemonic: String,
    },
    /// Submit a payload in a message
    SubmitPayload {
        /// The payload to send
        #[serde(rename = "payload")]
        payload_dto: PayloadDto,
    },
    //////////////////////////////////////////////////////////////////////
    // Node core API
    //////////////////////////////////////////////////////////////////////
    /// Get health
    GetHealth {
        /// Url
        url: String,
    },
    /// Get node info
    GetNodeInfo {
        /// Url
        url: String,
        /// Node authentication
        auth: Option<NodeAuth>,
    },
    /// Returns the node information together with the url of the used node
    GetInfo,
    /// Get peers
    GetPeers,
    /// Get tips
    GetTips,
    /// Post message (JSON)
    PostMessage {
        /// Message
        message: MessageDto,
    },
    /// Post message (raw)
    PostMessageRaw {
        /// Message
        message: MessageDto,
    },
    /// Get message
    GetMessage {
        /// Message ID
        #[serde(rename = "messageId")]
        message_id: MessageId,
    },
    /// Get message metadata with message_id
    GetMessageMetadata {
        /// Message ID
        #[serde(rename = "messageId")]
        message_id: MessageId,
    },
    /// Get message raw
    GetMessageRaw {
        /// Message ID
        #[serde(rename = "messageId")]
        message_id: MessageId,
    },
    /// Get message children
    GetMessageChildren {
        /// Message ID
        #[serde(rename = "messageId")]
        message_id: MessageId,
    },
    /// Get output
    GetOutput {
        /// Output ID
        #[serde(rename = "outputId")]
        output_id: OutputId,
    },
    /// Get the milestone by the given milestone id.
    GetMilestoneById {
        /// Milestone ID
        #[serde(rename = "milestoneId")]
        milestone_id: MilestoneId,
    },
    /// Get the raw milestone by the given milestone id.
    GetMilestoneByIdRaw {
        /// Milestone ID
        #[serde(rename = "milestoneId")]
        milestone_id: MilestoneId,
    },
    /// Get the milestone by the given index.
    GetMilestoneByIndex {
        /// Milestone Index
        index: u32,
    },
    /// Get the raw milestone by the given index.
    GetMilestoneByIndexRaw {
        /// Milestone Index
        index: u32,
    },
    /// Get the UTXO changes by the given milestone id.
    GetUtxoChangesById {
        /// Milestone ID
        #[serde(rename = "milestoneId")]
        milestone_id: MilestoneId,
    },
    /// Get the UTXO changes by the given milestone index.
    GetUtxoChangesByIndex {
        /// Milestone Index
        index: u32,
    },
    /// Get all receipts.
    GetReceipts,
    /// Get the receipts by the given milestone index.
    GetReceiptsMigratedAt {
        /// Milestone index
        #[serde(rename = "milestoneIndex")]
        milestone_index: u32,
    },
    /// Get the treasury output.
    GetTreasury,
    /// Returns the included message of the transaction.
    GetIncludedMessage {
        /// Transaction ID
        #[serde(rename = "transactionId")]
        transaction_id: TransactionId,
    },

    //////////////////////////////////////////////////////////////////////
    // Node indexer API
    //////////////////////////////////////////////////////////////////////
    /// Fetch basic output IDs
    BasicOutputIds {
        /// Query parameters for output requests
        #[serde(rename = "queryParameters")]
        query_parameters: Vec<QueryParameter>,
    },
    /// Fetch alias output IDs
    AliasOutputIds {
        /// Query parameters for output requests
        #[serde(rename = "queryParameters")]
        query_parameters: Vec<QueryParameter>,
    },
    /// Fetch alias output ID
    AliasOutputId {
        /// Alias id
        #[serde(rename = "aliasId")]
        alias_id: AliasId,
    },
    /// Fetch NFT output IDs
    NftOutputIds {
        /// Query parameters for output requests
        #[serde(rename = "queryParameters")]
        query_parameters: Vec<QueryParameter>,
    },
    /// Fetch NFT output ID
    NftOutputId {
        /// NFT ID
        #[serde(rename = "nftId")]
        nft_id: NftId,
    },
    /// Fetch foundry Output IDs
    FoundryOutputIds {
        /// Query parameters for output requests
        #[serde(rename = "queryParameters")]
        query_parameters: Vec<QueryParameter>,
    },
    /// Fetch foundry Output ID
    FoundryOutputId {
        /// Foundry ID
        #[serde(rename = "foundryId")]
        foundry_id: FoundryId,
    },

    //////////////////////////////////////////////////////////////////////
    // High level API
    //////////////////////////////////////////////////////////////////////
    /// Fetch OutputResponse from provided OutputIds (requests are sent in parallel)
    GetOutputs {
        /// Output IDs
        #[serde(rename = "outputIds")]
        output_ids: Vec<OutputId>,
    },
    /// Try to get OutputResponse from provided OutputIds (requests are sent in parallel and errors are ignored, can be
    /// useful for spent outputs)
    TryGetOutputs {
        /// Output IDs
        #[serde(rename = "outputIds")]
        output_ids: Vec<OutputId>,
    },
    /// Find all messages by provided message IDs.
    FindMessages {
        /// MessageIDs
        #[serde(rename = "messageIds")]
        message_ids: Vec<MessageId>,
    },
    /// Retries (promotes or reattaches) a message for provided message id. Message should only be
    /// retried only if they are valid and haven't been confirmed for a while.
    Retry {
        /// Message ID
        #[serde(rename = "messageId")]
        message_id: MessageId,
    },
    /// Retries (promotes or reattaches) a message for provided message id until it's included (referenced by a
    /// milestone). Default interval is 5 seconds and max attempts is 40. Returns the included message at first
    /// position and additional reattached messages
    RetryUntilIncluded {
        /// Message ID
        #[serde(rename = "messageId")]
        message_id: MessageId,
        /// Interval
        interval: Option<u64>,
        /// Maximum attempts
        #[serde(rename = "maxAttempts")]
        max_attempts: Option<u64>,
    },
    /// Function to consolidate all funds from a range of addresses to the address with the lowest index in that range
    /// Returns the address to which the funds got consolidated, if any were available
    ConsolidateFunds {
        /// Secret manager
        #[serde(rename = "secretManager")]
        secret_manager: SecretManagerDto,
        /// Account index
        #[serde(rename = "accountIndex")]
        account_index: u32,
        /// Address_range
        #[serde(rename = "addressRange")]
        address_range: Range<u32>,
    },
    /// Function to find inputs from addresses for a provided amount (useful for offline signing)
    FindInputs {
        /// Addresses
        addresses: Vec<String>,
        /// Amount
        amount: u64,
    },
    /// Find all outputs based on the requests criteria. This method will try to query multiple nodes if
    /// the request amount exceeds individual node limit.
    FindOutputs {
        /// Output IDs
        #[serde(rename = "outputIds")]
        output_ids: Vec<OutputId>,
        /// Addresses
        addresses: Vec<String>,
    },
    /// Reattaches messages for provided message id. Messages can be reattached only if they are valid and haven't been
    /// confirmed for a while.
    Reattach {
        /// Message ID
        #[serde(rename = "messageId")]
        message_id: MessageId,
    },
    /// Reattach a message without checking if it should be reattached
    ReattachUnchecked {
        /// Message ID
        #[serde(rename = "messageId")]
        message_id: MessageId,
    },
    /// Promotes a message. The method should validate if a promotion is necessary through get_message. If not, the
    /// method should error out and should not allow unnecessary promotions.
    Promote {
        /// Message ID
        #[serde(rename = "messageId")]
        message_id: MessageId,
    },
    /// Promote a message without checking if it should be promoted
    PromoteUnchecked {
        /// Message ID
        #[serde(rename = "messageId")]
        message_id: MessageId,
    },

    //////////////////////////////////////////////////////////////////////
    // Utils
    //////////////////////////////////////////////////////////////////////
    /// Transforms bech32 to hex
    Bech32ToHex {
        /// Bech32 encoded address
        bech32: String,
    },
    /// Transforms a hex encoded address to a bech32 encoded address
    HexToBech32 {
        /// Hex encoded bech32 address
        hex: String,
        /// Human readable part
        #[serde(rename = "bech32Hrp")]
        bech32_hrp: Option<String>,
    },
    /// Transforms a hex encoded public key to a bech32 encoded address
    HexPublicKeyToBech32Address {
        /// Hex encoded public key
        hex: String,
        /// Human readable part
        #[serde(rename = "bech32Hrp")]
        bech32_hrp: Option<String>,
    },
    /// Returns a valid Address parsed from a String.
    ParseBech32Address {
        /// Address
        address: String,
    },
    /// Checks if a String is a valid bech32 encoded address.
    IsAddressValid {
        /// Address
        address: String,
    },
    /// Generates a new mnemonic.
    GenerateMnemonic,
    /// Returns a hex encoded seed for a mnemonic.
    MnemonicToHexSeed {
        /// Mnemonic
        mnemonic: String,
    },
    /// Returns a message ID (Blake2b256 hash of message bytes) from a message
    MessageId {
        /// Message
        message: MessageDto,
    },
}

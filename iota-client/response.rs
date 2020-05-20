//! Response types
use anyhow::Result;
use iota_bundle_preview::{Address, Hash, Tag, Transaction, TransactionField};
use iota_ternary_preview::TryteBuf;
use serde::ser::{Serialize, Serializer, SerializeSeq, SerializeStruct};

// TODO: remove this struct once iota_bundle_preview::Transaction implements Serialize
/// a Transaction wrapper that can be serialized
#[derive(Serialize)]
pub struct TransactionDef {
    payload: String,
    address: String,
    value: String,
    obsolete_tag: String,
    timestamp: String,
    index: String,
    last_index: String,
    bundle: Vec<i8>,
    trunk: Vec<i8>,
    branch: Vec<i8>,
    tag: String,
    attachment_ts: String,
    attachment_lbts: String,
    attachment_ubts: String,
    nonce: String,
}

impl From<&Transaction> for TransactionDef {
    fn from(transaction: &Transaction) -> Self {
        TransactionDef {
            payload: serde_json::to_string(transaction.payload()).unwrap(),
            address: serde_json::to_string(transaction.address()).unwrap(),
            value: serde_json::to_string(transaction.value()).unwrap(),
            obsolete_tag: serde_json::to_string(transaction.obsolete_tag()).unwrap(),
            timestamp: serde_json::to_string(transaction.timestamp()).unwrap(),
            index: serde_json::to_string(transaction.index()).unwrap(),
            last_index: serde_json::to_string(transaction.last_index()).unwrap(),
            bundle: transaction.bundle().as_bytes().to_vec(),
            trunk: transaction.trunk().as_bytes().to_vec(),
            branch: transaction.branch().as_bytes().to_vec(),
            tag: serde_json::to_string(transaction.tag()).unwrap(),
            attachment_ts: serde_json::to_string(transaction.attachment_ts()).unwrap(),
            attachment_lbts: serde_json::to_string(transaction.attachment_lbts()).unwrap(),
            attachment_ubts: serde_json::to_string(transaction.attachment_ubts()).unwrap(),
            nonce: serde_json::to_string(transaction.nonce()).unwrap(),
        }
    }
}

fn transaction_serializer<S>(x: &Vec<Transaction>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(x.len()))?;
    for e in x {
        seq.serialize_element(&TransactionDef::from(e))?;
    }
    seq.end()
}

/// addNeighbors Response Type
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AddNeighborsResponse {
    #[serde(rename = "addedNeighbors")]
    /// Total number of added neighbors
    pub added_neighbors: Option<usize>,
}

/// checkConsistency Response Type
#[derive(Clone, Debug, Serialize)]
pub struct ConsistencyResponse {
    /// State of the given transactions in the `tails` parameter. A `true` value means
    /// that all given transactions are consistent. A `false` value means that one
    /// or more of the given transactions are inconsistent.
    pub state: bool,
    /// If the `state` field is false, this field contains information about why the transaction is inconsistent.
    pub info: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ConsistencyResponseBuilder {
    state: Option<bool>,
    info: Option<String>,
    exception: Option<String>,
    error: Option<String>,
}

impl ConsistencyResponseBuilder {
    pub(crate) async fn build(self) -> Result<ConsistencyResponse> {
        let mut state = false;
        if let Some(exception) = self.exception {
            return Err(anyhow!("{}", exception));
        } else if let Some(error) = self.error {
            return Err(anyhow!("{}", error));
        } else if let Some(s) = self.state {
            state = s;
        }

        Ok(ConsistencyResponse {
            state: state,
            info: self.info,
        })
    }
}

/// attachToTangle Response Type
#[derive(Serialize)]
pub struct AttachToTangleResponse {
    /// Transaction trytes that include a valid `nonce` field
    #[serde(serialize_with = "transaction_serializer")]
    pub trytes: Vec<Transaction>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct AttachToTangleResponseBuilder {
    trytes: Option<Vec<String>>,
    error: Option<String>,
    exception: Option<String>,
}

impl AttachToTangleResponseBuilder {
    pub(crate) async fn build(self) -> Result<AttachToTangleResponse> {
        let mut trytes = Vec::new();
        if let Some(exception) = self.exception {
            return Err(anyhow!("{}", exception));
        } else if let Some(error) = self.error {
            return Err(anyhow!("{}", error));
        } else if let Some(s) = self.trytes {
            s.iter().for_each(|x| {
                trytes.push(
                    Transaction::from_trits(TryteBuf::try_from_str(&x).unwrap().as_trits())
                        .unwrap(),
                )
            });
        }

        Ok(AttachToTangleResponse { trytes })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ErrorResponseBuilder {
    error: Option<String>,
    exception: Option<String>,
}

impl ErrorResponseBuilder {
    pub(crate) async fn build(self) -> Result<()> {
        if let Some(exception) = self.exception {
            return Err(anyhow!("{}", exception));
        } else if let Some(error) = self.error {
            return Err(anyhow!("{}", error));
        }

        Ok(())
    }
}

/// findTransactions Response Type
#[derive(Clone, Debug)]
pub struct FindTransactionsResponse {
    /// The transaction hashes which are returned depend on your input.
    /// * bundles: returns an array of transaction hashes that contain the given bundle hash.
    /// * addresses: returns an array of transaction hashes that contain the given address in the address field.
    /// * tags: returns an array of transaction hashes that contain the given value in the tag field.
    /// * approvees: returns an array of transaction hashes that contain the given transactions in their branchTransaction or trunkTransaction fields.
    pub hashes: Vec<Hash>,
}

// TODO: remove this when iota_bundle_preview::Hash implements Serialize
impl Serialize for FindTransactionsResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("FindTransactionsResponse", 1)?;
        let hashes: Vec<&[i8]> = self.hashes.iter().map(|hash| hash.as_bytes()).collect();
        state.serialize_field("hashes", &hashes)?;
        state.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct FindTransactionsResponseBuilder {
    hashes: Option<Vec<String>>,
    error: Option<String>,
    exception: Option<String>,
}

impl FindTransactionsResponseBuilder {
    pub(crate) async fn build(self) -> Result<FindTransactionsResponse> {
        let mut hashes: Vec<Hash> = Vec::new();
        if let Some(exception) = self.exception {
            return Err(anyhow!("{}", exception));
        } else if let Some(error) = self.error {
            return Err(anyhow!("{}", error));
        } else if let Some(s) = self.hashes {
            hashes = s
                .iter()
                .map(|s| {
                    Hash::from_inner_unchecked(
                        TryteBuf::try_from_str(&s).unwrap().as_trits().encode(),
                    )
                })
                .collect::<Vec<Hash>>();
        }

        Ok(FindTransactionsResponse { hashes })
    }
}

/// getBalances Response Type
#[derive(Clone, Debug)]
pub struct GetBalancesResponse {
    /// Array of balances in the same order as the `addresses` parameters were passed to the endpoint
    pub balances: Vec<u64>,
    /// The index of the milestone that confirmed the most recent balance
    pub milestone_index: i64,
    /// The referencing tips. If no `tips` parameter was passed to the endpoint,
    /// this field contains the hash of the latest milestone that confirmed the balance
    pub references: Vec<Hash>,
}

// TODO: remove this when iota_bundle_preview::Hash implements Serialize
impl Serialize for GetBalancesResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("GetBalancesResponse", 3)?;

        state.serialize_field("balances", &self.balances)?;
        state.serialize_field("milestone_index", &self.milestone_index)?;

        let references: Vec<&[i8]> = self.references.iter().map(|hash| hash.as_bytes()).collect();
        state.serialize_field("references", &references)?;

        state.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct GetBalancesResponseBuilder {
    balances: Option<Vec<String>>,
    #[serde(rename = "milestoneIndex")]
    milestone_index: Option<i64>,
    references: Option<Vec<String>>,
    error: Option<String>,
    exception: Option<String>,
}

impl GetBalancesResponseBuilder {
    pub(crate) async fn build(self) -> Result<GetBalancesResponse> {
        let mut res = GetBalancesResponse {
            balances: Vec::new(),
            milestone_index: 0,
            references: Vec::new(),
        };

        if let Some(exception) = self.exception {
            return Err(anyhow!("{}", exception));
        } else if let Some(error) = self.error {
            return Err(anyhow!("{}", error));
        }

        if let Some(s) = self.balances {
            res.balances = s.into_iter().map(|s| s.parse().unwrap()).collect();
        }

        if let Some(s) = self.milestone_index {
            res.milestone_index = s;
        }

        if let Some(s) = self.references {
            res.references = s
                .iter()
                .map(|s| {
                    Hash::from_inner_unchecked(
                        TryteBuf::try_from_str(&s).unwrap().as_trits().encode(),
                    )
                })
                .collect::<Vec<Hash>>();
        }

        Ok(res)
    }
}

/// getInclusionStatesResponse Response Type
#[derive(Clone, Debug, Serialize)]
pub struct GetInclusionStatesResponse {
    /// List of boolean values in the same order as the `transactions` parameters.
    /// A `true` value means the transaction was confirmed
    pub states: Vec<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct GetInclusionStatesResponseBuilder {
    states: Option<Vec<bool>>,
    error: Option<String>,
    exception: Option<String>,
}

impl GetInclusionStatesResponseBuilder {
    pub(crate) async fn build(self) -> Result<GetInclusionStatesResponse> {
        let mut states = Vec::new();
        if let Some(exception) = self.exception {
            return Err(anyhow!("{}", exception));
        } else if let Some(error) = self.error {
            return Err(anyhow!("{}", error));
        } else if let Some(s) = self.states {
            states = s;
        }

        Ok(GetInclusionStatesResponse { states })
    }
}

/// getNeighbors Response Type
#[derive(Clone, Debug, Serialize)]
pub struct GetNeighborsResponse {
    /// Vector of `NeighborResponse`
    pub neighbors: Vec<NeighborResponse>,
}

/// getNeighbors Response Type
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct GetNeighborsResponseBuilder {
    neighbors: Option<Vec<NeighborResponse>>,
    error: Option<String>,
    exception: Option<String>,
}

impl GetNeighborsResponseBuilder {
    pub(crate) async fn build(self) -> Result<GetNeighborsResponse> {
        let mut neighbors = Vec::new();
        if let Some(exception) = self.exception {
            return Err(anyhow!("{}", exception));
        } else if let Some(error) = self.error {
            return Err(anyhow!("{}", error));
        } else if let Some(s) = self.neighbors {
            neighbors = s;
        }

        Ok(GetNeighborsResponse { neighbors })
    }
}

/// getNodeAPIConfiguration Response Type
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetNodeAPIConfigurationResponse {
    /// Maximum number of transactions that may be returned by the findTransactions endpoint
    #[serde(rename = "maxFindTransactions")]
    pub max_find_transactions: Option<usize>,
    /// Maximum number of parameters in an API call
    #[serde(rename = "maxRequestsList")]
    pub max_requests_list: Option<usize>,
    /// Maximum number of trytes that may be returned by the getTrytes endpoint
    #[serde(rename = "maxGetTrytes")]
    pub max_get_trytes: Option<usize>,
    /// Maximum number of characters that the body of an API call may contain
    #[serde(rename = "maxBodyLength")]
    pub max_body_length: Option<usize>,
    /// See if the node runs on a network other than the Mainnet
    #[serde(rename = "testNet")]
    pub testnet: Option<bool>,
    /// Milestone start index on IRI node
    #[serde(rename = "milestoneStartIndex")]
    pub milestone_start_index: i64,
}

/// getNodeInfo Response Type
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetNodeInfoResponse {
    /// Name of IRI node
    #[serde(rename = "appName")]
    pub app_name: String,
    /// IRI version
    #[serde(rename = "appVersion")]
    pub app_version: String,
    /// Number of threads IRI is using
    #[serde(rename = "jreAvailableProcessors")]
    pub jre_available_processors: Option<u16>,
    /// Amount of free memory on IRI node
    #[serde(rename = "jreFreeMemory")]
    pub jre_free_memory: Option<u64>,
    /// Max amount of memory on IRI node
    #[serde(rename = "jreMaxMemory")]
    pub jre_max_memory: Option<u64>,
    /// Total amount of memory on IRI node
    #[serde(rename = "jreTotalMemory")]
    pub jre_total_memory: Option<u64>,
    /// JRE version of IRI node
    #[serde(rename = "jreVersion")]
    pub jre_version: Option<String>,
    /// Latest milestone on IRI node
    #[serde(rename = "latestMilestone")]
    pub latest_milestone: String,
    /// Latest milestone index on IRI node
    #[serde(rename = "latestMilestoneIndex")]
    pub latest_milestone_index: u32,
    /// Latest solid subtangle milestone on IRI node
    #[serde(rename = "latestSolidSubtangleMilestone")]
    pub latest_solid_subtangle_milestone: String,
    /// Latest solid subtangle milestone index on IRI node
    #[serde(rename = "latestSolidSubtangleMilestoneIndex")]
    pub latest_solid_subtangle_milestone_index: u32,
    /// Milestone start index on IRI node
    #[serde(rename = "milestoneStartIndex")]
    pub milestone_start_index: i64,
    /// Amount of neighbors connected to IRI node
    pub neighbors: u16,
    /// Packet queue size on IRI node
    #[serde(rename = "packetsQueueSize")]
    pub packets_queue_size: Option<u16>,
    /// Current time on IRI node (UNIX Seconds),
    pub time: u64,
    /// Amount of tips on IRI node
    pub tips: u32,
    /// Transactions to request on IRI node
    #[serde(rename = "transactionsToRequest")]
    pub transactions_to_request: u32,
}

/// getTips Response Type
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetTipsResponse {
    /// Vector of tip transaction hashes
    pub hashes: Vec<String>,
}

/// getTransactionsToApprove Response Type
#[derive(Clone, Debug)]
pub struct GTTAResponse {
    /// Valid trunk transaction hash
    pub trunk_transaction: Hash,
    /// Valid branch transaction hash
    pub branch_transaction: Hash,
}

// TODO: remove this when iota_bundle_preview::Hash implements Serialize
impl Serialize for GTTAResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("GTTAResponse", 2)?;

        state.serialize_field("trunk_transaction", &self.trunk_transaction.as_bytes())?;

        state.serialize_field("branch_transaction", &self.branch_transaction.as_bytes())?;

        state.end()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct GTTAResponseBuilder {
    #[serde(rename = "trunkTransaction")]
    trunk_transaction: Option<String>,
    #[serde(rename = "branchTransaction")]
    branch_transaction: Option<String>,
    error: Option<String>,
    exception: Option<String>,
}

impl GTTAResponseBuilder {
    pub(crate) async fn build(self) -> Result<GTTAResponse> {
        let mut res = GTTAResponse {
            trunk_transaction: Hash::zeros(),
            branch_transaction: Hash::zeros(),
        };

        if let Some(exception) = self.exception {
            return Err(anyhow!("{}", exception));
        } else if let Some(error) = self.error {
            return Err(anyhow!("{}", error));
        }

        if let Some(s) = self.trunk_transaction {
            res.trunk_transaction =
                Hash::from_inner_unchecked(TryteBuf::try_from_str(&s).unwrap().as_trits().encode());
        }

        if let Some(b) = self.branch_transaction {
            res.branch_transaction =
                Hash::from_inner_unchecked(TryteBuf::try_from_str(&b).unwrap().as_trits().encode());
        }

        Ok(res)
    }
}

/// Representation of neighbor node
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct NeighborResponse {
    /// IP address of neighbors
    pub address: String,
    /// Domain of neighbors
    pub domain: String,
    /// Number of all transactions
    #[serde(rename = "numberOfAllTransactions")]
    pub number_of_all_transactions: usize,
    /// Number of invalid transactions
    #[serde(rename = "numberOfInvalidTransactions")]
    pub number_of_invalid_transactions: usize,
    /// Number of new transactions
    #[serde(rename = "numberOfNewTransactions")]
    pub number_of_new_transactions: usize,
    /// Number of random transaction requests
    #[serde(rename = "numberOfRandomTransactionRequests")]
    pub number_of_random_transactions: usize,
    /// Number of sent transactions
    #[serde(rename = "numberOfSentTransactions")]
    pub number_of_sent_transactions: usize,
    /// Number of sent transactions
    #[serde(rename = "numberOfStaleTransactions")]
    pub number_of_stale_transactions: usize,
    /// Number of sent transactions
    #[serde(rename = "numberOfDroppedSentPackets")]
    pub number_of_dropped_sent_packets: usize,
    /// Type of connection, either tcp or udp
    #[serde(rename = "connectionType")]
    pub connection_type: String,
    /// Status of Neighbor connection
    pub connected: bool,
}

/// getTrytes Response Type
#[derive(Serialize)]
pub struct GetTrytesResponse {
    /// Vector of transaction trytes for the given transaction hashes (in the same order as the parameters)
    #[serde(serialize_with = "transaction_serializer")]
    pub trytes: Vec<Transaction>,
}

#[derive(Clone, Deserialize, Debug)]
pub(crate) struct GetTrytesResponseBuilder {
    trytes: Option<Vec<String>>,
    exception: Option<String>,
    error: Option<String>,
}

impl GetTrytesResponseBuilder {
    pub(crate) async fn build(self) -> Result<GetTrytesResponse> {
        let mut trytes = Vec::new();
        if let Some(exception) = self.exception {
            return Err(anyhow!("{}", exception));
        } else if let Some(error) = self.error {
            return Err(anyhow!("{}", error));
        } else if let Some(s) = self.trytes {
            s.iter().for_each(|x| {
                trytes.push(
                    Transaction::from_trits(TryteBuf::try_from_str(&x).unwrap().as_trits())
                        .unwrap(),
                )
            });
        }

        Ok(GetTrytesResponse { trytes })
    }
}

/// removeNeighbors Response Type
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RemoveNeighborsResponse {
    /// Total number of removed neighbors
    #[serde(rename = "removedNeighbors")]
    pub removed_neighbors: Option<usize>,
}

/// wereAddressesSpentFrom Response Type
#[derive(Clone, Debug)]
pub struct WereAddressesSpentFromResponse {
    /// States of the specified addresses in the same order as the values in the `addresses` parameter.
    /// A `true` value means that the address has been spent from.
    pub states: Vec<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WereAddressesSpentFromResponseBuilder {
    states: Option<Vec<bool>>,
    exception: Option<String>,
    error: Option<String>,
}

impl WereAddressesSpentFromResponseBuilder {
    pub(crate) async fn build(self) -> Result<WereAddressesSpentFromResponse> {
        let mut states = Vec::new();
        if let Some(exception) = self.exception {
            return Err(anyhow!("{}", exception));
        } else if let Some(error) = self.error {
            return Err(anyhow!("{}", error));
        } else if let Some(s) = self.states {
            states = s;
        }

        Ok(WereAddressesSpentFromResponse { states })
    }
}

#[derive(Clone, Debug, Serialize)]
/// Address can be used as input to spend balance
pub struct Input {
    pub(crate) address: Address,
    pub(crate) balance: u64,
    pub(crate) index: u64,
}

/// A transfer could be an input or an output for building a bundle.
/// input/output transfer depends on value, a negative value for an output transfer, a positive value for an input transfer
#[derive(Clone, Debug)]
pub struct Transfer {
    /// Transfer address
    pub address: Address,
    /// Transfer value
    pub value: u64,
    /// Optional message
    pub message: Option<String>,
    /// Optional message
    pub tag: Option<Tag>,
}

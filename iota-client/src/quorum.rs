//! Quorum module is a extension to iota client instance which can make sure result from API calls are verified by node
//! pool and guarantee to staisfy minimum quorum threshold.

use crate::error::*;
use crate::response::*;
use crate::Client;

use bee_crypto::ternary::Hash;
use bee_ternary::{T3B1Buf, TryteBuf};
use bee_transaction::bundled::{Address, BundledTransactionField};
use iota_conversion::Trinary;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use once_cell::sync::Lazy;

async fn get_synced_nodes(client: &Client) -> Result<HashSet<String>> {
    Ok(client
        .sync
        .read()
        .expect("Node pool read poinsened")
        .clone())
}

/// An instance of the client using HORNET URI
#[derive(Debug)]
pub struct Quorum {
    /// Quorum threshold
    pub(crate) threshold: AtomicU8,
    /// Minimum nodes quired to satisfy quorum threshold
    pub(crate) min: AtomicUsize,
    /// Timestamp of last syncing process
    pub(crate) time: Arc<RwLock<Instant>>,
}

impl Quorum {
    /// Get the quorum instance. It will init the instance if it's not created yet.
    pub fn get() -> &'static Quorum {
        static QUORUM: Lazy<Quorum> = Lazy::new(|| Quorum {
            threshold: AtomicU8::new(66),
            min: AtomicUsize::new(0),
            time: Arc::new(RwLock::new(Instant::now())),
        });

        &QUORUM
    }
}

/// Gets the confirmed balance of an address.
/// If the tips parameter is missing, the returned balance is correct as of the latest confirmed milestone.
/// This endpoint returns data only if the node is synchronized.
/// # Parameters
/// * [`addresses`] - Address for which to get the balance (do not include the checksum)
/// * [`threshold`] - (Optional) Confirmation threshold between 0 and 100, default is 100.
/// * [`tips`] - (Optional) Tips whose history of transactions to traverse to find the balance
///
/// [`addresses`]: struct.GetBalancesBuilder.html#method.addresses
/// [`threshold`]: struct.GetBalancesBuilder.html#method.threshold
/// [`tips`]: struct.GetBalancesBuilder.html#method.tips
pub fn get_balances() -> GetBalancesBuilder {
    GetBalancesBuilder::new()
}

/// Builder to construct getBalances API
#[derive(Debug)]
pub struct GetBalancesBuilder {
    addresses: Vec<String>,
    tips: Option<Vec<String>>,
}

impl GetBalancesBuilder {
    pub(crate) fn new() -> Self {
        Self {
            addresses: Default::default(),
            tips: Default::default(),
        }
    }

    /// Add address for which to get the balance (do not include the checksum)
    pub fn addresses(mut self, addresses: &[Address]) -> Self {
        self.addresses = addresses
            .iter()
            .map(|h| h.to_inner().as_i8_slice().trytes().unwrap())
            .collect();
        self
    }

    /// Add tips whose history of transactions to traverse to find the balance
    pub fn tips(mut self, tips: &[Hash]) -> Self {
        self.tips = Some(
            tips.iter()
                .map(|h| {
                    (*h).encode::<T3B1Buf>()
                        .iter_trytes()
                        .map(char::from)
                        .collect::<String>()
                })
                .collect(),
        );
        self
    }

    /// Send getBalances request
    pub async fn send(self, client: &Client) -> Result<GetBalancesResponse> {
        let quorum = Quorum::get();
        let mut body = json!({
            "command": "getBalances",
            "addresses": self.addresses,
        });

        if let Some(reference) = self.tips {
            body["tips"] = json!(reference);
        }

        let mut result = HashMap::new();
        for url in get_synced_nodes(client).await?.iter() {
            let body_ = body.clone();
            let res: GetBalancesResponseBuilder = response!(Client, body_, url);
            let res = res.build().await?;
            let counters = result.entry(res).or_insert(0);
            *counters += 1;
        }

        let res = result
            .into_iter()
            .max_by_key(|v| v.1)
            .ok_or(Error::QuorumError)?;

        if res.1 >= quorum.min.load(Ordering::Acquire) {
            Ok(res.0)
        } else {
            Err(Error::QuorumThreshold)
        }
    }
}

/// Gets the inclusion states of a set of transactions.
/// This endpoint determines if a transaction is confirmed by the network (referenced by a valid milestone).
/// You can search for multiple tips (and thus, milestones) to get past inclusion states of transactions.
/// This endpoint returns data only if the node is synchronized.
/// # Parameters
/// * [`transactions`] - List of transaction hashes for which you want to get the inclusion state
/// * [`tips`] - (Optional) List of tip transaction hashes (including milestones) you want to search for
///
/// [`transactions`]: ../core/struct.GetInclusionStatesBuilder.html#method.transactions
/// [`tips`]: ../core/struct.GetInclusionStatesBuilder.html#method.tips
pub fn get_inclusion_states() -> GetInclusionStatesBuilder {
    GetInclusionStatesBuilder::new()
}

/// Builder to construct getInclusionStates API
#[derive(Debug)]
pub struct GetInclusionStatesBuilder {
    transactions: Vec<String>,
}

impl GetInclusionStatesBuilder {
    pub(crate) fn new() -> Self {
        Self {
            transactions: Default::default(),
        }
    }

    /// Add list of transaction hashes for which you want to get the inclusion state
    pub fn transactions(mut self, transactions: &[Hash]) -> Self {
        self.transactions = transactions
            .iter()
            .map(|h| {
                (*h).encode::<T3B1Buf>()
                    .iter_trytes()
                    .map(char::from)
                    .collect::<String>()
            })
            .collect();
        self
    }

    /// Send getInclusionStates request
    pub async fn send(self, client: &Client) -> Result<GetInclusionStatesResponse> {
        let quorum = Quorum::get();
        let body = json!({
            "command": "getInclusionStates",
            "transactions": self.transactions,
        });

        let mut result = HashMap::new();
        for url in get_synced_nodes(client).await?.iter() {
            let body_ = body.clone();
            let res: GetInclusionStatesResponseBuilder = response!(Client, body_, url);
            let res = res.build().await?;
            let counters = result.entry(res).or_insert(0);
            *counters += 1;
        }

        let res = result
            .into_iter()
            .max_by_key(|v| v.1)
            .ok_or(Error::QuorumError)?;

        if res.1 >= quorum.min.load(Ordering::Acquire) {
            Ok(res.0)
        } else {
            Err(Error::QuorumThreshold)
        }
    }
}

/// Fetches inclusion states of the given transactions by calling GetInclusionStates
/// using the latest solid subtangle milestone from GetNodeInfo.
///
/// # Parameters
/// * [`transactions`] - List of transaction hashes for which you want to get the inclusion state
pub async fn get_latest_inclusion(transactions: &[Hash], client: &Client) -> Result<Vec<bool>> {
    let states = get_inclusion_states()
        .transactions(transactions)
        .send(client)
        .await?
        .states;
    Ok(states)
}

/// Gets latest solid subtangle milestone.
pub async fn get_latest_solid_subtangle_milestone(client: &Client) -> Result<Hash> {
    let quorum = Quorum::get();
    let body = json!( {
        "command": "getNodeInfo",
    });

    let mut result = HashMap::new();
    for url in get_synced_nodes(client).await?.iter() {
        let body_ = body.clone();
        let hash: GetNodeInfoResponse = response!(Client, body_, url);
        let hash = Hash::from_inner_unchecked(
            // TODO missing impl error on Hash
            TryteBuf::try_from_str(&hash.latest_solid_subtangle_milestone)
                .unwrap()
                .as_trits()
                .encode(),
        );
        let counters = result.entry(hash).or_insert(0);
        *counters += 1;
    }

    let res = result
        .into_iter()
        .max_by_key(|v| v.1)
        .ok_or(Error::QuorumError)?;

    if res.1 >= quorum.min.load(Ordering::Acquire) {
        Ok(res.0)
    } else {
        Err(Error::QuorumThreshold)
    }
}

/// Checks if an address was ever withdrawn from, either in the current epoch or in any previous epochs.
/// If an address has a pending transaction, it's also considered 'spent'.
/// # Parameters
/// * `address` - addresses to check (do not include the checksum)
pub async fn were_addresses_spent_from(
    addresses: &[Address],
    client: &Client,
) -> Result<WereAddressesSpentFromResponse> {
    let quorum = Quorum::get();
    let addresses: Vec<String> = addresses
        .iter()
        .map(|h| h.to_inner().as_i8_slice().trytes().unwrap())
        .collect();
    let body = json!({
        "command": "wereAddressesSpentFrom",
        "addresses": addresses,
    });

    let mut result = HashMap::new();
    for url in get_synced_nodes(client).await?.iter() {
        let body_ = body.clone();
        let res: WereAddressesSpentFromResponseBuilder = response!(Client, body_, url);
        let res = res.build().await?;
        let counters = result.entry(res).or_insert(0);
        *counters += 1;
    }

    let res = result
        .into_iter()
        .max_by_key(|v| v.1)
        .ok_or(Error::QuorumError)?;

    if res.1 >= quorum.min.load(Ordering::Acquire) {
        Ok(res.0)
    } else {
        Err(Error::QuorumThreshold)
    }
}

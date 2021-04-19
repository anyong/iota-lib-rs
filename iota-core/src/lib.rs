//! IOTA core

#![deny(unused_extern_crates)]
#![warn(
    //missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]

pub use bee_ternary as ternary;
pub use bee_transaction as transaction;
pub use crypto;
pub use iota_bundle_miner as bundle_miner;
pub use iota_client as client;

pub use client::{iota_core::AddressBuilder, Client, ClientBuilder};

// TODO prelude

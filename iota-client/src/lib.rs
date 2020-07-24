//! Provides access to the Iota Client API

#![deny(unused_extern_crates)]
#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]

#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_json;

pub mod builder;
#[macro_use]
pub mod client;
pub mod core;
pub mod error;
pub mod extended;
// #[cfg(feature = "quorum")]
// pub mod quorum;
pub mod response;
mod util;

pub use builder::ClientBuilder;
pub use client::Client;
pub use error::*;
pub use response::*;
pub use util::{bytes_to_trytes, str_to_trytes};

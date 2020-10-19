//! IOTA core

#![deny(unused_extern_crates)]
#![warn(
    //missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]

pub use bee_message as message;
pub use bee_signing_ext as signing;
pub use iota_client as client;

pub use client::{Client, ClientBuilder};

// TODO prelude
pub use client::*;
pub use message::prelude;
pub use prelude::*;
pub use signing::binary::BIP32Path;

//! This library provides pretty much anything you could need to work with
//! Iota. The documentation is a work in progress, but if you need any help
//! I can usually be found on the Iota discord rust or development chats.
//!
//! Heres a quick example of how to send a transaction (Note that trytes is being
//! used as a seed here...don't do that)
//!```
//! extern crate iota_lib_rs;
//! extern crate futures;
//! 
//! use iota_lib_rs::iota_api;
//! use iota_lib_rs::iota_api::SendTransferOptions;
//! use iota_lib_rs::utils::trytes_converter;
//! use iota_lib_rs::model::*;
//! use futures::executor::block_on;
//!
//! fn main() {
//!     let trytes = "HELLOWORLDHELLOWORLDHELLOWORLDHELLOWORLDHELLOWORLDHELLOWORLDHELLOWORLDHELLOWORLDD";
//!     let message = trytes_converter::to_trytes("Hello World").unwrap();
//!     let mut transfer = Transfer::default();
//!     *transfer.value_mut() = 0;
//!     *transfer.address_mut() = trytes.to_string();
//!     *transfer.message_mut() = message;
//!     let api = iota_api::API::new("https://trinity.iota.fm");
//!     let options = SendTransferOptions{
//!         seed: trytes.to_string(),
//!         depth: 3,
//!         min_weight_magnitude: 14,
//!         local_pow: true,
//!         threads: None,
//!         inputs: None,
//!         reference: None,
//!         remainder_address: None,
//!         security: None,
//!         hmac_key: None,
//!     };    
//!     let tx = block_on(api.send_transfers(vec![transfer], options)).unwrap();
//!     println!("{:?}", tx);
//! }
//!```
#![allow(dead_code)]
#![feature(rust_2018_preview)]
#![feature(rust_2018_idioms)]
#![feature(futures_api)]
#![feature(async_await)]
#![feature(await_macro)]

#[macro_use]
extern crate crunchy;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate crossbeam;
extern crate futures;
extern crate num_cpus;
extern crate rand;
extern crate regex;
extern crate reqwest;


/// Provides all crypto algorithms and data structures used by Iota
pub mod crypto;
/// Provides helper functions that make interacting with IRI easier
pub mod iota_api;
/// Provides methods to call IRI APIs
pub mod iri_api;
/// Provides the various struces used by Iota
pub mod model;
/// Provides multi-sig functionality
pub mod multisig;
/// Provides many useful helper functions that are used throughout
/// the library
pub mod utils;

use std::result;

/// This is the result type used throughout the library
pub type Result<T> = result::Result<T, failure::Error>;

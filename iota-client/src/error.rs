//! Error handling in iota-client crate.

use std::fmt;

/// Type alias of `Result` in iota-client
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
/// Error type of the iota client crate.
pub enum Error {
    /// Error when building transaction messages
    TransactionError,
    /// The wallet account doesn't have enough balance
    NotEnoughBalance(u64),
    /// Missing required parameters
    MissingParameter(String),
    /// Invalid parameters
    InvalidParameter(String),
    /// Found Spent Address that still has balance
    SpentAddress,
    /// Error from RestAPI calls with status code other than 200
    ResponseError(u16),
    /// No node available in the node pool
    NodePoolEmpty,
    /// Error on Url type conversion
    UrlError,
    /// Errors from reqwest api call
    ReqwestError(reqwest::Error),
    /// Hex string convert error
    FromHexError(hex::FromHexError),
    /// Message types error
    MessageError(bee_message::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::TransactionError => write!(f, "Error when building transaction message"),
            Error::MissingParameter(s) => write!(f, "Must provide required parameter:{}", s),
            Error::InvalidParameter(s) => write!(f, "Parameter is invalid:{}", s),
            Error::SpentAddress => "Found Spent Address that still has balance.".fmt(f),
            Error::NodePoolEmpty => "No node available".fmt(f),
            Error::ReqwestError(e) => e.fmt(f),
            Error::UrlError => "Fail to parse url".fmt(f),
            Error::NotEnoughBalance(v) => write!(
                f,
                "The wallet account doesn't have enough balance. It only has {:?}",
                v
            ),
            Error::FromHexError(e) => e.fmt(f),
            Error::ResponseError(s) => write!(f, "Response error with status code {}", s),
            Error::MessageError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::ReqwestError(error)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(error: hex::FromHexError) -> Self {
        Error::FromHexError(error)
    }
}

impl From<bee_message::Error> for Error {
    fn from(error: bee_message::Error) -> Self {
        Error::MessageError(error)
    }
}

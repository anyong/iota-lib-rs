use reqwest::r#async::{Client, Response};
use reqwest::Error;
use tokio::prelude::Future;

/// Store transactions into the local storage.
/// The trytes to be used for this call are
/// returned by attachToTangle.
pub(crate) fn store_transactions(
    client: &Client,
    uri: &str,
    trytes: &[String],
) -> impl Future<Item = Response, Error = Error> {
    let body = json!({
        "command": "storeTransactions",
        "trytes": trytes,
    });

    client
        .post(uri)
        .header("ContentType", "application/json")
        .header("X-IOTA-API-Version", "1")
        .body(body.to_string())
        .send()
}

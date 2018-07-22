use super::responses::BroadcastTransactionsResponse;
use crate::utils::input_validator;
use crate::Result;
use reqwest::header::{ContentType, Headers};
use reqwest::Client;

/// Broadcast a list of transactions to all neighbors.
/// The input trytes for this call are provided by attachToTangle.
pub async fn broadcast_transactions(
    client: &Client,
    uri: String,
    trytes: Vec<String>,
) -> Result<BroadcastTransactionsResponse> {
    ensure!(
        input_validator::is_array_of_attached_trytes(&trytes),
        "Provided trytes are not valid: {:?}",
        trytes
    );

    let mut headers = Headers::new();
    headers.set(ContentType::json());
    headers.set_raw("X-IOTA-API-Version", "1");

    let body = json!({
        "command": "broadcastTransactions",
        "trytes": trytes,
    });

    let resp: BroadcastTransactionsResponse = client
        .post(&uri)
        .headers(headers)
        .body(body.to_string())
        .send()?
        .json()?;

    if let Some(error) = resp.error() {
        return Err(format_err!("{}", error));
    }
    if let Some(exception) = resp.exception() {
        return Err(format_err!("{}", exception));
    }

    Ok(resp)
}

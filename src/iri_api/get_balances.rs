use super::responses::GetBalancesResponse;
use crate::Result;
use reqwest::header::{ContentType, Headers};
use reqwest::Client;

/// Returns the balance based on the latest confirmed milestone.
/// In addition to the balances, it also returns the referencing tips (or milestone),
/// as well as the index with which the confirmed balance was
/// determined. The balances is returned as a list in the same
/// order as the addresses were provided as input.
pub async fn get_balances(
    client: &Client,
    uri: String,
    addresses: Vec<String>,
    threshold: i32,
) -> Result<GetBalancesResponse> {
    let mut headers = Headers::new();
    headers.set(ContentType::json());
    headers.set_raw("X-IOTA-API-Version", "1");

    let body = json!({
        "command": "getBalances",
        "addresses": addresses,
        "threshold": threshold,
    });

    Ok(client
        .post(&uri)
        .headers(headers)
        .body(body.to_string())
        .send()?
        .json()?)
}

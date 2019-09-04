use reqwest::r#async::{Client, Response};
use reqwest::Error;
use tokio::prelude::Future;

/// Struct used to provide named arguments for `get_transactions_to_approve`
#[derive(Clone, Debug)]
pub struct GetTransactionsToApproveOptions<'a> {
    /// How deep to search
    pub depth: usize,
    /// Where to start search
    pub reference: Option<&'a str>,
}

/// Provide sane defaults for the fields
/// * `depth` - 3
/// * `reference` - None
impl<'a> Default for GetTransactionsToApproveOptions<'a> {
    fn default() -> Self {
        GetTransactionsToApproveOptions {
            depth: 3,
            reference: None,
        }
    }
}

/// Tip selection which returns `trunkTransaction` and
/// `branchTransaction`. The input value depth determines
/// how many milestones to go back to for finding the
/// transactions to approve. The higher your depth value,
/// the more work you have to do as you are confirming more
/// transactions. If the depth is too large (usually above 15,
/// it depends on the node's configuration) an error will be
/// returned. The reference is an optional hash of a transaction
/// you want to approve. If it can't be found at the specified
/// depth then an error will be returned.
pub(crate) fn get_transactions_to_approve(
    client: &Client,
    uri: &str,
    options: GetTransactionsToApproveOptions<'_>,
) -> impl Future<Item = Response, Error = Error> {
    let mut body = json!({
        "command": "getTransactionsToApprove",
        "depth": options.depth,
    });

    if let Some(reference) = options.reference {
        body["reference"] = json!(reference);
    }

    client
        .post(uri)
        .header("ContentType", "application/json")
        .header("X-IOTA-API-Version", "1")
        .body(body.to_string())
        .send()
}

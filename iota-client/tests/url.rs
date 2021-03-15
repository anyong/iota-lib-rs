#[tokio::test]
async fn invalid_url() {
    let client = iota_client::Client::builder().with_node("data:text/plain,Hello?World#");
    assert!(client.is_err());
}
#[tokio::test]
async fn valid_url() {
    let client = iota_client::Client::builder().with_node("https://api.lb-0.testnet.chrysalis2.com");
    assert!(client.is_ok());
}

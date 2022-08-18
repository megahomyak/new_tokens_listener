#[tokio::main]
async fn main() {
    let blockchain_client =
        web3::Web3::new(web3::transports::http::Http::new("https://eth.public-rpc.com").unwrap());
    let block_id = web3::types::BlockId::Number(web3::types::BlockNumber::Number(
        blockchain_client.eth().block_number().await.unwrap(),
    ));
    let block = blockchain_client
        .eth()
        .block(block_id)
        .await
        .unwrap()
        .unwrap();
    println!("{:?}", block);
}

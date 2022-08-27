use std::time::{Duration, SystemTime};

use polling::Poller;

mod block;
mod polling;
mod interval_ensurer;

#[tokio::main]
async fn main() {
    let blockchain_client =
        web3::Web3::new(web3::transports::http::Http::new("https://eth.public-rpc.com").unwrap());
    let last_block_number = blockchain_client
        .eth()
        .block_number()
        .await
        .unwrap()
        .as_u64();
    let mut poller = Poller::new(last_block_number, &blockchain_client);
    let poll_interval = Duration::from_millis(20);
    loop {
        let beginning = SystemTime::now();

        let new_blocks = poller.get_new_blocks().await.unwrap();
        let mut hashes_were_printed = false;
        for transaction in futures::future::join_all(
            new_blocks
                .into_iter()
                .flat_map(|block| block.transaction_ids)
                .map(|transaction_id| blockchain_client.eth().transaction(transaction_id.into())),
        )
        .await
        .into_iter()
        .filter_map(|transaction| transaction.ok().and_then(std::convert::identity))
        {
            println!("{:x}", transaction.hash);
            hashes_were_printed = true;
        }
        if hashes_were_printed {
            println!("-----");
        }
        match beginning.elapsed() {
            Ok(time_taken) => {
                if let Some(remaining_time) = poll_interval.checked_sub(time_taken) {
                    tokio::time::sleep(remaining_time).await;
                }
            }
            Err(_subtraction_error) => tokio::time::sleep(poll_interval).await,
        };
    }
}

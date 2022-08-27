use std::{
    future::Future,
    ops::ControlFlow,
    sync::Arc,
    time::{Duration, SystemTime},
};

use polling::Poller;
use tokio::sync::Mutex;

mod polling;
mod block;

/// Guaranteed to execute `action` with at least `duration` of time between two executions, trying
/// to invoke `action` as frequently as possible.
pub async fn every<Breaker: Send, Fut: Future<Output = ControlFlow<Breaker>> + Send>(
    duration: Duration,
    mut action: impl FnMut() -> Fut + Send,
) -> Breaker {
    loop {
        let beginning = SystemTime::now();
        if let ControlFlow::Break(breaker) = action().await {
            break breaker;
        }
        match beginning.elapsed() {
            Ok(time_taken) => {
                if let Some(remaining_time) = duration.checked_sub(time_taken) {
                    tokio::time::sleep(remaining_time).await;
                }
            }
            Err(_subtraction_error) => tokio::time::sleep(duration).await,
        };
    }
}

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
    let poller = Arc::new(Mutex::new(Poller::new(
        last_block_number,
        &blockchain_client,
    )));
    every(Duration::from_millis(20), move || {
        let poller = poller.clone();
        async move {
            let new_blocks = poller.lock().await.get_new_blocks().await.unwrap();
            if !new_blocks.is_empty() {
                println!("-----");
                for block in &new_blocks {
                    println!("{:x}", block.hash);
                }
            }
            ControlFlow::<(), ()>::Continue(())
        }
    })
    .await;
}

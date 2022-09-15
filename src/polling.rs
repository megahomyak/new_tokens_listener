use std::collections::TryReserveError;

use futures::{stream, Future, Stream, StreamExt};
use web3::Web3;

use crate::block::Block;

pub trait TryWithCapacity<T> {
    fn try_with_capacity(capacity: usize) -> Result<Vec<T>, TryReserveError>;
}

impl<T> TryWithCapacity<T> for Vec<T> {
    fn try_with_capacity(capacity: usize) -> Result<Self, TryReserveError> {
        let mut new_blocks = Self::new();
        new_blocks.try_reserve_exact(capacity)?;
        Ok(new_blocks)
    }
}

pub fn make_block_id(number: u64) -> web3::types::BlockId {
    web3::types::U64::from(number).into()
}

type BlockNumber = u64;

#[derive(Debug)]
pub enum CurrentBlockIdGettingError {
    /// An error occured when making a request to the blockchain RPC server
    ServerRequestError(web3::Error),
}

#[derive(Debug)]
pub enum BlockGettingError {
    NoSuchBlock,
    /// Emitted when the received block information is incomplete
    Incomplete,
    /// An error occured when making a request to the blockchain RPC server
    ServerRequestError(web3::Error),
}

pub async fn get_new_blocks<'client, Transport>(
    client: &'client Web3<Transport>,
    after: BlockNumber,
) -> Result<impl Stream<Item = Result<Block, BlockGettingError>> + 'client, CurrentBlockIdGettingError>
where
    Transport: web3::Transport + Send + Sync,
    Transport::Out: Send,
{
    let current_block_id = match client.eth().block_number().await {
        Ok(block_id) => block_id.as_u64(),
        Err(request_error) => {
            return Err(CurrentBlockIdGettingError::ServerRequestError(
                request_error,
            ))
        }
    };
    Ok(stream::iter(
        (after + 1..=current_block_id).map(|block_id| client.eth().block(make_block_id(block_id))),
    )
    .map(|block| async {
        match block.await {
            Ok(Some(block)) => {
                if let (Some(number), Some(hash)) = (block.number, block.hash) {
                    Ok(Block {
                        hash,
                        number: number.as_u64(),
                        transaction_ids: block.transactions,
                    })
                } else {
                    Err(BlockGettingError::Incomplete)
                }
            }
            Ok(None) => Err(BlockGettingError::NoSuchBlock),
            Err(request_error) => Err(BlockGettingError::ServerRequestError(request_error)),
        }
    }))
}

pub struct Poller<'poller, Transport: web3::Transport> {
    latest_known_block_number: BlockNumber,
    blockchain_provider_client: &'poller Web3<Transport>,
}

impl<'poller, Transport: web3::Transport> Poller<'poller, Transport> {
    pub const fn new(
        latest_known_block_number: BlockNumber,
        blockchain_provider_client: &'poller Web3<Transport>,
    ) -> Self {
        Self {
            latest_known_block_number,
            blockchain_provider_client,
        }
    }

    pub async fn get_new_blocks(&mut self) -> Result<Vec<Block>, BlocksGettingError>
    where
        Transport: Send + Sync,
        Transport::Out: Send,
    {
        let new_blocks = get_new_blocks(
            self.blockchain_provider_client,
            self.latest_known_block_number,
        )
        .await?;
        if !new_blocks.is_empty() {
            self.latest_known_block_number = new_blocks[new_blocks.len() - 1].number;
        }
        Ok(new_blocks)
    }
}

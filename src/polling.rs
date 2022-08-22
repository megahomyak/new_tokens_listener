use std::collections::TryReserveError;

use web3::{
    types::{Block, H256},
    Web3,
};

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
pub enum BlocksGettingError {
    /// Emitted when there are so many blocks that a vector can't be allocated for them
    TooManyBlocks,
    /// An error occured when making a request to the blockchain RPC server
    ServerRequestError(web3::Error),
    /// Block with the passed block ID does not exist yet
    BeginningDoesNotExist,
}

pub async fn get_new_blocks<Transport>(
    client: &Web3<Transport>,
    after: BlockNumber,
) -> Result<Vec<Block<H256>>, BlocksGettingError>
where
    Transport: web3::Transport + Send + Sync,
    Transport::Out: Send,
{
    let current_block_id = match client.eth().block_number().await {
        Ok(block_id) => block_id.as_u64(),
        Err(request_error) => return Err(BlocksGettingError::ServerRequestError(request_error)),
    };
    if current_block_id < after {
        return Err(BlocksGettingError::BeginningDoesNotExist);
    }
    let new_blocks_amount = current_block_id - after;
    let mut new_blocks = match Vec::try_with_capacity(match new_blocks_amount.try_into() {
        Ok(new_blocks_amount) => new_blocks_amount,
        Err(_conversion_error) => return Err(BlocksGettingError::TooManyBlocks),
    }) {
        Ok(new_blocks) => new_blocks,
        Err(_allocation_error) => return Err(BlocksGettingError::TooManyBlocks),
    };
    for block_id in (after + 1)..=current_block_id {
        new_blocks.push(match client.eth().block(make_block_id(block_id)).await {
            Ok(maybe_block) => maybe_block.expect("Block must exist"),
            Err(request_error) => {
                return Err(BlocksGettingError::ServerRequestError(request_error))
            }
        });
    }
    Ok(new_blocks)
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

    pub async fn get_new_blocks(&mut self) -> Result<Vec<Block<H256>>, BlocksGettingError>
    where
        Transport: Send + Sync,
        Transport::Out: Send,
    {
        let new_blocks = get_new_blocks(
            self.blockchain_provider_client,
            self.latest_known_block_number,
        )
        .await?;
        self.latest_known_block_number += u64::try_from(new_blocks.len())
            .expect("Blocks amount can't be bigger than u64 (by Ethereum's definition)");
        Ok(new_blocks)
    }

    pub const fn latest_known_block_number(&self) -> BlockNumber {
        self.latest_known_block_number
    }
}

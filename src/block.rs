use web3::types::H256;

pub struct Block {
    pub hash: H256,
    pub number: u64,
    pub transaction_ids: Vec<H256>,
}

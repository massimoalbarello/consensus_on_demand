use crate::consensus_layer::pool_reader::PoolReader;

pub struct BlockMaker {
    time: u64,
}

impl BlockMaker {
    pub fn new() -> Self {
        Self {
            time: 0,
        }
    }

    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> u64 {
        1
    }
}
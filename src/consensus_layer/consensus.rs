use super::{
    pool::ConsensusPoolImpl, 
    artifacts::{ChangeSet, ChangeAction},
    pool_reader::PoolReader,
    consensus_subcomponents::{notary::Notary, block_maker::BlockMaker},
};

pub struct ConsensusImpl {
    block_maker: BlockMaker,
    notary: Notary,
}

impl ConsensusImpl {
    pub fn new() -> Self {
        Self {
            block_maker: BlockMaker::new(),
            notary: Notary::new(),
        }
    }

    pub fn on_state_change(&self, pool: &ConsensusPoolImpl) -> ChangeSet {
        let pool_reader = PoolReader::new(pool);

        let make_block = || {
            self.block_maker.on_state_change(&pool_reader)
        };

        let notarize = || {
            self.notary.on_state_change(&pool_reader)
        };

        vec![ChangeAction::AddToValidated(String::from("Consensus message")), ChangeAction::MoveToValidated(String::from("Consensus message"))]
    }
}
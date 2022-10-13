use super::{
    pool::ConsensusPoolImpl, 
    artifacts::{ChangeSet, ChangeAction},
    pool_reader::PoolReader,
    consensus_subcomponents::{notary::{Notary, NotarizationShare}, block_maker::{BlockMaker, BlockProposal}},
};


// Rotate on_state_change calls with a round robin schedule to ensure fairness.
#[derive(Default)]
struct RoundRobin {
    index: std::cell::RefCell<usize>,
}

impl RoundRobin {
    // Call the next function in the given list of calls according to a round
    // robin schedule. Return as soon as a call returns a non-empty ChangeSet.
    // Otherwise try calling the next one, and return empty ChangeSet if all
    // calls from the given list have been tried.
    pub fn call_next<'a, T>(&self, calls: &[&'a dyn Fn() -> Vec<T>]) -> Vec<T> {
        let mut result;
        let mut index = self.index.borrow_mut();
        let mut next = *index;
        loop {
            result = calls[next]();
            next = (next + 1) % calls.len();
            if !result.is_empty() || *index == next {
                break;
            };
        }
        *index = next;
        result
    }
}

pub struct ConsensusImpl {
    block_maker: BlockMaker,
    notary: Notary,
    schedule: RoundRobin,
}

impl ConsensusImpl {
    pub fn new() -> Self {
        Self {
            block_maker: BlockMaker::new(),
            notary: Notary::new(),
            schedule: RoundRobin::default(),
        }
    }

    pub fn on_state_change(&self, pool: &ConsensusPoolImpl) -> ChangeSet {
        // Invoke `on_state_change` on each subcomponent in order.
        // Return the first non-empty [ChangeSet] as returned by a subcomponent.
        // Otherwise return an empty [ChangeSet] if all subcomponents return
        // empty.
        //
        // There are two decisions that ConsensusImpl makes:
        //
        // 1. It must return immediately if one of the subcomponent returns a
        // non-empty [ChangeSet]. It is important that a [ChangeSet] is fully
        // applied to the pool or timer before another subcomponent uses
        // them, because each subcomponent expects to see full state in order to
        // make correct decisions on what to do next.
        //
        // 2. The order in which subcomponents are called also matters. At the
        // moment it is important to call finalizer first, because otherwise
        // we'll just keep producing notarized blocks indefintely without
        // finalizing anything, due to the above decision of having to return
        // early. The order of the rest subcomponents decides whom is given
        // a priority, but it should not affect liveness or correctness.

        let pool_reader = PoolReader::new(pool);

        let make_block = || {
            add_to_validated(self.block_maker.on_state_change(&pool_reader))
        };

        let notarize = || {
            add_all_to_validated(self.notary.on_state_change(&pool_reader))
        };

        let calls: [&'_ dyn Fn() -> ChangeSet; 2] = [
            &notarize,
            &make_block,
        ];

        let changeset = self.schedule.call_next(&calls);

        changeset
    }
}


fn add_all_to_validated(messages: Vec<NotarizationShare>) -> ChangeSet {
    messages
        .into_iter()
        .map(|msg| ChangeAction::AddToValidated(String::from("Notarization Share")))
        .collect()
}

fn add_to_validated(msg: Option<BlockProposal>) -> ChangeSet {
    msg.map(|msg| ChangeAction::AddToValidated(String::from("Block Proposal")).into())
        .unwrap_or_default()
}
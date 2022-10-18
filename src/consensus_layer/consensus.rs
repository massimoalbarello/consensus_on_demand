use super::{
    pool::ConsensusPoolImpl, 
    artifacts::{ChangeSet, ChangeAction, ConsensusMessage},
    pool_reader::PoolReader,
    consensus_subcomponents::{notary::Notary, block_maker::BlockMaker, validator::Validator},
};


// Rotate on_state_change calls with a round robin schedule to ensure fairness.
#[derive(Default)]
pub struct RoundRobin {
    index: std::cell::RefCell<usize>,
}

impl RoundRobin {
    // Call the next function in the given list of calls according to a round
    // robin schedule. Return as soon as a call returns a non-empty ChangeSet.
    // Otherwise try calling the next one, and return empty ChangeSet if all
    // calls from the given list have been tried.
    pub fn call_next<'a, T>(&self, calls: &[&'a dyn Fn() -> (Vec<T>, bool)]) -> (Vec<T>, bool) {
        let mut result;
        let mut to_broadcast;
        let mut index = self.index.borrow_mut();
        let mut next = *index;
        loop {
            (result, to_broadcast) = calls[next]();
            next = (next + 1) % calls.len();
            if !result.is_empty() || *index == next {
                break;
            };
        }
        *index = next;
        (result, to_broadcast)
    }
}

pub struct ConsensusImpl {
    block_maker: BlockMaker,
    notary: Notary,
    validator: Validator,
    schedule: RoundRobin,
}

impl ConsensusImpl {
    pub fn new(node_number: u8) -> Self {
        Self {
            block_maker: BlockMaker::new(node_number),
            notary: Notary::new(node_number),
            validator: Validator::new(),
            schedule: RoundRobin::default(),
        }
    }

    pub fn on_state_change(&self, pool: &ConsensusPoolImpl) -> (ChangeSet, bool) {
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
        
        let notarize = || {
            let change_set = add_all_to_validated(self.notary.on_state_change(&pool_reader));
            let to_broadcast = true;
            (change_set, to_broadcast)
        };

        let make_block = || {
            let change_set = add_to_validated(self.block_maker.on_state_change(&pool_reader));
            let to_broadcast = true;
            (change_set, to_broadcast)
        };


        let validate = || {
            self.validator.on_state_change(&pool_reader)
        };

        let calls: [&'_ dyn Fn() -> (ChangeSet, bool); 3] = [
            &notarize,
            &make_block,
            &validate,
        ];

        let (changeset, to_broadcast) = self.schedule.call_next(&calls);

        (changeset, to_broadcast)
    }
}


fn add_all_to_validated(messages: Vec<ConsensusMessage>) -> ChangeSet {
    messages
        .into_iter()
        .map(|msg| ChangeAction::AddToValidated(msg))
        .collect()
}

fn add_to_validated(msg: Option<ConsensusMessage>) -> ChangeSet {
    msg.map(|msg| ChangeAction::AddToValidated(msg).into())
        .unwrap_or_default()
}
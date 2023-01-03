use crate::{
    consensus_layer::{
        pool_reader::PoolReader, artifacts::ConsensusMessage, consensus_subcomponents::{aggregator::{
            aggregate, 
            Notarization, NotarizationContent, 
            Finalization, FinalizationContent, 
        }, notary::NotarizationShareContent}
    },
    crypto::{Signed}, SubnetParams
};
use super::{finalizer::FinalizationShareContent, notary::NotarizationShareContentCOD};

/// A finalization share is a multi-signature share on a finalization content.
/// If sufficiently many replicas create finalization shares, the shares can be
/// aggregated into a full finalization.
pub type FinalizationShare = Signed<FinalizationShareContent, u8>;

pub struct Acknowledger {
    node_id: u8,
    subnet_params: SubnetParams,
}

impl Acknowledger {
    #[allow(clippy::too_many_arguments)]
    pub fn new(node_id: u8, subnet_params: SubnetParams) -> Self {
        Self {
            node_id,
            subnet_params,
        }
    }

    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        // println!("\n########## Acknowledger ##########");
        let height = pool.get_notarized_height() + 1;
        let notarization_shares = pool.get_notarization_shares(height);
        let grouped_shares = aggregate(notarization_shares);
        grouped_shares.into_iter().filter_map(|(notarization_content, committee)| {
            if let NotarizationShareContent::COD(notarization_content) = notarization_content {
                // CoD rule 2: acknowledge (FP-finalize) only blocks whose parent is finalized
                if 
                    notarization_content.is_ack == true && 
                    committee.len() >= (self.subnet_params.total_nodes_number - self.subnet_params.disagreeing_nodes_number) as usize &&
                    is_parent_finalized(pool, &notarization_content)
                {
                    println!("\nAcknowledgement of block with hash: {} at height {} by committee: {:?}", notarization_content.block.get_ref(), notarization_content.height, committee);
                    Some(notarization_content)
                }
                else {
                    None
                }.map(|notarization_content| {
                    vec![
                        ConsensusMessage::Notarization(
                            Notarization {
                                content: NotarizationContent {
                                    height: notarization_content.height,
                                    block: notarization_content.block.clone(),
                                },

                                signature: 0,   // committee signature
                            }
                        ),
                        ConsensusMessage::Finalization(
                            Finalization {
                                content: FinalizationContent {
                                    height: notarization_content.height,
                                    block: notarization_content.block,
                                },
                                signature: 10,   // committee signature
                            }
                        )
                    ]
                })
            }
            else {
                None
            }
        }).flatten().collect()
    }
}

fn is_parent_finalized(pool: &PoolReader<'_>, notarization_content: &NotarizationShareContentCOD) -> bool {
    let parent_hash = notarization_content.block_parent_hash.clone();
    let parent_height = notarization_content.height - 1;
    if parent_height == 0 {
        return true;    // genesis block is finalized
    }
    match pool.get_finalized_block_hash_at_height(parent_height) {
        Some(finalized_block_hash) => finalized_block_hash == parent_hash,
        None => false,
    }
}
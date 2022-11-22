use crate::{
    consensus_layer::{
        pool_reader::PoolReader, artifacts::ConsensusMessage, consensus_subcomponents::aggregator::{
            aggregate, 
            Notarization, NotarizationContent, 
            Finalization, FinalizationContent, 
        }
    },
    crypto::{Signed}, SubnetParams
};
use super::finalizer::FinalizationShareContent;

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
        println!("\n########## Acknowledger ##########");
        let height = pool.get_notarized_height() + 1;
        let notarization_shares = pool.get_notarization_shares(height);
        let grouped_shares = aggregate(notarization_shares);
        grouped_shares.into_iter().filter_map(|(notarization_content, committee)| {
            if notarization_content.is_ack == true && committee.len() >= (self.subnet_params.total_nodes_number - self.subnet_params.faulty_nodes_number) as usize {
                println!("Acknowledgement of block with hash: {} at height {} by committee: {:?}", notarization_content.block.get_ref(), notarization_content.height, committee);
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
        }).flatten().collect()
    }
}
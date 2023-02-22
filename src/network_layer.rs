use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock}, time::Duration, ops::Sub,
};

use async_std::task;
use crossbeam_channel::Receiver;
use futures::{prelude::stream::StreamExt, stream::SelectNextSome};
use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity::Keypair,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    multiaddr::Protocol,
    multihash::Multihash,
    swarm::SwarmEvent,
    NetworkBehaviour, PeerId, Swarm,
};
use serde::{Deserialize, Serialize};

use crate::{
    artifact_manager::ArtifactProcessorManager,
    consensus_layer::{
        artifacts::{ConsensusMessage, UnvalidatedArtifact},
        height_index::Height,
    },
    time_source::{SysTimeSource, TimeSource, system_time_now, Time},
    SubnetParams, HeightMetrics, crypto::CryptoHash, ArtifactDelayInfo,
};

// We create a custom network behaviour that combines floodsub and mDNS.
// Use the derive to generate delegating NetworkBehaviour impl.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent")]
pub struct P2PBehaviour {
    floodsub: Floodsub,
    mdns: Mdns,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum OutEvent {
    Floodsub(FloodsubEvent),
    Mdns(MdnsEvent),
}

impl From<MdnsEvent> for OutEvent {
    fn from(v: MdnsEvent) -> Self {
        Self::Mdns(v)
    }
}

impl From<FloodsubEvent> for OutEvent {
    fn from(v: FloodsubEvent) -> Self {
        Self::Floodsub(v)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    ConsensusMessage(ConsensusMessage),
    KeepAliveMessage,
}

pub struct Peer {
    replica_number: u8,
    id: PeerId,
    round: usize,
    rank: u64,
    floodsub_topic: Topic,
    swarm: Swarm<P2PBehaviour>,
    receiver_outgoing_artifact: Receiver<ConsensusMessage>,
    time_source: Arc<SysTimeSource>,
    manager: ArtifactProcessorManager,
    proposals_timings: Arc<RwLock<BTreeMap<CryptoHash, ArtifactDelayInfo>>>,
}

impl Peer {
    pub async fn new(
        replica_number: u8,
        subnet_params: SubnetParams,
        topic: &str,
        finalization_times: Arc<RwLock<BTreeMap<Height, Option<HeightMetrics>>>>,
        proposals_timings: Arc<RwLock<BTreeMap<CryptoHash, ArtifactDelayInfo>>>,
    ) -> Self {
        let starting_round = 1;
        // Create a random PeerId
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        // Set up an encrypted DNS-enabled TCP Transport
        let transport = libp2p::development_transport(local_key).await.unwrap();

        // Create a Floodsub topic
        let floodsub_topic = Topic::new(topic);

        // channel used to transmit locally generated artifacts from the consensus layer to the network layer so that they can be broadcasted to other peers
        let (sender_outgoing_artifact, receiver_outgoing_artifact) =
            crossbeam_channel::unbounded::<ConsensusMessage>();

        // Initialize the time source.
        let time_source = Arc::new(SysTimeSource::new());

        // Create a Swarm to manage peers and events
        let local_peer = Self {
            replica_number,
            id: local_peer_id,
            round: starting_round,
            rank: 0, // updated after Peer object is instantiated
            floodsub_topic: floodsub_topic.clone(),
            swarm: {
                let mdns = task::block_on(Mdns::new(MdnsConfig::default())).unwrap();
                let mut behaviour = P2PBehaviour {
                    floodsub: Floodsub::new(local_peer_id),
                    mdns,
                };

                behaviour.floodsub.subscribe(floodsub_topic);
                Swarm::new(transport, behaviour, local_peer_id)
            },
            receiver_outgoing_artifact,
            time_source: time_source.clone(),
            manager: ArtifactProcessorManager::new(
                replica_number,
                subnet_params,
                time_source,
                sender_outgoing_artifact,
                finalization_times,
            ),
            proposals_timings,
        };
        // println!(
        //     "Local node initialized with number: {} and peer id: {:?}",
        //     local_peer.replica_number, local_peer_id
        // );
        local_peer
    }

    pub fn listen_for_dialing(&mut self) {
        self.swarm
            .listen_on(
                "/ip4/0.0.0.0/tcp/0"
                    .parse()
                    .expect("can get a local socket"),
            )
            .expect("swarm can be started");
    }

    pub fn broadcast_message(&mut self, mean_simulated_network_delay: u64) {
        match self.receiver_outgoing_artifact.try_recv() {
            Ok(outgoing_artifact) => {
                // println!("Broadcasted locally generated artifact");
                match &outgoing_artifact {
                    ConsensusMessage::BlockProposal(block_proposal) => {
                        let block_hash = block_proposal.content.hash.clone();
                        let artifact_delay_info = ArtifactDelayInfo {
                            // recording timestamp as if it was sent "mean_simulated_network_delay" milliseconds before
                            // this is because in order to simulate the network delay, every artifact is delayed by "mean_simulated_network_delay" before being broadcasted
                            sent: Some(Time::from_duration(system_time_now().sub(Time::from_duration(Duration::from_millis(mean_simulated_network_delay))))),
                            received: None,
                        };
                        self.proposals_timings
                            .write()
                            .unwrap()
                            .insert(block_hash, artifact_delay_info);
                    },
                    _ => (),
                    
                }
                self.swarm.behaviour_mut().floodsub.publish(
                    self.floodsub_topic.clone(),
                    serde_json::to_string::<Message>(&Message::ConsensusMessage(outgoing_artifact))
                        .unwrap(),
                );
            }
            Err(_) => {
                self.swarm.behaviour_mut().floodsub.publish(
                    self.floodsub_topic.clone(),
                    serde_json::to_string::<Message>(&Message::KeepAliveMessage).unwrap(),
                );
            }
        }
    }

    pub fn get_next_event(&mut self) -> SelectNextSome<'_, Swarm<P2PBehaviour>> {
        self.swarm.select_next_some()
    }

    pub fn match_event<T>(&mut self, event: SwarmEvent<OutEvent, T>) {
        match event {
            SwarmEvent::NewListenAddr { mut address, .. } => {
                address.push(Protocol::P2p(
                    Multihash::from_bytes(&self.id.to_bytes()[..]).unwrap(),
                ));
                // println!("Listening on {:?}", address);
            }
            SwarmEvent::Behaviour(OutEvent::Floodsub(FloodsubEvent::Message(floodsub_message))) => {
                let floodsub_content = String::from_utf8_lossy(&floodsub_message.data);
                let message =
                    serde_json::from_str::<Message>(&floodsub_content).expect("can parse artifact");
                self.handle_incoming_message(message);
            }
            SwarmEvent::Behaviour(OutEvent::Mdns(MdnsEvent::Discovered(list))) => {
                for (peer, _) in list {
                    self.swarm
                        .behaviour_mut()
                        .floodsub
                        .add_node_to_partial_view(peer);
                }
            }
            SwarmEvent::Behaviour(OutEvent::Mdns(MdnsEvent::Expired(list))) => {
                for (peer, _) in list {
                    if !self.swarm.behaviour_mut().mdns.has_node(&peer) {
                        self.swarm
                            .behaviour_mut()
                            .floodsub
                            .remove_node_from_partial_view(&peer);
                    }
                }
                // println!("Ignoring Mdns expired event");
            }
            _ => {
                // println!("Unhandled swarm event");
            }
        }
    }

    pub fn handle_incoming_message(&mut self, message_variant: Message) {
        match message_variant {
            Message::KeepAliveMessage => (),
            Message::ConsensusMessage(consensus_message) => {
                match &consensus_message {
                    ConsensusMessage::BlockProposal(block_proposal) => {
                        let block_hash = block_proposal.content.hash.clone();
                        let artifact_delay_info = ArtifactDelayInfo {
                            sent: None,
                            received: Some(system_time_now()),
                        };
                        self.proposals_timings
                            .write()
                            .unwrap()
                            .insert(block_hash, artifact_delay_info);
                    },
                    _ => ()
                };
                self.manager.on_artifact(
                    UnvalidatedArtifact::new(consensus_message, self.time_source.get_relative_time()),
                );
            }
        }
    }
}

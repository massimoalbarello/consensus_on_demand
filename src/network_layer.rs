use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, RwLock}, time::Duration,
};
use std::thread::sleep;
use crossbeam_channel::{Receiver, Sender};
use futures::{prelude::stream::StreamExt, stream::SelectNextSome};
use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity::Keypair,
    multiaddr::Protocol,
    multihash::Multihash,
    swarm::SwarmEvent,
    NetworkBehaviour, PeerId, Swarm, Multiaddr,
};
use serde::{Deserialize, Serialize};

use crate::{
    artifact_manager::ArtifactProcessorManager,
    consensus_layer::{
        artifacts::{ConsensusMessage, UnvalidatedArtifact},
        height_index::Height, consensus_subcomponents::{block_maker::{BlockProposal, Block, Payload}, notary::{NotarizationShareContentICC, NotarizationShareContentCOD, NotarizationShareContent}},
    },
    time_source::{SysTimeSource, TimeSource, system_time_now},
    SubnetParams, HeightMetrics, crypto::{CryptoHash, Hashed, Signed}, ArtifactDelayInfo,
};

// We create a custom network behaviour that combines floodsub and mDNS.
// Use the derive to generate delegating NetworkBehaviour impl.
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "OutEvent")]
pub struct P2PBehaviour {
    floodsub: Floodsub,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum OutEvent {
    Floodsub(FloodsubEvent),
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
    first_block_delay: u64,
    can_start_proposing: bool,
    subnet_params: SubnetParams,
    round: usize,
    rank: u64,
    floodsub_topic: Topic,
    swarm: Swarm<P2PBehaviour>,
    peers_addresses: String,
    subscribed_peers: BTreeSet<PeerId>,
    connected_peers: BTreeSet<PeerId>,
    receiver_outgoing_artifact: Receiver<ConsensusMessage>,
    sender_outgoing_artifact: Sender<ConsensusMessage>,
    finalization_times: Arc<RwLock<BTreeMap<Height, Option<HeightMetrics>>>>,
    time_source: Arc<SysTimeSource>,
    manager: Option<ArtifactProcessorManager>,
}

impl Peer {
    pub async fn new(
        replica_number: u8,
        peers_addresses: String,
        subnet_params: SubnetParams,
        first_block_delay: u64,
        topic: &str,
        finalization_times: Arc<RwLock<BTreeMap<Height, Option<HeightMetrics>>>>,
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
            first_block_delay,
            can_start_proposing: false,
            subnet_params,
            round: starting_round,
            rank: 0, // updated after Peer object is instantiated
            floodsub_topic: floodsub_topic.clone(),
            swarm: {
                let mut behaviour = P2PBehaviour {
                    floodsub: Floodsub::new(local_peer_id),
                };

                behaviour.floodsub.subscribe(floodsub_topic);
                Swarm::new(transport, behaviour, local_peer_id)
            },
            peers_addresses,
            subscribed_peers: BTreeSet::new(),
            connected_peers: BTreeSet::new(),
            receiver_outgoing_artifact,
            sender_outgoing_artifact,
            finalization_times,
            time_source,
            manager: None,
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
                "/ip4/0.0.0.0/tcp/56789"
                    .parse()
                    .expect("can get a local socket"),
            )
            .expect("swarm can be started");
    }

    pub fn broadcast_message(&mut self) {
        match self.receiver_outgoing_artifact.try_recv() {
            Ok(outgoing_artifact) => {
                if self.replica_number == 1 {
                    match &outgoing_artifact {
                        ConsensusMessage::BlockProposal(proposal) => {
                            if proposal.content.value.height == 1 {
                                sleep(Duration::from_millis(500));
                            }
                        },
                        ConsensusMessage::NotarizationShare(share) => {
                            match &share.content {
                                NotarizationShareContent::COD(ack) => {
                                    if ack.height == 1 {
                                        println!("Rebroadcasting first block proposal");
                                        self.swarm.behaviour_mut().floodsub.publish(
                                            self.floodsub_topic.clone(),
                                            serde_json::to_string::<Message>(&Message::ConsensusMessage(ConsensusMessage::BlockProposal(Signed {
                                                content: Hashed {
                                                    hash: String::from("426d3a77ace30d95db82aaaa9c49dbb6718bfbf106968ae0218f0f588871e229"),
                                                    value: Block {
                                                        parent: String::from("8c43f94e4759170f3b528ba6ff62171f4d26fd12ca4f4cca1da81a6534746715"),
                                                        payload: Payload::new(),
                                                        height: 1,
                                                        rank: 0,
                                                    },
                                                },
                                                signature: self.replica_number,
                                            })))
                                                .unwrap(),
                                        );
                                    }
                                }
                                NotarizationShareContent::ICC(share) => {
                                    if share.height == 1 {
                                        // self.swarm.behaviour_mut().floodsub.publish(
                                        //     self.floodsub_topic.clone(),
                                        //     serde_json::to_string::<Message>(&Message::ConsensusMessage())
                                        //         .unwrap(),
                                        // );
                                    }
                                }
                            }
                        },
                        _ => (),
                    }
                }
                // println!("\nBroadcasted locally generated artifact: {:?}", outgoing_artifact);
                self.swarm.behaviour_mut().floodsub.publish(
                    self.floodsub_topic.clone(),
                    serde_json::to_string::<Message>(&Message::ConsensusMessage(outgoing_artifact))
                        .unwrap(),
                );
            }
            Err(_) => {
                // println!("Sending keepalive");
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
                println!("Local peer ID: {:?}", self.id);
                if self.replica_number == 1 {
                    for peer_address in self.peers_addresses.split(',') {
                        let remote_peer_multiaddr: Multiaddr = peer_address.parse().expect("valid address");
                        let remote_peer_id = PeerId::try_from_multiaddr(&remote_peer_multiaddr).expect("multiaddress with peer ID");
                        if !self.subscribed_peers.contains(&remote_peer_id) {
                            self.swarm.dial(remote_peer_multiaddr.clone()).expect("known peer");
                            self.swarm
                                .behaviour_mut()
                                .floodsub
                                .add_node_to_partial_view(remote_peer_id);
                            self.subscribed_peers.insert(remote_peer_id);
                            println!("Dialed remote peer: {:?} and added to broadcast list", peer_address);
                        }
                    }
                }
            }
            SwarmEvent::Behaviour(OutEvent::Floodsub(floodsub_event)) => {
                match floodsub_event {
                    FloodsubEvent::Message(floodsub_message) => {
                        let floodsub_content = String::from_utf8_lossy(&floodsub_message.data);
                        let message =
                            serde_json::from_str::<Message>(&floodsub_content).expect("can parse artifact");
                        self.handle_incoming_message(message);
                    },
                    FloodsubEvent::Subscribed { peer_id: remote_peer_id, .. } => {
                        if !self.subscribed_peers.contains(&remote_peer_id) {
                            if self.replica_number != 1 {
                                self.swarm
                                    .behaviour_mut()
                                    .floodsub
                                    .add_node_to_partial_view(remote_peer_id);
                                self.subscribed_peers.insert(remote_peer_id);
                                println!("Added peer with ID: {:?} to broadcast list", remote_peer_id);
                            }
                        }
                    },
                    _ => println!("Unhandled floodsub event"), 

                }
                
            },
            SwarmEvent::ConnectionEstablished {peer_id: remote_peer_id, ..} => {
                if !self.connected_peers.contains(&remote_peer_id) {
                    println!("Connection established with remote peer: {:?}", remote_peer_id);
                    self.connected_peers.insert(remote_peer_id);
                }
                if self.connected_peers.len() == (self.subnet_params.total_nodes_number-1) as usize {
                    println!("Can start proposing");
                    self.can_start_proposing = true;
                    self.manager = Some(ArtifactProcessorManager::new(
                        self.replica_number,
                        self.subnet_params.clone(),
                        Arc::clone(&self.time_source),
                        self.sender_outgoing_artifact.clone(),
                        Arc::clone(&self.finalization_times),
                    ));
                }
            },
            _ => println!("unhandled swarm event"),
        }
    }

    pub fn handle_incoming_message(&mut self, message_variant: Message) {
        match message_variant {
            Message::KeepAliveMessage => (),
            Message::ConsensusMessage(consensus_message) => {
                // println!("\nReceived message: {:?}", consensus_message);
                match &self.manager {
                    Some(manager) => {
                        manager.on_artifact(
                            UnvalidatedArtifact::new(consensus_message, self.time_source.get_relative_time())
                        );
                    },
                    None => (),
                };
            }
        }
    }

    pub fn can_start_proposing(&self) -> bool {
        self.can_start_proposing
    }
}

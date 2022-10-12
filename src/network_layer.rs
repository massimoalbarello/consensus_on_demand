use async_std::task;
use futures::{prelude::stream::StreamExt, stream::SelectNextSome};
use libp2p::{
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity::Keypair,
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    swarm::SwarmEvent,
    NetworkBehaviour, PeerId, Swarm,
};

use crate::artifact_manager::ArtifactProcessorManager;
use crate::consensus_layer::artifacts::{Artifact, Block, UnvalidatedArtifact};

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

pub struct Peer {
    node_number: u8,
    round: usize,
    rank: u64,
    floodsub_topic: Topic,
    swarm: Swarm<P2PBehaviour>,
    manager: ArtifactProcessorManager,
}

impl Peer {
    pub async fn new(node_number: u8, topic: &str) -> Self {
        let starting_round = 1;
        // Create a random PeerId
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        // Set up an encrypted DNS-enabled TCP Transport
        let transport = libp2p::development_transport(local_key).await.unwrap();

        // Create a Floodsub topic
        let floodsub_topic = Topic::new(topic);

        // Create a Swarm to manage peers and events
        let local_peer = Self {
            node_number,
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
            manager: ArtifactProcessorManager::new(),
        };
        println!(
            "Local node initialized with number: {} and peer id: {:?}",
            local_peer.node_number, local_peer_id
        );
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

    pub fn broadcast_block(&mut self) {
        
        let block = Block::new(
            self.round as u64,
            self.rank,
            self.node_number,
            String::from("parent_hash"),
            format!("Block: {}_{}", self.round, self.node_number),
        );
        println!("Sent block at height {}", block.height);
        self.swarm.behaviour_mut().floodsub.publish(
            self.floodsub_topic.clone(),
            serde_json::to_string::<Artifact>(&Artifact::Block(block.clone())).unwrap(),
        );
    }

    pub fn keep_alive(&mut self) {
        self.swarm.behaviour_mut().floodsub.publish(
            self.floodsub_topic.clone(),
            serde_json::to_string::<Artifact>(&Artifact::KeepAliveMessage).unwrap(),
        );
    }

    pub fn get_next_event(&mut self) -> SelectNextSome<'_, Swarm<P2PBehaviour>> {
        self.swarm.select_next_some()
    }

    pub fn match_event<T>(&mut self, event: SwarmEvent<OutEvent, T>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on {:?}", address);
            }
            SwarmEvent::Behaviour(OutEvent::Floodsub(FloodsubEvent::Message(message))) => {
                let message_content = String::from_utf8_lossy(&message.data);
                let artifact = serde_json::from_str::<Artifact>(&message_content)
                    .expect("can parse artifact");
                self.handle_incoming_artifact(artifact);
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

    pub fn handle_incoming_artifact(&mut self, artifact_variant: Artifact) {
        match artifact_variant {
            Artifact::KeepAliveMessage => println!("Received keep alive message"),
            _ => self.manager.on_artifact(UnvalidatedArtifact::new(artifact_variant)),
        }
    }
}

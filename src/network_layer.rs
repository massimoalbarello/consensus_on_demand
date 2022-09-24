pub mod networking {

    use async_std::{fs::File, prelude::*, task};
    use futures::{channel::mpsc::Sender, prelude::stream::StreamExt, stream::SelectNextSome};
    use libp2p::{
        floodsub::{Floodsub, FloodsubEvent, Topic},
        identity::Keypair,
        mdns::{Mdns, MdnsConfig, MdnsEvent},
        swarm::SwarmEvent,
        NetworkBehaviour, PeerId, Swarm,
    };

    use crate::consensus_layer::blockchain::{
        Artifact, Block, Blockchain, InputPayloads, NotarizationShare,
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

    pub struct Peer {
        node_number: u8,
        round: usize,
        rank: u8,
        floodsub_topic: Topic,
        swarm: Swarm<P2PBehaviour>,
        blockchain: Blockchain,
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
                rank: (starting_round as u8 + node_number - 2) % 4,
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
                blockchain: Blockchain::new(),
            };
            println!(
                "Local node initialized with number: {} and peer id: {:?}",
                local_peer.node_number, local_peer_id
            );
            println!(
                "Local node has rank: {} in round: {}",
                local_peer.rank, local_peer.round
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

        pub fn create_block(&mut self, mut tx: Sender<Block>) {
            // attach new block to last block in finalized blockchain
            if self.rank == 0 {
                let parent_hash = self
                    .blockchain
                    .block_tree
                    .get_parent_hash(self.round as u64, self.blockchain.finalized_chain_index)
                    .expect("can get parent hash");
                println!("Appending block to parent with hash: {}", parent_hash);
                let round = self.round;
                let local_peer_rank = self.rank;
                let local_node_number = self.node_number;
                task::spawn(async move {
                    // mine block in a separate non-blocking task
                    match get_next_block(round, local_peer_rank, local_node_number, parent_hash)
                        .await
                    {
                        Some(block) => tx.try_send(block).expect("can push into channel"), // push block into channel so that it can later be broadcasted
                        None => (),
                    };
                });
            } else {
                println!(
                    "Cannot propose in round: {} as local peer has rank: {}",
                    self.round, self.rank
                );
            }
        }

        pub fn broadcast_block(&mut self, block: Option<Block>) {
            match block {
                Some(block) => {
                    println!("Sent block at height {}", block.height);
                    self.swarm.behaviour_mut().floodsub.publish(
                        self.floodsub_topic.clone(),
                        serde_json::to_string::<Artifact>(&Artifact::Block(block.clone())).unwrap(),
                    );
                    self.blockchain
                        .block_tree
                        .create_child_at_index(self.blockchain.finalized_chain_index, block);
                }
                None => (),
            }
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

        pub fn handle_incoming_artifact(&mut self, artifact_content: Artifact) {
            match artifact_content {
                Artifact::NotarizationShare(share) => {
                    println!("\nReceived notarization share for block with hash: {} at height {} from peer with node number: {}", &share.block_hash, share.block_height, share.from_node_number);
                    self.blockchain.block_tree.update_recvd_notarization_shares(
                        share.from_node_number,
                        &share.block_hash,
                        share.block_height,
                        self.round as u64,
                    );
                }
                Artifact::Block(block) => {
                    let block_height = block.height;
                    let block_hash = block.hash.clone();
                    let block_from_rank = block.from_rank;
                    println!(
                        "\nReceived block with hash: {} attached to: {} from peer with rank: {}",
                        &block_hash, &block.parent_hash, block.from_rank
                    );
                    // local peer always adds block to its block tree as it might later send a notarization share for it (once corresponding timer has expired)
                    self.blockchain
                        .block_tree
                        .create_child_at_index(self.blockchain.finalized_chain_index, block);
                    // for now local peer sends notarization share only if if receives block from leader of current round
                    // TODO: check if timer corresponding to rank has expired, if so broadcast notarization share
                    if block_from_rank == 0 {
                        // local peer updtates recvd_notarization_shares for the share it sends
                        // required as local peer does not receive the share it broadcasts to others
                        self.blockchain.block_tree.update_recvd_notarization_shares(
                            self.node_number,
                            &block_hash,
                            block_height,
                            self.round as u64,
                        );
                        self.send_notarization_share(block_height, block_hash);
                    }
                }
                Artifact::KeepAliveMessage => (),
            }
        }

        fn send_notarization_share(&mut self, block_height: u64, block_hash: String) {
            let notarization_share =
                NotarizationShare::new(self.node_number, block_height, block_hash.clone());
            self.swarm.behaviour_mut().floodsub.publish(
                self.floodsub_topic.clone(),
                serde_json::to_string::<Artifact>(&Artifact::NotarizationShare(notarization_share))
                    .unwrap(),
            );
            println!(
                "Sent notarization share for block with hash: {} at height: {}",
                block_hash, block_height
            );
        }
    }

    async fn get_next_block(
        height: usize,
        local_peer_rank: u8,
        local_node_number: u8,
        parent_hash: String,
    ) -> Option<Block> {
        match get_next_payload(height).await {
            Some(payload) => {
                // setting block id according to the length of the local blockchain
                let new_block = Block::new(
                    height as u64,
                    local_peer_rank,
                    local_node_number,
                    parent_hash,
                    payload,
                );
                Some(new_block)
            }
            None => {
                println!("No more payloads");
                None
            }
        }
    }

    async fn get_next_payload(round: usize) -> Option<String> {
        let input_payloads: InputPayloads = read_file("payloads_pool.txt").await;
        let next_payload = if round < input_payloads.len() {
            Some(input_payloads[round].clone())
        } else {
            None
        };
        next_payload
    }

    async fn read_file(path: &str) -> InputPayloads {
        let mut file = File::open(path).await.expect("txt file in path");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .await
            .expect("read content as string");

        let mut input_payloads: InputPayloads = vec![];
        for line in content.lines() {
            input_payloads.push(String::from(line));
        }
        input_payloads
    }
}

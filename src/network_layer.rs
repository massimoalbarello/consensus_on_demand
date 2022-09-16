pub mod networking {

    use async_std::{fs::File, prelude::*, task};
    use futures::{
        prelude::stream::StreamExt,
        stream::SelectNextSome,
    };
    use libp2p::{
        floodsub::{Floodsub, FloodsubEvent, Topic},
        identity::Keypair,
        mdns::{Mdns, MdnsConfig, MdnsEvent},
        swarm::SwarmEvent,
        Multiaddr, NetworkBehaviour, PeerId, Swarm,
    };

    use crate::consensus_layer::blockchain::{
        Blockchain,
        Block,
        InputPayloads,
    };

    // We create a custom network behaviour that combines floodsub and mDNS.
    // Use the derive to generate delegating NetworkBehaviour impl.
    #[derive(NetworkBehaviour)]
    #[behaviour(out_event = "OutEvent")]
    pub struct NetworkBehaviour {
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
        local_sn: usize,
        floodsub_topic: Topic,
        swarm:  Swarm<NetworkBehaviour>,
        blockchain: Blockchain,
    }

    impl Peer {
        pub async fn new(topic: &str) -> Self {
            // Create a random PeerId
            let local_key = Keypair::generate_ed25519();
            let local_peer_id = PeerId::from(local_key.public());
            println!("Local peer id: {:?}", local_peer_id);

            // Set up an encrypted DNS-enabled TCP Transport
            let transport = libp2p::development_transport(local_key).await.unwrap();

            // Create a Floodsub topic
            let floodsub_topic = Topic::new(topic);

            // Create a Swarm to manage peers and events
            Self {
                local_sn: 0,    // used to read next payload from local pool
                floodsub_topic: floodsub_topic.clone(),
                swarm: {
                    let mdns = task::block_on(Mdns::new(MdnsConfig::default())).unwrap();
                    let mut behaviour = NetworkBehaviour {
                        floodsub: Floodsub::new(local_peer_id),
                        mdns,
                    };

                    behaviour.floodsub.subscribe(floodsub_topic);
                    Swarm::new(transport, behaviour, local_peer_id)
                },
                blockchain: Blockchain::new(),
            }


        }

        pub fn dial_peer(&mut self, to_dial:String) {
            let addr: Multiaddr = to_dial.parse().unwrap();
            self.swarm.dial(addr).unwrap();
            println!("Dialed peer {:?}", to_dial);
        }

        pub fn listen_for_dialing(&mut self) {
            self.swarm.listen_on("/ip4/0.0.0.0/tcp/0"
                .parse().expect("can get a local socket")
            ).expect("swarm can be started");
        }

        pub async fn broadcast_block(&mut self) {
            match get_next_block(self).await {
                Some(block) => {
                    println!("Sent block with sequence number {}", block.id);
                    self.local_sn += 1;
                    self.swarm
                        .behaviour_mut()
                        .floodsub
                        .publish(self.floodsub_topic.clone(), serde_json::to_string(&block).unwrap())
                },
                None => println!("No more input blocks"),
            }
           
        }

        pub fn get_next_event(&mut self) -> SelectNextSome<'_, Swarm<NetworkBehaviour>> {
            self.swarm.select_next_some()
        }

        pub fn match_event<T>(&mut self, event: SwarmEvent<OutEvent, T>) {
            match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(OutEvent::Floodsub(
                    FloodsubEvent::Message(message)
                )) => handle_incoming_block(self, &String::from_utf8_lossy(&message.data), message.source),
                SwarmEvent::Behaviour(OutEvent::Mdns(
                    MdnsEvent::Discovered(list)
                )) => {
                    for (peer, _) in list {
                        self.swarm
                            .behaviour_mut()
                            .floodsub
                            .add_node_to_partial_view(peer);
                    }
                }
                SwarmEvent::Behaviour(OutEvent::Mdns(MdnsEvent::Expired(
                    list
                ))) => {
                    for (peer, _) in list {
                        if !self.swarm.behaviour_mut().mdns.has_node(&peer) {
                            self.swarm
                                .behaviour_mut()
                                .floodsub
                                .remove_node_from_partial_view(&peer);
                        }
                    }
                    // println!("Ignoring Mdns expired event");
                },
                _ => {
                    // println!("Unhandled swarm event");
                }
            }
        }
    }
    
    pub async fn get_next_block(local_peer: &mut Peer) -> Option<Block> {
        match get_next_payload(local_peer.local_sn).await {
            Some(payload) => {
                let previous_hash = local_peer.blockchain.blocks.last().expect("must have last block").hash.clone();
                let new_block = Block::new(local_peer.blockchain.blocks.len() as u64, previous_hash, payload);
                local_peer.blockchain.blocks.push(new_block.clone());
                Some(new_block)
            }
            None => None,
        }
    }

    async fn get_next_payload(local_sn: usize) -> Option<String> {
        let input_payloads: InputPayloads = read_file("payloads_pool.txt").await;
        let next_payload = if local_sn < input_payloads.len() {
            Some(input_payloads[local_sn].clone())
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

    pub fn handle_incoming_block(local_peer: &mut Peer, block_content: &str, block_source: PeerId) {
        println!("{}: {}", block_source, block_content);
        let block = serde_json::from_str::<Block>(block_content).expect("can parse block");
        local_peer.blockchain.try_add_block(block);
    }
}
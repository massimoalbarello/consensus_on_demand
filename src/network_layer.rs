pub mod networking {

    use async_std::task;
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

    // We create a custom network behaviour that combines floodsub and mDNS.
    // Use the derive to generate delegating NetworkBehaviour impl.
    #[derive(NetworkBehaviour)]
    #[behaviour(out_event = "OutEvent")]
    pub struct MyBehaviour {
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
        floodsub_topic: Topic,
        swarm:  Swarm<MyBehaviour>,

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
                floodsub_topic: floodsub_topic.clone(),
                swarm: {
                    let mdns = task::block_on(Mdns::new(MdnsConfig::default())).unwrap();
                    let mut behaviour = MyBehaviour {
                        floodsub: Floodsub::new(local_peer_id),
                        mdns,
                    };

                    behaviour.floodsub.subscribe(floodsub_topic);
                    Swarm::new(transport, behaviour, local_peer_id)
                },
            }


        }

        pub fn dial_peer(&mut self, to_dial:String) {
            let addr: Multiaddr = to_dial.parse().unwrap();
            self.swarm.dial(addr).unwrap();
            println!("Dialed peer {:?}", to_dial);
        }

        pub fn listen_for_dialing(&mut self) {
            self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();
        }

        pub fn broadcast(&mut self, line: &[u8]) {
            self.swarm
                .behaviour_mut()
                .floodsub
                .publish(self.floodsub_topic.clone(), line)
        }

        pub fn get_next_event(&mut self) -> SelectNextSome<'_, Swarm<MyBehaviour>> {
            self.swarm.select_next_some()
        }

        pub fn match_event<T>(&mut self, event: SwarmEvent<OutEvent, T>) {
            match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                SwarmEvent::Behaviour(OutEvent::Floodsub(
                    FloodsubEvent::Message(message)
                )) => {
                    println!(
                        "Received: '{:?}' from {:?}",
                        String::from_utf8_lossy(&message.data),
                        message.source
                    );
                }
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
                },
                _ => {}
            }
        }
    }
}
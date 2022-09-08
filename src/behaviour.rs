pub mod behaviour_config {

    use libp2p::{
        floodsub::{Floodsub, FloodsubEvent, Topic},
        mdns::{Mdns, MdnsEvent},
        NetworkBehaviour, PeerId, Swarm
    };

    // We create a custom network behaviour that combines floodsub and mDNS.
    // Use the derive to generate delegating NetworkBehaviour impl.
    #[derive(NetworkBehaviour)]
    #[behaviour(out_event = "OutEvent")]
    pub struct MyBehaviour {
        floodsub: Floodsub,
        mdns: Mdns,
    }

    impl MyBehaviour {
        pub fn new(local_peer_id: PeerId, mdns: Mdns) -> MyBehaviour{
            MyBehaviour {
                floodsub: Floodsub::new(local_peer_id),
                mdns,
            }
        }

        pub fn subscribe(&mut self, floodsub_topic: Topic) {
            self.floodsub.subscribe(floodsub_topic);
        }

        pub fn get_floodsub_mutable_reference(swarm: &mut Swarm<MyBehaviour>) -> &mut Floodsub {
            &mut swarm.behaviour_mut().floodsub
        }

        pub fn get_mdns_mutable_reference(swarm: &mut Swarm<MyBehaviour>) -> &mut Mdns {
            &mut swarm.behaviour_mut().mdns
        }

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

}
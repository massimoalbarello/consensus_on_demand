pub mod behaviour_config {

    use libp2p::kad::record::store::MemoryStore;
    use libp2p::kad::{
        Kademlia, KademliaEvent
    };
    use libp2p::{
        mdns::{Mdns, MdnsEvent},
        NetworkBehaviour,
        Swarm
    };

    // We create a custom network behaviour that combines Kademlia and mDNS.
    #[derive(NetworkBehaviour)]
    #[behaviour(out_event = "MyBehaviourEvent")]
    pub struct MyBehaviour {
        kademlia: Kademlia<MemoryStore>,
        mdns: Mdns,
    }

    impl MyBehaviour {
        pub fn new(kademlia: Kademlia<MemoryStore>, mdns: Mdns) -> MyBehaviour {
            MyBehaviour { kademlia, mdns }
        }
    }

    pub enum MyBehaviourEvent {
        Kademlia(KademliaEvent),
        Mdns(MdnsEvent),
    }

    impl From<KademliaEvent> for MyBehaviourEvent {
        fn from(event: KademliaEvent) -> Self {
            MyBehaviourEvent::Kademlia(event)
        }
    }

    impl From<MdnsEvent> for MyBehaviourEvent {
        fn from(event: MdnsEvent) -> Self {
            MyBehaviourEvent::Mdns(event)
        }
    }

    pub fn get_kademlia_behaviour_mut_reference(swarm: &mut Swarm<MyBehaviour>) -> &mut Kademlia<MemoryStore>{
        &mut swarm.behaviour_mut().kademlia
    }

}
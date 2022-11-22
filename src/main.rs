use async_std::{io, task::sleep};
use futures::{
    future::FutureExt,
    prelude::{stream::StreamExt, *},
    select,
};
use std::time::Duration;

pub mod network_layer;
use crate::network_layer::Peer;
pub mod artifact_manager;
pub mod crypto;
pub mod consensus_layer;
pub mod time_source;

#[derive(Clone)]
pub struct SubnetParams {
    total_nodes_number: u8,
    byzantine_nodes_number: u8,
    faulty_nodes_number: u8,
}

impl SubnetParams {
    fn new(n: u8, f: u8, p: u8) -> Self {
        Self {
            total_nodes_number: n,
            byzantine_nodes_number: f,
            faulty_nodes_number: p
        }
    }
}

async fn broadcast_message_future() {
    sleep(Duration::from_millis(100)).await;
}

#[async_std::main]
async fn main() {
    let mut cmd_line_args = std::env::args();
    cmd_line_args.next();    // ignore first parameter from command line
    // get local replica number
    match cmd_line_args.next() {
        Some(replica_number) => {
            let replica_number: u8 = replica_number
                .parse()
                .expect("cannot parse input from command line into replica number");
            // get total number of nodes in the subnet
            match cmd_line_args.next() {
                Some(n) => {
                    let n: u8 = n
                        .parse()
                        .expect("cannot parse input from command line into total number of nodes");
                    // get number of byzantine nodes in the subnet
                    match cmd_line_args.next() {
                        Some(f) => {
                            let f: u8 = f
                                .parse()
                                .expect("cannot parse input from command line into number of byzantine nodes");
                            // get number of faulty nodes in the subnet
                            match cmd_line_args.next() {
                                Some(p) => {
                                    let p: u8 = p
                                        .parse()
                                        .expect("cannot parse input from command line into number of faulty nodes");
                                    let mut my_peer = Peer::new(replica_number, SubnetParams::new(n, f, p), "gossip_blocks").await;

                                    // Listen on all interfaces and whatever port the OS assigns
                                    my_peer.listen_for_dialing();
                        
                                    // Read full lines from stdin
                                    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();
                        
                                    // Process events
                                    loop {
                                        select! {
                                            _ = stdin.select_next_some() => (),
                                            _ = broadcast_message_future().fuse() => {
                                                // prevent Mdns expiration event by periodically broadcasting keep alive messages to peers
                                                // if any locally generated artifact, broadcast it
                                                my_peer.broadcast_message();
                                            },
                                            event = my_peer.get_next_event() => my_peer.match_event(event),
                                        }
                                    }
                                },
                                None => panic!("must receive number of faulty nodes from the command line"),
                            }
                        },
                        None => panic!("must receive number of byzantine nodes from the command line"),
                    }
                },
                None => panic!("must receive total number of nodes from the command line"),
            }
        }
        None => panic!("must receive replica number from command line"),
    }
}
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

async fn keep_alive_future() {
    sleep(Duration::new(5, 0)).await;
}

#[async_std::main]
async fn main() {
    // get local peer id
    match std::env::args().nth(1) {
        Some(node_number) => {
            let node_number: u8 = node_number
                .parse()
                .expect("cannot parse input from command line into node number");
            let mut my_peer = Peer::new(node_number, "gossip_blocks").await;

            // Listen on all interfaces and whatever port the OS assigns
            my_peer.listen_for_dialing();

            // Read full lines from stdin
            let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

            // Process events
            loop {
                select! {
                    _ = stdin.select_next_some() => my_peer.broadcast_block(),
                    _ = keep_alive_future().fuse() => {
                        my_peer.keep_alive(); // prevent Mdns expiration event by periodically sending keep alive messages to peers,
                    },
                    event = my_peer.get_next_event() => my_peer.match_event(event),
                }
            }
        }
        None => panic!("Must receive input from command line"),
    }
}

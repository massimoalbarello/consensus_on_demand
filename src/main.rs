use async_std::{io, task::sleep};
use futures::{
    channel::mpsc,
    future::FutureExt,
    prelude::{stream::StreamExt, *},
    select,
};
use std::time::Duration;

pub mod network_layer;
use crate::network_layer::networking::Peer;
pub mod block_tree;
pub mod consensus_layer;

async fn keep_alive_future() {
    sleep(Duration::new(1, 0)).await;
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

            let (tx, mut rx) = mpsc::channel(1);

            let mut next_proposed_block_height: u32 = 1; // HP: each peer will not propose two blocks at the same height

            // Process events
            loop {
                select! {
                    _ = stdin.select_next_some() => my_peer.create_block(next_proposed_block_height, tx.clone()),
                    _ = keep_alive_future().fuse() => {
                        // if there is a block in the channel, broadcast it
                        // otherwise, send keep alive message
                        match rx.try_next() {
                            Ok(block) => {
                                my_peer.broadcast_block(next_proposed_block_height, block);
                                next_proposed_block_height += 1;
                            },
                            Err(_) => my_peer.keep_alive(), // prevent Mdns expiration event by periodically sending keep alive messages to peers,
                        }
                    },
                    event = my_peer.get_next_event() => my_peer.match_event(event),
                }
            }
        },
        None => panic!("Must receive input from command line"),
    }
}

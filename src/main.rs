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
pub mod consensus_layer;
pub mod block_tree;

async fn keep_alive_future() {
    sleep(Duration::new(1, 0)).await;
}

#[async_std::main]
async fn main() {
    let mut my_peer = Peer::new("gossip_blocks").await;

    // Dial another node if specified
    if let Some(to_dial) = std::env::args().nth(1) {
        my_peer.dial_peer(to_dial);
    }

    // Listen on all interfaces and whatever port the OS assigns
    my_peer.listen_for_dialing();

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

    let (tx, mut rx) = mpsc::channel(1);

    // Process events
    loop {
        select! {
            _ = stdin.select_next_some() => my_peer.create_block(tx.clone()),
            _ = keep_alive_future().fuse() => {
                // if there is a block in the channel, broadcast it
                // otherwise, send keep alive message
                match rx.try_next() {
                    Ok(block) => my_peer.broadcast_block(block),
                    Err(_) => my_peer.keep_alive(), // prevent Mdns expiration event by periodically sending keep alive messages to peers,
                }
            },
            event = my_peer.get_next_event() => my_peer.match_event(event),
        }
    }
}

use async_std::io;
use futures::{
    select,
    prelude::{stream::StreamExt, *}
};

pub mod network_layer;
use crate::network_layer::networking::Peer;

pub mod consensus_layer;

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

    // Process events
    loop {
        select! {
            _ = stdin.select_next_some() => my_peer.broadcast_block().await,
            event = my_peer.get_next_event() => my_peer.match_event(event),
        }
    }
}
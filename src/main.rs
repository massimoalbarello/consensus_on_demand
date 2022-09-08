use async_std::io;
use futures::{
    select,
    prelude::{stream::StreamExt, *}
};

pub mod network_layer;
use crate::network_layer::networking::Peer;

#[async_std::main]
async fn main() {
    let mut peer = Peer::new("chat").await;

    // Reach out to another node if specified
    if let Some(to_dial) = std::env::args().nth(1) {
        peer.dial_peer(to_dial);
    }

    // Listen on all interfaces and whatever port the OS assigns
    peer.listen_for_dialing();

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

    // Kick it off
    loop {
        select! {
            line = stdin.select_next_some() => peer.broadcast(line.expect("Stdin not to close").as_bytes()),
            event = peer.get_next_event() => peer.match_event(event),
        }
    }
}
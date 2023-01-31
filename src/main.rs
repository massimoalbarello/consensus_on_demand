use async_std::{io, task::sleep, fs::File};
use futures::{
    future::FutureExt,
    prelude::{stream::StreamExt, *},
    select,
};
use structopt::StructOpt;
use std::{time::Duration, sync::{RwLock, Arc}, collections::BTreeMap};
use serde::{Serialize, Deserialize};
use serde_json::to_string;

#[derive(Serialize, Deserialize, Debug)]
struct BenchmarkResult {
    results: BTreeMap<Height, Duration>,
}

pub mod network_layer;
use crate::{network_layer::Peer, time_source::{system_time_now, get_absolute_end_time}, consensus_layer::height_index::Height};
pub mod artifact_manager;
pub mod crypto;
pub mod consensus_layer;
pub mod time_source;


#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short, long)]
    r: u8,  // replica number
    #[structopt(short, long, default_value = "6")]
    n: u8,  // total number of nodes
    #[structopt(short, long, default_value = "1")]
    f: u8,  // number of byzantine nodes
    #[structopt(short, long, default_value = "1")]
    p: u8,  // number of disagreeing nodes 
    #[structopt(short, long)]
    cod: bool,  // enable Fast IC Consensus
    #[structopt(short, long, default_value = "300")]
    t: u64, // time to run replica
}

#[derive(Clone)]
pub struct SubnetParams {
    total_nodes_number: u8,
    byzantine_nodes_number: u8,
    disagreeing_nodes_number: u8,
    consensus_on_demand: bool,
}

impl SubnetParams {
    fn new(n: u8, f: u8, p: u8, cod: bool) -> Self {
        Self {
            total_nodes_number: n,
            byzantine_nodes_number: f,
            disagreeing_nodes_number: p,
            consensus_on_demand: cod,
        }
    }
}

async fn broadcast_message_future() {
    sleep(Duration::from_millis(100)).await;
}

#[async_std::main]
async fn main() {
    let opt = Opt::from_args();

    let finalizations_times = Arc::new(RwLock::new(BTreeMap::<Height, Duration>::new()));
    let cloned_finalization_times = Arc::clone(&finalizations_times);

    let mut my_peer = Peer::new(opt.r, SubnetParams::new(opt.n, opt.f, opt.p, opt.cod), "gossip_blocks", cloned_finalization_times).await;

    // Listen on all interfaces and whatever port the OS assigns
    my_peer.listen_for_dialing();

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

    let starting_time = system_time_now();
    let relative_duration = Duration::from_millis(opt.t * 1000);
    let absolute_end_time = get_absolute_end_time(starting_time, relative_duration);

    // Process events
    loop {
        // if !my_peer.manager.handle.as_ref().unwrap().is_finished() {
        if system_time_now() < absolute_end_time {
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
        else {
            println!("\nStopped replica");

            let benchmark_result = BenchmarkResult {
                results: finalizations_times.read().unwrap().clone(),
            };

            let encoded = to_string(&benchmark_result).unwrap();
            let mut file = File::create(format!("benchmark_result_{}.json", opt.r)).await.unwrap();
            file.write_all(encoded.as_bytes()).await.unwrap();

            break;
        }
    }

}
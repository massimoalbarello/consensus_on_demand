use async_std::{fs::File, io, task::sleep};
use futures::{
    future::FutureExt,
    prelude::{stream::StreamExt, *},
    select,
};
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use time_source::Time;
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
    time::Duration,
};
use structopt::StructOpt;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeightMetrics {
    latency: Duration,
    fp_finalization: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct BenchmarkResult {
    finalization_times: BTreeMap<Height, Option<HeightMetrics>>,
    proposals_timings: BTreeMap<CryptoHash, ArtifactDelayInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArtifactDelayInfo {
    sent: Option<Time>,
    received: Option<Time>,
}

pub mod network_layer;
use crate::{
    consensus_layer::height_index::Height,
    network_layer::Peer,
    time_source::{get_absolute_end_time, system_time_now}, crypto::CryptoHash,
};

pub mod artifact_manager;
pub mod consensus_layer;
pub mod crypto;
pub mod time_source;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(long)]
    r: u8, // replica number
    #[structopt(long, default_value = "6")]
    n: u8, // total number of nodes
    #[structopt(long, default_value = "1")]
    f: u8, // number of byzantine nodes
    #[structopt(long, default_value = "1")]
    p: u8, // number of disagreeing nodes
    #[structopt(long)]
    cod: bool, // enable Fast IC Consensus
    #[structopt(long, default_value = "300")]
    t: u64, // time to run replica
    #[structopt(long, default_value = "500")]
    d: u64, // notary delay
    #[structopt(long, default_value = "")]
    p2: String,    // address of peer to connect to
    #[structopt(long, default_value = "")]
    p3: String,    // address of peer to connect to
}

#[derive(Clone)]
pub struct SubnetParams {
    total_nodes_number: u8,
    byzantine_nodes_number: u8,
    disagreeing_nodes_number: u8,
    consensus_on_demand: bool,
    artifact_delay: u64,
}

impl SubnetParams {
    fn new(n: u8, f: u8, p: u8, cod: bool, d: u64) -> Self {
        Self {
            total_nodes_number: n,
            byzantine_nodes_number: f,
            disagreeing_nodes_number: p,
            consensus_on_demand: cod,
            artifact_delay: d,
        }
    }
}

async fn broadcast_message_future() {
    sleep(Duration::from_millis(1)).await;
}

#[async_std::main]
async fn main() {
    let opt = Opt::from_args();

    let finalizations_times = Arc::new(RwLock::new(BTreeMap::<Height, Option<HeightMetrics>>::new()));
    let cloned_finalization_times = Arc::clone(&finalizations_times);

    let proposals_timings = Arc::new(RwLock::new(BTreeMap::<CryptoHash, ArtifactDelayInfo>::new()));
    let cloned_proposals_timings = Arc::clone(&proposals_timings);

    let mut my_peer = Peer::new(
        opt.r,
        vec![opt.p2, opt.p3],
        SubnetParams::new(opt.n, opt.f, opt.p, opt.cod, opt.d),
        "gossip_blocks",
        cloned_finalization_times,
        cloned_proposals_timings,
    )
    .await;

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
        } else {
            // println!("\nStopped replica");

            let benchmark_result = BenchmarkResult {
                finalization_times: finalizations_times.read().unwrap().clone(),
                proposals_timings: proposals_timings.read().unwrap().clone(),
            };

            let encoded = to_string(&benchmark_result).unwrap();
            let mut file = File::create(format!("./benchmark/benchmark_results.json"))
                .await
                .unwrap();
            file.write_all(encoded.as_bytes()).await.unwrap();

            break;
        }
    }
}

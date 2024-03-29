use async_std::{fs::File, io, task, stream};
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
    time::Duration, thread,
};
use crossbeam_channel::{Receiver, Sender};
use structopt::StructOpt;
use tide::{Body, Request, Response, Result};

#[derive(Serialize, Deserialize, Debug, Clone)]
enum FinalizationType {
    IC,
    FP,
    DK,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeightMetrics {
    latency: Duration,
    fp_finalization: FinalizationType,
}

#[derive(Serialize, Deserialize, Debug)]
struct BenchmarkResult {
    finalization_times: BTreeMap<Height, Option<HeightMetrics>>,
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
    time_source::{get_absolute_end_time, system_time_now}
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
    #[structopt(long, default_value = "56789")]
    port: u64,    // port which the peers listen for connections
    #[structopt(name="broadcast_interval", long, default_value = "100")]
    broadcast_interval: u64, // interval after which artifacts are broadcasted
    #[structopt(name="artifact_manager_polling_interval", long, default_value = "200")]
    artifact_manager_polling_interval: u64, // periodic duration of `PollEvent` in milliseconds
}

#[derive(Clone)]
pub struct SubnetParams {
    total_nodes_number: u8,
    byzantine_nodes_number: u8,
    disagreeing_nodes_number: u8,
    consensus_on_demand: bool,
    artifact_delay: u64,
    artifact_manager_polling_interval: u64,
}

impl SubnetParams {
    fn new(n: u8, f: u8, p: u8, cod: bool, d: u64, artifact_manager_polling_interval: u64) -> Self {
        Self {
            total_nodes_number: n,
            byzantine_nodes_number: f,
            disagreeing_nodes_number: p,
            consensus_on_demand: cod,
            artifact_delay: d,
            artifact_manager_polling_interval,
        }
    }
}

async fn get_local_peer_id(req: Request<String>) -> Result {
    let peer_id = req.state();
    let res = Response::builder(200)
        .header("Content-Type", "application/json")
        .body(Body::from_json(peer_id)?)
        .build();
    Ok(res)
}

async fn post_remote_peers_addresses(mut req: Request<String>, sender: Arc<RwLock<Sender<String>>>) -> Result {
    let addresses = req.body_string().await?;
    sender.write().unwrap().send(addresses).unwrap();
    let res = Response::builder(200)
        .header("Content-Type", "application/json")
        .build();
    Ok(res)
}

#[async_std::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();
    println!("Replica number: {} running FICC: {}, with F: {}, P: {}, notarization delay: {}, broadcast_interval: {}, and artifact manager polling interval: {}", opt.r, opt.cod, opt.f, opt.p, opt.d, opt. broadcast_interval, opt.artifact_manager_polling_interval);

    let finalizations_times = Arc::new(RwLock::new(BTreeMap::<Height, Option<HeightMetrics>>::new()));
    let cloned_finalization_times = Arc::clone(&finalizations_times);

    let mut my_peer = Peer::new(
        opt.r,
        opt.port,
        SubnetParams::new(
            opt.n,
            opt.f,
            opt.p,
            opt.cod,
            opt.d,
            opt.artifact_manager_polling_interval
        ),
        "gossip_blocks",
        cloned_finalization_times,
    ).await;

    // Listen on all available interfaces at port specified in opt.port
    my_peer.listen_for_dialing();
    let local_peer_id = my_peer.id.to_string();

    let (sender_peers_addresses, receiver_peers_addresses) = 
    crossbeam_channel::unbounded::<String>();

    thread::spawn(move || {
        let mut peers_addresses = String::new();
        println!("Waiting to receive peers addresses...");
        match receiver_peers_addresses.recv() {
            Ok(addresses) => {
                peers_addresses.push_str(&addresses);
            },
            Err(_) => (),
        }
        println!("Received peers addresses: {}", peers_addresses);

        task::block_on(async {
            my_peer.dial_peers(peers_addresses);

            let starting_time = system_time_now();
            let relative_duration = Duration::from_millis(opt.t * 1000);
            let absolute_end_time = get_absolute_end_time(starting_time, relative_duration);
            loop {
                if system_time_now() < absolute_end_time {
                    let mut broadcast_interval = stream::interval(Duration::from_millis(opt.broadcast_interval));
                    select! {
                        _ = broadcast_interval.next().fuse() => {
                            // prevent Mdns expiration event by periodically broadcasting keep alive messages to peers
                            // if any locally generated artifact, broadcast it
                            if my_peer.artifact_manager_started() {
                                my_peer.broadcast_message();
                            }
                        },
                        event = my_peer.get_next_event() => my_peer.match_event(event),
                    }
                } else {
                    // println!("\nStopped replica");
                    let benchmark_result = BenchmarkResult {
                        finalization_times: finalizations_times.read().unwrap().clone(),
                    };

                    let encoded = to_string(&benchmark_result).unwrap();
                    let mut file = File::create(format!("./benchmark/benchmark_results.json"))
                        .await
                        .unwrap();
                    file.write_all(encoded.as_bytes()).await.unwrap();

                    break;
                }
            }
            std::process::exit(0);
        });
    });

    let mut app = tide::with_state(local_peer_id);

    app.at("/local_peer_id")
        .get(get_local_peer_id);

    let arc_sender_peers_addresses: Arc<RwLock<Sender<String>>> = Arc::new(RwLock::new(sender_peers_addresses));
    let cloned_arc_sender_peers_addresses = Arc::clone(&arc_sender_peers_addresses);
    app.at("/remote_peers_addresses")
        .post(move |req| post_remote_peers_addresses(req, Arc::clone(&cloned_arc_sender_peers_addresses)));

    app.listen(format!("0.0.0.0:{}", opt.port+1)).await?;

    Ok(())
}

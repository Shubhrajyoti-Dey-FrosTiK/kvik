pub mod listen;
pub mod rpc;
use clap::Parser;
use listen::listen_for_connections;
use ps::get_ps_instance;
use rpc::{election_timeout::run_election_timeout, kv::KV, leader_actions::run_leader_actions};
use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    spawn,
    sync::{broadcast, Mutex},
    time::sleep,
};
use tracing::info;
pub mod connect;
pub mod ps;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    host_port: u32,

    #[arg(short, long)]
    other_nodes: Vec<u32>,

    #[arg(short, long)]
    pr: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    info!("NODE STARTED AT  -  {:#?}", now);
    let args = Arc::new(Args::parse());
    let ps = get_ps_instance(args.host_port.clone(), args.other_nodes.clone())
        .await
        .unwrap();

    let kv_service = KV {
        host_port: args.host_port.clone(),
        start_time: now as u64,
        persistent_store: ps,
        commit_index: 0,
        last_applied: 0,
        leader_state: Arc::new(Mutex::new(None)), // Not a leader in the beginning
        next_election_time: Arc::new(Mutex::new(
            rand::random_range(
                Duration::from_millis(150).as_nanos()..Duration::from_millis(300).as_nanos(),
            ) + now,
        )), // in nanos
        last_append_entry_time: Arc::new(Mutex::new(None)),
        role: Arc::new(Mutex::new(rpc::kv::Role::Follower)),
        connected_hosts: Arc::new(Mutex::new(args.other_nodes.clone())),
    };

    let kv_service_cloned = Arc::new(kv_service.clone());
    let (tx, rx) = broadcast::channel::<bool>(1);
    let exit_rx = Arc::new(Mutex::new(rx));

    spawn(run_election_timeout(
        kv_service_cloned.clone(),
        exit_rx.clone(),
        args.other_nodes.len().clone() + 1,
    ));

    spawn(run_leader_actions(
        kv_service_cloned.clone(),
        exit_rx.clone(),
    ));

    // For actual use
    listen_for_connections(kv_service).await.unwrap();

    // For testing
    // spawn(listen_for_connections(kv_service));
    // sleep(Duration::from_secs(5)).await;
    // tx.send(true).unwrap(); // exit
}

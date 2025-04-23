pub mod listen;
pub mod rpc;
use clap::Parser;
use connect::connect_to_other_node;
use listen::listen_for_connections;
use ps::get_ps_instance;
use rpc::{
    election_timeout::run_election_timeout,
    kv::{ConnMap, KV},
};
use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    spawn,
    sync::{broadcast, Mutex, Notify},
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
    let ps = get_ps_instance(args.host_port.clone()).await.unwrap();

    let kv_service = KV {
        host_port: args.host_port.clone(),
        start_time: now as u64,
        persistent_store: ps,
        commit_index: 0,
        last_applied: 0,
        leader_state: None, // Not a leader in the beginning
        next_check_time: rand::random_range(150000000..300000000) + now, // in nanos
        conn_map: Arc::new(Mutex::new(vec![])),
    }; // Replace with your actual implementation

    let kv_service_cloned = Arc::new(kv_service.clone());
    let (tx, rx) = broadcast::channel::<bool>(1);

    spawn(run_election_timeout(
        kv_service_cloned.clone(),
        Arc::new(Mutex::new(rx)),
    ));

    for conn_port in args.other_nodes.clone() {
        let conn_client = connect_to_other_node(conn_port.clone()).await.unwrap();
        if conn_client.is_some() {
            kv_service.conn_map.lock().await.push(ConnMap {
                client: conn_client.unwrap(),
                conn_port,
            });
        }
    }

    // listen_for_connections(kv_service).await.unwrap();
    tx.send(true).unwrap(); // exit
}

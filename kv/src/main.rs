pub mod listen;
pub mod rpc;
use clap::Parser;
use init::connect_to_other_node;
use listen::listen_for_connections;
use tracing::info;
pub mod init;
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::{spawn, time::sleep};

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
    info!("NODE STARTED AT  -  {:#?}", SystemTime::now());
    let args = Arc::new(Args::parse());

    spawn(listen_for_connections(args.host_port.clone()));

    for other_nodes in args.other_nodes.clone() {
        spawn(connect_to_other_node(other_nodes.clone()));
    }

    // loop {}
    sleep(Duration::from_secs(5)).await;
}

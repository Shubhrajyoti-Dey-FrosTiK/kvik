pub mod listen;
pub mod rpc;
use clap::Parser;
use init::connect_to_other_node;
use listen::listen_for_connections;
pub mod init;
use std::{sync::Arc, time::Duration};
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
    let args = Arc::new(Args::parse());
    sleep(Duration::from_secs(2)).await;

    spawn(listen_for_connections(args.host_port.clone()));

    for other_nodes in args.other_nodes.clone() {
        spawn(connect_to_other_node(other_nodes.clone()));
    }

    loop {}
}

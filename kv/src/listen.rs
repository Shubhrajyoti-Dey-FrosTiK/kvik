use crate::rpc::kv::{kv_server::KvServer, KV};
use anyhow::Result;
use tonic::transport::Server;
use tracing::info;

pub async fn listen_for_connections(host_port: u32) -> Result<()> {
    let addr: std::net::SocketAddr = format!("0.0.0.0:{}", host_port.clone()).parse().unwrap();
    let kv_service = KV::default(); // Replace with your actual implementation

    info!("Starting to listen to {}", addr.clone());
    Server::builder()
        .add_service(KvServer::new(kv_service))
        .serve(addr)
        .await
        .unwrap();

    Ok(())
}

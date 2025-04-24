use crate::rpc::kv::{kv_server::KvServer, KV};
use anyhow::Result;
use tonic::transport::Server;
use tracing::{error, info};

pub async fn listen_for_connections(kv: KV) -> Result<()> {
    let addr: std::net::SocketAddr = format!("[::1]:{}", kv.host_port.clone()).parse().unwrap();

    info!("Starting to listen to {}", addr.clone());
    let server = Server::builder()
        .add_service(KvServer::new(kv))
        .serve(addr)
        .await;

    if server.is_err() {
        error!("{}", server.err().unwrap());
    }

    Ok(())
}

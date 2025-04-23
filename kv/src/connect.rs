use anyhow::Result;
use tonic::transport::Channel;
use tracing::{info, warn};

use crate::rpc::kv::kv_client::KvClient;

pub async fn connect_to_other_node(conn_port: u32) -> Result<Option<KvClient<Channel>>> {
    let addr = format!("http://0.0.0.0:{}", conn_port.clone());
    let conn = KvClient::connect(addr.clone()).await;

    if conn.is_ok() {
        info!("Connected to {}", addr.clone());
        return Ok(Some(conn.unwrap()));
    } else {
        warn!("Not connected to {}", addr.clone());
        return Ok(None);
    }
}

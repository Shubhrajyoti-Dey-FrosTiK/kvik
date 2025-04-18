use anyhow::Result;
use tracing::{info, warn};

use crate::rpc::kv::kv_client::KvClient;

pub async fn connect_to_other_node(host_port: u32) -> Result<()> {
    let addr = format!("http://0.0.0.0:{}", host_port.clone());
    let conn = KvClient::connect(addr.clone()).await;

    if conn.is_ok() {
        info!("Connected to {}", addr.clone());
    } else {
        warn!("Not connected to {}", addr.clone());
    }
    Ok(())
}

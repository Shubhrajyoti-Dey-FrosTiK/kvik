use anyhow::Result;
use std::sync::Arc;
use tracing::{info, warn};

use crate::rpc::kv::{kv_client::KvClient, ConnMap, KV};

pub async fn connect_to_other_node(conn_port: u32, kv: Arc<KV>) -> Result<()> {
    let addr = format!("http://0.0.0.0:{}", conn_port.clone());
    let conn = KvClient::connect(addr.clone()).await;

    if conn.is_ok() {
        info!("Connected to {}", addr.clone());
        kv.conn_map.lock().await.push(ConnMap {
            conn_port,
            client: conn.unwrap(),
        });
        return Ok(());
    } else {
        warn!("Not connected to {}", addr.clone());
        return Ok(());
    }
}

use crate::rpc::kv::kv_client::KvClient;
use tonic::transport::Channel;

pub async fn get_connection_client(conn_port: u32) -> Option<KvClient<Channel>> {
    let addr = format!("http://0.0.0.0:{}", conn_port.clone());
    let conn = KvClient::connect(addr.clone()).await;

    if conn.is_ok() {
        // info!("Connected to {}", addr.clone());
        return Some(conn.unwrap());
    } else {
        // warn!("Not connected to {}", addr.clone());
        return None;
    }
}

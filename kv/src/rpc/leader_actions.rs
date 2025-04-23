use tokio::{
    sync::{broadcast::Receiver, Mutex},
    time::sleep,
};
use tracing::{info, warn};

use super::kv::{AppendEntriesRequest, Role, KV};
use std::{sync::Arc, time::Duration};

pub async fn run_leader_actions(kv_service: Arc<KV>, exit: Arc<Mutex<Receiver<bool>>>) {
    loop {
        let exit = exit.lock().await.try_recv();
        if exit.is_ok() {
            info!("Exitting election timeouts");
            break;
        }

        if kv_service.get_role().await != Role::Leader {
            continue;
        }

        sleep(Duration::from_millis(150)).await;

        // Send AppendEntriesRPC to all connected clients
        for conn_map in kv_service.conn_map.lock().await.iter_mut() {
            let append_entries_response = conn_map
                .client
                .append_entries(AppendEntriesRequest {
                    term: kv_service.get_current_term().await,
                    leader_id: kv_service.host_port.clone(),
                    prev_log_index: kv_service.get_last_log_index().await,
                    prev_log_term: kv_service.get_last_log_term().await,
                    entries: vec![],
                    leader_commit: kv_service.commit_index.clone(),
                })
                .await;

            if append_entries_response.is_err() {
                warn!(
                    "Error appending entries to - {}",
                    append_entries_response.err().unwrap()
                );
                continue;
            }

            let append_entries_response = append_entries_response.unwrap().into_inner();
            if append_entries_response.success {
                if append_entries_response.term > kv_service.get_current_term().await {
                    kv_service
                        .set_current_term(append_entries_response.term)
                        .await;
                }
                info!(
                    "Success appending entries to {}",
                    conn_map.conn_port.clone()
                );
            } else {
                warn!(
                    "Didnt succeed appending entries to {}",
                    conn_map.conn_port.clone()
                )
            }
        }
    }
}

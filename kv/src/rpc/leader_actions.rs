use super::{
    kv::{AppendEntriesRequest, Role, KV},
    state::PRStatistics,
    utils::get_current_time_nanosecs,
};
use crate::connect::get_connection_client;
use std::{ops::AddAssign, sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast::Receiver, Mutex},
    time::sleep,
};
use tracing::{info, warn};

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

        let old_pr_statistics = kv_service.get_pr_statistics().await.to_proto();
        let mut current_cycle_pr_statistics = PRStatistics::empty();
        let mut average_leader_latency = 0;
        let mut active_nodes = 0;
        // Send AppendEntriesRPC to all connected clients
        for connected_host in kv_service.connected_hosts.lock().await.iter() {
            let client = get_connection_client(*connected_host).await;
            if client.is_none() {
                continue;
            }
            let mut client = client.unwrap();
            let request_start_time = get_current_time_nanosecs();
            let append_entries_response = client
                .append_entries(AppendEntriesRequest {
                    term: kv_service.get_current_term().await,
                    leader_id: kv_service.host_port.clone(),
                    prev_log_index: kv_service.get_last_log_index().await,
                    prev_log_term: kv_service.get_last_log_term().await,
                    entries: vec![],
                    leader_commit: kv_service.commit_index.clone(),
                    pr_statistics: Some(old_pr_statistics.clone()),
                })
                .await;
            let request_end_time = get_current_time_nanosecs();

            if append_entries_response.is_err() {
                warn!(
                    "Error appending entries to - {} | {}",
                    connected_host.clone(),
                    append_entries_response.err().unwrap()
                );

                current_cycle_pr_statistics
                    .crash_count
                    .insert(*connected_host, 1);
                continue;
            }

            let append_entries_response = append_entries_response.unwrap().into_inner();
            if append_entries_response.success {
                if append_entries_response.term > kv_service.get_current_term().await {
                    info!(
                        "{} chosen as the leader of the system",
                        connected_host.clone()
                    );
                    kv_service
                        .set_current_term(append_entries_response.term)
                        .await;
                }

                average_leader_latency.add_assign((request_end_time - request_start_time) as u64);
                active_nodes.add_assign(1);
                current_cycle_pr_statistics.average_latency.insert(
                    *connected_host,
                    (request_end_time - request_start_time) as u64,
                );
                current_cycle_pr_statistics
                    .request_served
                    .insert(*connected_host, 1);

                // info!("Success appending entries to {}", connected_host);
            } else {
                warn!("Didnt succeed appending entries to {}", connected_host)
            }
        }

        current_cycle_pr_statistics.average_latency.insert(
            kv_service.host_port.clone(),
            average_leader_latency / active_nodes,
        );
        current_cycle_pr_statistics
            .request_served
            .insert(kv_service.host_port.clone(), 1);

        kv_service
            .update_pr_statistics(current_cycle_pr_statistics)
            .await;

        sleep(Duration::from_millis(50)).await;
    }
}

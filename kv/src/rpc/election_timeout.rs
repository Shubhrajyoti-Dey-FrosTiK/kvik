use tokio::{
    sync::{broadcast::Receiver, Mutex},
    time::sleep,
};
use tracing::{info, warn};

use super::kv::{RequestVoteRequest, KV};
use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub async fn run_election_timeout(kv_service: Arc<KV>, exit: Arc<Mutex<Receiver<bool>>>) {
    loop {
        let exit = exit.lock().await.try_recv();
        if exit.is_ok() {
            info!("Exitting election timeouts");
            break;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        if now < kv_service.next_check_time {
            sleep(Duration::from_nanos(
                (kv_service.next_check_time - now) as u64,
            ))
            .await;
            continue;
        } else {
            // info!("GOING FOR ELECTIONS");
            for conn_map in kv_service.conn_map.lock().await.iter_mut() {
                let vote_response = conn_map
                    .client
                    .request_vote(RequestVoteRequest {
                        candidate_id: kv_service.host_port.clone(),
                        last_log_index: kv_service.commit_index,
                        last_log_term: kv_service.get_last_log_term().await,
                        term: kv_service.get_current_term().await,
                    })
                    .await;

                if vote_response.is_err() {
                    warn!("Error in getting vote - {}", vote_response.err().unwrap());
                    continue;
                }

                let vote_response = vote_response.unwrap().into_inner();
                if vote_response.vote_granted {
                    info!("Received vote from - {}", conn_map.conn_port.clone());
                } else {
                    warn!("Didnt receive vote from - {}", conn_map.conn_port.clone())
                }
            }
        }
    }
}

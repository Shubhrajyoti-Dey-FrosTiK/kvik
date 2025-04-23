use tokio::{
    sync::{broadcast::Receiver, Mutex},
    time::sleep,
};
use tracing::{info, warn};

use super::kv::{RequestVoteRequest, Role, KV};
use std::{
    ops::AddAssign,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub async fn run_election_timeout(
    kv_service: Arc<KV>,
    exit: Arc<Mutex<Receiver<bool>>>,
    total_nodes: usize,
) {
    loop {
        let exit = exit.lock().await.try_recv();
        if exit.is_ok() {
            info!("Exitting election timeouts");
            break;
        }

        if kv_service.get_role().await == Role::Leader {
            continue;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let next_election_time = kv_service.get_next_election_time().await;
        if now < next_election_time {
            sleep(Duration::from_nanos((next_election_time - now) as u64)).await;
            continue;
        } else {
            let mut vote_count = 1;
            for conn_map in kv_service.conn_map.lock().await.iter_mut() {
                let current_term = kv_service.get_current_term().await;
                let vote_response = conn_map
                    .client
                    .request_vote(RequestVoteRequest {
                        candidate_id: kv_service.host_port.clone(),
                        last_log_index: kv_service.commit_index,
                        last_log_term: kv_service.get_last_log_term().await,
                        term: current_term.clone(),
                    })
                    .await;

                if vote_response.is_err() {
                    warn!("Error in getting vote - {}", vote_response.err().unwrap());
                    continue;
                }

                let vote_response = vote_response.unwrap().into_inner();
                if vote_response.vote_granted {
                    info!("Received vote from - {}", conn_map.conn_port.clone());
                    vote_count.add_assign(1);
                } else {
                    if vote_response.term > current_term {
                        kv_service.set_current_term(vote_response.term).await;
                        break;
                    }
                    warn!("Didnt receive vote from - {}", conn_map.conn_port.clone())
                }
            }

            if vote_count >= total_nodes.div_ceil(2) {
                info!(
                    "Elevated to leader with {} / {} votes",
                    vote_count,
                    total_nodes.clone()
                );
                kv_service.set_role(Role::Leader).await;
            }
        }
    }
}

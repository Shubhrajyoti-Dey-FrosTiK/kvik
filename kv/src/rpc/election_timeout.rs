use super::kv::{RequestVoteRequest, Role, KV};
use crate::{
    connect::get_connection_client,
    rpc::{
        kv::{InformNextTermRequst, LeaderState},
        utils::{get_current_time_nanosecs, get_random_election_timeout},
    },
};
use std::{ops::AddAssign, sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast::Receiver, Mutex},
    time::sleep,
};
use tracing::{info, warn};

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

        let now = get_current_time_nanosecs();

        let next_election_time = kv_service.get_next_election_time().await;
        if now < next_election_time {
            if kv_service.get_role().await != Role::Follower {
                continue;
            }

            sleep(Duration::from_nanos((next_election_time - now) as u64)).await;
            // sleep(Duration::from_secs(1)).await;
            continue;
        }

        if kv_service.get_role().await != Role::Follower {
            continue;
        }

        if kv_service.pr_enabled {
            let pr_statistics = kv_service.get_pr_statistics().await;
            let mut nodes: Vec<(&u32, &f32)> = pr_statistics.pr.iter().collect();
            nodes.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

            if nodes.iter().all(|(_, pr)| *pr == nodes[0].1) {
                warn!("All nodes have equal PR values, no leader elected from pr");
            } else {
                let mut leader_selected = false;

                for (node, pr) in nodes {
                    let node = node.clone();

                    if node == kv_service.host_port {
                        kv_service.set_role(Role::Leader).await;
                        leader_selected = true;
                        info!(
                            "Switched to {:?} from follower",
                            kv_service.get_role().await
                        );
                        break;
                    }

                    let client = get_connection_client(node).await;

                    if client.is_none() {
                        warn!("{} is not selected to be leader as not ready", node.clone());
                        continue;
                    }
                    let mut client = client.unwrap();
                    let response = client
                        .inform_next_tern(InformNextTermRequst {
                            leader_id: node,
                            term: kv_service.get_current_term().await + 1,
                        })
                        .await;

                    if response.is_err() {
                        warn!("Errored out informing {}", node);
                        continue;
                    }

                    let response = response.unwrap().into_inner();
                    if response.success == false {
                        warn!("Refused to be leader {}", node);
                    }

                    leader_selected = true;
                    break;
                }

                if leader_selected {
                    kv_service
                        .set_next_election_time(get_random_election_timeout())
                        .await;
                    continue;
                }
            }
        }

        kv_service.set_role(Role::Candidate).await;
        info!(
            "Switched to {:?} from follower",
            kv_service.get_role().await
        );

        info!("PR Statistics {:#?}", kv_service.get_pr_statistics().await);

        let mut vote_count = 1;
        let current_term = kv_service.get_current_term().await;
        // for conn_map in kv_service.conn_map.lock().await.iter_mut() {
        for other_host in kv_service.connected_hosts.lock().await.iter() {
            let client = get_connection_client(*other_host).await;
            if client.is_none() {
                continue;
            }
            let mut client = client.unwrap();
            let vote_response = client
                .request_vote(RequestVoteRequest {
                    candidate_id: kv_service.host_port.clone(),
                    last_log_index: kv_service.commit_index,
                    last_log_term: kv_service.get_last_log_term().await,
                    term: current_term.clone() + 1,
                })
                .await;

            if vote_response.is_err() {
                warn!("Error in getting vote - {}", vote_response.err().unwrap());
                continue;
            }

            let vote_response = vote_response.unwrap().into_inner();
            if vote_response.vote_granted {
                info!("Received vote from - {}", other_host.clone());
                vote_count.add_assign(1);
            } else {
                if vote_response.term > current_term {
                    info!("{} chosen as the leader of the system", other_host.clone());
                    kv_service.set_current_term(vote_response.term).await;
                    break;
                }
                warn!("Didnt receive vote from - {}", other_host.clone())
            }
        }

        if vote_count >= total_nodes.div_ceil(2) {
            info!(
                "Elevated to leader with {} / {} votes",
                vote_count,
                total_nodes.clone()
            );
            *kv_service.leader_state.lock().await = Some(LeaderState {
                last_connected_hosts: vec![],
                match_index: vec![],
                next_index: vec![],
            });
            kv_service.set_role(Role::Leader).await;
            kv_service.set_current_term(current_term + 1).await;
        } else {
            info!("Didn't get elected as leader so turning down to follower");
            kv_service
                .set_next_election_time(get_random_election_timeout())
                .await;
            kv_service.set_role(Role::Follower).await;
        }
    }
}

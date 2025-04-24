use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::kv::{AppendEntriesRequest, AppendEntriesResponse, Role, KV};
use tonic::{Request, Response, Status};
use tracing::info;

impl KV {
    pub async fn append_entries_handler(
        &self,
        request: Request<AppendEntriesRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<AppendEntriesResponse>, Status> {
        let request = request.into_inner();
        let current_term = self.get_current_term().await;
        if current_term > request.term {
            return Ok(Response::new(AppendEntriesResponse {
                term: current_term,
                success: false,
            }));
        }

        if current_term < request.term {
            self.set_current_term(request.term.clone()).await;
            self.set_role(Role::Follower).await;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let upper_limit = Duration::from_millis(300).as_nanos();
        let lower_limit = Duration::from_millis(200).as_nanos();
        self.set_last_appendentry_time(now).await;
        self.set_next_election_time(rand::random_range(lower_limit..upper_limit) + now)
            .await;
        let current_role = self.get_role().await;
        if current_role != Role::Follower {
            info!("Switched to follower from {:?}", current_role);
            self.set_role(Role::Follower).await;
        }

        Ok(Response::new(AppendEntriesResponse {
            success: true,
            term: request.term,
        }))
    }
}

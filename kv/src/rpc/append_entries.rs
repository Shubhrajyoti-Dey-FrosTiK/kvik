use super::{
    kv::{AppendEntriesRequest, AppendEntriesResponse, Role, KV},
    utils::get_random_election_timeout,
};
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

        if self.get_voted_for().await.is_some() {
            self.set_voted_for(None).await;
        }

        if current_term < request.term {
            info!(
                "{} chosen as the leader of the system",
                request.leader_id.clone()
            );
            self.set_current_term(request.term.clone()).await;
            self.set_role(Role::Follower).await;
        }

        self.set_next_election_time(get_random_election_timeout())
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

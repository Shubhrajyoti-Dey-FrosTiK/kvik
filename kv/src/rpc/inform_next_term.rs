use crate::rpc::kv::LeaderState;

use super::kv::{InformNextTermRequst, InformNextTermResponse, Role, KV};
use tonic::{Request, Response, Status};
use tracing::info;

impl KV {
    pub async fn inform_next_term_handler(
        &self,
        request: Request<InformNextTermRequst>, // Accept request of type HelloRequest
    ) -> Result<Response<InformNextTermResponse>, Status> {
        let request = request.into_inner();
        let current_term = self.get_current_term().await;
        if current_term > request.term {
            return Ok(Response::new(InformNextTermResponse { success: false }));
        }

        *self.leader_state.lock().await = Some(LeaderState {
            last_connected_hosts: vec![],
            match_index: vec![],
            next_index: vec![],
        });
        self.set_current_term(request.term).await;
        self.set_role(Role::Leader).await;
        info!("Elected to be leader using PR");

        return Ok(Response::new(InformNextTermResponse { success: true }));
    }
}

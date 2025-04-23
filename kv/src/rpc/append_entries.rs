use super::kv::{AppendEntriesRequest, AppendEntriesResponse, KV};
use tonic::{Request, Response, Status};

impl KV {
    pub async fn append_entries_handler(
        &self,
        request: Request<AppendEntriesRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<AppendEntriesResponse>, Status> {
        let reply = AppendEntriesResponse {
            success: true,
            term: 1,
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

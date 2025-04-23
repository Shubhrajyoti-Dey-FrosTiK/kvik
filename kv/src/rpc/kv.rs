use microkv::MicroKV;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{transport::Channel, Request, Response, Status};
tonic::include_proto!("kv");

#[derive(Clone, Debug)]
pub struct LeaderState {
    pub next_index: Vec<u64>, // for each server, index of the next log entry to send to that server (initialized to leader last log index + 1)
    pub match_index: Vec<u64>, // for each server, index of highest log entry known to be replicated on server (initialized to 0, increases monotonically)
}

#[derive(Clone)]
pub struct ConnMap {
    pub conn_port: u32,
    pub client: kv_client::KvClient<Channel>,
}

#[derive(Clone)]
pub struct KV {
    pub host_port: u32,
    pub start_time: u64,
    pub persistent_store: MicroKV,
    pub commit_index: u64, // index of highest log entry known to be committed (initialized to 0, increases monotonically)
    pub last_applied: u64, // index of highest log entry applied to state machine (initialized to 0, increases monotonically)

    pub leader_state: Option<LeaderState>, // state of the leader node | Only if the node is the leader, this data is present

    pub next_check_time: u128, // this is the time the node should have been sent a AppendEntries Request
    pub conn_map: Arc<Mutex<Vec<ConnMap>>>,
}

#[tonic::async_trait]
impl kv_server::Kv for KV {
    async fn request_vote(
        &self,
        request: Request<RequestVoteRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<RequestVoteResponse>, Status> {
        return self.request_vote_handler(request).await;
    }

    async fn append_entries(
        &self,
        request: Request<AppendEntriesRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<AppendEntriesResponse>, Status> {
        return self.append_entries_handler(request).await;
    }
}

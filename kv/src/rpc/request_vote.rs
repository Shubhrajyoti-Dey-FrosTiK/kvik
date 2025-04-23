use super::kv::{RequestVoteRequest, RequestVoteResponse, KV};
use tonic::{Request, Response, Status};

impl KV {
    pub async fn request_vote_handler(
        &self,
        request: Request<RequestVoteRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<RequestVoteResponse>, Status> {
        let request = request.into_inner();

        let current_term = self.get_current_term().await;
        if request.term < current_term {
            return Ok(Response::new(RequestVoteResponse {
                term: current_term,
                vote_granted: false,
            }));
        }

        if request.term > current_term {
            self.set_current_term(request.term.clone()).await;
        }

        let voted_for = self.get_voted_for().await;
        let last_log_term = self.get_last_log_term().await;

        // If the voted_for is null or if the candidate has already voted the candidate
        // AND
        // If the candidate’s log is at least as up-to-date as receiver’s log, grant vote
        if (voted_for.is_none() || voted_for.unwrap() == request.candidate_id)
            && ((request.last_log_term > last_log_term)
                || (request.last_log_term == last_log_term
                    && request.last_log_index >= self.commit_index))
        {
            return Ok(Response::new(RequestVoteResponse {
                term: current_term,
                vote_granted: true,
            }));
        }

        return Ok(Response::new(RequestVoteResponse {
            term: current_term,
            vote_granted: false,
        }));
    }
}

use tonic::{Request, Response, Status};

tonic::include_proto!("kv");

#[derive(Debug, Default)]
pub struct KV {
    host_port: u32,
    start_time: u64,
}

#[tonic::async_trait]
impl kv_server::Kv for KV {
    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HeartbeatResponse>, Status> {
        // Return an instance of type HelloReply
        println!("Got a request: {:?}", request);

        let reply = HeartbeatResponse {
            active_time: self.start_time,
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

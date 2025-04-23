use tokio::{
    sync::{broadcast::Receiver, Mutex, Notify},
    time::sleep,
};
use tracing::info;

use super::kv::KV;
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
            info!("GOING FOR ELECTIONS");
        }
    }
}

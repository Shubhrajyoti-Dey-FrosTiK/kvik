use super::kv::KV;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

#[derive(Serialize, Deserialize)]
pub struct Log {
    pub command: String,
    pub term: u64,
}

impl KV {
    pub async fn get_current_term(&self) -> u64 {
        return self
            .persistent_store
            .get_unwrap::<u64>("currentTerm")
            .unwrap();
    }

    pub async fn set_current_term(&self, current_term: u64) {
        self.persistent_store
            .put::<u64>("currentTerm", &current_term)
            .unwrap();
    }

    pub async fn get_voted_for(&self) -> Option<u32> {
        let voted_for = self.persistent_store.get_unwrap::<u32>("votedFor");

        if voted_for.is_err() {
            return None;
        }

        return Some(voted_for.unwrap());
    }

    pub async fn get_last_log_term(&self) -> u64 {
        let log_string = self
            .persistent_store
            .get_unwrap::<String>("log")
            .unwrap_or("[]".to_string());

        let logs: Vec<Log> = from_str(&log_string).unwrap();

        if logs.len() > 0 {
            return logs.get(logs.len() - 1).unwrap().term;
        }

        return 0;
    }
}

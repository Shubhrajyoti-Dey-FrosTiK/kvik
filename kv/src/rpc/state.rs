use std::ops::DerefMut;

use super::kv::{Role, KV};
use serde::{Deserialize, Serialize};

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

    pub async fn set_voted_for(&self, voted_for: u32) {
        self.persistent_store
            .put::<u32>("votedFor", &voted_for)
            .unwrap();
    }

    pub async fn get_last_log_term(&self) -> u64 {
        let logs = self.persistent_store.get_unwrap::<Vec<Log>>("log").unwrap();

        if logs.len() > 0 {
            return logs.get(logs.len() - 1).unwrap().term;
        }

        return 0;
    }

    pub async fn get_last_log_index(&self) -> u64 {
        let logs = self.persistent_store.get_unwrap::<Vec<Log>>("log").unwrap();

        return logs.len() as u64;
    }

    pub async fn get_role(&self) -> Role {
        return self.role.lock().await.clone();
    }

    pub async fn set_role(&self, role: Role) {
        self.role.lock().await.set(role);
    }

    pub async fn get_next_election_time(&self) -> u128 {
        self.next_election_time.lock().await.clone()
    }

    pub async fn set_next_election_time(&self, time: u128) {
        let mut election_time = self.next_election_time.lock().await;
        *election_time.deref_mut() = time;
    }

    pub async fn get_last_appendentry_time(&self) -> Option<u128> {
        self.last_append_entry_time.lock().await.clone()
    }

    pub async fn set_last_appendentry_time(&self, time: u128) {
        let mut last_appendentry_time = self.last_append_entry_time.lock().await;
        *last_appendentry_time.deref_mut() = Some(time);
    }
}

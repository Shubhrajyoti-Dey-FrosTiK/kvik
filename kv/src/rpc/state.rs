use super::{
    kv::{self, PrStatistics, Role, KV},
    utils::get_pr_of_system,
};
use microkv::MicroKV;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::{AddAssign, DerefMut, MulAssign},
};

#[derive(Serialize, Deserialize)]
pub struct Log {
    pub command: String,
    pub term: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GlobalStatistics {
    pub request_served: u64, // total number of requests served. Leader - 1 (per cycle), Follower - 1 (n-1 followers). Total n (ideally) in a cycle
    pub heartbeat_cycles: u64, // total number of heartbeat cycles.
}

impl GlobalStatistics {
    pub fn from_proto(proto: kv::GlobalStatistics) -> Self {
        Self {
            heartbeat_cycles: proto.heartbeat_cycles,
            request_served: proto.request_served,
        }
    }

    pub fn to_proto(&self) -> kv::GlobalStatistics {
        kv::GlobalStatistics {
            heartbeat_cycles: self.heartbeat_cycles,
            request_served: self.request_served,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PRStatistics {
    pub request_served: HashMap<u32, u64>,
    pub average_latency: HashMap<u32, u64>,
    pub crash_count: HashMap<u32, u64>,
    pub pr: HashMap<u32, f32>,
    pub global: GlobalStatistics,
}

impl PRStatistics {
    pub fn empty() -> Self {
        let pr = HashMap::new();
        let zero_map = HashMap::new();
        Self {
            average_latency: zero_map.clone(),
            crash_count: zero_map.clone(),
            pr,
            request_served: zero_map.clone(),
            global: GlobalStatistics {
                request_served: 0,
                heartbeat_cycles: 0,
            },
        }
    }

    pub fn default(nodes: Vec<u32>) -> Self {
        let mut pr = HashMap::new();
        let mut zero_map = HashMap::new();
        for node in nodes.iter() {
            pr.insert(*node, 1.0 / (nodes.len() as f32));
            zero_map.insert(*node, 0 as u64);
        }
        Self {
            average_latency: zero_map.clone(),
            crash_count: zero_map.clone(),
            pr,
            request_served: zero_map.clone(),
            global: GlobalStatistics {
                request_served: 0,
                heartbeat_cycles: 0,
            },
        }
    }

    pub async fn setup(microkv: MicroKV, nodes: Vec<u32>) {
        let pr_statistics = microkv.get_unwrap::<Vec<Log>>("prStatistics");
        if pr_statistics.is_ok() {
            return;
        }

        microkv
            .put::<Self>("prStatistics", &Self::default(nodes))
            .unwrap();
    }

    pub fn to_proto(&self) -> PrStatistics {
        PrStatistics {
            request_served: self.request_served.clone(),
            average_latency: self.average_latency.clone(),
            crash_count: self.crash_count.clone(),
            pr: self.pr.clone(),
            global: Some(self.global.to_proto()),
        }
    }

    pub fn from_proto(pr_statistics: PrStatistics) -> Self {
        Self {
            request_served: pr_statistics.request_served,
            average_latency: pr_statistics.average_latency,
            crash_count: pr_statistics.crash_count,
            pr: pr_statistics.pr,
            global: GlobalStatistics::from_proto(pr_statistics.global.unwrap()),
        }
    }
}

impl KV {
    pub async fn get_all_nodes(&self) -> Vec<u32> {
        let mut nodes = self.connected_hosts.lock().await.clone();
        nodes.push(self.host_port.clone());
        nodes
    }

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
        self.persistent_store
            .get_unwrap::<Option<u32>>("votedFor")
            .unwrap()
    }

    pub async fn set_voted_for(&self, voted_for: Option<u32>) {
        self.persistent_store
            .put::<Option<u32>>("votedFor", &voted_for)
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

    pub async fn get_pr_statistics(&self) -> PRStatistics {
        self.persistent_store
            .get_unwrap::<PRStatistics>("prStatistics")
            .unwrap()
    }

    pub async fn set_pr_statistics(&self, pr_statistics: PRStatistics) {
        self.persistent_store
            .put::<PRStatistics>("prStatistics", &pr_statistics)
            .unwrap()
    }

    pub async fn update_pr_statistics(&self, pr_statistics: PRStatistics) {
        let old_pr_statistics = self.get_pr_statistics().await;
        let mut new_pr_statistics = old_pr_statistics.clone();
        for (host, average_latency) in pr_statistics.average_latency.clone() {
            // let crash_count = pr_statistics.crash_count.get(&host);
            let old_requests_served = old_pr_statistics.request_served.get(&host).unwrap();
            let old_average_latency = old_pr_statistics.average_latency.get(&host).unwrap();
            let new_average_latency = ((old_average_latency * old_requests_served.clone())
                + average_latency)
                .div_ceil(old_requests_served.clone() + 1);
            new_pr_statistics
                .average_latency
                .insert(host.clone(), new_average_latency);
            new_pr_statistics
                .request_served
                .insert(host.clone(), *old_requests_served + 1);
        }

        for (host, crash_count) in pr_statistics.crash_count {
            let old_crash_count = old_pr_statistics.crash_count.get(&host).unwrap();
            new_pr_statistics
                .crash_count
                .insert(host.clone(), crash_count + old_crash_count.clone());
        }

        new_pr_statistics
            .global
            .heartbeat_cycles
            .add_assign(pr_statistics.global.heartbeat_cycles); // Basically increment the heartbeat cycles
        new_pr_statistics
            .global
            .request_served
            .add_assign(pr_statistics.global.request_served);

        let num_of_nodes = self.get_all_nodes().await.len();
        let (leader_pr, follower_pr) = get_pr_of_system(num_of_nodes);

        let mut all_nodes = self.connected_hosts.lock().await.clone();
        all_nodes.push(self.host_port.clone());

        for node in all_nodes.iter() {
            let node_request_served = new_pr_statistics.request_served.get(node).unwrap_or(&0);
            let node_average_latency = new_pr_statistics.average_latency.get(node).unwrap_or(&0);
            let node_crashes = new_pr_statistics.crash_count.get(node).unwrap_or(&0);

            let request_served_ratio = (node_request_served.clone() as f32)
                / ((new_pr_statistics.global.request_served.max(1)) as f32);
            let crash_constant = f32::powi(
                (all_nodes.len() - 1) as f32 / all_nodes.len() as f32,
                node_crashes.clone() as i32,
            );
            let latency_constant = 1.0 / node_average_latency.clone().max(1) as f32;

            let mut pr = if node.clone() == self.host_port {
                leader_pr
            } else {
                follower_pr
            };

            pr.mul_assign(request_served_ratio * crash_constant * latency_constant);
            new_pr_statistics.pr.insert(node.clone(), pr);
        }

        self.persistent_store
            .put::<PRStatistics>("prStatistics", &new_pr_statistics)
            .unwrap();
    }

    pub async fn get_last_appendentry_time(&self) -> Option<u128> {
        self.last_append_entry_time.lock().await.clone()
    }

    pub async fn set_last_appendentry_time(&self, time: u128) {
        let mut last_appendentry_time = self.last_append_entry_time.lock().await;
        *last_appendentry_time.deref_mut() = Some(time);
    }
}

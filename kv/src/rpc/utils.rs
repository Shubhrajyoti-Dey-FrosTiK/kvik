use std::{
    collections::HashMap,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use pagerank_rs::Pagerank;

pub fn get_current_time_nanosecs() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

pub fn get_random_election_timeout() -> u128 {
    let upper_limit = Duration::from_millis(300).as_nanos();
    let lower_limit = Duration::from_millis(100).as_nanos();
    rand::random_range(lower_limit..upper_limit) + get_current_time_nanosecs()
}

pub fn get_pr_of_system(mum_nodes: usize) -> (f32, f32) {
    let mut graph = Pagerank::new(mum_nodes);
    graph.link(1, 0).unwrap();
    graph.link(2, 0).unwrap();

    let probability_of_following_a_link = 0.85;
    let tolerance = 0.0001;

    for i in 1..mum_nodes {
        graph.link(i, 0).unwrap();
    }

    let mut result = HashMap::new();
    graph.rank(
        probability_of_following_a_link,
        tolerance,
        |node_id, rank| {
            result.insert(node_id as u32, rank);
        },
    );

    return (
        result.get(&0).unwrap().clone() as f32,
        result.get(&1).unwrap().clone() as f32,
    );
}

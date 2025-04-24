use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

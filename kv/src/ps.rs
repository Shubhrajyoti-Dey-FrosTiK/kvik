use anyhow::Result;
use microkv::MicroKV;
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::rpc::state::Log;

pub async fn get_ps_instance(host_port: u32) -> Result<MicroKV> {
    let db_path = format!("./db/{}.json", host_port);
    if !Path::new(&db_path).exists() {
        fs::create_dir_all("./db").unwrap();
    }
    let micro_kv = MicroKV::open_with_base_path("db", PathBuf::from_str(&db_path).unwrap())
        .expect("Failed to create MicroKV from a stored file or create MicroKV for this file")
        .set_auto_commit(true);

    let current_term = micro_kv.get_unwrap::<u64>("currentTerm");
    if current_term.is_err() {
        micro_kv.put::<u64>("currentTerm", &0).unwrap();
    }

    let log = micro_kv.get_unwrap::<Vec<Log>>("log");
    if log.is_err() {
        micro_kv.put::<Vec<Log>>("log", &vec![]).unwrap();
    }

    Ok(micro_kv)
}

mod api;
mod config;
mod err;
mod utils;

use std::{path::Path, *};

fn main() {
    let mc_json_path = Path::new("mc-config.json");
    let mc_jar_path = Path::new("mc.jar");
    let mc_config = config::load_json(mc_json_path).unwrap();
    let latest_version = match mc_config.check_update().unwrap() {
        Some(o) => o,
        None => return,
    };
    let jar_download = latest_version.download().unwrap();
    fs::write(mc_jar_path, jar_download).unwrap();
    latest_version.write_json(mc_jar_path).unwrap();
}

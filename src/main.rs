mod api;
mod config;
mod err;
mod processes;
mod utils;

use crate::err::ErrorCaller;
use std::{path::Path, *};

fn mc(updated: &mut bool) -> utils::Result<()> {
    let mc_json_path = Path::new("mc-config.json");
    let mc_jar_path = Path::new("mc.jar");
    let mc_config = config::load_json(mc_json_path).e()?;
    let latest_version = match mc_config.check_update().e()? {
        Some(o) => o,
        None => return Ok(()),
    };
    let jar_download = latest_version.download().e()?;
    fs::write(mc_jar_path, jar_download).e()?;
    *updated = true;
    latest_version.write_json(mc_json_path).e()?;
    Ok(())
}
fn main() {
    let schedule = utils::parse_cron(env::args()).unwrap();
    let mut upcoming = schedule.upcoming(chrono::Local);
    processes::sys_update().unwrap();
    mc(&mut false).unwrap();
    let mut mc_state = processes::new();
    processes::mc_restart(&mut mc_state, &[""]).unwrap();
    loop {
        if let Some(next) = upcoming.next() {
            let sleep_duration = (next - chrono::Local::now()).to_std().unwrap_or_default();
            thread::sleep(sleep_duration);
            match processes::sys_update() {
                Ok(r) => r,
                Err(e) => eprintln!("{:#?}", e),
            };
            let mut updated = false;
            match mc(&mut updated) {
                Ok(r) => r,
                Err(e) => eprintln!("{:#?}", e),
            };
            if updated {
                processes::mc_restart(&mut mc_state, &[""]).unwrap();
            }
        } else {
            panic!("None")
        }
    }
}

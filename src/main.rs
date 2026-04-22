mod api;
mod config;
mod err;
mod utils;

use std::{path::Path, *};

use crate::err::ErrorCaller;

fn mc() -> utils::Result<()> {
    let mc_json_path = Path::new("mc-config.json");
    let mc_jar_path = Path::new("mc.jar");
    let mc_config = config::load_json(mc_json_path).e()?;
    let latest_version = match mc_config.check_update().e()? {
        Some(o) => o,
        None => return Ok(()),
    };
    let jar_download = latest_version.download().e()?;
    fs::write(mc_jar_path, jar_download).e()?;
    latest_version.write_json(mc_json_path).e()?;
    Ok(())
}
fn main() {
    let schedule = utils::parse_cron(env::args()).unwrap();
    let mut upcoming = schedule.upcoming(chrono::Local);
    mc().unwrap();
    loop {
        if let Some(next) = upcoming.next() {
            let sleep_duration = (next - chrono::Local::now()).to_std().unwrap_or_default();
            thread::sleep(sleep_duration);
            match mc() {
                Ok(r) => r,
                Err(e) => eprintln!("{:#?}", e),
            };
        }
    }
}

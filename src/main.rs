mod api;
mod config;
mod err;
mod processes;
mod utils;

use crate::err::ErrorCaller;
use std::{path::Path, *};

fn mc_update(updated: &mut bool) -> utils::Result<Vec<String>> {
    let mc_json_path = Path::new("mc-config.json");
    let mc_jar_path = Path::new("mc.jar");
    let mut mc_config = config::load_json(mc_json_path).e()?;
    let latest_version = match mc_config.need_update().e()? {
        Some(o) => o,
        None => return Ok(mc_config.get_start_flags()),
    };
    let jar_download = latest_version.download().e()?;
    fs::write(mc_jar_path, jar_download).e()?;
    *updated = true;
    let start_args = latest_version.get_start_flags();
    latest_version.write_json(mc_json_path).e()?;
    Ok(start_args)
}
fn main() {
    let schedule = utils::parse_cron().unwrap();
    let mut upcoming = schedule.upcoming(chrono::Local);
    let mc_state_mutex = processes::grab_stdin();
    println!("Initial System Update");
    processes::sys_update().unwrap();
    println!("Intial PaperMC API Check");
    let start_args = mc_update(&mut false).unwrap();
    let mut custom_args: Vec<String> = Vec::new();
    match env::var("JAVACMDADD") {
        Ok(r) => {
            for add in r.split(" ") {
                custom_args.push(add.to_string());
            }
        }
        _ => (),
    };
    println!("{:#?},{:#?}", start_args, custom_args);
    println!("Starting Minecraft");
    processes::mc_restart(mc_state_mutex.clone(), &custom_args, &start_args).unwrap();
    loop {
        if let Some(next) = upcoming.next() {
            while next >= chrono::Local::now() {
                thread::sleep(time::Duration::from_secs(5));
                let mut mc_state_lock = mc_state_mutex.write().unwrap();
                if let Some(mc_state) = (*mc_state_lock).as_mut() {
                    if mc_state.check_state().unwrap() {
                        (*mc_state_lock) = None;
                        break;
                    }
                } else {
                    println!("Minecraft Closed, Restarting ...");
                    break;
                }
            }
            println!("Checking for Updates");
            match processes::sys_update() {
                Ok(r) => r,
                Err(e) => eprintln!("{:#?}", e),
            };
            let mut updated = false;
            let start_args = match mc_update(&mut updated) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{:#?}", e);
                    Vec::new()
                }
            };
            if updated || mc_state_mutex.read().unwrap().is_none() {
                if updated {
                    println!("Updating Minecraft")
                }
                processes::mc_restart(mc_state_mutex.clone(), &custom_args, &start_args).unwrap();
            }
        } else {
            panic!("None")
        }
    }
}

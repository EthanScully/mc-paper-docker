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
    let schedule = utils::parse_cron().unwrap();
    let mut upcoming = schedule.upcoming(chrono::Local);
    println!("Initial System Update");
    processes::sys_update().unwrap();
    println!("Intial PaperMC API Check");
    mc(&mut false).unwrap();
    let mut java_command = vec![
        "-jar".to_string(),
        "mc.jar".to_string(),
        "nogui".to_string(),
    ];
    match env::var("JAVACMDADD") {
        Ok(r) => {
            for add in r.split(" ") {
                java_command.push(add.to_string());
            }
        }
        _ => (),
    };
    let mut mc_state = processes::new();
    println!("Starting Minecraft");
    processes::mc_restart(&mut mc_state, &java_command).unwrap();
    loop {
        if let Some(next) = upcoming.next() {
            while next >= chrono::Local::now() {
                thread::sleep(time::Duration::from_secs(5));
                if let Some(state) = mc_state {
                    mc_state = state.check_state().unwrap();
                    if mc_state.is_none() {
                        println!("Minecraft Closed, Restarting ...");
                        break;
                    }
                }
            }
            println!("Checking for Updates");
            match processes::sys_update() {
                Ok(r) => r,
                Err(e) => eprintln!("{:#?}", e),
            };
            let mut updated = false;
            match mc(&mut updated) {
                Ok(r) => r,
                Err(e) => eprintln!("{:#?}", e),
            };
            if updated || mc_state.is_none() {
                if updated {
                    println!("Updating Minecraft")
                }
                processes::mc_restart(&mut mc_state, &java_command).unwrap();
            }
        } else {
            panic!("None")
        }
    }
}

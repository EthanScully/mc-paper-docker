mod api;
mod config;
mod err;
mod utils;

use std::{
    io::Read,
    path::Path,
    sync::{Arc, OnceLock, RwLock},
    *,
};

use crate::err::{Caller, ErrorCaller};

type Hresult<T> = result::Result<T, String>;

fn mc() -> utils::Result<bool> {
    let mc_json_path = Path::new("mc-config.json");
    let mc_jar_path = Path::new("mc.jar");
    let mc_config = config::load_json(mc_json_path).e()?;
    let latest_version = match mc_config.check_update().e()? {
        Some(o) => o,
        None => return Ok(false),
    };
    let jar_download = latest_version.download().e()?;
    fs::write(mc_jar_path, jar_download).e()?;
    latest_version.write_json(mc_json_path).e()?;
    Ok(true)
}
fn main() {
    let schedule = utils::parse_cron(env::args()).unwrap();
    let mut upcoming = schedule.upcoming(chrono::Local);
    sys_update().unwrap();
    mc().unwrap();
    mc_restart();
    loop {
        if let Some(next) = upcoming.next() {
            let sleep_duration = (next - chrono::Local::now()).to_std().unwrap_or_default();
            thread::sleep(sleep_duration);
            sys_update().unwrap();
            let updated = match mc() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{:#?}", e);
                    false
                }
            };
            if updated {
                mc_restart();
            }
        }
    }
}
fn sys_update() -> utils::Result<()> {
    run_simple_command("apt", &["upgrade"]).e()?;
    run_simple_command("apt", &["install", "openjdk-21-jre-headless", "f"]).e()?;
    Ok(())
}
fn run_simple_command(arg0: &str, args: &[&str]) -> utils::Result<()> {
    let mut command = process::Command::new(arg0);
    for arg in args {
        _ = command.arg(arg)
    }
    let mut child = command
        .stderr(process::Stdio::piped())
        .stdout(process::Stdio::piped())
        .spawn()
        .e()?;
    let mut stdout = child.stdout.take().o()?;
    let mut stderr = child.stderr.take().o()?;
    let stdout_handle = thread::spawn(move || -> Hresult<String> {
        let mut out = String::new();
        match stdout.read_to_string(&mut out).e() {
            Err(e) => return Err(e.to_string()),
            _ => (),
        };
        Ok(out)
    });
    let stderr_handle = thread::spawn(move || -> Hresult<String> {
        let mut out = String::new();
        match stderr.read_to_string(&mut out).e() {
            Err(e) => return Err(e.to_string()),
            _ => (),
        };
        Ok(out)
    });
    let stdout = stdout_handle.join().o()??;
    let stderr = stderr_handle.join().o()??;
    if let Some(exit_status) = child.try_wait().e()? {
        if !exit_status.success() {
            return Err(err::new(format!("{}{}", stdout.trim(), stderr.trim())))?;
        }
    }
    return Ok(());
}


static MC_STATE: OnceLock<Arc<RwLock<String>>> = OnceLock::new();

fn mc_restart() {}

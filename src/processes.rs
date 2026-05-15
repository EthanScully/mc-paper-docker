use crate::{
    err::{self, Caller, ErrorCaller},
    utils,
};
use std::{
    io::{BufRead, Read, Write},
    sync::{Arc, RwLock},
    *,
};
pub fn sys_update() -> utils::Result<()> {
    run_simple_command("apt", &["update"]).e()?;
    run_simple_command("apt", &["install", "openjdk-25-jre-headless", "-y"]).e()?;
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
    let stdout_handle = thread::spawn(move || -> Result<String, String> {
        let mut out = String::new();
        match stdout.read_to_string(&mut out).e() {
            Err(e) => return Err(e.to_string()),
            _ => (),
        };
        Ok(out)
    });
    let stderr_handle = thread::spawn(move || -> Result<String, String> {
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

pub struct Process {
    child: process::Child,
    stdin: process::ChildStdin,
}
impl Process {
    /// returns true if process is dead
    pub fn check_state(&mut self) -> utils::Result<bool> {
        if self.child.try_wait().e()?.is_some() {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
pub fn mc_restart(mc_state_mutex: Arc<RwLock<Option<Process>>>,args: &Vec<String>, start_args: &Vec<String>) -> utils::Result<()> {
    if let Some(mut mc_state) = (*mc_state_mutex.write().o()?).take() {
        if mc_state.child.try_wait().e()?.is_none() {
            let msg = "\n/stop\n";
            mc_state.stdin.write_all(msg.as_bytes()).e()?;
            mc_state.stdin.flush().e()?;
            let mut exited = false;
            for _ in 0..300 {
                if mc_state.child.try_wait().e()?.is_some() {
                    exited = true;
                    break;
                }
                thread::sleep(time::Duration::from_secs(1));
            }
            if !exited {
                mc_state.child.kill().e()?;
            }
            mc_state.child.wait().e()?;
        }
    }
    let mut command = process::Command::new("java");
    for arg in args {
        _ = command.arg(arg)
    }
    for arg in start_args {
        _ = command.arg(arg)
    }
    let mut child = command
        .arg("-jar")
        .arg("mc.jar")
        .arg("nogui")
        .stdin(process::Stdio::piped())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .spawn()
        .e()?;
    let child_stdin = child.stdin.take().o()?;
    let state = Process {
        child,
        stdin: child_stdin,
    };
    *mc_state_mutex.write().o()? = Some(state);
    Ok(())
}

pub fn grab_stdin() -> Arc<RwLock<Option<Process>>> {
    let process: Arc<RwLock<Option<Process>>> = Arc::new(RwLock::new(None));
    let p = process.clone();
    thread::spawn(move || {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = String::new();
        loop {
            buffer.clear();
            match handle.read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => {
                    let mut child_process_option = match p.write() {
                        Ok(r) => r,
                        _ => break,
                    };
                    if let Some(child_process) = (*child_process_option).as_mut() {
                        if child_process.stdin.write_all(buffer.as_bytes()).is_err() {
                            continue;
                        }
                        _ = child_process.stdin.flush();
                    }
                }
                Err(_) => break,
            }
        }
    });
    process
}

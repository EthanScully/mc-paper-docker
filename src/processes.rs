use crate::{
    err::{self, Caller, ErrorCaller},
    utils,
};
use std::{
    io::{Read, Write},
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
    pub fn check_state(mut self) -> utils::Result<Option<Self>> {
        if self.child.try_wait().e()?.is_some() {
            return Ok(None)
        } else {
            return Ok(Some(self))
        }
    }
}

pub fn new() -> Option<Process> {
    None
}
pub fn mc_restart(mc_state: &mut Option<Process>, args: &Vec<String>) -> utils::Result<()> {
    if let Some(mut state) = mc_state.take() {
        if state.child.try_wait().e()?.is_none() {
            let msg = "/stop\n";
            state.stdin.write_all(msg.as_bytes()).e()?;
            state.stdin.flush().e()?;
            let mut exited = false;
            for _ in 0..300 {
                if state.child.try_wait().e()?.is_some() {
                    exited = true;
                    break;
                }
                thread::sleep(time::Duration::from_secs(1));
            }
            if !exited {
                state.child.kill().e()?;
            }
            state.child.wait().e()?;
        }
    }
    let mut command = process::Command::new("java");
    for arg in args {
        _ = command.arg(arg)
    }
    let mut child = command
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
    *mc_state = Some(state);
    Ok(())
}

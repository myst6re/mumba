#[cfg(windows)]
use crate::os::windows::DETACHED_PROCESS;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};

#[cfg(windows)]
pub mod regedit;
#[cfg(windows)]
pub mod windows;

fn run_detached(command: &mut Command) -> &mut Command {
    if cfg!(windows) {
        #[cfg(windows)]
        return command.creation_flags(DETACHED_PROCESS);
    }

    command
}

pub fn run_helper(command: &mut Command) -> &mut Command {
    run_detached(command)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
}

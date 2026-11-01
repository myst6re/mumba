use std::process::{Command, Stdio};

#[cfg(windows)]
pub mod regedit;
#[cfg(windows)]
pub mod windows;

pub fn run_helper(command: &mut Command) -> &mut Command {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
}

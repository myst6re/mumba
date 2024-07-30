use std::process::{Command, Stdio};
use std::os::windows::process::CommandExt;

const DETACHED_PROCESS: u32 = 0x8;

fn main() {
    match std::env::args().next() {
        Some(ff8_path) => {
            Command::new(&ff8_path)
                .creation_flags(DETACHED_PROCESS)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn().unwrap();
        },
        None => ()
    }
}

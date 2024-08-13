#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::{error, info};
use same_file::is_same_file;
use simplelog::{CombinedLogger, LevelFilter, SimpleLogger, WriteLogger};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

fn run() -> std::io::Result<()> {
    CombinedLogger::init(vec![
        SimpleLogger::new(LevelFilter::Debug, simplelog::Config::default()),
        WriteLogger::new(
            LevelFilter::Info,
            simplelog::Config::default(),
            File::create("launcher.log").unwrap(),
        ),
    ])
    .unwrap_or_default();
    let current_exe = std::env::current_exe()?;
    let path = match std::env::args().nth(1) {
        Some(ff8_path) => PathBuf::from(ff8_path),
        None => {
            let mut exe_path = current_exe.clone();
            exe_path.pop();
            let config_path = exe_path.join("moomba_path.txt");
            let path = std::fs::read_to_string(config_path);
            match path {
                Ok(path) => PathBuf::from(path.trim()),
                Err(e) => {
                    error!("Error reading link: {}", e);
                    exe_path.join("FF8_Launcher_Original.exe")
                }
            }
        }
    };
    let dir = match path.parent() {
        Some(dir) => dir,
        None => Path::new("."),
    };
    info!(
        "Path={} Dir={}",
        &path.to_str().unwrap(),
        &dir.to_str().unwrap()
    );

    if is_same_file(&path, current_exe)? {
        error!("Target exe is the current exe itself");
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Target exe is the current exe itself",
        ))
    } else {
        let mut child = Command::new(&path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(dir)
            .spawn()?;
        info!("Wait for child process...");
        let exit_status = child.wait()?;
        info!("Child process exited with status {}", exit_status);
        Ok(())
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            error!("Error: {:?}", e);
            ExitCode::FAILURE
        }
    }
}

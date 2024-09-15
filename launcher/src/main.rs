#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use log::{error, info};
use same_file::is_same_file;
use simplelog::{CombinedLogger, LevelFilter, SimpleLogger, WriteLogger};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

fn path_fallback(exe_path: &Path) -> PathBuf {
    exe_path.join("FF8_Launcher_Original.exe")
}

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
    let mut exe_path = current_exe.clone();
    exe_path.pop();
    let mut path = match std::env::args().nth(1) {
        Some(ff8_path) => {
            info!("Build ff8 path from arg");
            PathBuf::from(ff8_path)
        }
        None => {
            let config_path = exe_path.join("mumba_path.txt");
            info!(
                "Build ff8 path from file at {}",
                config_path.to_string_lossy()
            );
            let path = std::fs::read_to_string(config_path);
            match path {
                Ok(path) => PathBuf::from(path.trim()),
                Err(e) => {
                    error!("Error reading mumba_path.txt: {}", e);
                    path_fallback(&exe_path)
                }
            }
        }
    };

    if !path.exists() {
        error!(
            "Path {} does not exist, fallback to the original launcher",
            path.to_string_lossy()
        );
        path = path_fallback(&exe_path)
    }

    let dir = match path.parent() {
        Some(dir) => dir,
        None => Path::new("."),
    };
    info!(
        "Path={} Dir={}",
        &path.to_string_lossy(),
        &dir.to_string_lossy()
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
            error!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

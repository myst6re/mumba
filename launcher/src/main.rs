#![cfg_attr(feature = "release", windows_subsystem = "windows")]

use std::process::{Command, Stdio};
use std::path::{Path, PathBuf};
use same_file::is_same_file;

fn main() -> std::io::Result<()> {
    let current_exe = std::env::current_exe()?;
    let path = match std::env::args().nth(1) {
        Some(ff8_path) => PathBuf::from(ff8_path),
        None => {
            let mut exe_path = current_exe.clone();
            exe_path.pop();
            exe_path.join("FF8_Launcher_Original.exe")
        }
    };
    let dir = match path.parent() {
        Some(dir) => dir,
        None => Path::new(".")
    };

    if is_same_file(&path, current_exe)? {
        Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Target exe is the current exe itself"))
    } else {
        println!("Path={} Dir={}", &path.to_str().unwrap(), &dir.to_str().unwrap());
        let mut child = Command::new(&path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .current_dir(dir)
            .spawn()?;
        child.wait()?;
        Ok(())
    }
}

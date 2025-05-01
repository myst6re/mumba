use std::io;
use std::path::{Path, PathBuf};

pub struct Env {
    pub cache_dir: PathBuf,
    pub data_dir: PathBuf,
    pub config_path: PathBuf,
    pub mumba_dir: PathBuf,
    pub ffnx_dir: PathBuf,
    pub log_path: PathBuf,
}

impl Env {
    pub fn new(program_name: &str) -> Result<Self, std::io::Error> {
        let mut exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("./mumba"));
        exe_path.pop(); // Remove exe filename

        let cache_fallback = exe_path.clone();
        let data_fallback = exe_path.clone();
        let config_fallback = exe_path.clone();

        let config_path = config_fallback.join("mumba.toml");
        if config_path.exists() {
            let log_path = data_fallback.join(format!("{}.log", program_name));
            // Local installation
            return Ok(Self {
                cache_dir: cache_fallback,
                data_dir: data_fallback,
                config_path,
                mumba_dir: exe_path.clone(),
                ffnx_dir: exe_path,
                log_path,
            });
        }

        let base_dirs = directories::BaseDirs::new();
        let (cache_dir, data_dir, config_dir) = match base_dirs {
            Some(d) => (
                Self::add_app_dir(d.cache_dir()),
                Self::add_app_dir(d.data_dir()),
                Self::add_app_dir(d.config_dir()),
            ),
            None => (
                cache_fallback.clone(),
                data_fallback.clone(),
                config_fallback.clone(),
            ),
        };
        let cache_dir = Self::create_dir(&cache_dir)
            .and(Ok(cache_dir))
            .or_else(|_| Self::create_dir(&cache_fallback).and(Ok(cache_fallback)))?;
        let data_dir = Self::create_dir(&data_dir)
            .and(Ok(data_dir))
            .or_else(|_| Self::create_dir(&data_fallback).and(Ok(data_fallback)))?;
        let log_path = data_dir.join(format!("{}.log", program_name));
        let config_dir = Self::create_dir(&config_dir)
            .and(Ok(config_dir))
            .or_else(|_| Self::create_dir(&config_fallback).and(Ok(config_fallback)))?;
        let ffnx_dir = data_dir.join("game");

        Ok(Self {
            cache_dir,
            data_dir,
            config_path: config_dir.join("mumba.toml"),
            mumba_dir: exe_path,
            ffnx_dir,
            log_path,
        })
    }

    fn create_dir<P: AsRef<std::path::Path>>(path: P) -> io::Result<()> {
        match std::fs::create_dir_all(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn add_app_dir(path: &Path) -> PathBuf {
        path.to_path_buf().join("mumba")
    }

    pub fn get_resource_launcher_path(&self) -> PathBuf {
        let resource_launcher_path = self.mumba_dir.join("ff8_launcher.exe");
        if resource_launcher_path.exists() {
            resource_launcher_path
        } else {
            PathBuf::from("/var/lib/mumba/ff8_launcher.exe")
        }
    }
}

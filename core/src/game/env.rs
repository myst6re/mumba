use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub struct Env {
    pub cache_dir: PathBuf,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub moomba_dir: PathBuf,
    pub ffnx_dir: PathBuf,
}

impl Env {
    pub fn new() -> Result<Self, std::io::Error> {
        let base_dirs = directories::BaseDirs::new();
        let cache_fallback = PathBuf::from_str("./cache").unwrap();
        let data_fallback = PathBuf::from_str("./data").unwrap();
        let config_fallback = PathBuf::from_str("./config").unwrap();
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
        let config_dir = Self::create_dir(&config_dir)
            .and(Ok(config_dir))
            .or_else(|_| Self::create_dir(&config_fallback).and(Ok(config_fallback)))?;
        let mut exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("./"));
        exe_path.pop(); // Remove exe filename
        let ffnx_dir = data_dir.join("game");
        Ok(Self {
            cache_dir,
            data_dir,
            config_dir,
            moomba_dir: exe_path,
            ffnx_dir,
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
        path.to_path_buf().join("moomba")
    }
}

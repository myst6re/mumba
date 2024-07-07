use std::io;
use std::str::FromStr;
use std::path::PathBuf;

pub struct Env {
    pub cache_dir: PathBuf,
    pub moomba_dir: PathBuf,
    pub ffnx_dir: PathBuf
}

impl Env {
    pub fn new() -> Result<Self, std::io::Error> {
        let base_dirs = directories::BaseDirs::new();
        let cache_fallback = PathBuf::from_str("./cache").unwrap();
        let cache_dir = base_dirs.map_or(cache_fallback.clone(), |d| d.cache_dir().to_path_buf().join("Moomba"));
        let cache_dir = Self::create_dir(&cache_dir).and(Ok(cache_dir)).or_else(|_|
            Self::create_dir(&cache_fallback).and(Ok(cache_fallback))
        )?;
        let mut exe_path = std::env::current_exe()?;
        exe_path.pop(); // Remove exe filename
        let ffnx_dir = exe_path.clone().join("game");
        Ok(Self {
            cache_dir,
            moomba_dir: exe_path,
            ffnx_dir
        })
    }

    fn create_dir<P: AsRef<std::path::Path>>(path: P) -> io::Result<()> {
        match std::fs::create_dir(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
            Err(e) => Err(e)
        }
    }
}

use simplelog::{CombinedLogger, LevelFilter, SimpleLogger, WriteLogger};
use std::fs::File;
use std::path::Path;

pub fn init(log_path: &Path) {
    CombinedLogger::init(vec![
        SimpleLogger::new(LevelFilter::Debug, simplelog::Config::default()),
        WriteLogger::new(
            LevelFilter::Info,
            simplelog::Config::default(),
            File::create(log_path).unwrap(),
        ),
    ])
    .unwrap();
}

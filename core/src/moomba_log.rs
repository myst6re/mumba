use crate::game::env::Env;
use simplelog::{CombinedLogger, LevelFilter, SimpleLogger, WriteLogger};
use std::fs::File;

pub fn init(env: &Env, name: &str) {
    CombinedLogger::init(vec![
        SimpleLogger::new(LevelFilter::Debug, simplelog::Config::default()),
        WriteLogger::new(
            LevelFilter::Info,
            simplelog::Config::default(),
            File::create(env.data_dir.join(name)).unwrap(),
        ),
    ])
    .unwrap();
}

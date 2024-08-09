use crate::game::env::Env;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;

pub fn init(env: &Env, name: &str) {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
            simplelog::Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            simplelog::Config::default(),
            File::create(env.data_dir.join(name)).unwrap(),
        ),
    ])
    .unwrap();
}

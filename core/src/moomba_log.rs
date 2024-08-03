use simplelog::{CombinedLogger, TermLogger, WriteLogger, LevelFilter, TerminalMode, ColorChoice};
use std::fs::File;
use crate::game::env::Env;

pub fn init(env: &Env, name: &str) {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, simplelog::Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), File::create(env.data_dir.join(name)).unwrap()),
        ]
    ).unwrap();
}

use simplelog::{CombinedLogger, TermLogger, WriteLogger, LevelFilter, TerminalMode, ColorChoice};
use std::fs::File;

pub fn init() {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, simplelog::Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), File::create("moomba.log").unwrap()),
        ]
    ).unwrap();
}

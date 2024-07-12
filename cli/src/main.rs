use moomba_core::game::env::Env;
use std::path::PathBuf;
use std::io::Error;
use simplelog::{CombinedLogger, TermLogger, WriteLogger, LevelFilter, TerminalMode, ColorChoice};
use std::fs::File;
use log::info;

fn copy_launcher(target_dir: &String) -> Result<(), Error> {
    let env = Env::new()?;
    let launcher_name = "FF8_Launcher.exe";
    let target_file = PathBuf::new().join(target_dir).join(&launcher_name);
    let from = env.moomba_dir.join(&launcher_name);

    info!("Copy {:?} to {:?}", &from, &target_file);

    std::fs::copy(from, target_file).and(Ok(()))
}

fn main() -> Result<(), Error> {
    CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Debug, simplelog::Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), File::create("moomba_patcher.log").unwrap()),
        ]
    ).unwrap();

    let target_dir = std::env::args().nth(1).expect("Please set target directory");

    copy_launcher(&target_dir)
}

use clap::{arg, Command};
use moomba_core::game::env::Env;
use moomba_core::game::installation::{Edition, Installation};
use std::path::PathBuf;

fn cli() -> Command {
    Command::new("mmb")
        .about("Modern and fast FFNx configurator")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("replace_launcher")
                .about("Replaces the launcher")
                .arg(arg!(<APP_PATH> "The app path of the game"))
                .arg_required_else_help(true),
        )
}

fn main() -> std::io::Result<()> {
    let env = Env::new()?;
    moomba_core::moomba_log::init(&env, "mmb.log");

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("replace_launcher", sub_matches)) => {
            let app_path = sub_matches.get_one::<String>("APP_PATH").expect("required");
            match Installation::from_directory(PathBuf::from(app_path), Edition::Steam) {
                Some(installation) => Installation::replace_launcher_from_app_path(
                    &installation.app_path,
                    &env.ffnx_dir.join(installation.exe_name),
                    &env,
                ),
                None => Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "App not found",
                )),
            }
        }
        Some((_, _)) | None => unreachable!(),
    }
}

use clap::{arg, Command};
use mumba_core::game::env::Env;
use mumba_core::game::installation::{Edition, Installation};

#[cfg(feature = "git")]
include!(concat!(env!("OUT_DIR"), "/built.rs"));

#[cfg(feature = "git")]
pub static GIT_VERSION_MMB: Option<&str> = GIT_VERSION;
#[cfg(not(feature = "git"))]
pub static GIT_VERSION_MMB: Option<&str> = Some("dev");

fn cli() -> Command {
    Command::new("mmb")
        .version(GIT_VERSION_MMB)
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
    let env = Env::new("mmb")?;
    mumba_core::mumba_log::init(&env.log_path);

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("replace_launcher", sub_matches)) => {
            let app_path = sub_matches.get_one::<String>("APP_PATH").expect("required");
            match Installation::from_directory(app_path, Some(Edition::Steam)) {
                Some(installation) => installation.replace_launcher_from_app_path(&env),
                None => Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "App not found",
                )),
            }
        }
        Some((_, _)) | None => unreachable!(),
    }
}

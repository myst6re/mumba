use super::{AppWindow, Installations};
use crate::TextLevel;
use log::{error, info, warn};
use moomba_core::config::Config;
use moomba_core::game::env::Env;
use moomba_core::game::ffnx::Ffnx;
use moomba_core::game::ffnx_config::FfnxConfig;
use moomba_core::game::installation;
use moomba_core::pe_format;
use moomba_core::provision;
use slint::ComponentHandle;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use thiserror::Error;

#[cfg(windows)]
const DETACHED_PROCESS: u32 = 0x8;

#[derive(Debug)]
pub enum Message {
    Setup(slint::SharedString),
    LaunchGame,
    ConfigureGame,
    UpdateGame,
    Quit,
}

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("Install error: {0}")]
    ProvisionError(#[from] provision::ErrorBox),
    #[error("Install error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("4GB patch Error: {0}")]
    PeFormatError(#[from] pe_format::Error),
}

pub struct Worker {
    pub tx: Sender<Message>,
    thread: std::thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(ui: &AppWindow) -> Self {
        let (tx, rx) = mpsc::channel::<Message>();
        let thread = std::thread::spawn({
            let handle_weak = ui.as_weak();
            move || worker_loop(rx, handle_weak)
        });
        Self { tx, thread }
    }

    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.tx.send(Message::Quit);
        self.thread.join()
    }
}

fn set_task_text(handle: slint::Weak<AppWindow>, text_level: TextLevel, text: &'static str) {
    handle
        .upgrade_in_event_loop(move |h| {
            let installations = h.global::<Installations>();
            installations.set_task_text(slint::SharedString::from(text));
            installations.set_task_text_type(text_level)
        })
        .unwrap_or_default()
}

fn set_game_ready(handle: slint::Weak<AppWindow>, ready: bool) {
    handle
        .upgrade_in_event_loop(move |h| h.global::<Installations>().set_is_ready(ready))
        .unwrap_or_default()
}

fn set_game_exe_path(handle: slint::Weak<AppWindow>, text: String) {
    handle
        .upgrade_in_event_loop(move |h| {
            h.global::<Installations>()
                .set_game_exe_path(slint::SharedString::from(text))
        })
        .unwrap_or_default()
}

fn set_current_page(handle: slint::Weak<AppWindow>, page_id: i32) {
    handle
        .upgrade_in_event_loop(move |h| h.global::<Installations>().set_current_page(page_id))
        .unwrap_or_default()
}

fn upgrade_ffnx(
    handle: slint::Weak<AppWindow>,
    ffnx_version: &String,
    edition: &installation::Edition,
    env: &Env,
) {
    set_task_text(handle.clone(), TextLevel::Info, "Check for FFNx update…");
    let (url, version) = Ffnx::find_last_stable_version_on_github("julianxhokaxhiu/FFNx", edition);
    if *ffnx_version != version {
        set_task_text(handle.clone(), TextLevel::Info, "Upgrading FFNx…");
        set_game_ready(handle.clone(), false);
        match Ffnx::download(url.as_str(), &env.ffnx_dir, env) {
            Ok(()) => (),
            Err(e) => {
                error!("Error when installing FFNx: {:?}", e);
            }
        };
        set_game_ready(handle.clone(), true)
    };
    set_task_text(handle.clone(), TextLevel::Info, "")
}

#[cfg(windows)]
fn launch_game(ff8_path: &Path, ffnx_dir: &Path, _app_id: u64) {
    if let Err(e) = Command::new(ff8_path)
        .creation_flags(DETACHED_PROCESS)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(ffnx_dir)
        .spawn()
    {
        error!("Unable to launch game: {:?}", e)
    }
}

#[cfg(unix)]
fn launch_game(_ff8_path: &Path, _ffnx_dir: &Path, app_id: u64) {
    match app_id {
        0 => {
            error!("Unable to launch game: only the Steam version is supported")
        }
        app_id => {
            if let Err(e) = Command::new("steam")
                .args(["-applaunch", app_id.to_string().as_str()])
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
            {
                error!("Unable to launch game: {:?}", e)
            }
        }
    }
}

fn install_game_and_ffnx(
    handle: slint::Weak<AppWindow>,
    installation: &installation::Installation,
    ff8_path: &PathBuf,
    env: &Env,
) -> Result<Ffnx, InstallError> {
    let ffnx_installation = match Ffnx::from_directory(&env.ffnx_dir, &installation.edition) {
        Some(ffnx_installation) => {
            info!("Found FFNx version {}", ffnx_installation.version);
            ffnx_installation
        }
        None => {
            set_task_text(handle.clone(), TextLevel::Info, "Installing FFNx…");
            let (url, _) = Ffnx::find_last_stable_version_on_github(
                "julianxhokaxhiu/FFNx",
                &installation.edition,
            );
            Ffnx::download(url.as_str(), &env.ffnx_dir, env)?;
            if let Some(ffnx_installation) =
                Ffnx::from_directory(&env.ffnx_dir, &installation.edition)
            {
                ffnx_installation
            } else {
                return Err(InstallError::IOError(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid FFNx installation",
                )));
            }
        }
    };

    if matches!(installation.edition, installation::Edition::Steam) {
        installation.replace_launcher(ff8_path, env)?
    };

    if !ff8_path.exists() {
        moomba_core::provision::copy_file(&installation.exe_path(), ff8_path)?
    }

    let bink_dll_path = env.ffnx_dir.join("binkw32.dll");
    if !bink_dll_path.exists() {
        if matches!(installation.edition, installation::Edition::Standard) {
            if !matches!(installation.version, Some((installation::Version::V120, _))) {
                info!("Patch game to 1.02...");
                let file_name = match installation.language.as_str() {
                    "fre" | "FR" => Some("FF8EidosFre"),
                    "ger" | "DE" => Some("FF8EidosGerV12"),
                    "eng" | "EN" => Some("FF8SqeaPatch"),
                    "spa" | "ES" => Some("ff8ngspa"),
                    "ita" | "IT" => Some("ff8ngita"),
                    "jp" | "JP" => Some("FF8EasqPatch"),
                    _ => None,
                };
                match file_name {
                    Some(file_name) => {
                        let url = format!("https://www.ff8.fr/download/programs/{}.zip", file_name);
                        moomba_core::provision::download_zip(
                            url.as_str(),
                            "FF8Patch1.02.zip",
                            &env.ffnx_dir,
                            env,
                        )?;
                        moomba_core::provision::rename_file(
                            &env.ffnx_dir.join("FF8.exe"),
                            ff8_path,
                        )?;
                    }
                    None => {
                        error!("Cannot detect the language of your game!");
                    }
                }
            }
        } else {
            moomba_core::provision::copy_file(
                &PathBuf::from(&installation.app_path).join("binkw32.dll"),
                &bink_dll_path,
            )?;
            // Clean
            let eax_dll_path = env.ffnx_dir.join("eax.dll");
            if eax_dll_path.exists() {
                std::fs::remove_file(eax_dll_path)?;
            }
        }
    }
    moomba_core::pe_format::pe_patch_4bg(ff8_path)?;
    Ok(ffnx_installation)
}

fn setup(
    handle: slint::Weak<AppWindow>,
    moomba_config: &mut Config,
    moomba_config_path: &PathBuf,
    exe_path: &slint::SharedString,
) -> Option<installation::Installation> {
    info!("Setup with EXE path {}", exe_path);
    match installation::Installation::from_exe_path(&PathBuf::from(exe_path.as_str())) {
        Ok(installation) => {
            moomba_config.set_installation(&installation);
            set_game_exe_path(
                handle.clone(),
                installation.exe_path().to_string_lossy().to_string(),
            );
            match moomba_config.save(moomba_config_path) {
                Ok(()) => Some(installation),
                Err(e) => {
                    error!("Cannot save configuration to config.toml: {:?}", e);
                    set_task_text(
                        handle.clone(),
                        TextLevel::Error,
                        "Cannot save configuration to config.toml",
                    );
                    None
                }
            }
        }
        Err(installation::FromExeError::NotFound) => {
            error!("This file does not exist: {}", exe_path);
            set_task_text(handle.clone(), TextLevel::Error, "File not found");
            None
        }
        Err(installation::FromExeError::LauncherSelected) => {
            error!("Select the game exe, not the launcher: {}", exe_path);
            set_task_text(handle.clone(), TextLevel::Error, "File not found");
            None
        }
    }
}

fn go_to_setup_page(
    rx: &Receiver<Message>,
    handle: slint::Weak<AppWindow>,
    moomba_config: &mut Config,
    moomba_config_path: &PathBuf,
) -> Option<installation::Installation> {
    loop {
        set_current_page(handle.clone(), 1);
        match rx.recv() {
            Ok(Message::Setup(exe_path)) => {
                match setup(handle.clone(), moomba_config, moomba_config_path, &exe_path) {
                    Some(installation) => return Some(installation),
                    None => continue,
                }
            }
            Ok(Message::Quit) => return None,
            msg => {
                error!("Received unknown message: {:?}", msg);
                set_task_text(
                    handle.clone(),
                    TextLevel::Error,
                    "Fatal error: Unknown message received. See logs for more details.",
                );
                continue;
            }
        }
    }
}

fn retrieve_installation(
    rx: &Receiver<Message>,
    handle: slint::Weak<AppWindow>,
    moomba_config: &mut Config,
    moomba_config_path: &PathBuf,
) -> Option<installation::Installation> {
    match moomba_config.installation() {
        Ok(Some(installation)) => {
            set_game_exe_path(
                handle.clone(),
                installation.exe_path().to_string_lossy().to_string(),
            );
            Some(installation)
        }
        Ok(None) | Err(_) => {
            let installations = installation::Installation::search();
            for inst in installations {
                match inst.edition {
                    installation::Edition::Standard | installation::Edition::Steam => {
                        set_game_exe_path(
                            handle.clone(),
                            inst.exe_path().to_string_lossy().to_string(),
                        );
                    }
                    installation::Edition::Remastered => {
                        warn!(
                            "Ignore remaster at {}, as Moomba is not compatible yet",
                            inst.app_path.to_string_lossy()
                        )
                    }
                }
            }

            go_to_setup_page(rx, handle.clone(), moomba_config, moomba_config_path)
        }
    }
}

fn retrieve_ffnx_installation(
    rx: &Receiver<Message>,
    handle: slint::Weak<AppWindow>,
    moomba_config: &mut Config,
    installation: &mut installation::Installation,
    ff8_path: &PathBuf,
    env: &Env,
) -> Option<Ffnx> {
    loop {
        match install_game_and_ffnx(handle.clone(), installation, ff8_path, env) {
            Ok(ffnx_installation) => return Some(ffnx_installation),
            Err(e) => {
                error!("Installation error: {}", e);
                set_task_text(handle.clone(), TextLevel::Error, "Cannot install FFNx");

                *installation =
                    match go_to_setup_page(rx, handle.clone(), moomba_config, &env.config_path) {
                        Some(inst) => inst,
                        None => return None, // Exit
                    }
            }
        };
    }
}

fn worker_loop(rx: Receiver<Message>, handle: slint::Weak<AppWindow>) {
    #[allow(unused_mut)]
    let mut env = match moomba_core::game::env::Env::new() {
        Ok(env) => env,
        Err(e) => {
            error!("Cannot initialize environment: {:?}", e);
            set_task_text(
                handle.clone(),
                TextLevel::Error,
                "Cannot initialize environment",
            );
            return;
        }
    };
    let mut moomba_config = Config::from_file(&env.config_path).unwrap_or_else(|_| Config::new());

    let mut installation =
        match retrieve_installation(&rx, handle.clone(), &mut moomba_config, &env.config_path) {
            Some(installation) => installation,
            None => return, // Exit
        };

    info!(
        "Found Game at {:?}: {:?} {} {:?}",
        &installation.app_path,
        &installation.edition,
        &installation.language,
        &installation.version
    );

    let ff8_exe_name = if cfg!(unix) {
        // Steam + Proton refuses to launch the game if the exe is outside the base dir of the app
        env.ffnx_dir = PathBuf::from(&installation.app_path);
        &installation.exe_name
    } else if matches!(installation.edition, installation::Edition::Steam) {
        "FF8_Moomba_Steam.exe"
    } else {
        "FF8_Moomba.exe"
    };
    let ff8_path = env.ffnx_dir.join(ff8_exe_name);

    let mut ffnx_installation = match retrieve_ffnx_installation(
        &rx,
        handle.clone(),
        &mut moomba_config,
        &mut installation,
        &ff8_path,
        &env,
    ) {
        Some(ffnx_installation) => ffnx_installation,
        None => return, // Exit
    };

    set_task_text(handle.clone(), TextLevel::Info, "");
    set_game_ready(handle.clone(), true);

    upgrade_ffnx(
        handle.clone(),
        &ffnx_installation.version,
        &installation.edition,
        &env,
    );

    let ffnx_config_path = env.ffnx_dir.join("FFNx.toml");
    let mut ffnx_config = match FfnxConfig::from_file(&ffnx_config_path) {
        Ok(c) => c,
        Err(_e) => FfnxConfig::new(),
    };
    ffnx_config.set_app_path(installation.app_path.to_string_lossy().to_string());
    ffnx_config.save(&ffnx_config_path);

    for received in &rx {
        match received {
            Message::Setup(exe_path) => {
                set_game_ready(handle.clone(), false);
                installation = match setup(
                    handle.clone(),
                    &mut moomba_config,
                    &env.config_path,
                    &exe_path,
                ) {
                    Some(inst) => inst,
                    None => break,
                };
                ffnx_installation = match retrieve_ffnx_installation(
                    &rx,
                    handle.clone(),
                    &mut moomba_config,
                    &mut installation,
                    &ff8_path,
                    &env,
                ) {
                    Some(version) => version,
                    None => return, // Exit
                };
                set_game_ready(handle.clone(), true);
            }
            Message::UpdateGame => upgrade_ffnx(
                handle.clone(),
                &ffnx_installation.version,
                &installation.edition,
                &env,
            ),
            Message::LaunchGame => {
                info!("Launch {:?} in dir {:?}...", &ff8_path, &env.ffnx_dir);
                launch_game(&ff8_path, &env.ffnx_dir, installation.get_app_id())
            }
            Message::ConfigureGame => {
                ffnx_config.save(&ffnx_config_path);
            }
            Message::Quit => break,
        };
        set_task_text(handle.clone(), TextLevel::Info, "");
    }
}

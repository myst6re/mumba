use super::{AppWindow, Installations};
use crate::lazy_ffnx_config::LazyFfnxConfig;
use crate::TextLevel;
use log::{error, info, warn};
use moomba_core::config::Config;
use moomba_core::game::env::Env;
use moomba_core::game::ffnx_config;
use moomba_core::game::ffnx_installation::FfnxInstallation;
use moomba_core::game::installation;
use moomba_core::pe_format;
use moomba_core::provision;
use moomba_core::toml;
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
    ConfigureFfnx,
    CancelConfigureFfnx,
    SetFfnxConfigBool(slint::SharedString, bool),
    SetFfnxConfigInt(slint::SharedString, i64),
    SetFfnxConfigString(slint::SharedString, slint::SharedString),
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
    #[error("Configure Error: {0}")]
    TomlFileError(#[from] toml::FileError),
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

fn set_ffnx_config(handle: slint::Weak<AppWindow>, ffnx_config: &mut LazyFfnxConfig) {
    let config = crate::FfnxConfig {
        renderer_backend: ffnx_config.get_int(ffnx_config::CFG_RENDERER_BACKEND, 0),
        fullscreen: ffnx_config.get_bool(ffnx_config::CFG_FULLSCREEN, true),
        borderless: ffnx_config.get_bool(ffnx_config::CFG_BORDERLESS, false),
        enable_vsync: ffnx_config.get_bool(ffnx_config::CFG_ENABLE_VSYNC, true),
        enable_antialiasing: ffnx_config.get_int(ffnx_config::CFG_ENABLE_ANTIALIASING, 0),
        enable_anisotropic: ffnx_config.get_bool(ffnx_config::CFG_ENABLE_ANISOTROPIC, true),
        ff8_use_gamepad_icons: ffnx_config.get_bool(ffnx_config::CFG_FF8_USE_GAMEPAD_ICONS, true),
    };
    handle
        .upgrade_in_event_loop(move |h| h.global::<Installations>().set_ffnx_config(config))
        .unwrap_or_default()
}

fn upgrade_ffnx(
    handle: slint::Weak<AppWindow>,
    ffnx_installation: &FfnxInstallation,
    edition: &installation::Edition,
    env: &Env,
) {
    set_task_text(handle.clone(), TextLevel::Info, "Check for FFNx update…");
    let (url, version) =
        FfnxInstallation::find_last_stable_version_on_github("julianxhokaxhiu/FFNx", edition);
    if *ffnx_installation.version != version {
        set_task_text(handle.clone(), TextLevel::Info, "Upgrading FFNx…");
        set_game_ready(handle.clone(), false);
        match FfnxInstallation::download(url.as_str(), &ffnx_installation.path, env) {
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
    env: &Env,
) -> Result<FfnxInstallation, InstallError> {
    let ffnx_installation = match FfnxInstallation::from_directory(&env.ffnx_dir, installation) {
        Some(ffnx_installation) => {
            info!("Found FFNx version {}", ffnx_installation.version);
            ffnx_installation
        }
        None => {
            set_task_text(handle.clone(), TextLevel::Info, "Installing FFNx…");
            let (url, _) = FfnxInstallation::find_last_stable_version_on_github(
                "julianxhokaxhiu/FFNx",
                &installation.edition,
            );
            FfnxInstallation::download(url.as_str(), &env.ffnx_dir, env)?;
            if let Some(ffnx_installation) =
                FfnxInstallation::from_directory(&env.ffnx_dir, installation)
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

    let mut lazy_ffnx_config = LazyFfnxConfig::new(&ffnx_installation);
    let ffnx_config = lazy_ffnx_config.get();
    ffnx_config.set_bool("show_fps", false);
    ffnx_config.set_bool("show_renderer_backend", false);
    ffnx_config.set_bool("show_stats", false);
    ffnx_config.set_bool("show_version", false);
    lazy_ffnx_config.save()?;

    let exe_path = ffnx_installation.exe_path();

    if matches!(installation.edition, installation::Edition::Steam) {
        installation.replace_launcher(&exe_path, env)?
    };

    if !exe_path.exists() {
        provision::copy_file(&installation.exe_path(), &exe_path)?
    }

    let bink_dll_path = ffnx_installation.path.join("binkw32.dll");
    if !bink_dll_path.exists() {
        if matches!(installation.edition, installation::Edition::Standard) {
            let file_name = match &installation.version {
                Some((installation::Version::V100, publisher)) => match publisher {
                    installation::Publisher::EaJp => Some("FF8EasqPatch"),
                    installation::Publisher::EaUs | installation::Publisher::EidosUk => {
                        Some("FF8SqeaPatch")
                    }
                    installation::Publisher::EidosDe => Some("FF8EidosGerV12"),
                    installation::Publisher::EidosFr => Some("FF8EidosFre"),
                    installation::Publisher::EidosIt => Some("ff8ngita"),
                    installation::Publisher::EidosSp => Some("ff8ngspa"),
                },
                _ => None,
            };
            match file_name {
                Some(file_name) => {
                    let url = format!("https://www.ff8.fr/download/programs/{}.zip", file_name);
                    provision::download_zip(
                        url.as_str(),
                        "FF8Patch1.02.zip",
                        &ffnx_installation.path,
                        env,
                    )?;
                    provision::rename_file(&ffnx_installation.path.join("FF8.exe"), &exe_path)?;
                }
                None => {
                    error!("Cannot detect the language of your game!");
                }
            }
        } else {
            provision::copy_file(&installation.app_path.join("binkw32.dll"), &bink_dll_path)?;
        }
    }
    match installation.edition {
        installation::Edition::Steam => {
            let eax_dll_path = ffnx_installation.path.join("eax.dll");
            if !eax_dll_path.exists()
                || pe_format::pe_version_info(&eax_dll_path)?
                    .product_name
                    .unwrap_or_default()
                    != "FFNx"
            {
                provision::copy_file(&installation.app_path.join("eax.dll"), &eax_dll_path)?
            }
        }
        installation::Edition::Standard | installation::Edition::Remastered => {
            let eax_dll_path = ffnx_installation.path.join("creative_eax.dll");
            if !eax_dll_path.exists() {
                if let Err(e) = provision::copy_file(&env.moomba_dir.join("eax.dll"), &eax_dll_path)
                {
                    warn!("Cannot install eax.dll: {}", e);
                }
            }
        }
    }
    if matches!(&installation.edition, installation::Edition::Standard) {
        let ff8_input = ffnx_installation.path.join("override").join("ff8input.cfg");
        if !ff8_input.exists() {
            std::fs::create_dir_all(ffnx_installation.path.join("override"))?;
            provision::copy_file(&installation.app_path.join("ff8input.cfg"), &ff8_input).or_else(
                |e| {
                    warn!(
                        "Error when copying ff8input.cfg, creating a new one instead: {}",
                        e
                    );
                    moomba_core::game::input_config::InputConfig::new(&installation.edition)
                        .to_file(&ff8_input)
                },
            )?
        }
    }
    pe_format::pe_patch_4bg(&exe_path)?;
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
    env: &Env,
) -> Option<FfnxInstallation> {
    loop {
        match install_game_and_ffnx(handle.clone(), installation, env) {
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

    let mut ffnx_installation = match retrieve_ffnx_installation(
        &rx,
        handle.clone(),
        &mut moomba_config,
        &mut installation,
        &env,
    ) {
        Some(ffnx_installation) => ffnx_installation,
        None => return, // Exit
    };

    set_task_text(handle.clone(), TextLevel::Info, "");
    set_game_ready(handle.clone(), true);

    let mut ffnx_config = LazyFfnxConfig::new(&ffnx_installation);
    ffnx_config
        .get()
        .set_app_path(installation.app_path.to_string_lossy().to_string());

    set_ffnx_config(handle.clone(), &mut ffnx_config);
    ffnx_config.save();

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
                    &env,
                ) {
                    Some(version) => version,
                    None => return, // Exit
                };
                set_game_ready(handle.clone(), true);
            }
            Message::UpdateGame => upgrade_ffnx(
                handle.clone(),
                &ffnx_installation,
                &installation.edition,
                &env,
            ),
            Message::LaunchGame => {
                let exe_path = ffnx_installation.exe_path();
                info!(
                    "Launch {:?} in dir {:?}...",
                    &exe_path, &ffnx_installation.path
                );
                launch_game(
                    &exe_path,
                    &ffnx_installation.path,
                    installation.get_app_id(),
                )
            }
            Message::SetFfnxConfigBool(key, value) => {
                ffnx_config.get().set_bool(key.as_str(), value)
            }
            Message::SetFfnxConfigInt(key, value) => ffnx_config.get().set_int(key.as_str(), value),
            Message::SetFfnxConfigString(key, value) => {
                ffnx_config.get().set_string(key.as_str(), value)
            }
            Message::ConfigureFfnx => {
                set_ffnx_config(handle.clone(), &mut ffnx_config);
                ffnx_config.save();
            }
            Message::CancelConfigureFfnx => ffnx_config.clear(),
            Message::Quit => break,
        };
        set_task_text(handle.clone(), TextLevel::Info, "");
    }
}

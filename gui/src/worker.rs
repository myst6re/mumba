use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use super::{AppWindow, Installations};
use slint::ComponentHandle;
use log::{info, warn, error};
use std::{
    path::PathBuf,
    os::windows::process::CommandExt,
    process::{Command, Stdio},
};
use moomba_core::config::Config;
use moomba_core::game::installation;
use moomba_core::game::ffnx_config::FfnxConfig;
use crate::TextLevel;
use moomba_core::game::env::Env;

const DETACHED_PROCESS: u32 = 0x8;

#[derive(Debug)]
pub enum Message {
    Setup(slint::SharedString, bool),
    LaunchGame,
    ConfigureGame,
    UpdateGame,
    Quit
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
            move || {
                worker_loop(rx, handle_weak)
            }
        });
        Self {
            tx,
            thread,
        }
    }

    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.tx.send(Message::Quit);
        self.thread.join()
    }
}

fn set_task_text(handle: slint::Weak<AppWindow>, text_level: TextLevel, text: &'static str) -> () {
    handle.upgrade_in_event_loop(move |h| {
        let installations = h.global::<Installations>();
        installations.set_task_text(slint::SharedString::from(text));
        installations.set_task_text_type(text_level)
    }).unwrap_or_default()
}

fn set_game_ready(handle: slint::Weak<AppWindow>, ready: bool) -> () {
    handle.upgrade_in_event_loop(move |h| {
        h.global::<Installations>().set_is_ready(ready)
    }).unwrap_or_default()
}

fn set_game_exe_path(handle: slint::Weak<AppWindow>, text: String) -> () {
    handle.upgrade_in_event_loop(move |h| {
        h.global::<Installations>().set_game_exe_path(slint::SharedString::from(text))
    }).unwrap_or_default()
}

fn set_current_page(handle: slint::Weak<AppWindow>, page_id: i32) -> () {
    handle.upgrade_in_event_loop(move |h| {
        h.global::<Installations>().set_current_page(page_id)
    }).unwrap_or_default()
}

fn upgrade_ffnx(handle: slint::Weak<AppWindow>, ffnx_version: &Option<String>, edition: &installation::Edition, env: &Env) -> () {
    match ffnx_version {
        Some(version) => {
            set_task_text(handle.clone(), TextLevel::Info, "Check for FFNx update…");
            let url = moomba_core::game::ffnx::Ffnx::find_last_stable_version_on_github("julianxhokaxhiu/FFNx", edition);
            if ! url.contains(version) {
                set_task_text(handle.clone(), TextLevel::Info, "Upgrading FFNx…");
                set_game_ready(handle.clone(), false);
                match moomba_core::game::ffnx::Ffnx::from_url(url.as_str(), &env.ffnx_dir, env) {
                    Ok(()) => (),
                    Err(e) => {error!("Error when installing FFNx: {:?}", e);}
                };
                set_game_ready(handle.clone(), true)
            };
            set_task_text(handle.clone(), TextLevel::Info, "")
        },
        None => ()
    }
}

fn worker_loop(
    rx: Receiver<Message>,
    handle: slint::Weak<AppWindow>
) -> () {
    let env = moomba_core::game::env::Env::new().unwrap();
    let moomba_config_path = env.config_dir.join("config.toml");
    let ffnx_config_path = env.ffnx_dir.join("FFNx.toml");

    let mut moomba_config = Config::from_file(&moomba_config_path).unwrap_or(Config::new());

    let installation = match moomba_config.installation() {
        Ok(Some(installation)) => installation,
        Ok(None) | Err(_) => {
            let installations = installation::Installation::search();
            for inst in installations {
                match inst.edition {
                    installation::Edition::Standard | installation::Edition::Steam => {
                        set_game_exe_path(handle.clone(), inst.exe_path().as_os_str().to_os_string().into_string().unwrap());
                    },
                    installation::Edition::Remastered => {
                        warn!("Ignore remaster at {}, as Moomba is not compatible yet", inst.app_path)
                    }
                }
            }

            set_current_page(handle.clone(), 1);
            match rx.recv() {
                Ok(Message::Setup(exe_path, replace_launcher)) => {
                    info!("Setup with EXE path {} (replace launcher: {})", exe_path, replace_launcher);
                    match installation::Installation::from_exe_path(&PathBuf::from(exe_path.as_str())) {
                        Some(installation) => {
                            if replace_launcher {
                                installation.replace_launcher(&env);
                            };
                            installation
                        },
                        None => return
                    }
                },
                Ok(Message::Quit) => return,
                Ok(msg) => {
                    error!("Received unknown message: {:?}", msg);
                    return
                }
                Err(e) => {
                    error!("Received error: {}", e);
                    return
                }
            }
        }
    };

    info!("Found Game at {:?}: {:?} {} {:?}", &installation.app_path, &installation.edition, &installation.language, &installation.version);

    moomba_config.set_installation(&installation);
    moomba_config.save(&moomba_config_path);

    let app_path = installation.app_path;
    let source_ff8_path = PathBuf::from(&app_path).join(installation.exe_name);
    let mut ffnx_version = None;

    match moomba_core::game::ffnx::Ffnx::is_installed(&env.ffnx_dir, matches!(installation.edition, installation::Edition::Steam)) {
        Some(version) => {
            info!("Found FFNx version {}", version);
            ffnx_version = Some(version)
        },
        None => {
            set_task_text(handle.clone(), TextLevel::Info, "Installing FFNx…");
            let url = moomba_core::game::ffnx::Ffnx::find_last_stable_version_on_github("julianxhokaxhiu/FFNx", &installation.edition);
            match moomba_core::game::ffnx::Ffnx::from_url(url.as_str(), &env.ffnx_dir, &env) {
                Ok(()) => (),
                Err(e) => {error!("Error when installing FFNx: {:?}", e);}
            }
        }
    };
    let ff8_path = env.ffnx_dir.join(if matches!(installation.edition, installation::Edition::Steam) {
        "FF8_Moomba_Steam.exe"
    } else {
        "FF8_Moomba.exe"
    });

    if ! ff8_path.exists() {
        info!("Copy {:?} to {:?}...", &source_ff8_path, &ff8_path);
        moomba_core::provision::copy_file(&source_ff8_path, &ff8_path);
        ()
    }

    let bink_dll_path = env.ffnx_dir.join("binkw32.dll");
    if ! bink_dll_path.exists() {
        if matches!(installation.edition, installation::Edition::Standard) {
            if ! matches!(installation.version, Some((installation::Version::V120, _))) {
                info!("Patch game to 1.02...");
                let file_name = match installation.language.as_str() {
                    "fre" | "fr" => Some("FF8EidosFre"),
                    "ger" | "de" => Some("FF8EidosGerV12"),
                    "eng" | "en" => Some("FF8SqeaPatch"),
                    "spa" | "es" => Some("ff8ngspa"),
                    "ita" | "it" => Some("ff8ngita"),
                    "jp" => Some("FF8EasqPatch"),
                    _ => None
                };
                match file_name {
                    Some(file_name) => {
                        let url = format!("https://www.ff8.fr/download/programs/{}.zip", file_name);
                        moomba_core::provision::download_zip(url.as_str(), "FF8Patch1.02.zip", &env.ffnx_dir, &env);
                        moomba_core::provision::rename_file(&env.ffnx_dir.join("FF8.exe"), &ff8_path);
                    },
                    None => {
                        error!("Cannot detect the language of your game!");
                    }
                }
            }
        } else {
            moomba_core::provision::copy_file(&PathBuf::from(&app_path).join("binkw32.dll"), &bink_dll_path);
            // Clean
            let eax_dll_path = env.ffnx_dir.join("eax.dll");
            if eax_dll_path.exists() {
                std::fs::remove_file(eax_dll_path);
            }
        }
    }
    moomba_core::pe_format::pe_patch_4bg(&ff8_path);

    let mut config = match FfnxConfig::from_file(&ffnx_config_path) {
        Ok(c) => c,
        Err(_e) => FfnxConfig::new()
    };
    config.set_app_path(app_path.as_str());
    config.save(&ffnx_config_path);

    set_task_text(handle.clone(), TextLevel::Info, "");
    set_game_ready(handle.clone(), true);

    upgrade_ffnx(handle.clone(), &ffnx_version, &installation.edition, &env);

    for received in rx {
        match received {
            Message::Setup(_, _) => {

            },
            Message::UpdateGame => {
                upgrade_ffnx(handle.clone(), &ffnx_version, &installation.edition, &env)
            },
            Message::LaunchGame => {
                info!("Launch {:?} in dir {:?}...", &ff8_path, &env.ffnx_dir);
                let res = Command::new(&ff8_path)
                    .creation_flags(DETACHED_PROCESS)
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .current_dir(&env.ffnx_dir)
                    .spawn();

                match res {
                    Err(e) => {
                        error!("Unable to launch game: {:?}", e)
                    },
                    Ok(_) => ()
                };
            },
            Message::ConfigureGame => {
                config.save(&ffnx_config_path);
            },
            Message::Quit => break
        };
        set_task_text(handle.clone(), TextLevel::Info, "");
    }
}

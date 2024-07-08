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

const DETACHED_PROCESS: u32 = 0x8;

#[derive(Debug)]
pub enum Message {
    Setup(slint::SharedString),
    LaunchGame,
    ConfigureGame,
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

fn set_task_text(handle: slint::Weak<AppWindow>, text: &'static str) -> Result<(), slint::EventLoopError> {
    handle.upgrade_in_event_loop(move |h| {
        h.global::<Installations>().set_task_text(slint::SharedString::from(text))
    })
}

fn set_game_ready(handle: slint::Weak<AppWindow>, ready: bool) -> Result<(), slint::EventLoopError> {
    handle.upgrade_in_event_loop(move |h| {
        h.global::<Installations>().set_is_ready(ready)
    })
}

fn set_game_exe_path(handle: slint::Weak<AppWindow>, text: String) -> Result<(), slint::EventLoopError> {
    handle.upgrade_in_event_loop(move |h| {
        h.global::<Installations>().set_game_exe_path(slint::SharedString::from(text))
    })
}

fn set_current_page(handle: slint::Weak<AppWindow>, page_id: i32) -> Result<(), slint::EventLoopError> {
    handle.upgrade_in_event_loop(move |h| {
        h.global::<Installations>().set_current_page(page_id)
    })
}

fn worker_loop(
    rx: Receiver<Message>,
    handle: slint::Weak<AppWindow>
) -> () {
    let env = moomba_core::game::env::Env::new().unwrap();
    let moomba_config_path = env.moomba_dir.join("config.toml");
    let ffnx_config_path = env.ffnx_dir.join("FFNx.toml");

    let mut moomba_config = Config::from_file(&moomba_config_path).unwrap_or(Config::new());

    let installation = match moomba_config.installation() {
        Ok(Some(installation)) => installation,
        Ok(None) | Err(_) => {
            let installations = installation::Installation::search();
            for inst in installations {
                match inst.version {
                    installation::Version::Standard | installation::Version::Steam => {
                        set_game_exe_path(handle.clone(), inst.exe_path().as_os_str().to_os_string().into_string().unwrap());
                    },
                    installation::Version::Remastered => {
                        warn!("Ignore remaster at {}, as Moomba is not compatible yet", inst.app_path)
                    }
                }
            }

            set_current_page(handle.clone(), 1);
            match rx.recv() {
                Ok(Message::Setup(exe_path)) => {
                    info!("Setup with EXE path {}", exe_path);
                    installation::Installation::from_exe_path(&PathBuf::from(exe_path.as_str()))
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

    info!("Found Game at {:?}: {:?} {}", &installation.app_path, &installation.version, &installation.language);

    moomba_config.set_installation(&installation);
    moomba_config.save(&moomba_config_path);

    let app_path = installation.app_path;
    let source_ff8_path = PathBuf::from(&app_path).join(installation.exe_name);

    match moomba_core::game::ffnx::Ffnx::is_installed(&env.ffnx_dir, matches!(installation.version, installation::Version::Steam)) {
        Some(version) => {
            info!("Found FFNx version {}", version);
        },
        None => {
            set_task_text(handle.clone(), "Installing game…");
            match moomba_core::game::ffnx::Ffnx::from_url(
                if matches!(installation.version, installation::Version::Steam) {
                    "https://github.com/julianxhokaxhiu/FFNx/releases/download/1.18.1/FFNx-Steam-v1.18.1.0.zip"
                } else {
                    "https://github.com/julianxhokaxhiu/FFNx/releases/download/1.18.1/FFNx-FF8_2000-v1.18.1.0.zip"
                },
                &env.ffnx_dir,
                &env
            ) {
                Ok(()) => {
                    ()
                },
                Err(e) => {
                    error!("Error when installing FFNx: {:?}", e);
                    ()
                }
            }

        }
    };
    let ff8_path = env.ffnx_dir.join(if matches!(installation.version, installation::Version::Steam) {
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
        if matches!(installation.version, installation::Version::Standard) {
            info!("Patch game to 1.02...");
            moomba_core::provision::download_zip("https://www.ff8.fr/download/programs/FF8EidosFre.zip", "FF8Patch1.02.zip", &env.ffnx_dir, &env);
            moomba_core::provision::rename_file(&env.ffnx_dir.join("FF8.exe"), &ff8_path);
        } else {
            moomba_core::provision::copy_file(&PathBuf::from(&app_path).join("binkw32.dll"), &bink_dll_path);
            // Clean
            let eax_dll_path = env.ffnx_dir.join("eax.dll");
            if eax_dll_path.exists() {
                std::fs::remove_file(eax_dll_path);
            }
        }
    }
    let mut config = match FfnxConfig::from_file(&ffnx_config_path) {
        Ok(c) => c,
        Err(_e) => FfnxConfig::new()
    };
    config.set_app_path(app_path.as_str());
    config.save(&ffnx_config_path);

    set_task_text(handle.clone(), "");
    set_game_ready(handle.clone(), true);

    for received in rx {
        match received {
            Message::Setup(_) => {

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
        set_task_text(handle.clone(), "");
    }
}

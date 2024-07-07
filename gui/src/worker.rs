use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use super::{AppWindow, Installations};
use slint::ComponentHandle;
use log::{info, error};
use std::{
    path::PathBuf,
    os::windows::process::CommandExt,
    process::{Command, Stdio},
};
use moomba_core::game::installation;
use moomba_core::game::ffnx_config::FfnxConfig;

const DETACHED_PROCESS: u32 = 0x8;

#[derive(Debug)]
pub enum Message {
    InstallBase,
    LaunchGame,
    ConfigureGame(slint::SharedString),
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

fn set_game_path(handle: slint::Weak<AppWindow>, text: String) -> Result<(), slint::EventLoopError> {
    handle.upgrade_in_event_loop(move |h| {
        h.global::<Installations>().set_game_path(slint::SharedString::from(text))
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
    let ffnx_config_path = env.ffnx_dir.join("FFNx.toml");
    let mut config = match FfnxConfig::from_file(ffnx_config_path.clone()) {
        Ok(c) => c,
        Err(_e) => FfnxConfig::new()
    };
    let first_installation;

    let installation = match config.app_path() {
        Ok(app_path) => installation::Installation::from_directory(String::from(app_path)),
        Err(_e) => {
            let installations = installation::Installation::search();
            if installations.is_empty() {
                set_current_page(handle.clone(), 1);
                return;
            } else {
                first_installation = installations[0].clone();
                let app_path = first_installation.app_path.as_str();
                config.set_app_path(app_path);
                config.save(ffnx_config_path.clone());
                first_installation.clone()
            }
        }
    };

    info!("Found Game at {:?}: {:?} {}", &installation.app_path, &installation.version, &installation.language);

    let app_path = installation.app_path;
    let source_ff8_path = PathBuf::from(&app_path).join(installation.exe_name);
    let ff8_path = env.ffnx_dir.join("FF8_Moomba.exe");

    set_game_path(handle.clone(), app_path.clone());

    for received in rx {
        match received {
            Message::InstallBase => {
                match moomba_core::game::ffnx::Ffnx::is_installed(&env.ffnx_dir) {
                    Some(version) => {
                        info!("Found FFNx version {}", version);
                    },
                    None => {
                        set_task_text(handle.clone(), "Installing game…");
                        match moomba_core::game::ffnx::Ffnx::from_url(
                            "https://github.com/julianxhokaxhiu/FFNx/releases/download/1.18.1/FFNx-FF8_2000-v1.18.1.0.zip",
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
                if ! ff8_path.exists() {
                    info!("Copy {:?} to {:?}...", &source_ff8_path, &ff8_path);
                    moomba_core::provision::copy_file(&source_ff8_path, &ff8_path);
                    ()
                }
                if matches!(installation.version, installation::Version::Standard) && ! env.ffnx_dir.join("binkw32.dll").exists() {
                    info!("Patch game to 1.02...");
                    moomba_core::provision::download_zip("https://www.ff8.fr/download/programs/FF8EidosFre.zip", "FF8Patch1.02.zip", &env.ffnx_dir, &env);
                    moomba_core::provision::rename_file(&env.ffnx_dir.join("FF8.exe"), &ff8_path);
                }
                set_game_ready(handle.clone(), true);
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
            Message::ConfigureGame(path) => {
                let mut config = match FfnxConfig::from_file(ffnx_config_path.clone()) {
                    Ok(c) => c,
                    Err(_e) => FfnxConfig::new()
                };
                config.set_app_path(path.as_str());
                set_game_path(handle.clone(), config.app_path().unwrap().into());
                config.save(ffnx_config_path.clone())
                ;
            },
            Message::Quit => {
                break
            }
        };
        set_task_text(handle.clone(), "");
    }
}

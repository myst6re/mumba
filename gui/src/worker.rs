use super::AppWindow;
use crate::ui_helper::UiHelper;
use crate::worker_loop::WorkerLoop;
use log::error;
use mumba_core::config::UpdateChannel;
use mumba_core::i18n;
use slint::ComponentHandle;
use std::sync::mpsc::{self, Sender};

#[derive(Debug)]
pub enum Message {
    Setup(slint::SharedString, UpdateChannel, String),
    LaunchGame,
    ConfigureFfnx,
    CancelConfigureFfnx,
    SetFfnxConfigBool(slint::SharedString, bool),
    SetFfnxConfigInt(slint::SharedString, i64),
    SetFfnxConfigString(slint::SharedString, slint::SharedString),
    SetFfnxConfigCurrentRefreshRate(i32, i32),
    UpdateGame,
    Quit,
}

pub struct Worker {
    pub tx: Sender<Message>,
    thread: std::thread::JoinHandle<()>,
}

impl Worker {
    pub fn new(ui: &AppWindow, language: Option<String>) -> Self {
        let (tx, rx) = mpsc::channel::<Message>();
        let thread = std::thread::spawn({
            let handle_weak = ui.as_weak();
            move || {
                let env = match mumba_core::game::env::Env::new() {
                    Ok(env) => env,
                    Err(e) => {
                        error!("Cannot initialize environment: {}", e);
                        return;
                    }
                };
                let i18n = i18n::I18n::new(language);

                WorkerLoop::new(rx, env, UiHelper::new(handle_weak, i18n)).run()
            }
        });
        Self { tx, thread }
    }

    pub fn join(self) -> std::thread::Result<()> {
        let _ = self.tx.send(Message::Quit);
        self.thread.join()
    }
}

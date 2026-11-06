use crate::TextLevel;
use log::{error, info};
use mumba_core::config::UpdateChannel;
use mumba_core::game::ffnx_installation::FfnxInstallation;
use mumba_core::game::installation;
use mumba_core::provision;

use crate::worker_loop::WorkerLoop;

pub struct PageSetup {}

impl PageSetup {
    pub fn clean_ffnx_installation(
        ffnx_installation: &FfnxInstallation,
    ) -> Result<(), std::io::Error> {
        provision::remove_file_if_exists(&ffnx_installation.path.join("steam_appid.txt"))?;
        provision::remove_file_if_exists(&ffnx_installation.path.join("FFVIII.exe"))?;
        provision::remove_file_if_exists(&ffnx_installation.exe_path())
    }

    pub fn setup(
        worker_loop: &WorkerLoop,
        app_path: &slint::SharedString,
        update_channel: UpdateChannel,
        language: String,
    ) -> Option<installation::Installation> {
        info!("Setup with EXE path \"{}\"", app_path);
        match installation::Installation::from_directory(app_path.as_str(), None) {
            Some(installation) => {
                let mut mumba_config = worker_loop.open_mumba_config();
                mumba_config.set_installation(&installation);
                mumba_config.set_update_channel(update_channel.clone());
                mumba_config.set_language(&language);
                worker_loop
                    .ui
                    .set_game_app_path(installation.app_path.to_string_lossy().to_string());
                worker_loop.ui.set_update_channel(update_channel);
                if worker_loop.save_mumba_config(&mumba_config) {
                    Some(installation)
                } else {
                    None
                }
            }
            None => {
                error!("This file does not exist: \"{}\"", app_path);
                worker_loop
                    .ui
                    .set_task_text(TextLevel::Error, "message-error-file-not-found");
                None
            }
        }
    }
}

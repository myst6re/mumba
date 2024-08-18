#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::Path;
slint::include_modules!();
use log::error;
use moomba_core::game::env::Env;

pub mod lazy_ffnx_config;
pub mod worker;

use worker::Worker;

fn main() -> Result<(), slint::PlatformError> {
    let env = Env::new().unwrap();
    moomba_core::moomba_log::init(&env, "moomba.log");

    let ui = AppWindow::new()?;

    let worker = Worker::new(&ui);

    ui.global::<Installations>().on_setup({
        let tx = worker.tx.clone();
        move |path| {
            if let Err(e) = tx.send(worker::Message::Setup(path)) {
                error!("Error: {}", e)
            }
        }
    });

    ui.global::<Installations>().on_launch_game({
        let tx = worker.tx.clone();
        move || tx.send(worker::Message::LaunchGame).unwrap()
    });

    ui.global::<Installations>().on_browse_game({
        let ui = ui.as_weak();
        move |old_path| {
            let mut dialog = rfd::FileDialog::new();
            dialog = dialog.set_title("Select a directory");
            dialog = dialog.add_filter("EXE files", &["exe"]);
            dialog = dialog.set_parent(&ui.unwrap().window().window_handle());

            if !old_path.is_empty() {
                dialog = dialog.set_directory(old_path.as_str());
                dialog = dialog.set_file_name(
                    Path::new(&old_path.as_str())
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                );
            }

            match dialog.pick_file() {
                Some(new_path) => new_path.to_string_lossy().to_string().into(),
                None => old_path,
            }
        }
    });

    ui.global::<Installations>().on_configure_ffnx({
        let tx = worker.tx.clone();
        move || tx.send(worker::Message::ConfigureFfnx).unwrap()
    });

    ui.global::<Installations>().on_cancel_configure_ffnx({
        let tx = worker.tx.clone();
        move || tx.send(worker::Message::CancelConfigureFfnx).unwrap()
    });

    ui.global::<Installations>().on_upgrade_ffnx({
        let tx = worker.tx.clone();
        move || tx.send(worker::Message::UpdateGame).unwrap()
    });

    ui.global::<Installations>().on_set_ffnx_config_bool({
        let tx = worker.tx.clone();
        move |key, value| {
            tx.send(worker::Message::SetFfnxConfigBool(key, value))
                .unwrap()
        }
    });

    ui.global::<Installations>().on_set_ffnx_config_int({
        let tx = worker.tx.clone();
        move |key, value| {
            tx.send(worker::Message::SetFfnxConfigInt(key, value as i64))
                .unwrap()
        }
    });

    ui.global::<Installations>().on_set_ffnx_config_string({
        let tx = worker.tx.clone();
        move |key, value| {
            tx.send(worker::Message::SetFfnxConfigString(key, value))
                .unwrap()
        }
    });

    ui.run()?;

    worker.join().unwrap();
    Ok(())
}

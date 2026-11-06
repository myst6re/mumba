use crate::lazy_ffnx_config::LazyFfnxConfig;
use crate::TextLevel;
use log::error;
use mumba_core::game::ffnx_config;
use mumba_core::screen::Screen;
use slint::SharedString;

use crate::worker_loop::WorkerLoop;

pub struct PageOptions {}

impl PageOptions {
    pub fn set_ffnx_config_bool(ffnx_config: &mut LazyFfnxConfig, key: SharedString, value: bool) {
        ffnx_config.get().set_bool(key.as_str(), value)
    }

    pub fn set_ffnx_config_int(
        worker_loop: &WorkerLoop,
        screen_resolutions: &Screen,
        ffnx_config: &mut LazyFfnxConfig,
        key: SharedString,
        value: i64,
    ) {
        if key == "current_resolution" {
            let resolutions = screen_resolutions.resolutions.get(value as usize);
            let (w, h) = match resolutions {
                Some(e) => (e.w, e.h),
                None => (0, 0),
            };
            let fullscreen = ffnx_config
                .get()
                .get_bool(ffnx_config::CFG_FULLSCREEN, true)
                .unwrap_or(true);
            if fullscreen {
                ffnx_config.get().set_int("window_size_x", w);
                ffnx_config.get().set_int("window_size_y", h);
            } else {
                let window_x = ffnx_config
                    .get()
                    .get_int("window_size_x_window", 0)
                    .unwrap_or(0);
                let window_y = ffnx_config
                    .get()
                    .get_int("window_size_y_window", 0)
                    .unwrap_or(0);
                ffnx_config.get().set_int("window_size_x", window_x);
                ffnx_config.get().set_int("window_size_y", window_y);
            }
            ffnx_config.get().set_int("window_size_x_fullscreen", w);
            ffnx_config.get().set_int("window_size_y_fullscreen", h);
            worker_loop.ui.set_refresh_rates(match resolutions {
                Some(e) => e.freqs.clone(),
                None => vec![],
            })
        } else if key == "window_size_x" || key == "window_size_y" {
            let fullscreen = ffnx_config
                .get()
                .get_bool(ffnx_config::CFG_FULLSCREEN, true)
                .unwrap_or(true);
            if fullscreen {
                let fullscreen_value = ffnx_config
                    .get()
                    .get_int(format!("{}_fullscreen", key).as_str(), 0)
                    .unwrap_or(0);
                ffnx_config.get().set_int(key.as_str(), fullscreen_value);
            } else {
                ffnx_config.get().set_int(key.as_str(), value)
            }
            ffnx_config
                .get()
                .set_int(format!("{}_window", key).as_str(), value)
        } else {
            ffnx_config.get().set_int(key.as_str(), value)
        }
    }

    pub fn set_ffnx_config_string(
        ffnx_config: &mut LazyFfnxConfig,
        key: SharedString,
        value: SharedString,
    ) {
        ffnx_config.get().set_string(key.as_str(), value)
    }

    pub fn set_ffnx_config_current_refresh_rate(
        ffnx_config: &mut LazyFfnxConfig,
        screen_resolutions: &Screen,
        current_resolution: i32,
        current_refresh_rate: i32,
    ) {
        let freqs = screen_resolutions
            .resolutions
            .get(current_resolution as usize)
            .map(|r| r.freqs.clone())
            .unwrap_or_default();
        ffnx_config.get().set_int(
            "refresh_rate",
            *freqs.get(current_refresh_rate as usize).unwrap_or(&0) as i64,
        )
    }

    pub fn configure_ffnx(
        worker_loop: &WorkerLoop,
        screen_resolutions: &Screen,
        ffnx_config: &mut LazyFfnxConfig,
    ) {
        worker_loop
            .ui
            .set_ffnx_config(ffnx_config, &screen_resolutions);
        if let Err(error) = ffnx_config.save() {
            error!("Cannot save FFNx configuration: {}", error);
            worker_loop
                .ui
                .set_task_text(TextLevel::Error, "message-error-cannot-save-ffnx-config")
        }
    }
}

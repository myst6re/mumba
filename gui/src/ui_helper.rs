use super::{AppWindow, Installations};
use crate::lazy_ffnx_config::LazyFfnxConfig;
use crate::TextLevel;
use mumba_core::config::UpdateChannel;
use mumba_core::game::ffnx_config;
use mumba_core::i18n::I18n;
use mumba_core::screen::Screen;
use slint::ComponentHandle;

pub struct UiHelper {
    handle: slint::Weak<AppWindow>,
    i18n: I18n,
}

impl UiHelper {
    pub fn new(handle: slint::Weak<AppWindow>, i18n: I18n) -> Self {
        Self { handle, i18n }
    }

    pub fn clear_task_text(&self) {
        self.handle
            .clone()
            .upgrade_in_event_loop(move |h| {
                let installations = h.global::<Installations>();
                installations.set_task_text(slint::SharedString::from(""));
                installations.set_task_text_type(TextLevel::Info)
            })
            .unwrap_or_default()
    }

    pub fn set_task_text(&self, text_level: TextLevel, id: &'static str) {
        let text = self.i18n.tr(id);
        self.handle
            .clone()
            .upgrade_in_event_loop(move |h| {
                let installations = h.global::<Installations>();
                installations.set_task_text(slint::SharedString::from(text));
                installations.set_task_text_type(text_level)
            })
            .unwrap_or_default()
    }

    pub fn set_game_ready(&self, ready: bool) {
        self.handle
            .clone()
            .upgrade_in_event_loop(move |h| h.global::<Installations>().set_is_ready(ready))
            .unwrap_or_default()
    }

    pub fn set_game_exe_path(&self, text: String) {
        self.handle
            .clone()
            .upgrade_in_event_loop(move |h| {
                h.global::<Installations>()
                    .set_game_exe_path(slint::SharedString::from(text))
            })
            .unwrap_or_default()
    }

    pub fn set_update_channel(&self, update_channel: UpdateChannel) {
        self.handle
            .clone()
            .upgrade_in_event_loop(move |h| {
                h.global::<Installations>()
                    .set_update_channel(update_channel as i32)
            })
            .unwrap_or_default()
    }

    pub fn set_current_page(&self, page_id: i32) {
        self.handle
            .clone()
            .upgrade_in_event_loop(move |h| h.global::<Installations>().set_current_page(page_id))
            .unwrap_or_default()
    }

    pub fn set_resolutions(&self, screen_resolutions: &Screen, current_resolution: i32) {
        let resolutions: Vec<slint::SharedString> = screen_resolutions
            .resolutions
            .iter()
            .map(|screen| slint::SharedString::from(format!("{}x{}", screen.w, screen.h)))
            .collect();
        let refresh_rates: Vec<slint::SharedString> = screen_resolutions
            .resolutions
            .get(current_resolution as usize)
            .map(|sr| sr.freqs.clone())
            .unwrap_or_default()
            .iter()
            .map(|freq| slint::SharedString::from(format!("{} Hz", freq)))
            .collect();

        self.handle
            .clone()
            .upgrade_in_event_loop(move |h| {
                let installations = h.global::<Installations>();
                installations.set_resolutions(slint::ModelRc::<slint::SharedString>::from(
                    resolutions.as_slice(),
                ));
                installations.set_refresh_rates(slint::ModelRc::<slint::SharedString>::from(
                    refresh_rates.as_slice(),
                ));
            })
            .unwrap_or_default()
    }

    pub fn set_refresh_rates(&self, refresh_rates: Vec<u32>) {
        let refresh_rates: Vec<slint::SharedString> = refresh_rates
            .iter()
            .map(|freq| slint::SharedString::from(format!("{} Hz", freq)))
            .collect();

        self.handle
            .clone()
            .upgrade_in_event_loop(move |h| {
                h.global::<Installations>().set_refresh_rates(
                    slint::ModelRc::<slint::SharedString>::from(refresh_rates.as_slice()),
                );
            })
            .unwrap_or_default()
    }

    pub fn set_ffnx_config(
        &self,
        ffnx_config: &mut LazyFfnxConfig,
        screen_resolutions: &Screen,
    ) -> crate::FfnxConfig {
        let fullscreen = ffnx_config.get_bool(ffnx_config::CFG_FULLSCREEN, true);
        let win_or_full_size_x = ffnx_config.get_int(ffnx_config::CFG_WINDOW_SIZE_X, 0);
        let win_or_full_size_y = ffnx_config.get_int(ffnx_config::CFG_WINDOW_SIZE_Y, 0);
        let window_size_x = if fullscreen { 0 } else { win_or_full_size_x };
        let window_size_y = if fullscreen { 0 } else { win_or_full_size_x };
        let current_resolution = {
            let fullscreen_size_x = ffnx_config.get_int(
                ffnx_config::CFG_WINDOW_SIZE_X_FULLSCREEN,
                if fullscreen { win_or_full_size_x } else { 0 },
            ) as u32;
            let fullscreen_size_y = ffnx_config.get_int(
                ffnx_config::CFG_WINDOW_SIZE_Y_FULLSCREEN,
                if fullscreen { win_or_full_size_y } else { 0 },
            ) as u32;
            screen_resolutions
                .position(fullscreen_size_x, fullscreen_size_y)
                .unwrap_or(screen_resolutions.resolutions.len().saturating_sub(1))
        };
        let config = crate::FfnxConfig {
            renderer_backend: ffnx_config.get_int(ffnx_config::CFG_RENDERER_BACKEND, 0),
            fullscreen,
            borderless: ffnx_config.get_bool(ffnx_config::CFG_BORDERLESS, false),
            enable_vsync: ffnx_config.get_bool(ffnx_config::CFG_ENABLE_VSYNC, true),
            enable_antialiasing: ffnx_config.get_int(ffnx_config::CFG_ENABLE_ANTIALIASING, 0),
            enable_anisotropic: ffnx_config.get_bool(ffnx_config::CFG_ENABLE_ANISOTROPIC, true),
            enable_bilinear: ffnx_config.get_bool(ffnx_config::CFG_ENABLE_BILINEAR, false),
            ff8_use_gamepad_icons: ffnx_config
                .get_bool(ffnx_config::CFG_FF8_USE_GAMEPAD_ICONS, true),
            current_resolution: current_resolution as i32,
            current_refresh_rate: {
                let refresh_rate = ffnx_config.get_int(ffnx_config::CFG_REFRESH_RATE, 0) as u32;
                screen_resolutions
                    .refresh_rate_position(current_resolution, refresh_rate)
                    .unwrap_or(0) as i32
            },
            internal_resolution_scale: ffnx_config
                .get_int(ffnx_config::CFG_INTERNAL_RESOLUTION_SCALE, 0),
            window_size_x: ffnx_config.get_int(
                ffnx_config::CFG_WINDOW_SIZE_X_WINDOW,
                if window_size_x == 0 {
                    640
                } else {
                    window_size_x
                },
            ),
            window_size_y: ffnx_config.get_int(
                ffnx_config::CFG_WINDOW_SIZE_Y_WINDOW,
                if window_size_y == 0 {
                    480
                } else {
                    window_size_y
                },
            ),
        };
        let config2 = config.clone();
        self.handle
            .clone()
            .upgrade_in_event_loop(move |h| h.global::<Installations>().set_ffnx_config(config))
            .unwrap_or_default();
        config2
    }
}

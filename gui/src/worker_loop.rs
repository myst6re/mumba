use crate::lazy_ffnx_config::LazyFfnxConfig;
use crate::ui_helper::UiHelper;
use crate::worker::Message;
use crate::TextLevel;
use log::{error, info, warn};
use mumba_core::config::{Config, UpdateChannel};
use mumba_core::game::env::Env;
use mumba_core::game::ffnx_config;
use mumba_core::game::ffnx_installation::FfnxInstallation;
use mumba_core::game::installation;
use mumba_core::screen::Screen;
use mumba_core::steam::get_steam_exe;
use mumba_core::{pe_format, provision, toml};
use std::sync::mpsc::Receiver;
use thiserror::Error;

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

pub struct WorkerLoop {
    rx: Receiver<Message>,
    env: Env,
    ui: UiHelper,
}

impl WorkerLoop {
    pub fn new(rx: Receiver<Message>, env: Env, ui: UiHelper) -> Self {
        Self { rx, env, ui }
    }

    fn open_mumba_config(&self) -> Config {
        Config::from_file(&self.env.config_path).unwrap_or_else(|_| Config::new())
    }

    pub fn run(&mut self) {
        let (mut installation, mut update_channel) = match self.retrieve_installation() {
            Some(installation) => installation,
            None => return, // Exit
        };

        info!(
            "Found Game at \"{}\": {:?} {} {:?}",
            &installation.app_path.to_string_lossy(),
            &installation.edition,
            &installation.language,
            &installation.version
        );

        let mut ffnx_installation =
            match self.retrieve_ffnx_installation(&mut installation, &mut update_channel) {
                Some(ffnx_installation) => ffnx_installation,
                None => return, // Exit
            };

        self.ui.clear_task_text();
        self.ui.set_mumba_initialized(true);
        self.ui.set_game_ready(true);
        let screen_resolutions = Screen::list_screens_resolutions();

        let mut ffnx_config = LazyFfnxConfig::new(&ffnx_installation);
        let ui_ffnx_config = self
            .ui
            .set_ffnx_config(&mut ffnx_config, &screen_resolutions);
        let steam_exe = get_steam_exe().unwrap_or_default();

        self.ui
            .set_resolutions(&screen_resolutions, ui_ffnx_config.current_resolution);

        for received in &self.rx {
            match received {
                Message::Setup(exe_path, update_chan, language) => {
                    self.ui.set_game_ready(false);
                    update_channel = update_chan.clone();
                    installation = match self.setup(&exe_path, update_chan, language) {
                        Some(inst) => inst,
                        None => break,
                    };
                    ffnx_installation = match self
                        .retrieve_ffnx_installation(&mut installation, &mut update_channel)
                    {
                        Some(ffnx_inst) => ffnx_inst,
                        None => return, // Exit
                    };
                    self.ui.set_game_ready(true);
                }
                Message::UpdateGame => {
                    let _ = ffnx_config.get();
                    self.upgrade_ffnx(
                        &ffnx_installation,
                        &installation.edition,
                        update_channel.clone(),
                    );
                    if let Err(error) = ffnx_config.save() {
                        error!("Cannot save FFNx configuration: {}", error);
                        self.ui.set_task_text(
                            TextLevel::Error,
                            "message-error-cannot-save-ffnx-config",
                        )
                    }
                }
                Message::LaunchGame => {
                    ffnx_config.get().set_app_path(if cfg!(unix) {
                        String::from("..")
                    } else {
                        installation.app_path.to_string_lossy().to_string()
                    });
                    if let Err(error) = ffnx_config.save() {
                        error!("Cannot save FFNx configuration: {}", error);
                        self.ui.set_task_text(
                            TextLevel::Error,
                            "message-error-cannot-save-ffnx-config",
                        )
                    }
                    if ffnx_installation
                        .launch_game(&installation, &steam_exe)
                        .is_err()
                    {
                        self.ui
                            .set_task_text(TextLevel::Error, "message-error-cannot-launch-game")
                    }
                }
                Message::LaunchCW => {
                    if installation.launch_cw(&steam_exe).is_err() {
                        self.ui
                            .set_task_text(TextLevel::Error, "message-error-cannot-launch-game")
                    }
                }
                Message::SetFfnxConfigBool(key, value) => {
                    ffnx_config.get().set_bool(key.as_str(), value)
                }
                Message::SetFfnxConfigInt(key, value) => {
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
                        self.ui.set_refresh_rates(match resolutions {
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
                Message::SetFfnxConfigString(key, value) => {
                    ffnx_config.get().set_string(key.as_str(), value)
                }
                Message::SetFfnxConfigCurrentRefreshRate(
                    current_resolution,
                    current_refresh_rate,
                ) => {
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
                Message::ConfigureFfnx => {
                    self.ui
                        .set_ffnx_config(&mut ffnx_config, &screen_resolutions);
                    if let Err(error) = ffnx_config.save() {
                        error!("Cannot save FFNx configuration: {}", error);
                        self.ui.set_task_text(
                            TextLevel::Error,
                            "message-error-cannot-save-ffnx-config",
                        )
                    }
                }
                Message::CancelConfigureFfnx => ffnx_config.clear(),
                Message::Quit => break,
            };
            self.ui.clear_task_text();
        }
    }

    fn retrieve_installation(&mut self) -> Option<(installation::Installation, UpdateChannel)> {
        let mumba_config = self.open_mumba_config();
        let update_channel = mumba_config
            .update_channel()
            .unwrap_or(UpdateChannel::Stable);
        self.ui.set_update_channel(update_channel.clone());

        match mumba_config.installation() {
            Ok(Some(installation)) => {
                self.ui
                    .set_game_exe_path(installation.exe_path().to_string_lossy().to_string());
                Some((installation, update_channel))
            }
            Ok(None) | Err(_) => {
                let installations = installation::Installation::search();
                for inst in installations {
                    match inst.edition {
                        installation::Edition::Standard | installation::Edition::Steam => {
                            self.ui
                                .set_game_exe_path(inst.exe_path().to_string_lossy().to_string());
                        }
                        installation::Edition::Remastered => {
                            warn!(
                                "Ignore remaster at {}, as The Yellow Mumba is not compatible yet",
                                inst.app_path.to_string_lossy()
                            )
                        }
                    }
                }

                self.go_to_setup_page()
            }
        }
    }

    fn upgrade_ffnx(
        &self,
        ffnx_installation: &FfnxInstallation,
        edition: &installation::Edition,
        update_channel: UpdateChannel,
    ) {
        self.ui
            .set_task_text(TextLevel::Info, "message-info-check-ffnx-update");
        let url = FfnxInstallation::find_version_on_github(
            "julianxhokaxhiu/FFNx",
            edition,
            update_channel,
        );
        self.ui
            .set_task_text(TextLevel::Info, "message-info-upgrade-in-progress-ffnx");
        self.ui.set_game_ready(false);
        match FfnxInstallation::download(url.as_str(), &ffnx_installation.path, &self.env) {
            Ok(()) => (),
            Err(e) => {
                error!("Error when installing FFNx: {}", e);
            }
        };
        self.ui.set_game_ready(true);
        self.ui.clear_task_text()
    }

    fn install_game_and_ffnx(
        &self,
        installation: &installation::Installation,
        update_channel: UpdateChannel,
    ) -> Result<FfnxInstallation, InstallError> {
        let ffnx_dir = if cfg!(unix) {
            // The game runs inside a fake Windows filesystem
            installation.app_path.join("mumba")
        } else {
            self.env.ffnx_dir.clone()
        };
        let ffnx_installation = match FfnxInstallation::from_directory(&ffnx_dir, installation) {
            Some(ffnx_installation) => {
                info!(
                    "Found FFNx version {}",
                    if ffnx_installation.version == "0.0.0" {
                        "dev"
                    } else {
                        ffnx_installation.version.as_str()
                    }
                );
                ffnx_installation
            }
            None => {
                self.ui
                    .set_task_text(TextLevel::Info, "message-info-install-in-progress-ffnx");
                let url = FfnxInstallation::find_version_on_github(
                    "julianxhokaxhiu/FFNx",
                    &installation.edition,
                    update_channel,
                );
                FfnxInstallation::download(url.as_str(), &ffnx_dir, &self.env)?;
                if let Some(ffnx_installation) =
                    FfnxInstallation::from_directory(&ffnx_dir, installation)
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
            installation.replace_launcher(&self.env)?
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
                        match provision::download_zip(
                            url.as_str(),
                            "FF8Patch1.02.zip",
                            &ffnx_installation.path,
                            &self.env,
                        ) {
                            Ok(()) => provision::rename_file(
                                &ffnx_installation.path.join("FF8.exe"),
                                &exe_path,
                            )?,
                            Err(e) => {
                                error!("Cannot download 1.02 patch: {}", e)
                            }
                        }
                    }
                    None => {
                        error!("Cannot detect the language of your game")
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
                        == "FFNx"
                {
                    provision::copy_file(&self.env.mumba_dir.join("eax.dll"), &eax_dll_path)?
                }
            }
            installation::Edition::Standard | installation::Edition::Remastered => {
                let eax_dll_path = ffnx_installation.path.join("creative_eax.dll");
                if !eax_dll_path.exists() {
                    if let Err(e) =
                        provision::copy_file(&self.env.mumba_dir.join("eax.dll"), &eax_dll_path)
                    {
                        warn!("Cannot install creative_eax.dll: {}", e);
                    }
                }
            }
        }
        if matches!(&installation.edition, installation::Edition::Standard) {
            let ff8_input = ffnx_installation.path.join("override").join("ff8input.cfg");
            if !ff8_input.exists() {
                std::fs::create_dir_all(ffnx_installation.path.join("override"))?;
                provision::copy_file(&installation.app_path.join("ff8input.cfg"), &ff8_input)
                    .or_else(|e| {
                        warn!(
                            "Error when copying ff8input.cfg, creating a new one instead: {}",
                            e
                        );
                        mumba_core::game::input_config::InputConfig::new(&installation.edition)
                            .to_file(&ff8_input)
                    })?
            }
        }
        pe_format::pe_patch_4bg(&exe_path)?;
        Ok(ffnx_installation)
    }

    fn setup(
        &self,
        exe_path: &slint::SharedString,
        update_channel: UpdateChannel,
        language: String,
    ) -> Option<installation::Installation> {
        info!("Setup with EXE path {}", exe_path);
        match installation::Installation::from_exe_path(exe_path.as_str()) {
            Ok(installation) => {
                let mut mumba_config = self.open_mumba_config();
                mumba_config.set_installation(&installation);
                mumba_config.set_update_channel(update_channel.clone());
                mumba_config.set_language(&language);
                self.ui
                    .set_game_exe_path(installation.exe_path().to_string_lossy().to_string());
                self.ui.set_update_channel(update_channel);
                match mumba_config.save(&self.env.config_path) {
                    Ok(()) => Some(installation),
                    Err(e) => {
                        error!("Cannot save configuration to mumba.toml: {}", e);
                        self.ui.set_task_text(
                            TextLevel::Error,
                            "message-error-cannot-save-mumba-config",
                        );
                        None
                    }
                }
            }
            Err(installation::FromExeError::NotFound) => {
                error!("This file does not exist: {}", exe_path);
                self.ui
                    .set_task_text(TextLevel::Error, "message-error-file-not-found");
                None
            }
            Err(installation::FromExeError::LauncherSelected) => {
                error!("Select the game exe, not the launcher: {}", exe_path);
                self.ui
                    .set_task_text(TextLevel::Error, "message-error-file-not-found");
                None
            }
        }
    }

    fn go_to_setup_page(&self) -> Option<(installation::Installation, UpdateChannel)> {
        loop {
            self.ui.set_current_page(1);
            match self.rx.recv() {
                Ok(Message::Setup(exe_path, update_channel, language)) => {
                    match self.setup(&exe_path, update_channel.clone(), language) {
                        Some(installation) => return Some((installation, update_channel)),
                        None => continue,
                    }
                }
                Ok(Message::Quit) => return None,
                msg => {
                    error!("Received unknown message: {:?}", msg);
                    self.ui
                        .set_task_text(TextLevel::Error, "message-fatal-unknown-action");
                    continue;
                }
            }
        }
    }

    fn retrieve_ffnx_installation(
        &self,
        installation: &mut installation::Installation,
        update_channel: &mut UpdateChannel,
    ) -> Option<FfnxInstallation> {
        loop {
            match self.install_game_and_ffnx(installation, update_channel.clone()) {
                Ok(ffnx_installation) => return Some(ffnx_installation),
                Err(e) => {
                    error!("Installation error: {}", e);
                    self.ui
                        .set_task_text(TextLevel::Error, "message-error-cannot-install-ffnx");

                    match self.go_to_setup_page() {
                        Some((inst, update_chan)) => {
                            *installation = inst;
                            *update_channel = update_chan
                        }
                        None => return None, // Exit
                    }
                }
            };
        }
    }
}

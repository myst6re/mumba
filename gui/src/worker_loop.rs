use crate::lazy_ffnx_config::LazyFfnxConfig;
use crate::ui_helper::{Page, UiHelper};
use crate::worker::Message;
use crate::TextLevel;
use log::{error, info, warn};
use mumba_core::config::{Config, UpdateChannel};
use mumba_core::game::env::Env;
use mumba_core::game::ffnx_config;
use mumba_core::game::ffnx_installation::FfnxInstallation;
use mumba_core::game::installation::{self, Edition};
use mumba_core::screen::Screen;
use mumba_core::{pe_format, provision, toml};
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("Install error: {0}")]
    ProvisionError(#[from] provision::Error),
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

    fn save_mumba_config(&self, mumba_config: &Config) -> bool {
        match mumba_config.save(&self.env.config_path) {
            Ok(()) => true,
            Err(e) => {
                error!("Cannot save configuration to mumba.toml: {}", e);
                self.ui
                    .set_task_text(TextLevel::Error, "message-error-cannot-save-mumba-config");
                false
            }
        }
    }

    pub fn run(&mut self) {
        let (ffnx_path, mut installation, mut update_channel) = match self.retrieve_installation() {
            Some(installation) => installation,
            None => return, // Exit
        };

        info!(
            "Found Game {:?} {} at \"{}\"",
            installation.edition,
            installation.language,
            installation.app_path.to_string_lossy()
        );

        let mut ffnx_installation = match self.retrieve_ffnx_installation(
            &ffnx_path,
            &mut installation,
            &mut update_channel,
        ) {
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

        self.ui
            .set_resolutions(&screen_resolutions, ui_ffnx_config.current_resolution);

        for received in &self.rx {
            match received {
                Message::Setup(app_path, update_chan, language) => {
                    Self::clean_ffnx_installation(&ffnx_installation).unwrap();
                    update_channel = update_chan.clone();
                    installation = match self.setup(&app_path, update_chan, language) {
                        Some(inst) => inst,
                        None => continue,
                    };
                    self.ui.set_game_ready(false);
                    ffnx_installation = match self.retrieve_ffnx_installation(
                        &ffnx_path,
                        &mut installation,
                        &mut update_channel,
                    ) {
                        Some(ffnx_inst) => ffnx_inst,
                        None => continue,
                    };
                    self.ui.set_game_ready(true);
                }
                Message::SetFfnxPath(_ffnx_path) => (),
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
                    ffnx_config.get().set_string(
                        ffnx_config::CFG_APP_PATH,
                        if cfg!(unix) {
                            String::from("..")
                        } else {
                            installation.app_path.to_string_lossy().to_string()
                        },
                    );
                    if let Err(error) = ffnx_config.save() {
                        error!("Cannot save FFNx configuration: {}", error);
                        self.ui.set_task_text(
                            TextLevel::Error,
                            "message-error-cannot-save-ffnx-config",
                        )
                    }
                    if ffnx_installation.launch_game().is_err() {
                        self.ui
                            .set_task_text(TextLevel::Error, "message-error-cannot-launch-game")
                    }
                }
                Message::LaunchCW => {
                    if installation.launch_cw().is_err() {
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
                Message::OpenLogs => self.open_logs(),
                Message::CancelConfigureFfnx => ffnx_config.clear(),
                Message::Quit => return,
            };
            self.ui.clear_task_text();
        }
    }

    fn open_logs(&self) {
        if let Err(e) = opener::open(self.env.log_path.clone()) {
            error!("Cannot open {}: {}", self.env.log_path.to_string_lossy(), e)
        }
    }

    fn retrieve_installation(
        &mut self,
    ) -> Option<(PathBuf, installation::Installation, UpdateChannel)> {
        let mumba_config = self.open_mumba_config();
        let update_channel = mumba_config
            .update_channel()
            .unwrap_or(UpdateChannel::Stable);
        self.ui.set_update_channel(update_channel.clone());

        let ret = match mumba_config.installation() {
            Ok(Some(installation)) => {
                self.ui
                    .set_game_app_path(installation.app_path.to_string_lossy().to_string());
                Some((installation, update_channel))
            }
            Ok(None) | Err(_) => {
                let installations = installation::Installation::search();
                if !installations.is_empty() {
                    self.ui.set_game_app_path(
                        installations
                            .first()
                            .unwrap()
                            .app_path
                            .to_string_lossy()
                            .to_string(),
                    );
                }

                self.go_to_setup_page()
            }
        };

        if let Some((installation, update_channel)) = ret {
            let ffnx_path = match mumba_config.data_path() {
                Ok(Some(ffnx_path)) => PathBuf::from(ffnx_path),
                Ok(None) | Err(_) => installation.app_path.join("mumba"),
            };
            self.ui
                .set_ffnx_path(ffnx_path.to_string_lossy().to_string());
            Some((ffnx_path, installation, update_channel))
        } else {
            None
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
        ffnx_dir: &PathBuf,
        installation: &installation::Installation,
        update_channel: UpdateChannel,
    ) -> Result<FfnxInstallation, InstallError> {
        let ffnx_installation = match FfnxInstallation::from_directory(ffnx_dir, installation) {
            Some(ffnx_installation) => {
                info!(
                    "Found FFNx version {} at \"{}\"",
                    if ffnx_installation.version == "0.0.0" {
                        "dev"
                    } else {
                        ffnx_installation.version.as_str()
                    },
                    ffnx_installation.path.to_string_lossy()
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
                FfnxInstallation::download(url.as_str(), ffnx_dir, &self.env)?;
                if let Some(ffnx_installation) =
                    FfnxInstallation::from_directory(ffnx_dir, installation)
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

        if matches!(installation.edition, Edition::Steam) {
            installation.replace_launcher(&self.env)?;
            provision::copy_file(
                &installation
                    .app_path
                    .join(format!("FF8_{}.exe", installation.language)),
                &exe_path,
            )?
        } else if matches!(installation.edition, Edition::Remastered) {
            File::create(ffnx_installation.path.join("FFVIII.exe"))?;
        };

        let steam_appid = ffnx_installation.path.join("steam_appid.txt");
        if !steam_appid.exists() {
            let mut file = File::create(steam_appid)?;
            file.write_all(format!("{}", installation.edition.clone() as u32).as_bytes())?;
        }

        if !exe_path.exists() {
            match provision::download_zip(
                "https://www.ff8.fr/download/programs/FF8SqeaPatch.zip",
                "FF8Patch1.02.zip",
                &ffnx_installation.path,
                &self.env,
            ) {
                Ok(()) => {
                    provision::rename_file(&ffnx_installation.path.join("FF8.exe"), &exe_path)?
                }
                Err(e) => {
                    error!("Cannot download 1.02 patch: {}", e)
                }
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
        if matches!(&installation.edition, Edition::Standard) {
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

    fn clean_ffnx_installation(ffnx_installation: &FfnxInstallation) -> Result<(), std::io::Error> {
        provision::remove_file(&ffnx_installation.path.join("steam_appid.txt"))?;
        provision::remove_file(&ffnx_installation.path.join("FFVIII.exe"))?;
        provision::remove_file(&ffnx_installation.exe_path())
    }

    fn setup(
        &self,
        app_path: &slint::SharedString,
        update_channel: UpdateChannel,
        language: String,
    ) -> Option<installation::Installation> {
        info!("Setup with EXE path \"{}\"", app_path);
        match installation::Installation::from_directory(app_path.as_str(), None) {
            Some(installation) => {
                let mut mumba_config = self.open_mumba_config();
                mumba_config.set_installation(&installation);
                mumba_config.set_update_channel(update_channel.clone());
                mumba_config.set_language(&language);
                self.ui
                    .set_game_app_path(installation.app_path.to_string_lossy().to_string());
                self.ui.set_update_channel(update_channel);
                if self.save_mumba_config(&mumba_config) {
                    Some(installation)
                } else {
                    None
                }
            }
            None => {
                error!("This file does not exist: \"{}\"", app_path);
                self.ui
                    .set_task_text(TextLevel::Error, "message-error-file-not-found");
                None
            }
        }
    }

    fn go_to_setup_page(&self) -> Option<(installation::Installation, UpdateChannel)> {
        loop {
            self.ui.set_current_page(Page::Setup);
            match self.rx.recv() {
                Ok(Message::Setup(exe_path, update_channel, language)) => {
                    match self.setup(&exe_path, update_channel.clone(), language) {
                        Some(installation) => return Some((installation, update_channel)),
                        None => continue,
                    }
                }
                Ok(Message::OpenLogs) => self.open_logs(),
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

    fn set_ffnx_path(&self, ffnx_path: &Path) -> bool {
        let mut mumba_config = self.open_mumba_config();
        mumba_config.set_data_path(ffnx_path);
        self.save_mumba_config(&mumba_config)
    }

    fn go_to_select_dir_page(&self) -> Option<String> {
        loop {
            self.ui.set_current_page(Page::SelectDir);
            match self.rx.recv() {
                Ok(Message::SetFfnxPath(ffnx_path)) => {
                    let path = PathBuf::from(String::from(ffnx_path.clone()));
                    if self.set_ffnx_path(&path) {
                        return Some(String::from(ffnx_path));
                    }
                    continue;
                }
                Ok(Message::OpenLogs) => self.open_logs(),
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
        ffnx_dir: &Path,
        installation: &mut installation::Installation,
        update_channel: &mut UpdateChannel,
    ) -> Option<FfnxInstallation> {
        let mut ffnx_dir = ffnx_dir.to_path_buf();
        loop {
            match self.install_game_and_ffnx(&ffnx_dir, installation, update_channel.clone()) {
                Ok(ffnx_installation) => {
                    let mut mumba_config = self.open_mumba_config();
                    mumba_config.set_data_path(&ffnx_dir);
                    self.save_mumba_config(&mumba_config);
                    return Some(ffnx_installation);
                }
                Err(InstallError::ProvisionError(provision::Error::IoError(e)))
                    if e.kind() == std::io::ErrorKind::PermissionDenied =>
                {
                    error!("Installation error: {}", e);
                    // Fallback to default path
                    self.ui
                        .set_ffnx_path(self.env.ffnx_dir.to_string_lossy().to_string());

                    let ffnx_path = self.go_to_select_dir_page()?;
                    ffnx_dir = PathBuf::from(ffnx_path);
                }
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

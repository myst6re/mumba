use crate::toml;
use std::path::Path;
use toml_edit::DocumentMut;

pub const CFG_APP_PATH: &str = "app_path";
pub const CFG_RENDERER_BACKEND: &str = "renderer_backend";
pub const CFG_FULLSCREEN: &str = "fullscreen";
pub const CFG_BORDERLESS: &str = "borderless";
pub const CFG_ENABLE_VSYNC: &str = "enable_vsync";
pub const CFG_ENABLE_ANTIALIASING: &str = "enable_antialiasing";
pub const CFG_ENABLE_ANISOTROPIC: &str = "enable_anisotropic";
pub const CFG_ENABLE_BILINEAR: &str = "enable_bilinear";
pub const CFG_FF8_USE_GAMEPAD_ICONS: &str = "ff8_use_gamepad_icons";
pub const CFG_REFRESH_RATE: &str = "refresh_rate";
pub const CFG_INTERNAL_RESOLUTION_SCALE: &str = "internal_resolution_scale";
pub const CFG_WINDOW_SIZE_X: &str = "window_size_x";
pub const CFG_WINDOW_SIZE_Y: &str = "window_size_y";
pub const CFG_WINDOW_SIZE_X_FULLSCREEN: &str = "window_size_x_fullscreen";
pub const CFG_WINDOW_SIZE_Y_FULLSCREEN: &str = "window_size_y_fullscreen";
pub const CFG_WINDOW_SIZE_X_WINDOW: &str = "window_size_x_window";
pub const CFG_WINDOW_SIZE_Y_WINDOW: &str = "window_size_y_window";

pub struct FfnxConfig {
    inner: DocumentMut,
}

impl FfnxConfig {
    pub fn new() -> Self {
        Self {
            inner: DocumentMut::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, toml::FileError> {
        Ok(Self {
            inner: toml::parse_from_file(path)?,
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), toml::FileError> {
        toml::save_to_file(&self.inner, path)
    }

    pub fn root(&self) -> &toml_edit::Table {
        self.inner.as_table()
    }

    pub fn app_path(&self) -> Result<&str, toml::Error> {
        toml::get_string(self.root(), CFG_APP_PATH, "")
    }

    pub fn set_app_path<V: Into<String>>(&mut self, value: V) {
        self.set_string(CFG_APP_PATH, value)
    }

    pub fn set_bool<V: Into<bool>>(&mut self, key: &str, value: V) {
        self.inner[key] = toml_edit::value(value.into())
    }

    pub fn get_bool(&self, key: &str, default: bool) -> Result<bool, toml::Error> {
        toml::get_boolean(self.root(), key, default)
    }

    pub fn set_int<V: Into<i64>>(&mut self, key: &str, value: V) {
        self.inner[key] = toml_edit::value(value.into())
    }

    pub fn get_int(&self, key: &str, default: i64) -> Result<i64, toml::Error> {
        toml::get_integer(self.root(), key, default)
    }

    pub fn set_string<V: Into<String>>(&mut self, key: &str, value: V) {
        self.inner[key] = toml_edit::value(value.into())
    }

    pub fn get_string<'a>(&'a self, key: &str, default: &'a str) -> Result<&'a str, toml::Error> {
        toml::get_string(self.root(), key, default)
    }
}

impl Default for FfnxConfig {
    fn default() -> Self {
        Self::new()
    }
}

use crate::toml;
use std::path::Path;
use toml_edit::DocumentMut;

const CFG_APP_PATH: &str = "app_path";
const CFG_FULLSCREEN: &str = "fullscreen";

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

    pub fn fullscreen(&self) -> Result<bool, toml::Error> {
        toml::get_boolean(self.root(), CFG_FULLSCREEN, true)
    }

    pub fn set_bool<V: Into<bool>>(&mut self, key: &str, value: V) {
        self.inner[key] = toml_edit::value(value.into())
    }

    pub fn set_int<V: Into<i64>>(&mut self, key: &str, value: V) {
        self.inner[key] = toml_edit::value(value.into())
    }

    pub fn set_string<V: Into<String>>(&mut self, key: &str, value: V) {
        self.inner[key] = toml_edit::value(value.into())
    }
}

impl Default for FfnxConfig {
    fn default() -> Self {
        Self::new()
    }
}

use crate::game::installation::Installation;
use crate::toml;
use std::path::Path;
use thiserror::Error;
use toml_edit::DocumentMut;

const CFG_EXE_PATH: &str = "exe_path";
const CFG_UPDATE_CHANNEL: &str = "update_channel";
const CFG_LANGUAGE: &str = "language";

#[derive(Error, Debug)]
pub enum Error {
    #[error("The key {0} is not the type {1}")]
    WrongTypeError(String, String),
    #[error("The key {0} is absent")]
    DoesNotExist(String),
}

#[derive(Debug, Clone)]
pub enum UpdateChannel {
    Stable = 0,
    Beta = 1,
    Alpha = 2,
}

pub struct Config {
    inner: DocumentMut,
}

impl Config {
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

    pub fn installation(&self) -> Result<Option<Installation>, toml::Error> {
        let key = CFG_EXE_PATH;
        let exe_path = toml::get_string(self.root(), key, "")?;
        if exe_path.is_empty() {
            return Ok(None);
        }
        Installation::from_exe_path(exe_path)
            .map(Some)
            .map_err(|_| toml::Error::DoesNotExist(String::from(key)))
    }

    pub fn set_installation(&mut self, installation: &Installation) {
        if let Some(exe_path) = installation.exe_path().to_str() {
            self.inner[CFG_EXE_PATH] = toml_edit::Item::Value(exe_path.into())
        }
    }

    pub fn update_channel(&self) -> Result<UpdateChannel, toml::Error> {
        Ok(
            match toml::get_integer(self.root(), CFG_UPDATE_CHANNEL, 0)? {
                0 => UpdateChannel::Stable,
                1 => UpdateChannel::Beta,
                2 => UpdateChannel::Alpha,
                _ => UpdateChannel::Stable,
            },
        )
    }

    pub fn set_update_channel(&mut self, update_channel: UpdateChannel) {
        self.inner[CFG_UPDATE_CHANNEL] = toml_edit::Item::Value((update_channel as i64).into())
    }

    pub fn language(&self) -> Result<String, toml::Error> {
        Ok(String::from(toml::get_string(
            self.root(),
            CFG_LANGUAGE,
            "",
        )?))
    }

    pub fn set_language(&mut self, lang: &String) {
        self.inner[CFG_LANGUAGE] = toml_edit::Item::Value(lang.into())
    }

    fn root(&self) -> &toml_edit::Table {
        self.inner.as_table()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

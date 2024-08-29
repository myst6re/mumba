use crate::game::installation::Installation;
use crate::toml;
use std::path::{Path, PathBuf};
use thiserror::Error;
use toml_edit::DocumentMut;

const CFG_EXE_PATH: &str = "exe_path";
const CFG_UPDATE_CHANNEL: &str = "update_channel";

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

    pub fn installation(&self) -> Result<Option<Installation>, Error> {
        match self.inner.get("game") {
            Some(toml_edit::Item::Table(t)) => Ok(Some(Self::installation_from_table(t)?)),
            Some(_) => Err(Error::WrongTypeError(
                String::from("game"),
                String::from("Table"),
            )),
            None => Ok(None),
        }
    }

    pub fn set_installation(&mut self, installation: &Installation) {
        self.inner["game"] = toml_edit::Item::Table(
            Self::set_installation_to_table(installation).unwrap_or_default(),
        )
    }

    pub fn update_channel(&mut self) -> Result<UpdateChannel, Error> {
        match self.inner.get("ffnx") {
            Some(toml_edit::Item::Table(t)) => Ok(Self::ffnx_from_table(t)?),
            Some(_) => Err(Error::WrongTypeError(
                String::from("game"),
                String::from("Table"),
            )),
            None => Ok(UpdateChannel::Stable),
        }
    }

    pub fn set_update_channel(&mut self, update_channel: UpdateChannel) {
        self.inner["ffnx"] =
            toml_edit::Item::Table(Self::set_ffnx_to_table(update_channel).unwrap_or_default())
    }

    fn installation_from_table(table: &toml_edit::Table) -> Result<Installation, Error> {
        let key = CFG_EXE_PATH;
        let exe_path = PathBuf::from(match table.get(key) {
            Some(toml_edit::Item::Value(toml_edit::Value::String(exe_path))) => exe_path.value(),
            _ => {
                return Err(Error::WrongTypeError(
                    String::from(key),
                    String::from("String"),
                ))
            }
        });
        Installation::from_exe_path(&exe_path).map_err(|_| Error::DoesNotExist(String::from(key)))
    }

    fn set_installation_to_table(installation: &Installation) -> Option<toml_edit::Table> {
        let mut ret = toml_edit::Table::new();
        ret[CFG_EXE_PATH] = toml_edit::Item::Value(installation.exe_path().to_str()?.into());
        Some(ret)
    }

    fn ffnx_from_table(table: &toml_edit::Table) -> Result<UpdateChannel, Error> {
        let key = CFG_UPDATE_CHANNEL;
        match table.get(key) {
            Some(toml_edit::Item::Value(toml_edit::Value::Integer(update_channel))) => {
                Ok(match update_channel.value() {
                    0 => UpdateChannel::Stable,
                    1 => UpdateChannel::Beta,
                    2 => UpdateChannel::Alpha,
                    _ => UpdateChannel::Stable,
                })
            }
            _ => Err(Error::WrongTypeError(
                String::from(key),
                String::from("String"),
            )),
        }
    }

    fn set_ffnx_to_table(update_channel: UpdateChannel) -> Option<toml_edit::Table> {
        let mut ret = toml_edit::Table::new();
        ret[CFG_UPDATE_CHANNEL] = toml_edit::Item::Value((update_channel as i64).into());
        Some(ret)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

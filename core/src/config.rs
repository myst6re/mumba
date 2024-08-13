use crate::game::installation::Installation;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use thiserror::Error;
use toml_edit::DocumentMut;

pub struct Config {
    inner: DocumentMut,
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Error with the config file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("TOML format error: {0}")]
    TomlError(#[from] toml_edit::TomlError),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("The key {0} is not the type {1}")]
    WrongTypeError(String, String),
    #[error("The key {0} is absent")]
    DoesNotExist(String),
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            inner: DocumentMut::new(),
        }
    }

    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, FileError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Self {
            inner: contents.parse::<DocumentMut>()?,
        })
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), FileError> {
        let mut file = File::create(path)?;
        file.write_all(self.inner.to_string().as_bytes())?;
        Ok(())
    }

    pub fn installation(&self) -> Result<Option<Installation>, Error> {
        match self.inner.get("game") {
            Some(toml_edit::Item::Table(t)) => Ok(Some(Self::installation_from_table(t)?)),
            Some(_) => Err(Error::WrongTypeError(
                String::from("game"),
                String::from("table"),
            )),
            None => Ok(None),
        }
    }

    pub fn set_installation(&mut self, installation: &Installation) {
        self.inner["game"] = toml_edit::Item::Table(
            Self::set_installation_to_table(installation).unwrap_or_default(),
        )
    }

    fn installation_from_table(table: &toml_edit::Table) -> Result<Installation, Error> {
        let key = "exe_path";
        let exe_path = PathBuf::from(match table.get(key) {
            Some(toml_edit::Item::Value(toml_edit::Value::String(exe_path))) => exe_path.value(),
            _ => {
                return Err(Error::WrongTypeError(
                    String::from(key),
                    String::from("value"),
                ))
            }
        });
        Installation::from_exe_path(&exe_path).map_err(|_| Error::DoesNotExist(String::from(key)))
    }

    fn set_installation_to_table(installation: &Installation) -> Option<toml_edit::Table> {
        let mut ret = toml_edit::Table::new();
        ret["exe_path"] = toml_edit::Item::Value(installation.exe_path().to_str()?.into());
        Some(ret)
    }
}

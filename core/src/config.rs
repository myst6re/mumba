use toml_edit::DocumentMut;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use crate::game::installation::Installation;

pub struct Config {
    inner: DocumentMut
}

pub enum FileError {
    IoError(std::io::Error),
    TomlError(toml_edit::TomlError)
}

impl From<std::io::Error> for FileError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<toml_edit::TomlError> for FileError {
    fn from(e: toml_edit::TomlError) -> Self {
        Self::TomlError(e)
    }
}

pub enum Error {
    WrongTypeError,
    OsStringError(core::convert::Infallible)
}

impl From<core::convert::Infallible> for Error {
    fn from(e: core::convert::Infallible) -> Self {
        Self::OsStringError(e)
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            inner: DocumentMut::new()
        }
    }

    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, FileError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Self {
            inner: contents.parse::<DocumentMut>()?
        })
    }

    pub fn save<P: AsRef<std::path::Path>>(self: &Self, path: P) -> Result<(), FileError> {
        let mut file = File::create(path)?;
        file.write_all(self.inner.to_string().as_bytes())?;
        Ok(())
    }

    pub fn installation(self: &Self) -> Result<Option<Installation>, Error> {
        match self.inner.get("game") {
            Some(toml_edit::Item::Table(t)) => Ok(Some(Self::installation_from_table(t)?)),
            Some(_) => Err(Error::WrongTypeError),
            None => Ok(None),
        }
    }

    pub fn set_installation(self: &mut Self, installation: &Installation) -> () {
        self.inner["game"] = toml_edit::Item::Table(Self::set_installation_to_table(installation).unwrap_or_default())
    }

    fn installation_from_table(table: &toml_edit::Table) -> Result<Installation, Error> {
        let exe_path = PathBuf::from(match table.get("exe_path") {
            Some(toml_edit::Item::Value(toml_edit::Value::String(exe_path))) => exe_path.value(),
            _ => return Err(Error::WrongTypeError)
        });
        Ok(Installation::from_exe_path(&exe_path))
    }

    fn set_installation_to_table(installation: &Installation) -> Option<toml_edit::Table> {
        let mut ret = toml_edit::Table::new();
        ret["exe_path"] = toml_edit::Item::Value(installation.exe_path().to_str()?.into());
        Some(ret)
    }
}

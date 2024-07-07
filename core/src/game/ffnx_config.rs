use toml_edit::DocumentMut;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub struct FfnxConfig {
    inner: DocumentMut
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Error {
    WrongTypeError,
    NotAValueError
}

const CFG_APP_PATH: &str = "app_path";

impl FfnxConfig {
    pub fn new() -> Self {
        Self {
            inner: DocumentMut::new()
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, FileError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Self {
            inner: contents.parse::<DocumentMut>()?
        })
    }

    pub fn save<P: AsRef<Path>>(self: &Self, path: P) -> Result<(), FileError> {
        let mut file = File::create(path)?;
        file.write_all(self.inner.to_string().as_bytes())?;
        Ok(())
    }

    pub fn app_path(self: &Self) -> Result<&str, Error> {
        match self.inner.get(CFG_APP_PATH) {
            Some(toml_edit::Item::Value(v)) => match v.as_str() {
                Some(s) => Ok(s),
                None => Err(Error::WrongTypeError)
            },
            Some(_) => Err(Error::NotAValueError),
            None => Ok(""),
        }
    }

    pub fn set_app_path(self: &mut Self, app_path: &str) -> () {
        self.inner[CFG_APP_PATH] = toml_edit::value(app_path)
    }
}

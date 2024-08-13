use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use thiserror::Error;
use toml_edit::DocumentMut;

pub struct FfnxConfig {
    inner: DocumentMut,
}

#[derive(Error, Debug)]
pub enum FileError {
    #[error("Error with the FFNx config file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("TOML format error: {0}")]
    TomlError(#[from] toml_edit::TomlError),
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("The key {0} is not a string")]
    WrongTypeError(String),
    #[error("The key {0} is not a string value")]
    NotAValueError(String),
}

const CFG_APP_PATH: &str = "app_path";

impl FfnxConfig {
    pub fn new() -> Self {
        Self {
            inner: DocumentMut::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, FileError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Self {
            inner: contents.parse::<DocumentMut>()?,
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
                None => Err(Error::WrongTypeError(String::from(CFG_APP_PATH))),
            },
            Some(_) => Err(Error::NotAValueError(String::from(CFG_APP_PATH))),
            None => Ok(""),
        }
    }

    pub fn set_app_path(self: &mut Self, app_path: &str) -> () {
        self.inner[CFG_APP_PATH] = toml_edit::value(app_path)
    }
}

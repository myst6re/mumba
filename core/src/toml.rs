use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use thiserror::Error;

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
    #[error("The key {0} is not a value")]
    NotAValueError(String),
    #[error("The key {0} is absent")]
    DoesNotExist(String),
}

pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<toml_edit::DocumentMut, FileError> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.parse::<toml_edit::DocumentMut>()?)
}

pub fn save_to_file<P: AsRef<Path>>(
    document: &toml_edit::DocumentMut,
    path: P,
) -> Result<(), FileError> {
    let mut file = File::create(path)?;
    file.write_all(document.to_string().as_bytes())?;
    Ok(())
}

pub fn get_string<'a>(
    table: &'a toml_edit::Table,
    key: &str,
    default: &'a str,
) -> Result<&'a str, Error> {
    match table.get(key) {
        Some(toml_edit::Item::Value(v)) => match v.as_str() {
            Some(s) => Ok(s),
            None => Err(Error::WrongTypeError(
                String::from(key),
                String::from("String"),
            )),
        },
        Some(_) => Err(Error::NotAValueError(String::from(key))),
        None => Ok(default),
    }
}

pub fn get_boolean(table: &toml_edit::Table, key: &str, default: bool) -> Result<bool, Error> {
    match table.get(key) {
        Some(toml_edit::Item::Value(v)) => match v.as_bool() {
            Some(s) => Ok(s),
            None => Err(Error::WrongTypeError(
                String::from(key),
                String::from("Boolean"),
            )),
        },
        Some(_) => Err(Error::NotAValueError(String::from(key))),
        None => Ok(default),
    }
}

pub fn get_integer(table: &toml_edit::Table, key: &str, default: i64) -> Result<i64, Error> {
    match table.get(key) {
        Some(toml_edit::Item::Value(v)) => match v.as_integer() {
            Some(s) => Ok(s),
            None => Err(Error::WrongTypeError(
                String::from(key),
                String::from("Integer"),
            )),
        },
        Some(_) => Err(Error::NotAValueError(String::from(key))),
        None => Ok(default),
    }
}

#[cfg(feature = "network")]
use crate::game::env::Env;
use log::info;
#[cfg(feature = "network")]
use serde::de::DeserializeOwned;
#[cfg(feature = "network")]
use std::fs::File;
#[cfg(feature = "network")]
use std::io::Read;
use std::path::PathBuf;
use thiserror::Error;
#[cfg(feature = "zip")]
use zip_extensions::*;

#[cfg(feature = "network")]
#[derive(Error, Debug)]
#[error(transparent)]
pub struct ErrorBox(Box<Error>);

impl<E> From<E> for ErrorBox
where
    Error: From<E>,
{
    fn from(err: E) -> Self {
        ErrorBox(Box::new(Error::from(err)))
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(feature = "network")]
    #[error("HTTP Error: {0}")]
    HttpError(#[from] ureq::Error),
    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),
    #[cfg(feature = "zip")]
    #[error("Zip Error: {0}")]
    ZipError(#[from] zip::result::ZipError),
}

#[cfg(feature = "network")]
#[derive(Error, Debug)]
#[error(transparent)]
pub struct ToJsonErrorBox(Box<ToJsonError>);

impl<E> From<E> for ToJsonErrorBox
where
    ToJsonError: From<E>,
{
    fn from(err: E) -> Self {
        ToJsonErrorBox(Box::new(ToJsonError::from(err)))
    }
}

#[cfg(feature = "network")]
#[derive(Error, Debug)]
pub enum ToJsonError {
    #[error("HTTP Error downloading JSON format: {0}")]
    HttpError(#[from] ureq::Error),
    #[error("I/O Error downloading JSON format: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(feature = "network")]
pub fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, ToJsonErrorBox> {
    Ok(ureq::get(url).call()?.into_json()?)
}

#[cfg(all(feature = "network", feature = "zip"))]
pub fn download_zip(
    url: &str,
    local_zip_name: &str,
    target_dir: &PathBuf,
    env: &Env,
) -> Result<(), ErrorBox> {
    let temp_dir = env.cache_dir.as_path();
    let archive_path = temp_dir.join(local_zip_name);
    let mut reader = ureq::get(url).call()?.into_reader().take(250_000_000);
    let ret = from_reader(&mut reader, &archive_path, target_dir);
    match std::fs::remove_file(&archive_path) {
        Ok(ok) => ok,
        Err(e) => warn!(
            "Cannot remove file {}: {}",
            archive_path.to_string_lossy(),
            e
        ),
    };
    ret
}

#[cfg(feature = "zip")]
pub fn extract_zip(
    source_file: &PathBuf,
    target_dir: &PathBuf,
) -> Result<(), zip::result::ZipError> {
    info!("Extract zip from {:?} to {:?}...", source_file, target_dir);
    zip_extract(source_file, target_dir)
}

pub fn copy_file(source_file: &PathBuf, target_file: &PathBuf) -> Result<(), std::io::Error> {
    info!("Copy {:?} to {:?}...", source_file, target_file);
    std::fs::copy(source_file, target_file).and(Ok(()))
}

pub fn rename_file(source_file: &PathBuf, target_file: &PathBuf) -> Result<(), std::io::Error> {
    info!("Rename {:?} to {:?}...", source_file, target_file);
    std::fs::rename(source_file, target_file).and(Ok(()))
}

#[cfg(all(feature = "network", feature = "zip"))]
fn from_reader<R: Read + ?Sized>(
    reader: &mut R,
    archive_path: &PathBuf,
    target_dir: &PathBuf,
) -> Result<(), ErrorBox> {
    let mut file = File::create(archive_path)?;
    info!("Create file: {}", &archive_path.to_string_lossy());
    std::io::copy(reader, &mut file)?;
    Ok(zip_extract(archive_path, target_dir)?)
}

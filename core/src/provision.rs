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

#[derive(Error, Debug)]
pub enum Error {
    #[cfg(feature = "network")]
    #[error("HTTP Error: {0}")]
    HttpError(ureq::Error),
    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),
    #[cfg(feature = "zip")]
    #[error("Zip Error: {0}")]
    ZipError(zip::result::ZipError),
}

#[cfg(feature = "network")]
impl From<ureq::Error> for Error {
    fn from(err: ureq::Error) -> Self {
        match err {
            ureq::Error::Io(io) => Error::IoError(io),
            e => Error::HttpError(e),
        }
    }
}

#[cfg(feature = "zip")]
impl From<zip::result::ZipError> for Error {
    fn from(err: zip::result::ZipError) -> Self {
        match err {
            zip::result::ZipError::Io(io) => Error::IoError(io),
            e => Error::ZipError(e),
        }
    }
}

#[cfg(feature = "network")]
pub fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, Error> {
    Ok(ureq::get(url).call()?.body_mut().read_json::<T>()?)
}

#[cfg(all(feature = "network", feature = "zip"))]
pub fn download_zip(
    url: &str,
    local_zip_name: &str,
    target_dir: &PathBuf,
    env: &Env,
) -> Result<(), Error> {
    let temp_dir = env.cache_dir.as_path();
    let archive_path = temp_dir.join(local_zip_name);
    info!(
        "Download file from \"{}\" to \"{}\"",
        url,
        archive_path.to_string_lossy()
    );
    let mut response = ureq::get(url).call()?;
    let mut reader = response.body_mut().as_reader().take(250_000_000);
    let ret = from_reader(&mut reader, &archive_path, target_dir);
    info!(
        "Remove temporary file \"{}\"",
        archive_path.to_string_lossy()
    );
    match std::fs::remove_file(&archive_path) {
        Ok(ok) => ok,
        Err(e) => warn!(
            "Cannot remove file \"{}\": {}",
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
    info!(
        "Extract zip from \"{}\" to \"{}\"...",
        source_file.to_string_lossy(),
        target_dir.to_string_lossy()
    );
    zip_extract(source_file, target_dir)
}

pub fn copy_file(source_file: &PathBuf, target_file: &PathBuf) -> Result<(), std::io::Error> {
    info!(
        "Copy \"{}\" to \"{}\"...",
        source_file.to_string_lossy(),
        target_file.to_string_lossy()
    );
    std::fs::copy(source_file, target_file).and(Ok(()))
}

pub fn rename_file(source_file: &PathBuf, target_file: &PathBuf) -> Result<(), std::io::Error> {
    info!(
        "Rename \"{}\" to \"{}\"...",
        source_file.to_string_lossy(),
        target_file.to_string_lossy()
    );
    std::fs::rename(source_file, target_file).and(Ok(()))
}

#[cfg(all(feature = "network", feature = "zip"))]
fn from_reader<R: Read + ?Sized>(
    reader: &mut R,
    archive_path: &PathBuf,
    target_dir: &PathBuf,
) -> Result<(), Error> {
    let mut file = File::create(archive_path)?;
    std::io::copy(reader, &mut file)?;
    info!(
        "Extract file \"{}\" to \"{}\"",
        archive_path.to_string_lossy(),
        target_dir.to_string_lossy()
    );
    Ok(zip_extract(archive_path, target_dir)?)
}

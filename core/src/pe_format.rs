use std::path::Path;
use pelite::pe32::Pe;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    PeliteError(pelite::Error),
    PeliteFindError(pelite::resources::FindError),
    NoVersion
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl From<pelite::Error> for Error {
    fn from(e: pelite::Error) -> Self {
        Self::PeliteError(e)
    }
}

impl From<pelite::resources::FindError> for Error {
    fn from(e: pelite::resources::FindError) -> Self {
        Self::PeliteFindError(e)
    }
}

pub fn pe_version_info(path: &Path) -> Result<pelite::image::VS_FIXEDFILEINFO, Error> {
    // Map the file into memory
    let file_map = pelite::FileMap::open(path)?;

    // Interpret as a PE image
    let image = pelite::PeFile::from_bytes(file_map.as_ref())?;

    // Extract the resources from the image
    let resources = image.resources()?;

    // Extract the version info from the resources
    let version_info = resources.version_info()?;

    Ok(version_info.fixed().ok_or(Error::NoVersion)?.clone())
}

pub fn pe_patch_4bg(path: &Path) -> Result<bool, Error> {
    // Map the file into memory
    let file_map = pelite::FileMap::open(path)?;

    // Interpret as a PE image
    let image = pelite::pe32::PeFile::from_bytes(file_map.as_ref())?;

    // Already patched
    if (image.nt_headers().FileHeader.Characteristics & pelite::image::IMAGE_FILE_LARGE_ADDRESS_AWARE) != 0 {
        return Ok(false)
    }

    let characteristics_offset = image.dos_header().e_lfanew as u64 + 22;
    let checksum_offset = image.dos_header().e_lfanew as u64 + 88;
    let characteristics = image.nt_headers().FileHeader.Characteristics | pelite::image::IMAGE_FILE_LARGE_ADDRESS_AWARE;

    drop(file_map);

    // Modify file
    let mut f = File::options().read(false).write(true).open(path)?;
    f.seek(SeekFrom::Start(characteristics_offset))?;
    f.write(&characteristics.to_le_bytes())?;
    drop(f);

    let file_map = pelite::FileMap::open(path)?;
    let image = pelite::pe32::PeFile::from_bytes(file_map.as_ref())?;
    // Recalculate checksum
    let checksum = image.headers().check_sum();
    drop(file_map);

    // Modify file
    let mut f = File::options().read(false).write(true).open(path)?;
    f.seek(SeekFrom::Start(checksum_offset))?;
    f.write(&checksum.to_le_bytes())?;

    return Ok(true)
}

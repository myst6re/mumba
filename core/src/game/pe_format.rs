
use std::path::Path;

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

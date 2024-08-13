use pelite::pe32::Pe;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error when reading/writting EXE/DLL metadata: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Error when analyzing EXE/DLL metadata: {0}")]
    PeliteError(#[from] pelite::Error),
    #[error("Error extracting version info from EXE/DLL: {0}")]
    PeliteFindError(#[from] pelite::resources::FindError),
    #[error("No version available")]
    NoVersion,
}

pub struct VersionInfo {
    pub product_version: pelite::image::VS_VERSION,
    pub product_name: Option<String>,
    pub original_filename: Option<String>,
}

struct QueryStringsMultiLang<F> {
    f: F,
}

impl<'a, F: FnMut(&str, &str)> pelite::resources::version_info::Visit<'a>
    for QueryStringsMultiLang<F>
{
    fn string_table(&mut self, _lang: &'a [u16]) -> bool {
        true
    }
    fn string(&mut self, key: &'a [u16], value: &'a [u16]) {
        let key = String::from_utf16_lossy(key);
        let value = String::from_utf16_lossy(value);
        (self.f)(&key, &value);
    }
}

pub fn pe_version_info<P: AsRef<Path> + ?Sized>(path: &P) -> Result<VersionInfo, Error> {
    // Map the file into memory
    let file_map = pelite::FileMap::open(path)?;

    // Interpret as a PE image
    let image = pelite::PeFile::from_bytes(file_map.as_ref())?;

    // Extract the resources from the image
    let resources = image.resources()?;

    // Extract the version info from the resources
    let version_info = resources.version_info()?;
    let mut product_name = None;
    let mut original_filename = None;

    version_info.visit(&mut QueryStringsMultiLang {
        f: |name: &str, str: &str| {
            if name == "ProductName" && !str.is_empty() && product_name.is_none() {
                product_name = Some(String::from(str))
            } else if name == "OriginalFilename" && !str.is_empty() && original_filename.is_none() {
                original_filename = Some(String::from(str))
            }
        },
    });

    match version_info.fixed() {
        Some(info) => Ok(VersionInfo {
            product_version: info.dwProductVersion,
            product_name,
            original_filename,
        }),
        None => Err(Error::NoVersion),
    }
}

pub fn pe_patch_4bg(path: &Path) -> Result<bool, Error> {
    // Map the file into memory
    let file_map = pelite::FileMap::open(path)?;

    // Interpret as a PE image
    let image = pelite::pe32::PeFile::from_bytes(file_map.as_ref())?;

    // Already patched
    if (image.nt_headers().FileHeader.Characteristics
        & pelite::image::IMAGE_FILE_LARGE_ADDRESS_AWARE)
        != 0
    {
        return Ok(false);
    }

    let characteristics_offset = image.dos_header().e_lfanew as u64 + 22;
    let checksum_offset = image.dos_header().e_lfanew as u64 + 88;
    let characteristics = image.nt_headers().FileHeader.Characteristics
        | pelite::image::IMAGE_FILE_LARGE_ADDRESS_AWARE;

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

    return Ok(true);
}

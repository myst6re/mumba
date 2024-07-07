use std::path::PathBuf;
use crate::game::env::Env;
use crate::game::pe_format;
use crate::provision;

pub struct Ffnx {

}

impl Ffnx {
    pub fn from_url(url: &str, target_dir: &PathBuf, env: &Env) -> Result<(), provision::Error> {
        provision::download_zip(url, "FFNx.zip", target_dir, env)
    }

    pub fn from_file(source_file: &PathBuf, target_dir: &PathBuf) -> Result<(), zip::result::ZipError> {
        provision::extract_zip(source_file, target_dir)
    }

    pub fn is_installed(target_dir: &PathBuf) -> Option<String> {
        match pe_format::pe_version_info(target_dir.join("eax.dll").as_path()) {
            Ok(version) => Some(format!("{}.{}.{}", version.dwProductVersion.Major, version.dwProductVersion.Minor, version.dwProductVersion.Patch)),
            Err(pe_format::Error::IoError(e)) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                warn!("Cannot obtain eax.dll infos: {:?}", e);
                None
            }
        }
    }
}

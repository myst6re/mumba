use std::path::PathBuf;
use crate::game::env::Env;
use crate::pe_format;
use crate::provision;
use crate::game::installation::Edition;

pub struct Ffnx {

}

impl Ffnx {
    pub fn from_url(url: &str, target_dir: &PathBuf, env: &Env) -> Result<(), provision::Error> {
        provision::download_zip(url, "FFNx.zip", target_dir, env)
    }

    pub fn from_file(source_file: &PathBuf, target_dir: &PathBuf) -> Result<(), zip::result::ZipError> {
        provision::extract_zip(source_file, target_dir)
    }

    pub fn is_installed(target_dir: &PathBuf, steam: bool) -> Option<String> {
        match pe_format::pe_version_info(target_dir.join(if steam { "AF3DN.P" } else { "eax.dll" }).as_path()) {
            Ok(version) => Some(format!("{}.{}.{}", version.dwProductVersion.Major, version.dwProductVersion.Minor, version.dwProductVersion.Patch)),
            Err(pe_format::Error::IoError(e)) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                warn!("Cannot obtain eax.dll infos: {:?}", e);
                None
            }
        }
    }

    pub fn find_last_stable_version_on_github(repo_name: &str, edition: &Edition) -> String {
        let last_tag = crate::github::find_last_tag_version(repo_name).unwrap_or(String::from("1.19.1"));

        let url = "https://github.com/julianxhokaxhiu/FFNx/releases/download";
        let filename_prefix = if matches!(edition, Edition::Steam) {
            "Steam"
        } else {
            "FF8_2000"
        };

        format!("{}/{}/FFNx-{}-v{}.0.zip", url, last_tag, filename_prefix, last_tag)
    }
}

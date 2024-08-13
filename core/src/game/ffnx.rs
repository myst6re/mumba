#[cfg(all(feature = "network", feature = "zip"))]
use crate::game::env::Env;
#[cfg(feature = "network")]
use crate::game::installation::Edition;
#[cfg(feature = "pe")]
use crate::pe_format;
#[cfg(any(feature = "network", feature = "zip"))]
use crate::provision;
use std::path::{Path, PathBuf};

pub struct Ffnx {}

impl Ffnx {
    #[cfg(all(feature = "network", feature = "zip"))]
    pub fn from_url(url: &str, target_dir: &PathBuf, env: &Env) -> Result<(), provision::ErrorBox> {
        provision::download_zip(url, "FFNx.zip", target_dir, env)
    }

    #[cfg(feature = "zip")]
    pub fn from_file(
        source_file: &PathBuf,
        target_dir: &PathBuf,
    ) -> Result<(), zip::result::ZipError> {
        provision::extract_zip(source_file, target_dir)
    }

    #[cfg(feature = "pe")]
    pub fn is_installed(target_dir: &Path, steam: bool) -> Option<String> {
        match pe_format::pe_version_info(
            target_dir
                .join(if steam { "AF3DN.P" } else { "eax.dll" })
                .as_path(),
        ) {
            Ok(infos) => Some(format!(
                "{}.{}.{}",
                infos.product_version.Major,
                infos.product_version.Minor,
                infos.product_version.Patch
            )),
            Err(pe_format::Error::IoError(e)) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                warn!("Cannot obtain eax.dll infos: {:?}", e);
                None
            }
        }
    }

    #[cfg(feature = "network")]
    pub fn find_last_stable_version_on_github(repo_name: &str, edition: &Edition) -> String {
        let last_tag =
            crate::github::find_last_tag_version(repo_name).unwrap_or(String::from("1.19.1"));

        let url = "https://github.com/julianxhokaxhiu/FFNx/releases/download";
        let filename_prefix = if matches!(edition, Edition::Steam) {
            "Steam"
        } else {
            "FF8_2000"
        };

        format!(
            "{}/{}/FFNx-{}-v{}.0.zip",
            url, last_tag, filename_prefix, last_tag
        )
    }
}

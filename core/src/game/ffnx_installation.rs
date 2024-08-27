#[cfg(all(feature = "network", feature = "zip"))]
use crate::game::env::Env;
#[cfg(any(feature = "network", feature = "pe"))]
use crate::game::installation::{Edition, Installation};
#[cfg(feature = "pe")]
use crate::pe_format;
#[cfg(any(feature = "network", feature = "zip"))]
use crate::provision;
use std::path::Path;
#[cfg(any(feature = "network", feature = "zip"))]
use std::path::PathBuf;

pub struct FfnxInstallation {
    pub version: String,
    pub path: PathBuf,
    pub exe_name: String,
}

impl FfnxInstallation {
    #[cfg(all(feature = "network", feature = "zip"))]
    pub fn download(url: &str, target_dir: &PathBuf, env: &Env) -> Result<(), provision::ErrorBox> {
        provision::download_zip(url, "FFNx.zip", target_dir, env)
    }

    #[cfg(feature = "pe")]
    pub fn from_directory(
        target_dir: &Path,
        installation: &Installation,
    ) -> Option<FfnxInstallation> {
        let dll_name = if matches!(installation.edition, Edition::Steam) {
            "AF3DN.P"
        } else {
            "eax.dll"
        };
        let ff8_exe_name = if matches!(installation.edition, Edition::Standard) {
            // Rename exe to prevent Windows Compatibility patches for 2000 edition
            "FF8_Moomba.exe"
        } else {
            &installation.exe_name
        };
        match pe_format::pe_version_info(target_dir.join(dll_name).as_path()) {
            Ok(infos) => Some(FfnxInstallation {
                version: format!(
                    "{}.{}.{}",
                    infos.product_version.Major,
                    infos.product_version.Minor,
                    infos.product_version.Patch
                ),
                path: PathBuf::from(target_dir),
                exe_name: String::from(ff8_exe_name),
            }),
            Err(pe_format::Error::IoError(e)) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                warn!("Cannot obtain {} infos: {:?}", dll_name, e);
                None
            }
        }
    }

    #[cfg(feature = "network")]
    pub fn find_last_stable_version_on_github(
        repo_name: &str,
        edition: &Edition,
    ) -> (String, String) {
        let last_tag = crate::github::find_last_tag_version(repo_name)
            .and_then(|tag| Ok(tag.name))
            .unwrap_or(String::from("1.19.1"));

        let url = "https://github.com/julianxhokaxhiu/FFNx/releases/download";
        let filename_prefix = if matches!(edition, Edition::Steam) {
            "Steam"
        } else {
            "FF8_2000"
        };

        (
            format!(
                "{}/{}/FFNx-{}-v{}.0.zip",
                url, last_tag, filename_prefix, last_tag
            ),
            last_tag,
        )
    }

    pub fn exe_path(&self) -> PathBuf {
        self.path.join(&self.exe_name)
    }

    pub fn config_path(&self) -> PathBuf {
        self.path.join("FFNx.toml")
    }
}

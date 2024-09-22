#[cfg(feature = "network")]
use crate::config::UpdateChannel;
#[cfg(all(feature = "network", feature = "zip"))]
use crate::game::env::Env;
use crate::game::installation::{Edition, Installation};
#[cfg(feature = "network")]
use crate::github::GitHubReleaseAsset;
use crate::os::run_helper;
#[cfg(feature = "pe")]
use crate::pe_format;
#[cfg(any(feature = "network", feature = "zip"))]
use crate::provision;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Child, Command};

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
            "FF8_Mumba.exe"
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
                warn!("Cannot obtain {} infos: {}", dll_name, e);
                None
            }
        }
    }

    #[cfg(feature = "network")]
    pub fn find_version_on_github(
        repo_name: &str,
        edition: &Edition,
        update_channel: UpdateChannel,
    ) -> String {
        let last_release = match crate::github::find_last_release(repo_name) {
            Ok(last_release) => Some(last_release),
            Err(e) => {
                warn!("Unable to find the last release from GitHub: {}", e);
                None
            }
        };

        let release = match update_channel {
            UpdateChannel::Stable => last_release.and_then(|r| r.latest),
            UpdateChannel::Beta => last_release.and_then(|r| r.latest_not_recent),
            UpdateChannel::Alpha => last_release.and_then(|r| r.prerelease),
        };

        release.and_then(|release| {
            Self::find_asset_from_github_release(&release, edition)
        }).map(|asset| asset.browser_download_url)
        .unwrap_or_else(|| {
            String::from(match edition {
                Edition::Steam => "https://github.com/julianxhokaxhiu/FFNx/releases/download/canary/FFNx-Steam-v1.19.1.114.zip",
                Edition::Standard | Edition::Remastered => "https://github.com/julianxhokaxhiu/FFNx/releases/download/canary/FFNx-FF8_2000-v1.19.1.114.zip",
            })
        })
    }

    #[cfg(feature = "network")]
    pub fn find_asset_from_github_release(
        release: &crate::github::GitHubRelease,
        edition: &Edition,
    ) -> Option<GitHubReleaseAsset> {
        let keyword = match edition {
            Edition::Steam => "steam",
            Edition::Standard | Edition::Remastered => "ff8_2000",
        };
        release
            .assets
            .iter()
            .find(|asset| {
                asset
                    .name
                    .to_ascii_lowercase()
                    .replace(['-', ' '], "_")
                    .contains(keyword)
            })
            .cloned()
    }

    pub fn exe_path(&self) -> PathBuf {
        self.path.join(&self.exe_name)
    }

    pub fn config_path(&self) -> PathBuf {
        self.path.join("FFNx.toml")
    }

    fn launch_game_directly(&self, ff8_path: &PathBuf) -> Result<Child, std::io::Error> {
        Installation::launch_game_directly(ff8_path, &self.path)
    }

    pub fn launch_game(
        &self,
        game_installation: &Installation,
        steam_exe: &Path,
    ) -> std::io::Result<()> {
        if let Err(e) = match game_installation.edition {
            Edition::Standard => self.launch_game_directly(&self.exe_path()),
            Edition::Steam | Edition::Remastered => game_installation
                .launch_game_via_steam(&self.exe_path(), steam_exe, &self.path)
                .or_else(|_| self.launch_game_directly(&game_installation.get_launcher_path())),
        } {
            error!("Unable to launch game: {}", e);
            Err(e)
        } else {
            Ok(())
        }
    }
}

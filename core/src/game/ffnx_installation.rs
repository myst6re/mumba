#[cfg(feature = "network")]
use crate::config::UpdateChannel;
#[cfg(all(feature = "network", feature = "zip"))]
use crate::game::env::Env;
use crate::game::installation::{Edition, Installation};
#[cfg(feature = "network")]
use crate::github::GitHubReleaseAsset;
#[cfg(feature = "pe")]
use crate::pe_format;
#[cfg(any(feature = "network", feature = "zip"))]
use crate::provision;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;
use std::path::PathBuf;

pub struct FfnxInstallation {
    pub version: String,
    pub path: PathBuf,
}

impl FfnxInstallation {
    #[cfg(all(feature = "network", feature = "zip"))]
    pub fn download(url: &str, target_dir: &PathBuf, env: &Env) -> Result<(), provision::Error> {
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
        if !matches!(installation.edition, Edition::Steam)
            && target_dir.join("AF3DN.P").as_path().exists()
        {
            warn!("Steam FFNx installed, renaming needed");
            provision::rename_file(&target_dir.join("AF3DN.P"), &target_dir.join(dll_name)).unwrap()
        }
        match pe_format::pe_version_info(target_dir.join(dll_name).as_path()) {
            Ok(infos) => {
                if infos.product_name.unwrap_or_default() == "FFNx" {
                    Some(FfnxInstallation {
                        version: format!(
                            "{}.{}.{}",
                            infos.product_version.Major,
                            infos.product_version.Minor,
                            infos.product_version.Patch,
                        ),
                        path: PathBuf::from(target_dir),
                    })
                } else {
                    None
                }
            }
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
        // Rename exe to prevent Windows Compatibility patches for 2000 edition
        self.path.join(String::from("FF8_Mumba.exe"))
    }

    pub fn config_path(&self) -> PathBuf {
        self.path.join("FFNx.toml")
    }

    pub fn launch_game(&self) -> std::io::Result<()> {
        if let Err(e) = Installation::launch_game_directly(&self.exe_path(), &self.path) {
            error!("Unable to launch game: {}", e);
            Err(e)
        } else {
            Ok(())
        }
    }

    pub fn fix_game_freeze_at_startup(&self) -> std::io::Result<()> {
        let path = self.exe_path();
        let mut f = File::options().read(false).write(true).open(path)?;
        f.seek(SeekFrom::Start(0x13F1))?;
        f.write_all(b"\x75")?;
        f.seek(SeekFrom::Start(0x706E0))?;
        f.write_all(b"\x85")?;
        f.seek(SeekFrom::Start(0x791F2C))?;
        f.write_all(b"_")?;
        f.seek(SeekFrom::Start(0x791F34))?;
        f.write_all(b"_")?;
        f.seek(SeekFrom::Start(0x791F3C))?;
        f.write_all(b"_")?;
        f.seek(SeekFrom::Start(0x791F44))?;
        f.write_all(b"_")
    }
}

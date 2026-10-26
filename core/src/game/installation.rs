#[cfg(any(feature = "pe", feature = "zip"))]
use crate::game::env::Env;
#[cfg(windows)]
use crate::os::regedit;
use crate::os::run_helper;
#[cfg(feature = "pe")]
use crate::pe_format;
#[cfg(any(feature = "pe", feature = "zip"))]
use crate::provision;
use crate::steam;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use thiserror::Error;

#[derive(Clone, Debug)]
pub enum Edition {
    Standard,
    Steam = 39150,
    Remastered = 1026680,
}

#[derive(Clone, Debug)]
pub enum Version {
    Unknown,
    V100,
    V120,
    V120NV,
}

#[derive(Clone, Debug)]
pub enum Publisher {
    EaJp,
    EaUs,
    EidosDe,
    EidosFr,
    EidosIt,
    EidosSp,
    EidosUk,
}

#[derive(Clone)]
pub struct Installation {
    pub app_path: PathBuf,
    pub edition: Edition,
    pub language: String,
    pub config_path: Option<PathBuf>,
}

#[derive(Error, Debug)]
pub enum FromExeError {
    #[error("EXE file not found")]
    NotFound,
    #[error("The launcher was selected, please select the FF8 executable")]
    LauncherSelected,
}

impl Installation {
    pub fn from_directory<P: AsRef<Path>>(app_path: P, edition: Option<Edition>) -> Option<Self> {
        let app_path = app_path.as_ref();
        if !app_path.exists() {
            return None;
        }
        let edition = edition.unwrap_or_else(|| Self::detect_edition(app_path));
        let language = Self::detect_language(app_path, edition.clone());
        let config_path = Self::get_config_path(&edition, app_path);

        Some(Self {
            app_path: PathBuf::from(app_path),
            edition,
            language,
            config_path,
        })
    }

    pub fn search() -> Vec<Self> {
        let mut installations = Vec::new();
        #[cfg(windows)]
        if let Some(installation) = Self::search_original_version() {
            installations.push(installation)
        };
        let steam_library_folders = match steam::Steam::from_config() {
            Ok(lib_folders) => Some(lib_folders),
            Err(e) => {
                warn!("Cannot find Steam Client installation: {}", e);
                None
            }
        };
        if let Some(installation) = Self::search_steam_edition(&steam_library_folders) {
            installations.push(installation)
        };
        if let Some(installation) = Self::search_remastered_edition(&steam_library_folders) {
            installations.push(installation)
        };
        installations
    }

    fn detect_edition(app_path: &Path) -> Edition {
        if app_path.join("FFVIII.exe").exists() {
            Edition::Remastered
        } else if app_path.join("AF3DN.P").exists() {
            Edition::Steam
        } else {
            Edition::Standard
        }
    }

    fn detect_language(app_path: &Path, edition: Edition) -> String {
        match edition {
            Edition::Standard => {
                Self::get_standard_edition_lang(app_path).unwrap_or(String::from("eng"))
            }
            Edition::Steam => Self::get_steam_edition_lang(app_path).unwrap_or(String::from("en")),
            Edition::Remastered => String::from("multi"),
        }
    }

    fn get_standard_edition_lang(app_path: &Path) -> std::io::Result<String> {
        let contents = std::fs::read_to_string(app_path.join("Data").join("main.fl"))?;
        Ok(String::from(
            match contents.match_indices("ff8\\data\\").next() {
                Some((idx, _matched)) => {
                    let lang = &contents[idx + 9..idx + 12];
                    if lang.ends_with('\\') {
                        &contents[idx + 9..idx + 11]
                    } else {
                        lang
                    }
                }
                None => "eng",
            },
        ))
    }

    fn get_steam_edition_lang(path_lang_dat: &Path) -> std::io::Result<String> {
        let contents = std::fs::read_to_string(path_lang_dat.join("lang.dat"))?;
        Ok(contents.to_ascii_uppercase())
    }

    fn get_version_from_exe(exe_path: &PathBuf) -> std::io::Result<Option<(Version, Publisher)>> {
        let mut f = File::options().read(true).write(false).open(exe_path)?;
        f.seek(SeekFrom::Start(0x1004))?;
        let mut bytes = [0u8; 4];
        f.read_exact(&mut bytes)?;
        let version_check1 = u32::from_le_bytes(bytes);
        f.seek(SeekFrom::Start(0x1404))?;
        f.read_exact(&mut bytes)?;
        let version_check2 = u32::from_le_bytes(bytes);

        if version_check1 == 0x3885048D && version_check2 == 0x159618 {
            Ok(Some((Version::V120, Publisher::EaUs)))
        } else if version_check1 == 0x3885048D && version_check2 == 0x1597C8 {
            Ok(Some((Version::V120NV, Publisher::EaUs)))
        } else if version_check1 == 0x1085048D && version_check2 == 0x159B48 {
            Ok(Some((Version::V120, Publisher::EidosFr)))
        } else if version_check1 == 0x1085048D && version_check2 == 0x159CF8 {
            Ok(Some((Version::V120NV, Publisher::EidosFr)))
        } else if version_check1 == 0xA885048D && version_check2 == 0x159C48 {
            Ok(Some((Version::V120, Publisher::EidosDe)))
        } else if version_check1 == 0xA885048D && version_check2 == 0x159DF8 {
            Ok(Some((Version::V120NV, Publisher::EidosDe)))
        } else if version_check1 == 0x8085048D && version_check2 == 0x159C38 {
            Ok(Some((Version::V120, Publisher::EidosSp)))
        } else if version_check1 == 0x8085048D && version_check2 == 0x159DE8 {
            Ok(Some((Version::V120NV, Publisher::EidosSp)))
        } else if version_check1 == 0xB885048D && version_check2 == 0x159BC8 {
            Ok(Some((Version::V120, Publisher::EidosIt)))
        } else if version_check1 == 0xB885048D && version_check2 == 0x159D78 {
            Ok(Some((Version::V120NV, Publisher::EidosIt)))
        } else if version_check1 == 0x2885048D && version_check2 == 0x159598 {
            Ok(Some((Version::V120, Publisher::EidosUk)))
        } else if version_check1 == 0x2885048D && version_check2 == 0x159748 {
            Ok(Some((Version::V120NV, Publisher::EidosUk)))
        } else if version_check1 == 0x1B6E9CC && version_check2 == 0x7C8DFFC9 {
            f.seek(SeekFrom::Start(0x1010))?;
            f.read_exact(&mut bytes)?;
            let version_check3 = u32::from_le_bytes(bytes);

            if version_check3 == 0x24AC {
                Ok(Some((Version::V120NV, Publisher::EaJp)))
            } else {
                Ok(Some((Version::V120, Publisher::EaJp)))
            }
        } else {
            Ok(None)
        }
    }

    #[cfg(windows)]
    pub fn get_config_path(edition: &Edition, app_path: &Path) -> Option<PathBuf> {
        match edition {
            Edition::Standard => Some(app_path.to_path_buf()),
            Edition::Steam => Some(
                crate::os::windows::my_documents_path()
                    .join("Square Enix")
                    .join("FINAL FANTASY VIII Steam"),
            ),
            Edition::Remastered => Self::find_first_directory_is_a_number(
                crate::os::windows::saved_games_path()
                    .join("FINAL FANTASY VIII Remastered")
                    .join("Steam"),
            ),
        }
    }

    fn find_first_directory_is_a_number(path: PathBuf) -> Option<PathBuf> {
        match path.read_dir() {
            Ok(it) => {
                for entry in it {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            if path.is_dir()
                                && path
                                    .file_name()
                                    .unwrap()
                                    .to_string_lossy()
                                    .parse::<u64>()
                                    .is_ok()
                            {
                                return Some(path);
                            }
                        }
                        _ => break,
                    }
                }
                None
            }
            Err(_) => None,
        }
    }

    #[cfg(unix)]
    pub fn get_config_path(edition: &Edition, app_path: &Path) -> Option<PathBuf> {
        match edition {
            Edition::Standard => Some(app_path.to_path_buf()),
            Edition::Steam => Some(
                app_path
                    .join("..")
                    .join("..")
                    .join("compatdata")
                    .join((Edition::Steam as u64).to_string())
                    .join("pfx")
                    .join("drive_c")
                    .join("users")
                    .join("steamuser")
                    .join("My Documents")
                    .join("Square Enix")
                    .join("FINAL FANTASY VIII Steam"),
            ),
            Edition::Remastered => Some(
                app_path
                    .join("..")
                    .join("..")
                    .join("compatdata")
                    .join((Edition::Remastered as u64).to_string())
                    .join("pfx")
                    .join("drive_c")
                    .join("users")
                    .join("steamuser")
                    .join("My Documents")
                    .join("My Games")
                    .join("FINAL FANTASY VIII Remastered")
                    .join("game_data"),
            ),
        }
    }

    pub fn get_app_id(&self) -> u64 {
        self.edition.clone() as u64
    }

    pub fn get_launcher_path(&self) -> PathBuf {
        match self.edition {
            Edition::Standard => self.app_path.join("FF8Config.exe"),
            Edition::Steam => self.app_path.join("FF8_Launcher.exe"),
            Edition::Remastered => self.app_path.join("FFVIII_LAUNCHER.exe"),
        }
    }

    #[cfg(windows)]
    fn search_original_version() -> Option<Self> {
        let locations = [regedit::RegLocation::Machine, regedit::RegLocation::User];
        for loc in locations {
            match regedit::reg_value_str(
                regedit::RegTarget::Wow32,
                loc,
                r"SOFTWARE\\Square Soft, Inc\\Final Fantasy VIII\\1.00",
                r"AppPath",
            ) {
                Ok(app_path) => return Self::from_directory(app_path, Some(Edition::Standard)),
                Err(_) => continue,
            }
        }
        None
    }

    #[cfg(windows)]
    fn search_steam_edition(steam_library_folders: &Option<steam::Steam>) -> Option<Self> {
        steam_library_folders
            .as_ref()
            .and_then(|lib_folders| {
                lib_folders.find_app(Edition::Steam as u64, "FINAL FANTASY VIII")
            })
            .or_else(|| {
                warn!(
                    "Cannot find FINAL FANTASY VIII installation path, try with uninstall entries"
                );
                regedit::reg_search_installed_app_by_key(
                    r"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App 39150",
                )
                .map(PathBuf::from)
            })
            .and_then(|app_path| Self::from_directory(app_path, Some(Edition::Steam)))
    }

    #[cfg(unix)]
    fn search_steam_edition(steam_library_folders: &Option<steam::Steam>) -> Option<Self> {
        steam_library_folders
            .as_ref()
            .and_then(|lib_folders| {
                lib_folders.find_app(Edition::Steam as u64, "FINAL FANTASY VIII")
            })
            .and_then(|app_path| Self::from_directory(app_path, Some(Edition::Steam)))
    }

    #[cfg(windows)]
    fn search_remastered_edition(steam_library_folders: &Option<steam::Steam>) -> Option<Self> {
        steam_library_folders
            .as_ref()
            .and_then(|lib_folders| lib_folders.find_app(Edition::Remastered as u64, "FINAL FANTASY VIII Remastered"))
            .or_else(|| {
                warn!("Cannot find FINAL FANTASY VIII Remastered installation path, try with uninstall entries");
                regedit::reg_search_installed_app_by_key(
                    r"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App 1026680",
                ).map(PathBuf::from)
            })
            .and_then(|app_path| Self::from_directory(app_path, Some(Edition::Remastered)))
    }

    #[cfg(unix)]
    fn search_remastered_edition(steam_library_folders: &Option<steam::Steam>) -> Option<Self> {
        steam_library_folders
            .as_ref()
            .and_then(|lib_folders| {
                lib_folders.find_app(Edition::Remastered as u64, "FINAL FANTASY VIII Remastered")
            })
            .and_then(|app_path| Self::from_directory(app_path, Some(Edition::Remastered)))
    }

    #[cfg(all(feature = "network", feature = "zip"))]
    pub fn install_patch_remote(
        url: &str,
        target_dir: &PathBuf,
        env: &Env,
    ) -> Result<(), provision::Error> {
        provision::download_zip(url, "FF8-patch.zip", target_dir, env)
    }

    #[cfg(feature = "zip")]
    pub fn install_patch_local(
        source_file: &PathBuf,
        target_dir: &PathBuf,
    ) -> Result<(), zip::result::ZipError> {
        provision::extract_zip(source_file, target_dir)
    }

    #[cfg(feature = "pe")]
    pub fn replace_launcher(self: &Installation, env: &Env) -> std::io::Result<()> {
        match self.replace_launcher_from_app_path(env) {
            Ok(o) => Ok(o),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                if cfg!(windows) {
                    #[cfg(windows)]
                    crate::os::windows::run_as(
                        &String::from(env.mumba_dir.join("mmb.exe").to_str().unwrap()),
                        &format!(
                            "replace_launcher \"{}\"",
                            self.app_path.to_string_lossy().to_string().replace('"', "")
                        ),
                    )?;
                    Ok(())
                } else {
                    Err(e)
                }
            }
            Err(e) => Err(e)?,
        }
    }

    #[cfg(feature = "pe")]
    pub fn replace_launcher_from_app_path(&self, env: &Env) -> std::io::Result<()> {
        let launcher_path = self.get_launcher_path();
        let resource_launcher_path = env.get_resource_launcher_path();
        let (launcher_product_name, legal_copyright) =
            match pe_format::pe_version_info(&launcher_path) {
                Ok(infos) => (
                    infos.product_name.unwrap_or_default(),
                    infos.legal_copyright.unwrap_or_default(),
                ),
                Err(_) => (String::new(), String::new()),
            };
        info!(
            "Launcher product name: {} (path: \"{}\")",
            launcher_product_name,
            launcher_path.to_string_lossy()
        );
        let backup_path = self.app_path.join(format!(
            "{}_Original.exe",
            launcher_path
                .with_extension("")
                .file_name()
                .unwrap()
                .to_string_lossy()
        ));
        if !backup_path.exists() || launcher_product_name == "FINAL FANTASY VIII for PC" {
            provision::copy_file(&launcher_path, &backup_path)?
        }
        if legal_copyright
            != pe_format::pe_version_info(&resource_launcher_path)
                .map(|infos| infos.legal_copyright.unwrap_or_default())
                .unwrap_or_default()
        {
            provision::copy_file(&resource_launcher_path, &launcher_path)?
        }

        Ok(())
    }

    pub fn launch_game_via_steam(
        &self,
        ff8_path: &Path,
        steam_exe: &Path,
        current_dir: &Path,
    ) -> Result<Child, std::io::Error> {
        let app_id = self.get_app_id();
        info!(
            "Launch \"{} -applaunch {} '{}'\" in dir \"{}\"...",
            steam_exe.to_string_lossy(),
            app_id,
            ff8_path.to_string_lossy(),
            current_dir.to_string_lossy()
        );
        run_helper(&mut Command::new(steam_exe))
            .args(["-applaunch", app_id.to_string().as_str()])
            .arg(ff8_path.as_os_str())
            .arg("--debug")
            .current_dir(current_dir)
            .spawn()
    }

    pub fn launch_game_directly(
        ff8_path: &PathBuf,
        current_dir: &Path,
    ) -> Result<Child, std::io::Error> {
        info!(
            "Launch \"{}\" in dir \"{}\"...",
            ff8_path.to_string_lossy(),
            current_dir.to_string_lossy()
        );
        run_helper(&mut Command::new(ff8_path))
            .current_dir(current_dir)
            .spawn()
    }

    fn get_cw_path(&self) -> PathBuf {
        let path = self.app_path.join("Chocobo.exe");
        let path_lang = self.app_path.join(format!("Chocobo_{}.exe", self.language));

        if path.exists() {
            path
        } else if path_lang.exists() {
            path_lang
        } else {
            self.app_path.join("Chocobo_FR_IT_DE_ES.exe")
        }
    }

    pub fn launch_cw(&self) -> std::io::Result<()> {
        if let Err(e) = Self::launch_game_directly(&self.get_cw_path(), &self.app_path) {
            error!("Unable to launch game: {}", e);
            Err(e)
        } else {
            Ok(())
        }
    }
}

use crate::game::env::Env;
#[cfg(windows)]
use crate::os::regedit;
#[cfg(feature = "pe")]
use crate::pe_format;
use crate::provision;
use crate::steam;
use std::fs::File;
#[cfg(feature = "pe")]
use std::io::Write;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::str::FromStr;
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
    pub exe_name: String,
    pub edition: Edition,
    pub version: Option<(Version, Publisher)>,
    pub language: String,
}

#[derive(Error, Debug)]
pub enum FromExeError {
    #[error("EXE file not found")]
    NotFound,
    #[error("The launcher was selected, please select the FF8 executable")]
    LauncherSelected,
}

impl Installation {
    pub fn new(
        app_path: PathBuf,
        exe_name: String,
        edition: Edition,
        version: Option<(Version, Publisher)>,
        language: String,
    ) -> Self {
        Self {
            app_path,
            exe_name,
            edition,
            version,
            language,
        }
    }

    pub fn from_exe_path(exe_path: &Path) -> Result<Self, FromExeError> {
        if !exe_path.exists() {
            return Err(FromExeError::NotFound);
        };

        let app_path = exe_path
            .parent()
            .map(|e| e.to_path_buf())
            .unwrap_or_default();
        // Detect edition and language
        let (edition, language) = match Self::get_steam_edition_lang(&app_path) {
            Ok(lang) => (Edition::Steam, lang),
            Err(_) => (
                Edition::Standard,
                Self::get_standard_edition_lang(&app_path).unwrap_or(String::from("eng")),
            ),
        };

        let mut exe_name = String::from(
            exe_path
                .file_name()
                .and_then(|e| e.to_str())
                .unwrap_or_default(),
        );
        let lower_exe_name = exe_name.to_ascii_lowercase();

        if lower_exe_name.contains("launcher") || lower_exe_name.contains("chocobo") {
            if !matches!(edition, Edition::Steam) {
                return Err(FromExeError::LauncherSelected);
            }
            exe_name = format!("FF8_{}.exe", language);
            if !app_path.join(&exe_name).exists() {
                return Err(FromExeError::LauncherSelected);
            }
        }

        // Detect version
        let version = Self::get_version_from_exe(&app_path.join(&exe_name)).unwrap_or(None);

        Ok(Self {
            app_path,
            exe_name,
            edition,
            version,
            language,
        })
    }

    pub fn from_directory(app_path: PathBuf) -> Self {
        // Detect exe name and edition
        let (exe_name, edition, language) = match Self::get_steam_edition_lang(&app_path) {
            Ok(lang) => {
                let mut exe_name = String::new();
                exe_name.push_str("FF8_");
                exe_name.push_str(&lang);
                exe_name.push_str(".exe");
                (exe_name, Edition::Steam, lang)
            }
            Err(_) => (
                String::from_str("FF8.exe").unwrap(),
                Edition::Standard,
                Self::get_standard_edition_lang(&app_path).unwrap_or(String::from("eng")),
            ),
        };
        // Detect version
        let version = Self::get_version_from_exe(&app_path.join(&exe_name)).unwrap_or(None);

        Self {
            app_path,
            exe_name,
            edition,
            version,
            language,
        }
    }

    pub fn exe_path(&self) -> PathBuf {
        PathBuf::from(&self.app_path).join(&self.exe_name)
    }

    pub fn search() -> Vec<Self> {
        let mut installations = Vec::new();
        #[cfg(windows)]
        if let Some((app_path, exe_name, language)) = Self::search_original_version() {
            let version =
                Self::get_version_from_exe(&PathBuf::new().join(&app_path).join(&exe_name))
                    .unwrap_or(None);
            installations.push(Self::new(
                app_path,
                exe_name,
                Edition::Standard,
                version,
                language,
            ))
        };
        let steam_library_folders = match steam::Steam::from_config() {
            Ok(lib_folders) => Some(lib_folders),
            Err(e) => {
                warn!("Cannot find Steam Client installation: {:?}", e);
                None
            }
        };
        if let Some((app_path, exe_name, language)) =
            Self::search_steam_edition(&steam_library_folders)
        {
            let version = Self::get_version_from_exe(&app_path.join(&exe_name)).unwrap_or(None);
            installations.push(Self::new(
                app_path,
                exe_name,
                Edition::Steam,
                version,
                language,
            ))
        };
        if let Some((app_path, exe_name, language)) =
            Self::search_remastered_edition(&steam_library_folders)
        {
            let version = Self::get_version_from_exe(&app_path.join(&exe_name)).unwrap_or(None);
            installations.push(Self::new(
                app_path,
                exe_name,
                Edition::Remastered,
                version,
                language,
            ))
        };
        installations
    }

    pub fn get_standard_edition_lang(path_lang_dat: &Path) -> std::io::Result<String> {
        let contents = std::fs::read_to_string(path_lang_dat.join("Data").join("main.fl"))?;
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

    pub fn get_steam_edition_lang(path_lang_dat: &Path) -> std::io::Result<String> {
        let contents = std::fs::read_to_string(path_lang_dat.join("lang.dat"))?;
        Ok(contents.to_ascii_uppercase())
    }

    pub fn get_version_from_exe(
        exe_path: &PathBuf,
    ) -> std::io::Result<Option<(Version, Publisher)>> {
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

    pub fn get_app_id(&self) -> u64 {
        self.edition.clone() as u64
    }

    #[cfg(windows)]
    fn search_original_version() -> Option<(PathBuf, String, String)> {
        let locations = [regedit::RegLocation::Machine, regedit::RegLocation::User];
        for loc in locations {
            match regedit::reg_value_str(
                regedit::RegTarget::Wow32,
                loc,
                r"SOFTWARE\\Square Soft, Inc\\Final Fantasy VIII\\1.00",
                r"AppPath",
            ) {
                Ok(app_path) => {
                    let app_path = PathBuf::from(app_path);
                    let lang =
                        Self::get_standard_edition_lang(&app_path).unwrap_or(String::from("eng"));
                    return Some((app_path, String::from("FF8.exe"), lang));
                }
                Err(_) => continue,
            }
        }
        None
    }

    #[cfg(windows)]
    fn search_steam_edition(
        steam_library_folders: &Option<steam::Steam>,
    ) -> Option<(PathBuf, String, String)> {
        steam_library_folders
            .as_ref()
            .and_then(|lib_folders| lib_folders.find_app(39150, "FINAL FANTASY VIII"))
            .or_else(|| {
                warn!(
                    "Cannot find FINAL FANTASY VIII installation path, try with uninstall entries"
                );
                regedit::reg_search_installed_app_by_key(
                    r"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App 39150",
                )
                .map(PathBuf::from)
            })
            .and_then(|app_path| match Self::get_steam_edition_lang(&app_path) {
                Ok(lang) => {
                    let mut exe_name = String::new();
                    exe_name.push_str("FF8_");
                    exe_name.push_str(lang.as_str());
                    exe_name.push_str(".exe");
                    Some((app_path, exe_name, lang))
                }
                Err(e) => {
                    warn!("Open FF8 lang.dat: {:?}", e);
                    None
                }
            })
    }

    #[cfg(unix)]
    fn search_steam_edition(
        steam_library_folders: &Option<steam::Steam>,
    ) -> Option<(PathBuf, String, String)> {
        steam_library_folders
            .as_ref()
            .and_then(|lib_folders| lib_folders.find_app(39150, "FINAL FANTASY VIII"))
            .and_then(|app_path| match Self::get_steam_edition_lang(&app_path) {
                Ok(lang) => {
                    let mut exe_name = String::new();
                    exe_name.push_str("FF8_");
                    exe_name.push_str(lang.as_str());
                    exe_name.push_str(".exe");
                    Some((app_path, exe_name, lang))
                }
                Err(e) => {
                    warn!("Open FF8 lang.dat: {:?}", e);
                    None
                }
            })
    }

    #[cfg(windows)]
    fn search_remastered_edition(
        steam_library_folders: &Option<steam::Steam>,
    ) -> Option<(PathBuf, String, String)> {
        steam_library_folders
            .as_ref()
            .and_then(|lib_folders| lib_folders.find_app(1026680, "FINAL FANTASY VIII Remastered"))
            .or_else(|| {
                warn!("Cannot find FINAL FANTASY VIII Remastered installation path, try with uninstall entries");
                regedit::reg_search_installed_app_by_key(
                    r"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App 1026680",
                ).map(PathBuf::from)
            })
            .map(|app_path| (app_path, String::from("FFVIII.exe"), String::from("en")))
        // TODO: lang
    }

    #[cfg(unix)]
    fn search_remastered_edition(
        steam_library_folders: &Option<steam::Steam>,
    ) -> Option<(PathBuf, String, String)> {
        steam_library_folders
            .as_ref()
            .and_then(|lib_folders| lib_folders.find_app(1026680, "FINAL FANTASY VIII Remastered"))
            .map(|app_path| (app_path, String::from("FFVIII.exe"), String::from("en")))
        // TODO: lang
    }

    #[cfg(all(feature = "network", feature = "zip"))]
    pub fn install_patch_remote(
        url: &str,
        target_dir: &PathBuf,
        env: &Env,
    ) -> Result<(), provision::ErrorBox> {
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
    pub fn replace_launcher(
        self: &Installation,
        ff8_path: &Path,
        env: &Env,
    ) -> std::io::Result<()> {
        match Self::replace_launcher_from_app_path(&self.app_path, ff8_path, env) {
            Ok(o) => Ok(o),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                if cfg!(windows) {
                    #[cfg(windows)]
                    crate::os::windows::run_as(
                        &String::from(env.moomba_dir.join("mmb.exe").to_str().unwrap()),
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
    pub fn replace_launcher_from_app_path(
        app_path: &Path,
        ff8_path: &Path,
        env: &Env,
    ) -> std::io::Result<()> {
        Self::create_launcher_config_file(app_path, ff8_path)?;
        let launcher_path = app_path.join("FF8_Launcher.exe");
        let launcher_product_name = match pe_format::pe_version_info(&launcher_path) {
            Ok(infos) => infos.product_name.unwrap_or_default(),
            Err(_) => String::new(),
        };
        info!(
            "Launcher product name: {} (path: {})",
            launcher_product_name,
            launcher_path.to_string_lossy()
        );
        let backup_path = app_path.join("FF8_Launcher_Original.exe");
        if !backup_path.exists() || launcher_product_name == "FINAL FANTASY VIII for PC" {
            provision::copy_file(&launcher_path, &backup_path)?
        }
        provision::copy_file(&env.moomba_dir.join("ff8_launcher.exe"), &launcher_path)?;
        Ok(())
    }

    #[cfg(feature = "pe")]
    fn create_launcher_config_file(app_path: &Path, ff8_path: &Path) -> std::io::Result<()> {
        let config_path = app_path.join("moomba_path.txt");
        let mut file = File::create(config_path)?;
        file.write_all(ff8_path.to_string_lossy().as_bytes())?;
        Ok(())
    }
}

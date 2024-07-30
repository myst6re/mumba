use std::path::PathBuf;
use std::str::FromStr;
use crate::game::env::Env;
use crate::provision;
use crate::regedit;
use std::fs::File;
use std::io::{Seek, SeekFrom, Read};

#[derive(Clone, Debug)]
pub enum Edition {
    Standard,
    Steam,
    Remastered
}

#[derive(Clone, Debug)]
pub enum Version {
    Unknown,
    V100,
    V120,
    V120NV
}

#[derive(Clone, Debug)]
pub enum Publisher {
    EaJp,
    EaUs,
    EidosDe,
    EidosFr,
    EidosIt,
    EidosSp,
    EidosUk
}

#[derive(Clone)]
pub struct Installation {
    pub app_path: String,
    pub exe_name: String,
    pub edition: Edition,
    pub version: Option<(Version, Publisher)>,
    pub language: String
}

#[derive(Debug)]
pub enum LauncherInstallError {
    LibLoadingError(libloading::Error),
    IoError(std::io::Error)
}

impl From<libloading::Error> for LauncherInstallError {
    fn from(e: libloading::Error) -> Self {
        Self::LibLoadingError(e)
    }
}

impl From<std::io::Error> for LauncherInstallError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}

impl Installation {
    pub fn new(app_path: String, exe_name: String, edition: Edition, version: Option<(Version, Publisher)>, language: String) -> Self {
        Self {
            app_path,
            exe_name,
            edition,
            version,
            language
        }
    }

    pub fn from_exe_path(exe_path: &PathBuf) -> Option<Self> {
        if ! exe_path.exists() {
            return None
        };

        let exe_name = String::from(exe_path.file_name().and_then(|e| e.to_str()).unwrap_or_default());
        let app_path = String::from(exe_path.parent().map(|e| e.as_os_str()).and_then(|e| e.to_str()).unwrap_or_default());
        // Detect exe name and edition
        let (edition, language) = match Self::get_steam_edition_lang(&app_path) {
            Ok(lang) => (Edition::Steam, lang),
            Err(_) => (Edition::Standard, Self::get_standard_edition_lang(&app_path).unwrap_or(String::from("eng")))
        };
        // Detect version
        let version = Self::get_version_from_exe(&PathBuf::new().join(&app_path).join(&exe_name)).unwrap_or(None);

        Some(Self {
            app_path,
            exe_name,
            edition,
            version,
            language
        })
    }

    pub fn from_directory(app_path: String) -> Self {
        // Detect exe name and edition
        let (exe_name, edition, language) = match Self::get_steam_edition_lang(&app_path) {
            Ok(lang) => {
                let mut exe_name = String::new();
                exe_name.push_str("FF8_");
                exe_name.push_str(&lang);
                exe_name.push_str(".exe");
                (exe_name, Edition::Steam, lang)
            }
            Err(_) => (String::from_str("FF8.exe").unwrap(), Edition::Standard, Self::get_standard_edition_lang(&app_path).unwrap_or(String::from("eng")))
        };
        // Detect version
        let version = Self::get_version_from_exe(&PathBuf::new().join(&app_path).join(&exe_name)).unwrap_or(None);

        Self {
            app_path,
            exe_name,
            edition,
            version,
            language
        }
    }

    pub fn exe_path(self: &Self) -> PathBuf {
        PathBuf::from(&self.app_path).join(&self.exe_name)
    }

    pub fn search() -> Vec<Self> {
        let mut installations = Vec::new();
        match Self::search_original_version() {
            Some((app_path, exe_name, language)) => {
                let version = Self::get_version_from_exe(&PathBuf::new().join(&app_path).join(&exe_name)).unwrap_or(None);
                installations.push(Self::new(app_path, exe_name, Edition::Standard, version, language))
            },
            None => ()
        }
        ;
        match Self::search_steam_edition() {
            Some((app_path, exe_name, language)) => {
                let version = Self::get_version_from_exe(&PathBuf::new().join(&app_path).join(&exe_name)).unwrap_or(None);
                installations.push(Self::new(app_path, exe_name, Edition::Steam, version, language))
            },
            None => ()
        }
        ;
        match Self::search_remastered_edition() {
            Some((app_path, exe_name, language)) => {
                let version = Self::get_version_from_exe(&PathBuf::new().join(&app_path).join(&exe_name)).unwrap_or(None);
                installations.push(Self::new(app_path, exe_name, Edition::Remastered, version, language))
            },
            None => ()
        }
        ;
        installations
    }

    pub fn get_standard_edition_lang(app_path: &String) -> std::io::Result<String> {
        let path_lang_dat = PathBuf::from(app_path);
        let contents = std::fs::read_to_string(path_lang_dat.join("Data").join("main.fl"))?;
        Ok(String::from(match contents.match_indices("ff8\\data\\").next() {
            Some((idx, _matched)) => {
                let lang = &contents[idx + 9..idx + 12];
                if lang.ends_with('\\') {
                    &contents[idx + 9..idx + 11]
                } else {
                    lang
                }
            }
            None => "eng"
        }))
    }

    pub fn get_steam_edition_lang(app_path: &String) -> std::io::Result<String> {
        let path_lang_dat = PathBuf::from(app_path);
        let contents = std::fs::read_to_string(path_lang_dat.join("lang.dat"))?;
        Ok(contents.to_ascii_uppercase())
    }

    pub fn get_version_from_exe(exe_path: &PathBuf) -> std::io::Result<Option<(Version, Publisher)>> {
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

    fn search_original_version() -> Option<(String, String, String)> {
        let locations = [regedit::RegLocation::Machine, regedit::RegLocation::User];
        for loc in locations {
            match regedit::reg_value_str(
                regedit::RegTarget::Wow32,
                loc,
                r"SOFTWARE\\Square Soft, Inc\\Final Fantasy VIII\\1.00",
                r"AppPath"
            ) {
                Ok(app_path) => {
                    let lang = Self::get_standard_edition_lang(&app_path).unwrap_or(String::from("eng"));
                    return Some((app_path, String::from("FF8.exe"), lang))
                },
                Err(_) => continue
            }
        }
        ;
        None
    }

    fn search_steam_edition() -> Option<(String, String, String)> {
        match regedit::reg_search_installed_app_by_key(r"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App 39150") {
            Some(app_path) => {
                match Self::get_steam_edition_lang(&app_path) {
                    Ok(lang) => {
                        let mut exe_name = String::new();
                        exe_name.push_str("FF8_");
                        exe_name.push_str(lang.as_str());
                        exe_name.push_str(".exe");
                        Some((app_path, exe_name, lang))
                    },
                    Err(e) => {
                        warn!("Open FF8 lang.dat: {:?}", e);
                        None
                    }
                }
            },
            None => None
        }
    }

    fn search_remastered_edition() -> Option<(String, String, String)> {
        match regedit::reg_search_installed_app_by_key(r"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App 1026680") {
            Some(app_path) => Some((app_path, String::from("FFVIII.exe"), String::from("en"))), // TODO: lang
            None => None
        }
    }

    pub fn install_patch_remote(url: &str, target_dir: &PathBuf, env: &Env) -> Result<(), provision::Error> {
        provision::download_zip(url, "FF8-patch.zip", target_dir, env)
    }

    pub fn install_patch_local(source_file: &PathBuf, target_dir: &PathBuf) -> Result<(), zip::result::ZipError> {
        provision::extract_zip(source_file, target_dir)
    }

    pub fn replace_launcher(self: &Installation, env: &Env) -> Result<(), LauncherInstallError> {
        match provision::copy_file(&env.moomba_dir.join("ff8_launcher.exe"), &PathBuf::new().join(&self.app_path).join("FF8_Launcher.exe")) {
            Ok(o) => Ok(o),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                crate::windows::run_as(&String::from(env.moomba_dir.join("moomba_cli.exe").to_str().unwrap()), &self.app_path)?;
                Ok(())
            },
            Err(e) => Err(e)?
        }
    }
}

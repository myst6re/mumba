use std::path::PathBuf;
use std::str::FromStr;
use crate::game::env::Env;
use crate::provision;
use crate::regedit;

#[derive(Clone, Debug)]
pub enum Version {
    Standard,
    Steam,
    Remastered
}

#[derive(Clone)]
pub struct Installation {
    pub app_path: String,
    pub exe_name: String,
    pub version: Version,
    pub language: String
}

impl Installation {
    pub fn new(app_path: String, exe_name: String, version: Version, language: String) -> Self {
        Self {
            app_path,
            exe_name,
            version,
            language
        }
    }

    pub fn from_directory(app_path: String) -> Self {
        // Detect exe name and version
        let (exe_name, version, language) = match Self::get_steam_version_lang(&app_path) {
            Ok(lang) => {
                let mut exe_name = String::new();
                exe_name.push_str("FF8_");
                exe_name.push_str(&lang);
                exe_name.push_str(".exe");
                (exe_name, Version::Steam, lang)
            }
            Err(_) => (String::from_str("FF8.exe").unwrap(), Version::Standard, Self::get_standard_version_lang(&app_path).unwrap_or(String::from("eng")))
        };

        Self {
            app_path,
            exe_name,
            version,
            language
        }
    }

    pub fn search() -> Vec<Self> {
        let mut installations = Vec::new();
        match Self::search_original_version() {
            Some((app_path, exe_name, language)) => {
                installations.push(Self::new(app_path, exe_name, Version::Standard, language))
            },
            None => ()
        }
        ;
        match Self::search_steam_version() {
            Some((app_path, exe_name, language)) => {
                installations.push(Self::new(app_path, exe_name, Version::Steam, language))
            },
            None => ()
        }
        ;
        match Self::search_remastered_version() {
            Some((app_path, exe_name, language)) => {
                installations.push(Self::new(app_path, exe_name, Version::Remastered, language))
            },
            None => ()
        }
        ;
        installations
    }

    pub fn get_standard_version_lang(app_path: &String) -> std::io::Result<String> {
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

    pub fn get_steam_version_lang(app_path: &String) -> std::io::Result<String> {
        let path_lang_dat = PathBuf::from(app_path);
        let contents = std::fs::read_to_string(path_lang_dat.join("lang.dat"))?;
        Ok(contents.to_ascii_uppercase())
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
                    let lang = Self::get_standard_version_lang(&app_path).unwrap_or(String::from("eng"));
                    return Some((app_path, String::from("FF8.exe"), lang))
                },
                Err(_) => continue
            }
        }
        ;
        None
    }

    fn search_steam_version() -> Option<(String, String, String)> {
        match regedit::reg_search_installed_app_by_key(r"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App 39150") {
            Some(app_path) => {
                match Self::get_steam_version_lang(&app_path) {
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

    fn search_remastered_version() -> Option<(String, String, String)> {
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
}

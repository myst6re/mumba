#[cfg(windows)]
use crate::os::regedit::*;
use keyvalues_parser::Vdf;
use keyvalues_serde::from_vdf;
use serde::Deserialize;
use std::{borrow::Cow, collections::HashMap, fs, path::PathBuf};

#[derive(Deserialize)]
struct SteamLibraryFolders {
    libraries: Vec<Library>,
}

#[derive(Deserialize)]
struct Library {
    path: PathBuf,
    apps: HashMap<u64, u64>,
}

pub struct Steam {
    library_folders: Option<SteamLibraryFolders>,
    pub path: PathBuf,
}

impl Steam {
    pub fn from_config() -> Result<Self, Box<dyn std::error::Error>> {
        let steam_path = get_steam_path()?;

        Ok(Steam {
            library_folders: Self::list_library_folders(&steam_path).ok(),
            path: PathBuf::from(steam_path),
        })
    }

    pub fn find_app(self: &Self, app_id: u64) -> Option<&PathBuf> {
        match &self.library_folders {
            Some(library_folders) => {
                for lib in &library_folders.libraries {
                    let apps = &lib.apps;
                    if apps.into_iter().any(|g| *g.0 == app_id) {
                        return Some(&lib.path);
                    }
                }
                None
            }
            None => None,
        }
    }

    fn list_library_folders(
        steam_path: &String,
    ) -> Result<SteamLibraryFolders, Box<dyn std::error::Error>> {
        let asset_path = format!("{}\\config\\libraryfolders.vdf", steam_path);
        let vdf_text = fs::read_to_string(asset_path)?;
        let mut vdf = Vdf::parse(&vdf_text)?;
        let obj = vdf.value.get_mut_obj().unwrap();

        // Switch all the entries with keys that are an index (0, 1, ...) to `"libraries"`
        let mut index = 0;
        while let Some(mut library) = obj.remove(index.to_string().as_str()) {
            obj.entry(Cow::from("libraries"))
                .or_insert(Vec::new())
                .push(library.pop().unwrap());

            index += 1;
        }

        Ok(from_vdf(vdf)?)
    }
}

#[cfg(windows)]
pub fn get_steam_path() -> Result<String, crate::regedit::Error> {
    let location = RegLocation::Machine;
    let path = "Software\\Valve\\Steam";
    let key = "InstallPath";
    let value = reg_value_str(RegTarget::None, location, path, key)
        .or_else(|_| reg_value_str(RegTarget::Wow32, location, path, key))
        .or_else(|_| reg_value_str(RegTarget::Wow64, location, path, key))?;

    Ok(value)
}

#[cfg(unix)]
pub fn get_steam_path() -> Result<String, std::env::VarError> {
    let home = std::env::var("HOME")?;
    Ok(format!("{}/.steam/steam", home))
}

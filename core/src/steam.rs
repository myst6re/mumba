#[cfg(windows)]
use crate::os::regedit::*;
#[cfg(feature = "steam")]
use keyvalues_parser::Vdf;
#[cfg(feature = "steam")]
use keyvalues_serde::from_vdf;
#[cfg(feature = "steam")]
use serde::Deserialize;
#[cfg(feature = "steam")]
use std::path::Path;
use std::path::PathBuf;
#[cfg(feature = "steam")]
use std::{borrow::Cow, collections::HashMap, fs};

#[cfg(feature = "steam")]
#[derive(Deserialize, Debug)]
struct SteamLibraryFolders {
    libraries: Vec<Library>,
}

#[cfg(feature = "steam")]
#[derive(Deserialize, Debug)]
struct Library {
    path: PathBuf,
    apps: HashMap<u64, u64>,
}

pub struct Steam {
    #[cfg(feature = "steam")]
    library_folders: Option<SteamLibraryFolders>,
    pub path: PathBuf,
}

impl Steam {
    pub fn from_config() -> Result<Self, Box<dyn std::error::Error>> {
        let path = get_steam_path()?;

        Ok(Steam {
            #[cfg(feature = "steam")]
            library_folders: Self::list_library_folders(&path).ok(),
            path,
        })
    }

    #[allow(unused_variables)]
    pub fn find_app(&self, app_id: u64, app_name: &'static str) -> Option<PathBuf> {
        let steam_path = if cfg!(feature = "steam") {
            #[cfg(feature = "steam")]
            match &self.library_folders {
                Some(library_folders) => {
                    Self::find_app_in_library_folders(library_folders, app_id).unwrap_or(&self.path)
                }
                None => &self.path,
            }
            #[cfg(not(feature = "steam"))]
            &self.path
        } else {
            &self.path
        };
        let app_path = steam_path.join("steamapps").join("common").join(app_name);
        if app_path.exists() {
            Some(app_path)
        } else {
            None
        }
    }

    #[cfg(feature = "steam")]
    fn find_app_in_library_folders(
        library_folders: &SteamLibraryFolders,
        app_id: u64,
    ) -> Option<&PathBuf> {
        for lib in &library_folders.libraries {
            let apps = &lib.apps;
            if apps.iter().any(|g| *g.0 == app_id) {
                return Some(&lib.path);
            }
        }
        None
    }

    #[cfg(feature = "steam")]
    fn list_library_folders(
        steam_path: &Path,
    ) -> Result<SteamLibraryFolders, Box<dyn std::error::Error>> {
        let asset_path = steam_path
            .to_path_buf()
            .join("config")
            .join("libraryfolders.vdf");
        info!("list_library_folders {:?}", asset_path);
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

        let res = from_vdf(vdf)?;

        info!("Found Libaries: {:?}", res);

        Ok(res)
    }
}

#[cfg(windows)]
pub fn get_steam_path() -> Result<PathBuf, crate::os::regedit::Error> {
    let location = RegLocation::Machine;
    let path = "Software\\Valve\\Steam";
    let key = "InstallPath";
    let value = reg_value_str(RegTarget::None, location, path, key)
        .or_else(|_| reg_value_str(RegTarget::Wow32, location, path, key))
        .or_else(|_| reg_value_str(RegTarget::Wow64, location, path, key))?;

    Ok(PathBuf::from(value))
}

#[cfg(unix)]
pub fn get_steam_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    Ok(std::fs::canonicalize(format!("{}/.steam/steam", home))?)
}

#[cfg(windows)]
pub fn get_steam_exe() -> Result<PathBuf, crate::os::regedit::Error> {
    Ok(get_steam_path()?.join("steam.exe"))
}

#[cfg(unix)]
pub fn get_steam_exe() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(get_steam_path()?.join("steam"))
}

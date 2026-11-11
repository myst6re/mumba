#[cfg(windows)]
use crate::os::regedit::*;
use crate::os::run_helper;
#[cfg(feature = "steam")]
use keyvalues_parser;
#[cfg(feature = "steam")]
use keyvalues_serde::from_vdf;
#[cfg(feature = "steam")]
use serde::Deserialize;
#[cfg(feature = "steam")]
use std::path::Path;
use std::path::PathBuf;
use std::process::{Child, Command};
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
    path: PathBuf,
    exe: PathBuf,
}

impl Steam {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::get_steam_path()?;
        let exe = Self::get_steam_exe(&path)?;

        Ok(Steam {
            #[cfg(feature = "steam")]
            library_folders: None,
            path,
            exe,
        })
    }

    pub fn from_config() -> Result<Self, Box<dyn std::error::Error>> {
        if cfg!(feature = "steam") {
            let mut steam = Self::new()?;
            steam.library_folders = Self::list_library_folders(&steam.path).ok();
            Ok(steam)
        } else {
            Self::new()
        }
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
        info!("list_library_folders \"{}\"", asset_path.to_string_lossy());
        let vdf_text = fs::read_to_string(asset_path)?;
        let mut vdf = keyvalues_parser::parse(&vdf_text)?.into_vdf();
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

    pub fn launch_app(
        &self,
        app_id: u64,
        current_dir: &Path,
        arg1: &Path,
    ) -> Result<Child, std::io::Error> {
        info!(
            "Launch \"{} -applaunch {} '{}'\" in dir \"{}\"...",
            self.exe.to_string_lossy(),
            app_id,
            arg1.to_string_lossy(),
            current_dir.to_string_lossy()
        );
        run_helper(&mut Command::new(&self.exe))
            .args(["-applaunch", app_id.to_string().as_str()])
            .arg(arg1.as_os_str())
            .current_dir(current_dir)
            .spawn()
    }

    #[cfg(windows)]
    fn get_steam_path() -> Result<PathBuf, crate::os::regedit::Error> {
        let location = RegLocation::Machine;
        let path = "Software\\Valve\\Steam";
        let key = "InstallPath";
        let value = reg_value_str(RegTarget::None, location, path, key)
            .or_else(|_| reg_value_str(RegTarget::Wow32, location, path, key))
            .or_else(|_| reg_value_str(RegTarget::Wow64, location, path, key))?;

        Ok(PathBuf::from(value))
    }

    #[cfg(unix)]
    fn get_steam_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let home = std::env::var("HOME")?;
        Ok(std::fs::canonicalize(format!("{}/.steam/steam", home))?)
    }

    #[cfg(windows)]
    fn get_steam_exe(steam_path: &Path) -> Result<PathBuf, crate::os::regedit::Error> {
        Ok(steam_path.join("steam.exe"))
    }

    #[cfg(unix)]
    fn get_steam_exe(_steam_path: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(PathBuf::from("steam"))
    }
}

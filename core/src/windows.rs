use windows::Win32::UI::Shell;
use windows::Win32::Foundation::HANDLE;
use std::path::PathBuf;

pub fn saved_games_path() -> String {
    let path = unsafe {
        Shell::SHGetKnownFolderPath(&Shell::FOLDERID_SavedGames, Shell::KF_FLAG_DEFAULT, HANDLE::default()).map(|e| e.to_string().unwrap_or_default()).unwrap_or_default()
    };
    if path.is_empty() {
        let dirs = directories::UserDirs::new();
        String::from(dirs.map_or(PathBuf::new(), |d| d.document_dir().unwrap().to_path_buf()).join("My Games").to_str().unwrap())
    } else {
        path
    }
}

use windows::Win32::UI::Shell;
use windows::Win32::Foundation::{HANDLE, HWND, WIN32_ERROR, self};
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ffi::OsStr;

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

pub fn run_as(program: &String, parameters: &String) -> Result<u32, std::io::Error> {
    let hinstance = unsafe {
        let hwnd: HWND = std::mem::zeroed();
        windows::Win32::UI::Shell::ShellExecuteW(
            hwnd,
            windows::core::PCWSTR::from_raw(OsStr::new("runas\0").encode_wide().collect::<Vec<_>>().as_ptr()),
            windows::core::PCWSTR::from_raw(OsStr::new(program).encode_wide().chain(Some(0)).collect::<Vec<_>>().as_ptr()),
            windows::core::PCWSTR::from_raw(OsStr::new(parameters).encode_wide().chain(Some(0)).collect::<Vec<_>>().as_ptr()),
            windows::core::PCWSTR::null(),
            windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
        )
    };
    match WIN32_ERROR(hinstance.0 as u32) {
        WIN32_ERROR(0u32) => Err(std::io::Error::new(std::io::ErrorKind::OutOfMemory, "Out of memory")),
        Foundation::ERROR_FILE_NOT_FOUND => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found")),
        Foundation::ERROR_PATH_NOT_FOUND => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Path not found")),
        Foundation::ERROR_BAD_FORMAT => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Exe is invalid")),
        WIN32_ERROR(Shell::SE_ERR_ACCESSDENIED) => Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied")),
        WIN32_ERROR(Shell::SE_ERR_ASSOCINCOMPLETE) => Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "File name association incomplete")),
        WIN32_ERROR(Shell::SE_ERR_DDEBUSY) => Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, "DDE busy")),
        WIN32_ERROR(Shell::SE_ERR_DDEFAIL) => Err(std::io::Error::new(std::io::ErrorKind::Other, "DDE transaction fail")),
        WIN32_ERROR(Shell::SE_ERR_DDETIMEOUT) => Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "DDE timeout")),
        WIN32_ERROR(Shell::SE_ERR_DLLNOTFOUND) => Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "DLL not found")),
        WIN32_ERROR(Shell::SE_ERR_NOASSOC) => Err(std::io::Error::new(std::io::ErrorKind::Other, "No association")),
        WIN32_ERROR(Shell::SE_ERR_OOM) => Err(std::io::Error::new(std::io::ErrorKind::OutOfMemory, "Out of memory")),
        WIN32_ERROR(Shell::SE_ERR_SHARE) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Sharing violation")),
        WIN32_ERROR(num) if num > 32 => Ok(num - 32),
        WIN32_ERROR(num) => Err(std::io::Error::new(std::io::ErrorKind::Unsupported, num.to_string()))
    }
}

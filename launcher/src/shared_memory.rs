use log::{error, warn, info, debug};
use windows::Win32::System::Memory::{CreateFileMappingA, MapViewOfFile, PAGE_READWRITE, FILE_MAP_ALL_ACCESS, MEMORY_MAPPED_VIEW_ADDRESS};
use windows::Win32::System::Threading::{CreateSemaphoreA, WaitForSingleObject, ReleaseSemaphore, INFINITE};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows::core::PCSTR;
use std::ffi::{OsString, OsStr};
use std::os::windows::prelude::*;
use std::path::PathBuf;
use windows::Win32::Foundation::{HWND, MAX_PATH, WAIT_OBJECT_0, WAIT_TIMEOUT, WAIT_FAILED, GetLastError};
use windows::Win32::UI::Shell;
use std::process::Child;

pub const GAME_METRICS: u32 = 3;
pub const GAME_READY: u32 = 4;
pub const USER_SAVE_DIR: u32 = 9;
pub const END_USER_INFO: u32 = 24;

pub struct SharedMemory {
    map_view: MEMORY_MAPPED_VIEW_ADDRESS,
    game_can: HANDLE,
    game_did: HANDLE,
    launcher_can: HANDLE,
    launcher_did: HANDLE,
    save_dir: PathBuf,
}

impl SharedMemory {
    pub fn new(is_cw: bool) -> Option<Self> {
        if !is_cw {
            return None // For now we only try to communicate with the game if it's the CW
        }
        let save_dir = save_path_2013();

        if save_dir.is_none() {
            return None
        }

        match Self::create_shared_memory(is_cw) {
            (Some(map_view), Some(game_can), Some(game_did), Some(launcher_can), Some(launcher_did)) => {
                Some(SharedMemory {
                    map_view,
                    game_can,
                    game_did,
                    launcher_can,
                    launcher_did,
                    save_dir: save_dir.unwrap()
                })
            },
            (_, _, _, _, _) => None
        }
    }

    fn create_semaphore(key: String) -> Option<HANDLE> {
        unsafe {
            match CreateSemaphoreA(
                None,
                0,
                1,
                PCSTR::from_raw(key.as_ptr())
            ) {
                Ok(handle) => Some(handle),
                Err(e) => {
                    error!("Cannot create semaphore {}: {}", key, e);
                    None
                }
            }
        }
    }

    fn create_shared_memory(is_cw: bool) -> (Option<MEMORY_MAPPED_VIEW_ADDRESS>, Option<HANDLE>, Option<HANDLE>, Option<HANDLE>, Option<HANDLE>) {
        let key = if is_cw { "choco" } else { "ff8" };

        info!("Create file mapping {}", key);

        let map_view_of_file = unsafe {
            match CreateFileMappingA(
                INVALID_HANDLE_VALUE,
                None,
                PAGE_READWRITE,
                0,
                0x20000,
                PCSTR::from_raw(format!("{}_sharedMemoryWithLauncher\0", key).as_ptr())
            ) {
                Ok(mapping) => {
                    Some(MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, 0))
                },
                Err(e) => {
                    error!("Cannot create file mapping: {}", e);
                    None
                }
            }
        };

        (
            map_view_of_file,
            Self::create_semaphore(format!("{}_gameCanReadMsgSem\0", key)),
            Self::create_semaphore(format!("{}_gameDidReadMsgSem\0", key)),
            Self::create_semaphore(format!("{}_launcherCanReadMsgSem\0", key)),
            Self::create_semaphore(format!("{}_launcherDidReadMsgSem\0", key))
        )
    }

    fn read_command_from_game(&self, duration_ms: u32) -> Option<u32> {
        unsafe {
            match WaitForSingleObject(self.launcher_can, duration_ms) {
                WAIT_OBJECT_0 => (),
                WAIT_TIMEOUT => return None,
                WAIT_FAILED => {
                    error!("Error when waiting for launcher can semaphore: #{}", GetLastError().0);
                    return None
                },
                e => {
                    error!("Error when waiting for launcher can semaphore: Unknown {}", e.0);
                    return None
                }
            }
            let data = self.map_view.Value as *const u32;
            let command = *data;
            info!("Received command: {}", command);
            let _ = ReleaseSemaphore(self.launcher_did, 1, None);
            debug!("launcher_did released");

            Some(command)
        }
    }

    fn send_command_to_game(&self, command: u32, param: Option<&OsStr>) {
        unsafe {
            let data = self.map_view.Value.byte_add(0x10000) as *mut u32;
            let data_param = data.byte_add(4);

            *data = command;

            match param {
                Some(str) => {
                    let param: Vec<u16> = str.encode_wide().collect();
                    *data = param.len() as u32;
                    std::ptr::copy_nonoverlapping(param.as_ptr(), data_param as *mut u16, param.len());
                    info!("Send command {} with param {}", command, str.to_string_lossy())
                },
                None => info!("Send command {}", command)
            }

            let _ = ReleaseSemaphore(self.game_can, 1, None);
            debug!("game_can released");
            let _ = WaitForSingleObject(self.game_did, INFINITE);
            debug!("game_did awaited");
        }
    }

    pub fn wait(&self, child: &mut Child) {
        if self.read_command_from_game(5000) != Some(GAME_READY) {
            warn!("The game did not send the GAME_READY command on time");
            return
        }

        let dir = self.save_dir.clone();
        self.send_command_to_game(USER_SAVE_DIR, Some(&dir.as_os_str()));
        self.send_command_to_game(END_USER_INFO, None);

        loop {
            if !matches!(child.try_wait(), Ok(None)) {
                return
            }
            self.read_command_from_game(700);
        }
    }
}

fn my_documents_path() -> PathBuf {
    let mut path = [0u16; MAX_PATH as usize];
    unsafe {
        // Steam 2013 version uses this obsolete implementation instead of SHGetKnownFolderPath
        Shell::SHGetFolderPathW(
            HWND::default(),
            (Shell::CSIDL_MYDOCUMENTS | Shell::CSIDL_FLAG_CREATE) as i32,
            HANDLE::default(),
            0,
            &mut path,
        )
        .unwrap_or_default()
    };
    let len = path.iter().position(|e| *e == 0).unwrap_or(0);
    PathBuf::from(OsString::from_wide(&path[0..len]))
}

pub fn save_path_2013() -> Option<PathBuf> {
    let steam_path_2013 = my_documents_path().join("Square Enix\\FINAL FANTASY VIII Steam");

    find_user_id(steam_path_2013)
}

fn find_user_id(steam_path_2013: PathBuf) -> Option<PathBuf> {
    match steam_path_2013.read_dir() {
        Ok(it) => {
            for entry in it {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_dir() && path.file_name().unwrap().to_string_lossy().starts_with("user_") {
                            return Some(path)
                        }

                    },
                    _ => break
                }
            }
            None
        },
        Err(_) => None
    }
}

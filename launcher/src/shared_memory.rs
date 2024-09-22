use log::{error, info};
use windows::Win32::System::Memory::{CreateFileMappingA, MapViewOfFile, PAGE_READWRITE, FILE_MAP_ALL_ACCESS, MEMORY_MAPPED_VIEW_ADDRESS};
use windows::Win32::System::Threading::{CreateSemaphoreA, WaitForSingleObject, ReleaseSemaphore};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows::core::PCSTR;
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::path::PathBuf;
use windows::Win32::Foundation::{HWND, MAX_PATH};
use windows::Win32::UI::Shell;

struct SharedMemory {
    map_view: MEMORY_MAPPED_VIEW_ADDRESS,
    game_can: HANDLE,
    game_did: HANDLE,
    launcher_can: HANDLE,
    launcher_did: HANDLE,
    save_dir: PathBuf,
}

impl SharedMemory {
    pub fn new(is_cw: bool) -> Option<Self> {
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

    pub fn wait_shared_memory(&self) {
        /* if let Some(launcher_can) = launcher_can {
            unsafe {
                let _ = WaitForSingleObject(launcher_can, 5000);
            }
        }

        info!("launcher_can awaited");

        if let Some(shared_memory) = shared_memory {
            unsafe {
                let data = shared_memory.Value as *const u32;
                let command = *data;
                info!("Received command: {}", command)
            }
        }

        if let Some(launcher_did) = launcher_did {
            unsafe {
                let _ = ReleaseSemaphore(launcher_did, 1, None);
            }
        }

        info!("launcher_did released"); */

        unsafe {
            /*
            let _ = WaitForSingleObject(launcher_can, 5000);
            let data = shared_memory.Value as *const u32;
            let command = *data;
            info!("Received command: {}", command)
             */
            let data = self.map_view.Value.byte_add(0x10000) as *mut u32;
            *data = 9;
            let param = data.byte_add(4);
            let dir = self.save_dir.clone();
            let len = dir.to_string_lossy().len();
            info!("Dir={} size={}", dir.to_string_lossy(), len);
            let dir: Vec<u16> = dir.as_os_str().encode_wide().collect();
            info!("len={}", dir.len());
            *param = dir.len() as u32;
            let src_ptr = dir.as_ptr();
            std::ptr::copy_nonoverlapping(src_ptr, param.byte_add(4) as *mut u16, dir.len());
            let _ = ReleaseSemaphore(self.game_can, 1, None);
            info!("game_can released");
            let _ = WaitForSingleObject(self.game_did, 5000);
            info!("game_did awaited");
            let data = self.map_view.Value.byte_add(0x10000) as *mut u32;
            *data = 24;
            let _ = ReleaseSemaphore(self.game_can, 1, None);
            info!("game_can released");
            let _ = WaitForSingleObject(self.game_did, 5000);
            info!("game_did awaited");
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

    steam_path_2013.read_dir();

    my_documents.join("Square Enix\\FINAL FANTASY VIII Steam\\user_199178")
}

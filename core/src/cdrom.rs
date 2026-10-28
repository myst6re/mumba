#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::Storage::FileSystem;

pub struct CDRom {
    pub path: PathBuf,
}

impl CDRom {
    #[cfg(windows)]
    pub fn list() -> Vec<CDRom> {
        let mut available_drive_bitmap = unsafe { FileSystem::GetLogicalDrives() };

        let mut ret: Vec<CDRom> = vec![];
        let mut letter = 'A';
        while available_drive_bitmap != 0 {
            if (available_drive_bitmap & 1) != 0 {
                let path = format!("{}:\\\0", letter);
                let path_w =
                    PCWSTR::from_raw(OsStr::new(&path).encode_wide().collect::<Vec<_>>().as_ptr());
                let drive_type = unsafe { FileSystem::GetDriveTypeW(path_w) };
                if drive_type == windows::Win32::System::WindowsProgramming::DRIVE_CDROM {
                    ret.push(CDRom {
                        path: PathBuf::from(format!("{}:\\", letter)),
                    })
                }
            }

            letter = char::from_u32((letter as u32) + 1).unwrap();
            available_drive_bitmap >>= 1;
        }

        ret
    }

    #[cfg(unix)]
    pub fn list() -> Vec<CDRom> {
        vec![]
    }
}

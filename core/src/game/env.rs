use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

/* Windows:
 *   %binary_dir%\
 *   ├── mumba.exe
 *   ├── mmb.exe
 *   ├── resources\
 *   │   ├── ff8_launcher.exe
 *   │   └── eax.dll
 *   ├── lang\
 *   │   └── mumba.en.ftl
 *   └── License.rtf
 *   C:\Users\Alice\AppData\Roaming\mumba\
 *   ├── mumba.toml
 *   ├── game\
 *   │   └── ffnx + ff8 exe
 *   ├── mods\
 *   │   └── ffnx + ff8 exe
 *   └── mumba.log
 * Linux:
 *   /usr/bin/
 *   ├── mumba
 *   └── mmb
 *   /var/lib/mumba/
 *   ├── resources/
 *   │   ├── ff8_launcher.exe
 *   │   └── eax.dll
 *   └── lang/
 *       └── mumba.en.ftl
 *   /home/alice/.config/mumba/
 *   └── mumba.toml
 *   /home/alice/.local/share/mumba/
 *   ├── game\
 *   │   └── ffnx + ff8 exe
 *   ├── mods\
 *   │   └── ffnx + ff8 exe
 *   └── mumba.log
 * Portable:
 *   ├── mumba.exe
 *   ├── mmb.exe
 *   ├── resources\
 *   │   ├── ff8_launcher.exe
 *   │   └── eax.dll
 *   ├── lang\
 *   │   └── mumba.en.ftl
 *   ├── License.rtf
 *   ├── mumba.toml
 *   ├── game\
 *   │   └── ffnx + ff8 exe
 *   ├── mods\
 *   │   └── ffnx + ff8 exe
 *   └── mumba.log
 */

pub struct Env {
    program_name: String,
    project_dirs: Option<ProjectDirs>,
    // Where mumba binary is
    pub binary_dir: PathBuf,
    // Windows: C:\Users\Alice\AppData\Local\mumba, Linux: /home/alice/.cache/mumba, macOS: /Users/Alice/Library/Caches/mumba
    pub cache_dir: PathBuf,
    // Windows: C:\Users\Alice\AppData\Roaming\mumba, Linux: /home/alice/.config/mumba, macOS: /Users/Alice/Library/Application Support/mumba
    pub config_path: PathBuf,
    // Windows: C:\Users\Alice\AppData\Roaming\mumba\game, Linux: /home/alice/.local/share/mumba/game, macOS: /Users/Alice/Library/Application Support/mumba/game
    pub ffnx_dir: PathBuf,
    // Windows: C:\Users\Alice\AppData\Roaming\mumba\mods
    pub mods_dir: PathBuf,
    // Windows: C:\Users\Alice\AppData\Roaming\mumba\mumba.log
    pub log_path: PathBuf,
}

impl Env {
    pub fn new(program_name: &str) -> Result<Self, std::io::Error> {
        let mut exe_path = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from(format!("./{}", program_name)));
        exe_path.pop(); // Remove exe filename
        let local_env = Self {
            program_name: String::from(program_name),
            project_dirs: None,
            binary_dir: exe_path.clone(),
            cache_dir: exe_path.clone(),
            config_path: exe_path.clone().join(format!("{}.toml", program_name)),
            ffnx_dir: exe_path.join("game"),
            mods_dir: exe_path.join("mods"),
            log_path: exe_path.join(format!("{}.log", program_name)),
        };

        if local_env.config_path.exists() {
            // Local installation
            return Ok(local_env);
        }

        let project_dirs = directories::ProjectDirs::from("", "", program_name);
        let (cache_dir, data_dir, config_dir) = match &project_dirs {
            Some(d) => (
                d.cache_dir().to_path_buf(),
                d.data_dir().to_path_buf(),
                d.config_dir().to_path_buf(),
            ),
            None => (exe_path.clone(), exe_path.clone(), exe_path.clone()),
        };
        let cache_dir = Self::create_dir(&cache_dir)
            .and(Ok(cache_dir))
            .or_else(|_| Self::create_dir(&local_env.cache_dir).and(Ok(local_env.cache_dir)))?;
        let data_dir = Self::create_dir(&data_dir).and(Ok(data_dir)).or_else(|_| {
            Self::create_dir(&local_env.binary_dir).and(Ok(local_env.binary_dir.clone()))
        })?;
        let config_dir = Self::create_dir(&config_dir)
            .and(Ok(config_dir))
            .or_else(|_| Self::create_dir(&local_env.binary_dir).and(Ok(local_env.binary_dir)))?;

        Ok(Self {
            program_name: local_env.program_name,
            project_dirs,
            cache_dir,
            ffnx_dir: data_dir.join("game"),
            mods_dir: data_dir.join("mods"),
            config_path: config_dir.join(format!("{}.toml", program_name)),
            binary_dir: exe_path,
            log_path: data_dir.join(format!("{}.log", program_name)),
        })
    }

    fn find_data<P: AsRef<Path>>(&self, dirname: &str, name: P) -> PathBuf {
        let path = PathBuf::from(&dirname).join(&name);
        if path.exists() {
            return path;
        }
        info!("File not found at {}", path.to_string_lossy());
        let path = self.binary_dir.join(dirname).join(&name);
        if path.exists() {
            return path;
        }
        info!("File not found at {}", path.to_string_lossy());
        if let Some(project_dirs) = &self.project_dirs {
            let path = project_dirs.data_dir().join(dirname).join(&name);
            if path.exists() {
                return path;
            }
            info!("File not found at {}", path.to_string_lossy());
        }
        #[cfg(unix)]
        let path = PathBuf::from("/usr/share")
            .join(&self.program_name)
            .join(&dirname)
            .join(&name);
        #[cfg(unix)]
        if path.exists() {
            return path;
        }
        #[cfg(unix)]
        info!("File not found at {}", path.to_string_lossy());
        PathBuf::new().join(name)
    }

    pub fn find_resource(&self, name: &str) -> PathBuf {
        self.find_data("resources", name)
    }

    pub fn find_lang(&self, lang: &str) -> PathBuf {
        let name = format!("{}.{}.ftl", self.program_name, lang);
        self.find_data("lang", name)
    }

    fn create_dir<P: AsRef<std::path::Path>>(path: P) -> io::Result<()> {
        match std::fs::create_dir_all(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Ok(()),
            Err(e) => Err(e),
        }
    }
}

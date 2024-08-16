use moomba_core::game::env::Env;
use moomba_core::game::ffnx_config::FfnxConfig;
use moomba_core::toml::FileError;
use std::path::PathBuf;

pub struct LazyFfnxConfig {
    config: Option<FfnxConfig>,
    config_path: PathBuf,
}

impl LazyFfnxConfig {
    pub fn new(env: &Env) -> Self {
        Self {
            config: None,
            config_path: env.ffnx_dir.join("FFNx.toml"),
        }
    }

    pub fn get(&mut self) -> &mut FfnxConfig {
        if self.config.is_none() {
            self.config = Some(FfnxConfig::from_file(&self.config_path).unwrap_or_default())
        }

        self.config.as_mut().unwrap()
    }

    pub fn save(&mut self) -> Result<(), FileError> {
        if let Some(config) = &self.config {
            config.save(&self.config_path)?
        }
        Ok(self.clear())
    }

    pub fn clear(&mut self) {
        self.config = None
    }
}

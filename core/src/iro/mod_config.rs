use crate::iro::mod_xml::ModInfo;
use crate::toml;
use std::path::Path;
use toml_edit::DocumentMut;

pub struct ModConfig {
    inner: DocumentMut,
}

impl ModConfig {
    pub fn new() -> Self {
        Self {
            inner: DocumentMut::new(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, toml::FileError> {
        Ok(Self {
            inner: toml::parse_from_file(path)?,
        })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), toml::FileError> {
        toml::save_to_file(&self.inner, path)
    }

    pub fn root(&self) -> &toml_edit::Table {
        self.inner.as_table()
    }

    pub fn set_info_config(&mut self, mod_info: &ModInfo) {
        for opt in mod_info.config_option.iter() {
            let key = opt.id.as_str();
            if !self.inner.contains_key(key) {
                self.inner[key] = toml_edit::Item::Value(opt.default.into())
            }
        }
    }
}

impl Default for ModConfig {
    fn default() -> Self {
        Self::new()
    }
}

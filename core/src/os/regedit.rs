use registry::{Data, Hive, RegKey, Security};
use thiserror::Error;
use utfx::U16CString;

#[derive(Copy, Clone)]
pub enum RegLocation {
    Machine,
    User,
}

#[derive(Copy, Clone)]
pub enum RegTarget {
    None,
    Wow32,
    Wow64,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Registry Key not found: {0}")]
    KeyNotFound(String),
    #[error("Registry Value not found: {0}")]
    ValueNotFound(String),
    #[error("Registry Key is not a string")]
    NotAString,
    #[error("Registry Unknown Error")]
    OtherError,
}

pub fn reg_search_installed_app_by_key(path: &'static str) -> Option<String> {
    let targets = [RegTarget::None, RegTarget::Wow64, RegTarget::Wow32];
    let locations = [RegLocation::Machine, RegLocation::User];
    for target in targets {
        for loc in locations {
            match reg_value_str(target, loc, path, r"InstallLocation") {
                Ok(path) => return Some(path),
                Err(_) => continue,
            }
        }
    }
    None
}

fn reg_open<Q>(target: RegTarget, loc: RegLocation, path: Q) -> Result<RegKey, registry::key::Error>
where
    Q: TryInto<U16CString>,
    Q::Error: Into<registry::key::Error>,
    Q::Error: Into<registry::value::Error>,
{
    let hive = match loc {
        RegLocation::Machine => Hive::LocalMachine,
        RegLocation::User => Hive::CurrentUser,
    };

    // Force target 32 or 64-bit
    let sec = Security::Read
        | match target {
            RegTarget::None => Security::empty(),
            RegTarget::Wow32 => Security::Wow6432Key,
            RegTarget::Wow64 => Security::Wow6464Key,
        };

    hive.open(path, sec)
}

pub fn reg_value_str<Q>(
    target: RegTarget,
    loc: RegLocation,
    path: Q,
    value_name: Q,
) -> Result<String, Error>
where
    Q: TryInto<U16CString>,
    Q::Error: Into<registry::key::Error>,
    Q::Error: Into<registry::value::Error>,
{
    match reg_open(target, loc, path) {
        Ok(reg_key) => match reg_key.value(value_name) {
            Ok(Data::String(s)) => Ok(s.to_string_lossy()),
            Ok(_) => Err(Error::NotAString),
            Err(registry::value::Error::NotFound(val, _)) => Err(Error::ValueNotFound(val)),
            Err(e) => {
                warn!("Get regedit value error: {:?}", e);
                Err(Error::OtherError)
            }
        },
        Err(registry::key::Error::NotFound(key, _)) => Err(Error::KeyNotFound(key)),
        Err(e) => {
            warn!("Get regedit key error: {:?}", e);
            Err(Error::OtherError)
        }
    }
}

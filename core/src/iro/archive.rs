use crate::iro::mod_xml::{deserialize_mod_xml, ModInfo};
use iroga::error::Error;
use iroga::iro_archive::IroArchive;
use std::io::{BufWriter, Cursor};
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum IroError {
    #[error(transparent)]
    IroError(#[from] Error),
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    IntoInnerErrorBufWriterU8(#[from] ::std::io::IntoInnerError<std::io::BufWriter<Vec<u8>>>),
    #[error(transparent)]
    DeError(#[from] quick_xml::de::DeError),
}

fn parse_utf16(bytes: &[u8]) -> Result<String, Error> {
    let bytes_u16 = bytes
        .chunks(2)
        .map(|e| e.try_into().map(u16::from_le_bytes))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| Error::InvalidUtf16("uneven bytes".to_owned()))?;

    String::from_utf16(&bytes_u16)
        .map_err(|_| Error::InvalidUtf16("bytes in u16 cannot be converted to string".to_owned()))
}

pub fn unpack_mod_xml<P: AsRef<Path>>(iro_path: P) -> Result<Option<ModInfo>, IroError> {
    let iro_file = std::fs::File::open(iro_path)?;
    let mut iro_archive = IroArchive::open(iro_file);
    let iro_header = iro_archive.read_header()?;
    let iro_entries = iro_archive.read_iro_entries(&iro_header)?;

    for iro_entry in iro_entries {
        let iro_entry_path = parse_utf16(&iro_entry.path)?;

        if iro_entry_path == "mod.xml" {
            let mut buf_writer = BufWriter::new(Vec::new());
            iro_archive.seek_and_read_file_entry(&iro_entry, &mut buf_writer)?;

            let bytes = buf_writer.into_inner()?;
            let mod_info = deserialize_mod_xml(Cursor::new(&bytes))?;
            return Ok(Some(mod_info));
        }
    }

    Ok(None)
}

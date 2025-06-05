use crate::iro::mod_xml::ModInfo;
use iroga::error::Error;
use iroga::iro_archive::{IroArchive as IrogaArchive, IroEntry};
use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::path::Path;

pub struct IroArchive {
    inner: IrogaArchive<File>,
    entries: Vec<IroEntry>,
}

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

impl IroArchive {
    pub fn from_path<P: AsRef<Path>>(iro_path: P) -> Result<IroArchive, IroError> {
        let iro_file = File::open(iro_path)?;
        let mut iro_archive = IrogaArchive::open(iro_file);
        let iro_header = iro_archive.read_header()?;
        let iro_entries = iro_archive.read_iro_entries(&iro_header)?;

        Ok(IroArchive {
            inner: iro_archive,
            entries: iro_entries,
        })
    }

    fn parse_utf16(bytes: &[u8]) -> Result<String, Error> {
        let bytes_u16 = bytes
            .chunks(2)
            .map(|e| e.try_into().map(u16::from_le_bytes))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| Error::InvalidUtf16("uneven bytes".to_owned()))?;

        String::from_utf16(&bytes_u16).map_err(|_| {
            Error::InvalidUtf16("bytes in u16 cannot be converted to string".to_owned())
        })
    }

    pub fn unpack_mod_xml(&mut self) -> Result<Option<ModInfo>, IroError> {
        if let Ok(Some(iro_entry)) = Self::search_file(&self.entries, "mod.xml") {
            let mut buf_writer = BufWriter::new(Vec::new());
            self.inner
                .seek_and_read_file_entry(iro_entry, &mut buf_writer)?;

            let bytes = buf_writer.into_inner()?;
            let mod_info = ModInfo::from_reader(Cursor::new(&bytes))?;
            return Ok(Some(mod_info));
        }

        Ok(None)
    }

    pub fn unpack_all(&mut self, output_path: &Path) -> Result<(), IroError> {
        for iro_entry in &self.entries {
            let iro_entry_path = Self::parse_utf16(&iro_entry.path)?;

            let entry_path = output_path.join(&iro_entry_path);
            std::fs::create_dir_all(
                entry_path
                    .parent()
                    .ok_or(Error::ParentPathDoesNotExist(entry_path.clone()))?,
            )?;

            let mut entry_file = std::fs::File::create(&entry_path).unwrap();

            self.inner
                .seek_and_read_file_entry(iro_entry, &mut entry_file)?;
        }
        Ok(())
    }

    fn search_file<'a>(
        entries: &'a Vec<IroEntry>,
        file_name: &str,
    ) -> Result<Option<&'a IroEntry>, Error> {
        for iro_entry in entries {
            let iro_entry_path = Self::parse_utf16(&iro_entry.path)?;

            if iro_entry_path == file_name {
                return Ok(Some(iro_entry));
            }
        }

        Ok(None)
    }
}

use itertools::Itertools;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Error)]
pub enum EntryError {
    #[error(display = "error reading line in entry file: {}", _0)]
    Line(io::Error),
    #[error(display = "linux field is missing")]
    MissingLinux,
    #[error(display = "title field is missing")]
    MisisngTitle,
    #[error(display = "entry is not a file")]
    NotAFile,
    #[error(display = "entry does not have a file name")]
    NoFilename,
    #[error(display = "initrd was defined without a value")]
    NoValueForInitrd,
    #[error(display = "linux was defined without a value")]
    NoValueForLinux,
    #[error(display = "error opening entry file: {}", _0)]
    Open(io::Error),
    #[error(display = "entry has a file name that is not UTF-8")]
    Utf8Filename,
}

#[derive(Debug, Default, Clone)]
pub struct Entry {
    pub filename: String,
    pub initrd: Option<String>,
    pub linux: String,
    pub options: Vec<String>,
    pub title: String,
}

impl Entry {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, EntryError> {
        let path = path.as_ref();

        if ! path.is_file() {
            return Err(EntryError::NotAFile);
        }

        let file_name = match path.file_stem() {
            Some(file_name) => match file_name.to_str() {
                Some(file_name) => file_name.to_owned(),
                None => return Err(EntryError::Utf8Filename)
            },
            None => return Err(EntryError::NoFilename)
        };

        let file = File::open(path).map_err(EntryError::Open)?;

        let mut entry = Entry::default();
        entry.filename = file_name;

        for line in BufReader::new(file).lines() {
            let line = line.map_err(EntryError::Line)?;
            let mut fields = line.split_whitespace();
            match fields.next() {
                Some("title") => entry.title = fields.join(" "),
                Some("linux") => match fields.next() {
                    Some(value) => entry.linux = value.to_owned(),
                    None => return Err(EntryError::NoValueForLinux)
                },
                Some("initrd") => match fields.next() {
                    Some(value) => entry.initrd = Some(value.into()),
                    None => return Err(EntryError::NoValueForInitrd)
                },
                Some("options") => entry.options = fields.map(String::from).collect(),
                _ => ()
            }
        }

        if entry.title.is_empty() {
            return Err(EntryError::MisisngTitle);
        }

        if entry.linux.is_empty() {
            return Err(EntryError::MissingLinux);
        }

        Ok(entry)
    }
}

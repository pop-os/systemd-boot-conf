use itertools::Itertools;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Error)]
pub enum EntryError {
    #[error("error reading line in entry file")]
    Line(#[source] io::Error),
    #[error("linux field is missing")]
    MissingLinux,
    #[error("title field is missing")]
    MisisngTitle,
    #[error("entry is not a file")]
    NotAFile,
    #[error("entry does not have a file name")]
    NoFilename,
    #[error("initrd was defined without a value")]
    NoValueForInitrd,
    #[error("linux was defined without a value")]
    NoValueForLinux,
    #[error("error opening entry file")]
    Open(#[source] io::Error),
    #[error("entry has a file name that is not UTF-8")]
    Utf8Filename,
}

#[derive(Debug, Default, Clone)]
pub struct Entry {
    pub id: Box<str>,
    pub initrd: Option<Box<str>>,
    pub linux: Box<str>,
    pub options: Vec<Box<str>>,
    pub title: Box<str>,
}

impl Entry {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, EntryError> {
        let path = path.as_ref();

        if !path.is_file() {
            return Err(EntryError::NotAFile);
        }

        let file_name = match path.file_stem() {
            Some(file_name) => match file_name.to_str() {
                Some(file_name) => file_name.to_owned(),
                None => return Err(EntryError::Utf8Filename),
            },
            None => return Err(EntryError::NoFilename),
        };

        let file = File::open(path).map_err(EntryError::Open)?;

        let mut entry = Entry::default();
        entry.id = file_name.into();

        for line in BufReader::new(file).lines() {
            let line = line.map_err(EntryError::Line)?;
            let mut fields = line.split_whitespace();
            match fields.next() {
                Some("title") => entry.title = fields.join(" ").into(),
                Some("linux") => match fields.next() {
                    Some(value) => entry.linux = value.into(),
                    None => return Err(EntryError::NoValueForLinux),
                },
                Some("initrd") => match fields.next() {
                    Some(value) => entry.initrd = Some(value.into()),
                    None => return Err(EntryError::NoValueForInitrd),
                },
                Some("options") => entry.options = fields.map(Box::from).collect(),
                _ => (),
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

    /// Determines if this boot entry is the current boot entry
    ///
    /// # Implementation
    ///
    /// This is determined by a matching the entry's initd and options to `/proc/cmdline`.
    pub fn is_current(&self) -> bool {
        let initrd = self
            .initrd
            .as_ref()
            .map(|x| ["initrd=", &x.replace('/', "\\")].concat());

        let initrd = initrd.as_ref().map(String::as_str);
        let options = self.options.iter().map(Box::as_ref);

        let expected_cmdline = initrd.iter().cloned().chain(options);

        crate::kernel_cmdline()
            .iter()
            .cloned()
            .zip(expected_cmdline)
            .all(|(a, b)| a == b)
    }
}

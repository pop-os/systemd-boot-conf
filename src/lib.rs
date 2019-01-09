//! Rust crate for managing the systemd-boot loader configuration.

#[macro_use]
extern crate err_derive;
extern crate itertools;

pub mod entry;
pub mod loader;

use self::entry::*;
use self::loader::*;

use std::fs;
use std::fs::File;
use std::io::{self, BufWriter};
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "error reading loader enrties directory: {}", _0)]
    EntriesDir(io::Error),
    #[error(display = "error parsing entry at {:?}: {}", path, why)]
    Entry { path: PathBuf, why: EntryError },
    #[error(display = "error writing entry file: {}", _0)]
    EntryWrite(io::Error),
    #[error(display = "error reading entry in loader entries directory: {}", _0)]
    FileEntry(io::Error),
    #[error(display = "error parsing loader conf at {:?}: {}", path, why)]
    Loader { path: PathBuf, why: LoaderError },
    #[error(display = "entry not found in data structure")]
    NotFound
}

#[derive(Debug, Default, Clone)]
pub struct SystemdBootConf {
    pub efi_mount: PathBuf,
    pub entries_path: PathBuf,
    pub loader_path: PathBuf,
    pub entries: Vec<Entry>,
    pub loader_conf: LoaderConf,
}

impl SystemdBootConf {
    pub fn new<P: Into<PathBuf>>(efi_mount: P) -> Result<Self, Error> {
        let efi_mount = efi_mount.into();
        let entries_path = efi_mount.join("loader/entries");
        let loader_path = efi_mount.join("loader/loader.conf");

        let mut manager = Self { efi_mount, entries_path, loader_path, .. Default::default() };
        manager.load_conf()?;
        manager.load_entries()?;

        Ok(manager)
    }

    /// Attempt to load all of the available entries in the system.
    pub fn load_entries(&mut self) -> Result<(), Error> {
        let &mut SystemdBootConf { ref mut entries, ref entries_path, .. } = self;
        let dir_entries = fs::read_dir(entries_path).map_err(Error::EntriesDir)?;

        entries.clear();
        for entry in dir_entries {
            let entry = entry.map_err(Error::FileEntry)?;
            let path = entry.path();

            // Only consider conf files in the directory.
            if ! path.is_file() || path.extension().map_or(true, |ext| ext != "conf") {
                continue
            }

            let entry = Entry::from_path(&path).map_err(move |why| Error::Entry {
                path: path.to_path_buf(),
                why
            })?;

            entries.push(entry);
        }

        Ok(())
    }

    /// Attempt to re-read the loader configuration.
    pub fn load_conf(&mut self) -> Result<(), Error> {
        let &mut SystemdBootConf { ref mut loader_conf, ref loader_path, .. } = self;

        *loader_conf = LoaderConf::from_path(loader_path).map_err(move |why| Error::Loader {
            path: loader_path.to_path_buf(),
            why
        })?;

        Ok(())
    }

    /// Validate that the default entry exists.
    pub fn default_entry_exists(&self) -> DefaultState {
        match self.loader_conf.default {
            Some(ref default) => if self.entry_exists(default) {
                DefaultState::Exists
            } else {
                DefaultState::DoesNotExist
            }
            None => DefaultState::NotDefined
        }
    }

    /// Validates that an entry exists with this name.
    pub fn entry_exists(&self, entry: &str) -> bool {
        self.entries.iter().any(|e| e.filename == entry)
    }

    /// Get the entry that corresponds to the given name.
    pub fn get(&self, entry: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.filename == entry)
    }

    /// Get a mutable entry that corresponds to the given name.
    pub fn get_mut(&mut self, entry: &str) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|e| e.filename == entry)
    }

    /// Overwrite the conf file with stored values.
    pub fn overwrite_loader_conf(&self) -> io::Result<()> {
        Self::try_io(&self.loader_path, |file| {
            if let Some(ref default) = self.loader_conf.default {
                writeln!(file, "default {}", default)?;
            }

            if let Some(timeout) = self.loader_conf.timeout {
                writeln!(file, "timeout {}", timeout)?;
            }

            Ok(())
        })
    }

    /// Overwrite the entry conf for the given entry.
    pub fn overwrite_entry_conf(&self, entry: &str) -> Result<(), Error> {
        let entry = match self.get(entry) {
            Some(entry) => entry,
            None => return Err(Error::NotFound)
        };

        let result = Self::try_io(&self.entries_path.join(format!("{}.conf", entry.filename)), move |file| {
            writeln!(file, "title {}", entry.title)?;
            writeln!(file, "linux {}", entry.linux)?;

            if let Some(ref initrd) = entry.initrd {
                writeln!(file, "initrd {}", initrd)?;
            }

            if !entry.options.is_empty() {
                writeln!(file, "options: {}", entry.options.join(" "))?;
            }

            Ok(())
        });

        result.map_err(Error::EntryWrite)
    }

    fn try_io<F: FnMut(&mut BufWriter<File>) -> io::Result<()>>(path: &Path, mut instructions: F) -> io::Result<()> {
        instructions(&mut BufWriter::new(File::create(path)?))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DefaultState {
    NotDefined,
    Exists,
    DoesNotExist,
}

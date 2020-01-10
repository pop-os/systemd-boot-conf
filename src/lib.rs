//! Rust crate for managing the systemd-boot loader configuration.

#[macro_use]
extern crate thiserror;

pub mod entry;
pub mod loader;

use self::entry::*;
use self::loader::*;

use once_cell::sync::OnceCell;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};

#[derive(Debug, Error)]
pub enum Error {
    #[error("error reading loader enrties directory")]
    EntriesDir(#[source] io::Error),
    #[error("error parsing entry at {:?}", path)]
    Entry { path: PathBuf, source: EntryError },
    #[error("error writing entry file")]
    EntryWrite(#[source] io::Error),
    #[error("error reading entry in loader entries directory")]
    FileEntry(#[source] io::Error),
    #[error("error parsing loader conf at {:?}", path)]
    Loader { path: PathBuf, source: LoaderError },
    #[error("error writing loader file")]
    LoaderWrite(#[source] io::Error),
    #[error("entry not found in data structure")]
    NotFound,
}

#[derive(Debug, Clone)]
pub struct SystemdBootConf {
    pub efi_mount: Box<Path>,
    pub entries_path: Box<Path>,
    pub loader_path: Box<Path>,
    pub entries: Vec<Entry>,
    pub loader_conf: LoaderConf,
}

impl SystemdBootConf {
    pub fn new<P: Into<PathBuf>>(efi_mount: P) -> Result<Self, Error> {
        let efi_mount = efi_mount.into();
        let entries_path = efi_mount.join("loader/entries").into();
        let loader_path = efi_mount.join("loader/loader.conf").into();

        let mut manager = Self {
            efi_mount: efi_mount.into(),
            entries_path,
            loader_path,
            entries: Vec::default(),
            loader_conf: LoaderConf::default(),
        };

        manager.load_conf()?;
        manager.load_entries()?;

        Ok(manager)
    }

    /// Find the boot entry which matches the current boot
    ///
    /// # Implementation
    ///
    /// The current boot option is determined by a matching the entry's initd and options
    /// to `/proc/cmdline`.
    pub fn current_entry(&self) -> Option<&Entry> {
        self.entries.iter().find(|e| e.is_current())
    }

    /// Validate that the default entry exists.
    pub fn default_entry_exists(&self) -> DefaultState {
        match self.loader_conf.default {
            Some(ref default) => {
                if self.entry_exists(default) {
                    DefaultState::Exists
                } else {
                    DefaultState::DoesNotExist
                }
            }
            None => DefaultState::NotDefined,
        }
    }

    /// Validates that an entry exists with this name.
    pub fn entry_exists(&self, entry: &str) -> bool {
        self.entries.iter().any(|e| e.id.as_ref() == entry)
    }

    /// Get the entry that corresponds to the given name.
    pub fn get(&self, entry: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id.as_ref() == entry)
    }

    /// Get a mutable entry that corresponds to the given name.
    pub fn get_mut(&mut self, entry: &str) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|e| e.id.as_ref() == entry)
    }

    /// Attempt to re-read the loader configuration.
    pub fn load_conf(&mut self) -> Result<(), Error> {
        let &mut SystemdBootConf {
            ref mut loader_conf,
            ref loader_path,
            ..
        } = self;

        *loader_conf = LoaderConf::from_path(loader_path).map_err(move |source| Error::Loader {
            path: loader_path.to_path_buf(),
            source,
        })?;

        Ok(())
    }

    /// Attempt to load all of the available entries in the system.
    pub fn load_entries(&mut self) -> Result<(), Error> {
        let &mut SystemdBootConf {
            ref mut entries,
            ref entries_path,
            ..
        } = self;
        let dir_entries = fs::read_dir(entries_path).map_err(Error::EntriesDir)?;

        entries.clear();
        for entry in dir_entries {
            let entry = entry.map_err(Error::FileEntry)?;
            let path = entry.path();

            // Only consider conf files in the directory.
            if !path.is_file() || path.extension().map_or(true, |ext| ext != "conf") {
                continue;
            }

            let entry = Entry::from_path(&path).map_err(move |source| Error::Entry {
                path: path.to_path_buf(),
                source,
            })?;

            entries.push(entry);
        }

        Ok(())
    }

    /// Overwrite the conf file with stored values.
    pub fn overwrite_loader_conf(&self) -> Result<(), Error> {
        let result = Self::try_io(&self.loader_path, |file| {
            if let Some(ref default) = self.loader_conf.default {
                writeln!(file, "default {}", default)?;
            }

            if let Some(timeout) = self.loader_conf.timeout {
                writeln!(file, "timeout {}", timeout)?;
            }

            Ok(())
        });

        result.map_err(Error::LoaderWrite)
    }

    /// Overwrite the entry conf for the given entry.
    pub fn overwrite_entry_conf(&self, entry: &str) -> Result<(), Error> {
        let entry = match self.get(entry) {
            Some(entry) => entry,
            None => return Err(Error::NotFound),
        };

        let result = Self::try_io(
            &self.entries_path.join(format!("{}.conf", entry.id)),
            move |file| {
                writeln!(file, "title {}", entry.title)?;
                writeln!(file, "linux {}", entry.linux)?;

                if let Some(ref initrd) = entry.initrd {
                    writeln!(file, "initrd {}", initrd)?;
                }

                if !entry.options.is_empty() {
                    writeln!(file, "options: {}", entry.options.join(" "))?;
                }

                Ok(())
            },
        );

        result.map_err(Error::EntryWrite)
    }

    fn try_io<F: FnMut(&mut BufWriter<File>) -> io::Result<()>>(
        path: &Path,
        mut instructions: F,
    ) -> io::Result<()> {
        instructions(&mut BufWriter::new(File::create(path)?))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DefaultState {
    NotDefined,
    Exists,
    DoesNotExist,
}

/// Fetches the kernel command line, and lazily initialize it if it has not been fetched.
pub fn kernel_cmdline() -> &'static [&'static str] {
    static CMDLINE_BUF: OnceCell<Box<str>> = OnceCell::new();
    static CMDLINE: OnceCell<Box<[&'static str]>> = OnceCell::new();

    CMDLINE.get_or_init(|| {
        let cmdline = CMDLINE_BUF.get_or_init(|| {
            fs::read_to_string("/proc/cmdline")
                .unwrap_or_default()
                .into()
        });

        cmdline
            .split_ascii_whitespace()
            .collect::<Vec<&'static str>>()
            .into()
    })
}

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("error reading line in loader conf")]
    Line(#[source] io::Error),
    #[error("loader conf is not a file")]
    NotAFile,
    #[error("default was defined without a value")]
    NoValueForDefault,
    #[error("timeout was defined without a value")]
    NoValueForTimeout,
    #[error("error opening loader file")]
    Open(#[source] io::Error),
    #[error("timeout was defined with a value ({}) which is not a number", _0)]
    TimeoutNaN(String),
}

#[derive(Debug, Default, Clone)]
pub struct LoaderConf {
    pub default: Option<Box<str>>,
    pub timeout: Option<u32>,
}

impl LoaderConf {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, LoaderError> {
        let path = path.as_ref();

        let mut loader = LoaderConf::default();
        if !path.exists() {
            return Ok(loader);
        }

        if !path.is_file() {
            return Err(LoaderError::NotAFile);
        }

        let file = File::open(path).map_err(LoaderError::Open)?;

        for line in BufReader::new(file).lines() {
            let line = line.map_err(LoaderError::Line)?;
            let mut fields = line.split_whitespace();
            match fields.next() {
                Some("default") => match fields.next() {
                    Some(default) => loader.default = Some(default.into()),
                    None => return Err(LoaderError::NoValueForDefault),
                },
                Some("timeout") => match fields.next() {
                    Some(timeout) => {
                        if let Ok(timeout) = timeout.parse::<u32>() {
                            loader.timeout = Some(timeout);
                        } else {
                            return Err(LoaderError::TimeoutNaN(timeout.into()));
                        }
                    }
                    None => return Err(LoaderError::NoValueForTimeout),
                },
                _ => (),
            }
        }

        Ok(loader)
    }
}

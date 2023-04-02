use std::io;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("{message:?}")]
    General { message: String },
    #[error("I/O Error: {message:?}")]
    IO { message: String },
    #[error("Not implemented: {message:?}")]
    NotImplemented { message: String },
}

impl Error {
    pub fn new<T>(m: T) -> Self
    where
        T: Into<String>,
    {
        Self::General { message: m.into() }
    }
    pub fn new_io_error<T>(m: T) -> Self
    where
        T: Into<String>,
    {
        Self::IO { message: m.into() }
    }
    pub fn not_impl<T>(m: T) -> Self
    where
        T: Into<String>,
    {
        Self::NotImplemented { message: m.into() }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IO {
            message: format!("{}", error),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl<T> ErrorContext<T> for std::result::Result<T, std::str::Utf8Error> {
    fn context(self, message: &str) -> Result<T> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(error!("{}: (utf8error: {})", message, e)),
        }
    }
}

impl<T> ErrorContext<T> for std::result::Result<T, toml::de::Error> {
    fn context(self, message: &str) -> Result<T> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(error!("{}: (toml decoding error: {})", message, e)),
        }
    }
}

impl ErrorContext<()> for std::io::Result<()> {
    fn context(self, message: &str) -> Result<()> {
        match self {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::new_io_error(format!(
                "{}: (io error: {})",
                message, e
            ))),
        }
    }
}

impl ErrorContext<()> for std::fmt::Result {
    fn context(self, message: &str) -> Result<()> {
        match self {
            Ok(_) => Ok(()),
            Err(e) => Err(error!("{}: (fmt error: {})", message, e)),
        }
    }
}

pub trait ErrorContext<T> {
    fn context(self, message: &str) -> Result<T>;
}

macro_rules! error {
    ($($args:expr),+) => {{
        $crate::error::Error::new(format!($($args),+))
    }};
}
pub(crate) use error;
macro_rules! not_impl {
    ($($args:expr),+) => {{
        $crate::error::Error::not_impl(format!($($args),+))
    }};
}
pub(crate) use not_impl;

macro_rules! ensure {
    ($arg:expr) => {{
        if !$arg {
            return Err($crate::error::Error::new(format!(
                "({}) is false",
                stringify!($arg)
            )));
        }
    }};
}
pub(crate) use ensure;

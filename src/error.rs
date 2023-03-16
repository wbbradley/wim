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
macro_rules! not_impl {
    ($($args:expr),+) => {{
        $crate::error::Error::not_impl(format!($($args),+))
    }};
}
pub(crate) use not_impl;

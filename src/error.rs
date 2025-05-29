use std::fmt;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    TmpPersist(tempfile::PersistError),
    Utf8(std::string::FromUtf8Error),
    Fzf(std::process::ExitStatus),
    Custom(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => err.fmt(f),
            Self::TmpPersist(err) => err.fmt(f),
            Self::Utf8(err) => err.fmt(f),
            Self::Fzf(err) => write!(f, "{err}"),
            Self::Custom(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<tempfile::PersistError> for Error {
    fn from(err: tempfile::PersistError) -> Self {
        Self::TmpPersist(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::Utf8(err)
    }
}

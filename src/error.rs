use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    TmpPersist(#[from] tempfile::PersistError),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    Fzf(std::process::ExitStatus),
    #[error(transparent)]
    Clap(#[from] clap::Error),
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),
}

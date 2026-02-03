use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandsError {
    #[error("PATH env var not set")]
    PathNotSet,
    #[error("HOME env var not set")]
    HomeNotSet,
    #[error("Could not obtain current directory")]
    InvalidCurrentDirectory(#[from] io::Error),
}

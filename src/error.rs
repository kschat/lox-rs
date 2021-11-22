use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, LoxError>;

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("Failed to parse.")]
    ParseError,

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

use std::io;

use thiserror::Error;

use crate::token::Token;

pub type Result<T> = std::result::Result<T, LoxError>;

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("Failed to parse.")]
    ParseError,

    #[error("Runtime Error: {message}")]
    RuntimeError { message: String, token: Token },

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

use crate::{stmt::Stmt, token::Token, value::Value};
use std::io;
use thiserror::Error;

pub type Result<T, E = LoxError> = std::result::Result<T, E>;

#[derive(Error, Debug)]
#[error("{message}")]
pub struct ScannerErrorDetails {
    pub message: String,
    pub line: usize,
}

#[derive(Error, Debug)]
#[error("{message}")]
pub struct ParserErrorDetails {
    pub message: String,
    pub token: Token,
}

#[derive(Error, Debug)]
#[error("{message}")]
pub struct ResolverErrorDetails {
    pub message: String,
    pub token: Token,
}

#[derive(Error, Debug)]
pub enum LoxError {
    #[error("Scanning Error: {details:?}")]
    ScanningError {
        tokens: Vec<Token>,
        details: Vec<ScannerErrorDetails>,
    },

    #[error("Failed to parse literal value.")]
    LiteralParseError,

    #[error("Parse Error: {details:?}")]
    ParseError {
        statements: Vec<Stmt>,
        details: Vec<ParserErrorDetails>,
    },

    #[error("Resolution Error: {0:?}")]
    ResolutionError(Vec<ResolverErrorDetails>),

    #[error("Runtime Error: {message}")]
    RuntimeError { message: String, token: Token },

    #[error("Can only call functions and classes.")]
    NotCallableError,

    #[error("Arguments did not match parameters")]
    IncorrectArityError,

    #[error("Return jump signal")]
    ReturnJump(Value),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

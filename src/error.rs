use crate::{
    stmt::Stmt,
    token::{Token, TokenLiteral},
};
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

    #[error("Runtime Error: {message}")]
    RuntimeError { message: String, token: Token },

    #[error("Can only call functions and classes.")]
    NotCallableError,

    #[error("Arguments did not match parameters")]
    IncorrectArityError,

    #[error("Return jump signal")]
    ReturnJump(TokenLiteral),

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

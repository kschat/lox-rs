use std::fmt::{Debug, Display};

use crate::token_kind::TokenKind;

#[derive(Debug)]
pub enum TokenLiteral {
    String(String),
    Number(f64),
}

impl Display for TokenLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => Display::fmt(value, f),
            Self::Number(value) => Display::fmt(value, f),
        }
    }
}

impl From<String> for TokenLiteral {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<f64> for TokenLiteral {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub literal: Option<TokenLiteral>,
    pub line: usize,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.literal {
            Some(value) => write!(f, "{:?} {} {}", self.kind, self.lexeme, value),
            None => write!(f, "{:?} {}", self.kind, self.lexeme),
        }
    }
}

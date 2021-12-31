use std::fmt::{Debug, Display};

use crate::{token_kind::TokenKind, value::Value};

#[derive(Debug, Clone)]
pub struct Token {
    pub id: usize,
    pub kind: TokenKind,
    pub lexeme: String,
    pub literal: Option<Value>,
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

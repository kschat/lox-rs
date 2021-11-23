use std::collections::HashMap;

use crate::{
    error::{LoxError, Result},
    token::{Token, TokenLiteral},
};

pub struct Environment {
    values: HashMap<String, TokenLiteral>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: TokenLiteral) {
        // TODO can we do this without creatinga new string?
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Result<&TokenLiteral> {
        self.values
            .get(&name.lexeme)
            .ok_or_else(|| LoxError::RuntimeError {
                // TODO get rid of clone
                token: name.clone(),
                message: format!("Undefined variable '{}'.", name.lexeme),
            })
    }

    pub fn assign(&mut self, name: &Token, value: &TokenLiteral) -> Result<()> {
        if !self.values.contains_key(&name.lexeme) {
            return Err(LoxError::RuntimeError {
                token: name.clone(),
                message: format!("variable '{}' not defined.", name.lexeme),
            });
        }

        self.values.insert(name.lexeme.to_string(), value.clone());
        Ok(())
    }
}

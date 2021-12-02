use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{LoxError, Result},
    token::{Token, TokenLiteral},
};

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, TokenLiteral>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            enclosing: None,
            values: HashMap::new(),
        }))
    }

    pub fn new_with_parent(enclosing: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }))
    }

    pub fn define(&mut self, name: &str, value: TokenLiteral) {
        self.values.insert(name.into(), value);
    }

    pub fn get(&self, name: &Token) -> Result<TokenLiteral> {
        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => match &self.enclosing {
                Some(enclosing) => enclosing.borrow().get(name),
                None => Err(LoxError::RuntimeError {
                    // TODO get rid of clone
                    token: name.clone(),
                    message: format!("Undefined variable '{}'.", name.lexeme),
                }),
            },
        }
    }

    pub fn assign(&mut self, name: &Token, value: &TokenLiteral) -> Result<()> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.to_string(), value.clone());

            return Ok(());
        }

        match &self.enclosing {
            Some(enclosing) => enclosing.borrow_mut().assign(name, value),
            None => Err(LoxError::RuntimeError {
                // TODO get rid of clone
                token: name.clone(),
                message: format!("variable '{}' not defined.", name.lexeme),
            }),
        }
    }
}
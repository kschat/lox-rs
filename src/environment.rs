use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{LoxError, Result},
    token::Token,
    value::Value,
};

#[derive(Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Value>,
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

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.into(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
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

    pub fn get_at(&self, distance: usize, name: &Token) -> Result<Value> {
        match distance {
            0 => Ok(self.values.get(&name.lexeme).unwrap().clone()),
            _ => match &self.enclosing {
                Some(enclosing) => enclosing.borrow().get_at(distance - 1, name),
                None => Err(LoxError::RuntimeError {
                    // TODO get rid of clone
                    token: name.clone(),
                    message: format!("Undefined variable '{}'.", name.lexeme),
                }),
            },
        }
    }

    pub fn get_keyword_at(&self, distance: usize, name: &str) -> Result<Value> {
        match distance {
            0 => Ok(self.values.get(name).unwrap().clone()),
            _ => match &self.enclosing {
                Some(enclosing) => enclosing.borrow().get_keyword_at(distance - 1, name),
                None => Err(LoxError::UnresolvedKeywordError {
                    keyword: name.to_string(),
                }),
            },
        }
    }

    pub fn assign(&mut self, name: &Token, value: &Value) -> Result<()> {
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

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: &Value) -> Result<()> {
        if distance == 0 {
            self.values.insert(name.lexeme.to_string(), value.clone());

            return Ok(());
        }

        match &self.enclosing {
            Some(enclosing) => enclosing.borrow_mut().assign_at(distance - 1, name, value),
            None => Err(LoxError::RuntimeError {
                // TODO get rid of clone
                token: name.clone(),
                message: format!("variable '{}' not defined.", name.lexeme),
            }),
        }
    }
}

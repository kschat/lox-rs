use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

use crate::{
    error::{LoxError, Result},
    token::{Token, TokenLiteral},
};

type ScopeId = usize;

pub struct Environment {
    arena: Vec<Option<RefCell<Scope>>>,
    current: ScopeId,
}

impl Environment {
    pub fn new() -> Self {
        let current = 0;
        Self {
            arena: vec![Some(RefCell::new(Scope::new(current, None)))],
            current,
        }
    }

    pub fn new_scope(&mut self) {
        let new_id = self.arena.len();
        let enclosing = match self.arena.get(self.current) {
            Some(Some(current)) => Some(current.borrow().id),
            _ => None,
        };

        self.arena
            .push(Some(RefCell::new(Scope::new(new_id, enclosing))));
        self.current = new_id;
    }

    pub fn end_scope(&mut self) {
        let new_current = self.get_scope(self.current).enclosing.unwrap();
        self.arena[self.current] = None;
        self.current = new_current;
    }

    pub fn define(&mut self, name: &str, value: TokenLiteral) {
        // TODO can we do this without creating a new string?
        self.get_scope_mut(self.current)
            .values
            .insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Result<TokenLiteral> {
        let current = self.get_scope(self.current);
        self.get_at(current, name)
    }

    fn get_at(&self, scope: Ref<Scope>, name: &Token) -> Result<TokenLiteral> {
        match scope.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => match scope.enclosing {
                Some(parent_id) => self.get_at(self.get_scope(parent_id), name),
                None => Err(LoxError::RuntimeError {
                    // TODO get rid of clone
                    token: name.clone(),
                    message: format!("Undefined variable '{}'.", name.lexeme),
                }),
            },
        }
    }

    pub fn assign(&self, name: &Token, value: &TokenLiteral) -> Result<()> {
        let current = self.get_scope_mut(self.current);
        self.assign_at(current, name, value)
    }

    fn assign_at(
        &self,
        mut scope: RefMut<Scope>,
        name: &Token,
        value: &TokenLiteral,
    ) -> Result<()> {
        if scope.values.contains_key(&name.lexeme) {
            scope.values.insert(name.lexeme.to_string(), value.clone());

            return Ok(());
        }

        match scope.enclosing {
            Some(parent_id) => self.assign_at(self.get_scope_mut(parent_id), name, value),
            None => Err(LoxError::RuntimeError {
                // TODO get rid of clone
                token: name.clone(),
                message: format!("variable '{}' not defined.", name.lexeme),
            }),
        }
    }

    fn get_scope(&self, id: ScopeId) -> Ref<Scope> {
        self.arena[id].as_ref().unwrap().borrow()
    }

    fn get_scope_mut(&self, id: ScopeId) -> RefMut<Scope> {
        self.arena[id].as_ref().unwrap().borrow_mut()
    }
}

pub struct Scope {
    id: ScopeId,
    enclosing: Option<ScopeId>,
    values: HashMap<String, TokenLiteral>,
}

impl Scope {
    pub fn new(id: ScopeId, enclosing: Option<ScopeId>) -> Self {
        Self {
            id,
            enclosing,
            values: HashMap::new(),
        }
    }
}

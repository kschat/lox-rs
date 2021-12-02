use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{
    callable::Callable,
    environment::Environment,
    error::{LoxError, Result},
    interpreter::Interpreter,
    stmt::Stmt,
    token_kind::TokenKind,
};

#[derive(Debug, Clone)]
pub enum TokenLiteral {
    String(String),
    Number(f64),
    Boolean(bool),
    Function(Box<Token>, Vec<Token>, Vec<Stmt>, Rc<RefCell<Environment>>),
    NativeFunction(Box<dyn Callable>),
    Nil,
}

impl Display for TokenLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => Display::fmt(value, f),
            Self::Number(value) => Display::fmt(value, f),
            Self::Boolean(value) => Display::fmt(value, f),
            Self::NativeFunction(_) => Display::fmt("<native fn>", f),
            Self::Function(name, _, _, _) => write!(f, "<fn {}>", name.lexeme),
            Self::Nil => Display::fmt("nil", f),
        }
    }
}

impl Callable for TokenLiteral {
    fn invoke(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[TokenLiteral],
    ) -> Result<TokenLiteral> {
        match self {
            TokenLiteral::NativeFunction(callee) => callee.call(interpreter, arguments),
            TokenLiteral::Function(_name, parameters, body, closure) => {
                let new_scope = Environment::new_with_parent(closure.clone());

                for (i, parameter) in parameters.iter().enumerate() {
                    new_scope
                        .borrow_mut()
                        .define(&parameter.lexeme, arguments[i].clone())
                }

                match interpreter.execute_block(body, new_scope) {
                    Ok(()) => Ok(TokenLiteral::Nil),
                    Err(LoxError::ReturnJump(value)) => Ok(value),
                    Err(error) => Err(error),
                }
            }
            _ => Err(LoxError::NotCallableError),
        }
    }

    fn arity(&self) -> usize {
        match self {
            TokenLiteral::Function(_, parameters, _, _) => parameters.len(),
            TokenLiteral::NativeFunction(callable) => callable.arity(),
            _ => 0,
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

impl TryFrom<TokenLiteral> for f64 {
    type Error = LoxError;

    fn try_from(value: TokenLiteral) -> Result<Self, Self::Error> {
        match value {
            TokenLiteral::Number(v) => Ok(v),
            _ => Err(Self::Error::LiteralParseError),
        }
    }
}

#[derive(Debug, Clone)]
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

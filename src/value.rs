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
    token::Token,
};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Function(Box<Token>, Vec<Token>, Vec<Stmt>, Rc<RefCell<Environment>>),
    NativeFunction(Box<dyn Callable>),
    Nil,
}

impl Display for Value {
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

impl Callable for Value {
    fn invoke(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> Result<Value> {
        match self {
            Value::NativeFunction(callee) => callee.call(interpreter, arguments),
            Value::Function(_name, parameters, body, closure) => {
                let new_scope = Environment::new_with_parent(closure.clone());

                for (i, parameter) in parameters.iter().enumerate() {
                    new_scope
                        .borrow_mut()
                        .define(&parameter.lexeme, arguments[i].clone())
                }

                match interpreter.execute_block(body, new_scope) {
                    Ok(()) => Ok(Value::Nil),
                    Err(LoxError::ReturnJump(value)) => Ok(value),
                    Err(error) => Err(error),
                }
            }
            _ => Err(LoxError::NotCallableError),
        }
    }

    fn arity(&self) -> usize {
        match self {
            Value::Function(_, parameters, _, _) => parameters.len(),
            Value::NativeFunction(callable) => callable.arity(),
            _ => 0,
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl TryFrom<Value> for f64 {
    type Error = LoxError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(v) => Ok(v),
            _ => Err(Self::Error::LiteralParseError),
        }
    }
}

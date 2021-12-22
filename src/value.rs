use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Debug, Display},
    ops::Deref,
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
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Value>,
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, Value>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, name: &str) -> Option<&Value> {
        self.methods.get(name)
    }
}

impl Callable for LoxClass {
    fn invoke(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> Result<Value> {
        let instance = LoxInstance::new(self.clone());

        if let Some(initializer) = self.find_method("init") {
            initializer.bind(&instance)?.call(interpreter, arguments)?;
        }

        Ok(Value::Instance(instance))
    }

    fn arity(&self) -> usize {
        match self.find_method("init") {
            Some(value) => value.arity(),
            None => 0,
        }
    }

    fn bind(&self, _instance: &LoxInstance) -> Result<Value> {
        Err(LoxError::NotBindableError)
    }
}

impl Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
pub struct LoxInstanceData {
    class: LoxClass,
    fields: HashMap<String, Value>,
}

impl LoxInstanceData {
    pub fn new(class: LoxClass) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }
}

impl Display for LoxInstanceData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {{", self.class.name)?;
        for (i, (name, value)) in self.fields.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }

            write!(f, " {}: {}", name, value)?;
        }

        write!(f, " }}")
    }
}

#[derive(Debug)]
pub struct LoxInstance(Rc<RefCell<LoxInstanceData>>);

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self(Rc::new(RefCell::new(LoxInstanceData::new(class))))
    }

    pub fn get(&self, name: &Token) -> Result<Value> {
        let data = self.0.borrow();

        if let Some(value) = data.fields.get(&name.lexeme) {
            return Ok(value.clone());
        }

        if let Some(value) = data.class.find_method(&name.lexeme) {
            return value.bind(self);
        }

        Err(LoxError::RuntimeError {
            token: name.clone(),
            message: format!("Undefined property '{}'.", name.lexeme),
        })
    }

    pub fn set(&mut self, name: &Token, value: &Value) {
        self.0
            .borrow_mut()
            .fields
            .insert(name.lexeme.to_string(), value.clone());
    }
}

impl Deref for LoxInstance {
    type Target = Rc<RefCell<LoxInstanceData>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for LoxInstance {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&*self.borrow(), f)
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    // TODO create LoxFunction struct
    Function {
        name: Box<Token>,
        parameters: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    },
    NativeFunction(Box<dyn Callable>),
    Class(LoxClass),
    Instance(LoxInstance),
    Nil,
}

impl Value {
    pub fn is_equal(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Nil, _) => false,
            (Value::Boolean(v1), Value::Boolean(v2)) => v1 == v2,
            #[allow(clippy::float_cmp)]
            (Value::Number(v1), Value::Number(v2)) => v1 == v2,
            (Value::String(v1), Value::String(v2)) => v1 == v2,
            (_, _) => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match *self {
            Value::Nil => false,
            Value::Boolean(value) => value,
            _ => true,
        }
    }

    pub fn to_number(&self, token: &Token) -> Result<f64> {
        self.try_into().map_err(|_| LoxError::RuntimeError {
            token: token.clone(),
            message: "Operand must be a number.".into(),
        })
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(value) => Display::fmt(value, f),
            Self::Number(value) => Display::fmt(value, f),
            Self::Boolean(value) => Display::fmt(value, f),
            Self::NativeFunction(_) => Display::fmt("<native fn>", f),
            Self::Function { name, .. } => write!(f, "<fn {}>", name.lexeme),
            Self::Class(class) => Display::fmt(class, f),
            Self::Instance(instance) => Display::fmt(instance, f),
            Self::Nil => Display::fmt("nil", f),
        }
    }
}

impl Callable for Value {
    fn invoke(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> Result<Value> {
        match self {
            Value::NativeFunction(callee) => callee.call(interpreter, arguments),
            Value::Class(callee) => callee.call(interpreter, arguments),
            Value::Function {
                parameters,
                body,
                closure,
                is_initializer,
                ..
            } => {
                let new_scope = Environment::new_with_parent(closure.clone());

                for (i, parameter) in parameters.iter().enumerate() {
                    new_scope
                        .borrow_mut()
                        .define(&parameter.lexeme, arguments[i].clone())
                }

                match interpreter.execute_block(body, new_scope) {
                    Ok(()) => Ok(match is_initializer {
                        false => Value::Nil,
                        true => closure.borrow().get_keyword_at(0, "this")?,
                    }),
                    Err(LoxError::ReturnJump(value)) => Ok(match is_initializer {
                        false => value,
                        true => closure.borrow().get_keyword_at(0, "this")?,
                    }),
                    Err(error) => Err(error),
                }
            }
            _ => Err(LoxError::NotCallableError),
        }
    }

    fn arity(&self) -> usize {
        match self {
            Value::Function { parameters, .. } => parameters.len(),
            Value::NativeFunction(callable) => callable.arity(),
            Value::Class(class) => class.arity(),
            _ => 0,
        }
    }

    fn bind(&self, instance: &LoxInstance) -> Result<Value> {
        match self {
            Value::Function {
                name,
                parameters,
                body,
                closure,
                is_initializer,
            } => {
                let environment = Environment::new_with_parent(closure.clone());
                environment
                    .borrow_mut()
                    .define("this", Value::Instance(instance.clone()));

                Ok(Value::Function {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    body: body.clone(),
                    closure: environment,
                    is_initializer: *is_initializer,
                })
            }
            _ => Err(LoxError::NotBindableError),
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

impl TryFrom<&Value> for f64 {
    type Error = LoxError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(v) => Ok(*v),
            _ => Err(Self::Error::LiteralParseError),
        }
    }
}

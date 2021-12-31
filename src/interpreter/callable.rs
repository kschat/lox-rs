use crate::{
    error::{LoxError, Result},
    interpreter::Interpreter,
    value::{LoxInstance, Value},
};

use dyn_clone::DynClone;
use std::fmt::Debug;

pub trait Callable: DynClone + Debug {
    fn arity(&self) -> usize;

    fn invoke(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> Result<Value>;

    fn call(&self, interpreter: &mut Interpreter, arguments: &[Value]) -> Result<Value> {
        self.validate(arguments)?;
        self.invoke(interpreter, arguments)
    }

    fn validate(&self, arguments: &[Value]) -> Result<()> {
        if arguments.len() != self.arity() {
            return Err(LoxError::IncorrectArityError);
        }

        Ok(())
    }

    fn bind(&self, instance: &LoxInstance) -> Result<Value>;
}

dyn_clone::clone_trait_object!(Callable);

use crate::{
    error::{LoxError, Result},
    interpreter::Interpreter,
    token::TokenLiteral,
};

use dyn_clone::DynClone;
use std::fmt::Debug;

pub trait Callable: DynClone + Debug {
    fn arity(&self) -> usize;

    fn invoke(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[TokenLiteral],
    ) -> Result<TokenLiteral>;

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[TokenLiteral],
    ) -> Result<TokenLiteral> {
        self.validate(arguments)?;
        self.invoke(interpreter, arguments)
    }

    fn validate(&self, arguments: &[TokenLiteral]) -> Result<()> {
        if arguments.len() != self.arity() {
            return Err(LoxError::IncorrectArityError);
        }

        Ok(())
    }
}

dyn_clone::clone_trait_object!(Callable);

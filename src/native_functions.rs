use std::time::{SystemTime, UNIX_EPOCH};

use crate::{callable::Callable, error::Result, interpreter::Interpreter, token::TokenLiteral};

#[derive(Debug, Clone)]
pub struct ClockCallable;

impl Callable for ClockCallable {
    fn invoke(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: &[TokenLiteral],
    ) -> Result<TokenLiteral> {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(TokenLiteral::Number(elapsed as f64))
    }

    fn arity(&self) -> usize {
        0
    }
}

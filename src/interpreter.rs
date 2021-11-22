use crate::{
    error::{LoxError, Result},
    expr::{Expr, Visitor},
    token::{Token, TokenLiteral},
    token_kind::TokenKind,
};

pub struct Interpreter<'a> {
    error_handler: Box<dyn FnMut(&LoxError) + 'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new<F>(handler: F) -> Self
    where
        F: FnMut(&LoxError) + 'a,
    {
        Self {
            error_handler: Box::new(handler),
        }
    }

    pub fn interpret(&mut self, expr: Expr) {
        match self.evaluate(&expr) {
            Ok(value) => println!("{}", value),
            Err(error) => (self.error_handler)(&error),
        }
    }

    fn evaluate(&self, expr: &Expr) -> Result<TokenLiteral> {
        expr.accept(self)
    }

    fn is_truthy(literal: TokenLiteral) -> bool {
        match literal {
            TokenLiteral::Nil => false,
            TokenLiteral::Boolean(value) => value,
            _ => true,
        }
    }

    fn to_number(literal: TokenLiteral, token: &Token) -> Result<f64> {
        literal.try_into().map_err(|_| LoxError::RuntimeError {
            // TODO get rid of clone
            token: token.clone(),
            message: "Operand must be a number.".into(),
        })
    }

    fn is_equal(a: TokenLiteral, b: TokenLiteral) -> bool {
        match (a, b) {
            (TokenLiteral::Nil, TokenLiteral::Nil) => true,
            (TokenLiteral::Nil, _) => false,
            (TokenLiteral::Boolean(v1), TokenLiteral::Boolean(v2)) => v1 == v2,
            #[allow(clippy::float_cmp)]
            (TokenLiteral::Number(v1), TokenLiteral::Number(v2)) => v1 == v2,
            (TokenLiteral::String(v1), TokenLiteral::String(v2)) => v1 == v2,
            (_, _) => false,
        }
    }
}

impl<'a> Visitor<Result<TokenLiteral>> for Interpreter<'a> {
    fn visit_binary_expr(
        &self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<TokenLiteral> {
        let left_value = self.evaluate(left)?;
        let right_value = self.evaluate(right)?;

        Ok(match operator.kind {
            TokenKind::Minus => TokenLiteral::Number(
                Interpreter::to_number(left_value, operator)?
                    - Interpreter::to_number(right_value, operator)?,
            ),
            TokenKind::Slash => TokenLiteral::Number(
                Interpreter::to_number(left_value, operator)?
                    / Interpreter::to_number(right_value, operator)?,
            ),
            TokenKind::Star => TokenLiteral::Number(
                Interpreter::to_number(left_value, operator)?
                    * Interpreter::to_number(right_value, operator)?,
            ),
            TokenKind::Plus => match (left_value, right_value) {
                (TokenLiteral::Number(l), TokenLiteral::Number(r)) => TokenLiteral::Number(l + r),
                (TokenLiteral::String(l), TokenLiteral::String(r)) => {
                    TokenLiteral::String(format!("{}{}", l, r))
                }
                _ => {
                    return Err(LoxError::RuntimeError {
                        // TODO get rid of clone
                        token: operator.clone(),
                        message: "Operands must be two numbers or two strings.".into(),
                    });
                }
            },
            TokenKind::Greater => TokenLiteral::Boolean(
                Interpreter::to_number(left_value, operator)?
                    > Interpreter::to_number(right_value, operator)?,
            ),
            TokenKind::GreaterEqual => TokenLiteral::Boolean(
                Interpreter::to_number(left_value, operator)?
                    >= Interpreter::to_number(right_value, operator)?,
            ),
            TokenKind::Less => TokenLiteral::Boolean(
                Interpreter::to_number(left_value, operator)?
                    < Interpreter::to_number(right_value, operator)?,
            ),
            TokenKind::LessEqual => TokenLiteral::Boolean(
                Interpreter::to_number(left_value, operator)?
                    <= Interpreter::to_number(right_value, operator)?,
            ),
            TokenKind::BangEqual => {
                TokenLiteral::Boolean(!Interpreter::is_equal(left_value, right_value))
            }
            TokenKind::EqualEqual => {
                TokenLiteral::Boolean(Interpreter::is_equal(left_value, right_value))
            }
            _ => unreachable!(),
        })
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> Result<TokenLiteral> {
        let right_value = self.evaluate(right)?;

        Ok(match operator.kind {
            TokenKind::Minus => TokenLiteral::Number(-f64::try_from(right_value).unwrap()),
            TokenKind::Bang => TokenLiteral::Boolean(!Interpreter::is_truthy(right_value)),
            _ => unreachable!(),
        })
    }

    fn visit_group_expr(&self, expr: &Expr) -> Result<TokenLiteral> {
        self.evaluate(expr)
    }

    fn visit_literal_expr(&self, literal: &TokenLiteral) -> Result<TokenLiteral> {
        Ok(literal.clone())
    }
}

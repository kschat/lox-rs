use std::{cell::RefCell, rc::Rc};

use crate::{
    callable::Callable,
    environment::Environment,
    error::{LoxError, Result},
    expr::{Expr, ExprVisitor},
    native_functions::ClockCallable,
    stmt::{Stmt, StmtVisitor},
    token::{Token, TokenLiteral},
    token_kind::TokenKind,
};

pub struct Interpreter {
    pub environment: Rc<RefCell<Environment>>,
    pub globals: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();
        let environment = globals.clone();

        globals.borrow_mut().define(
            "clock",
            TokenLiteral::NativeFunction(Box::new(ClockCallable)),
        );

        Self {
            environment,
            globals,
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), Vec<LoxError>> {
        let mut errors: Vec<LoxError> = vec![];
        for stmt in stmts {
            if let Err(error) = self.execute(&stmt) {
                errors.push(error);
            }
        }

        match errors.len() {
            0 => Ok(()),
            _ => Err(errors),
        }
    }

    pub(crate) fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Result<()> {
        let previous = self.environment.clone();
        self.environment = environment;

        for statement in statements {
            if let error @ Err(_) = self.execute(statement) {
                self.environment = previous;
                return error;
            }
        }

        self.environment = previous;

        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<TokenLiteral> {
        expr.accept(self)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn is_truthy(literal: &TokenLiteral) -> bool {
        match *literal {
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

impl ExprVisitor<Result<TokenLiteral>> for Interpreter {
    fn visit_binary_expr(
        &mut self,
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

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<TokenLiteral> {
        let right_value = self.evaluate(right)?;

        Ok(match operator.kind {
            TokenKind::Minus => TokenLiteral::Number(-f64::try_from(right_value).unwrap()),
            TokenKind::Bang => TokenLiteral::Boolean(!Interpreter::is_truthy(&right_value)),
            _ => unreachable!(),
        })
    }

    fn visit_group_expr(&mut self, expr: &Expr) -> Result<TokenLiteral> {
        self.evaluate(expr)
    }

    fn visit_literal_expr(&mut self, literal: &TokenLiteral) -> Result<TokenLiteral> {
        Ok(literal.clone())
    }

    fn visit_variable_expr(&mut self, name: &Token) -> Result<TokenLiteral> {
        self.environment.borrow().get(name)
    }

    fn visit_assign_expr(&mut self, name: &Token, expr: &Expr) -> Result<TokenLiteral> {
        let value = self.evaluate(expr)?;
        self.environment.borrow_mut().assign(name, &value)?;

        Ok(value)
    }

    fn visit_logicial_expr(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<TokenLiteral> {
        let left_value = self.evaluate(left)?;
        match operator.kind {
            TokenKind::Or => {
                if Interpreter::is_truthy(&left_value) {
                    return Ok(left_value);
                }
            }
            TokenKind::And => {
                if !Interpreter::is_truthy(&left_value) {
                    return Ok(left_value);
                }
            }
            _ => unreachable!(),
        }

        self.evaluate(right)
    }

    fn visit_call_expr(
        &mut self,
        callee: &Expr,
        arguments: &[Expr],
        paren: &Token,
    ) -> Result<TokenLiteral> {
        let callee = self.evaluate(callee)?;

        let arguments = arguments
            .iter()
            .map(|arg| self.evaluate(arg))
            .collect::<Result<Vec<_>>>()?;

        callee.call(self, &arguments).map_err(|error| match error {
            LoxError::IncorrectArityError => LoxError::RuntimeError {
                message: format!(
                    "Expected {} arguments but got {}.",
                    callee.arity(),
                    arguments.len()
                ),
                token: paren.clone(),
            },
            LoxError::NotCallableError => LoxError::RuntimeError {
                message: "Can only call functions and classes.".into(),
                token: paren.clone(),
            },
            _ => error,
        })
    }
}

impl StmtVisitor<Result<()>> for Interpreter {
    fn visit_expression_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.evaluate(expr)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<()> {
        let value = self.evaluate(expr)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: Option<&Expr>) -> Result<()> {
        let value = match initializer {
            Some(v) => self.evaluate(v)?,
            None => TokenLiteral::Nil,
        };

        self.environment.borrow_mut().define(&name.lexeme, value);

        Ok(())
    }

    fn visit_block_stmt(&mut self, statements: &[Stmt]) -> Result<()> {
        self.execute_block(
            statements,
            Environment::new_with_parent(self.environment.clone()),
        )
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) -> Result<()> {
        if Interpreter::is_truthy(&self.evaluate(condition)?) {
            self.execute(then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(else_branch)?;
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<()> {
        while Interpreter::is_truthy(&self.evaluate(condition)?) {
            self.execute(body)?;
        }

        Ok(())
    }

    fn visit_function_stmt(
        &mut self,
        name: &Token,
        parameters: &[Token],
        block: &[Stmt],
    ) -> Result<()> {
        let function = TokenLiteral::Function(
            name.clone().into(),
            parameters.to_vec(),
            block.to_vec(),
            self.environment.clone(),
        );

        self.environment.borrow_mut().define(&name.lexeme, function);

        Ok(())
    }

    fn visit_return_stmt(&mut self, _keyword: &Token, value: Option<&Expr>) -> Result<()> {
        Err(LoxError::ReturnJump(match value {
            Some(v) => self.evaluate(v)?,
            None => TokenLiteral::Nil,
        }))
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

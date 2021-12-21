use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    callable::Callable,
    environment::Environment,
    error::{LoxError, Result},
    expr::{Expr, ExprVisitor},
    native_functions::ClockCallable,
    stmt::{Stmt, StmtVisitor},
    token::Token,
    token_kind::TokenKind,
    value::{LoxClass, Value},
};

pub struct Interpreter {
    pub environment: Rc<RefCell<Environment>>,
    pub globals: Rc<RefCell<Environment>>,
    locals: HashMap<usize, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();
        let environment = globals.clone();

        globals
            .borrow_mut()
            .define("clock", Value::NativeFunction(Box::new(ClockCallable)));

        Self {
            environment,
            globals,
            locals: HashMap::new(),
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

    pub(crate) fn resolve(&mut self, name: &Token, depth: usize) {
        self.locals.insert(name.id, depth);
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value> {
        expr.accept(self)
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn lookup_variable(&mut self, name: &Token) -> Result<Value> {
        match self.locals.get(&name.id) {
            Some(distance) => self.environment.borrow().get_at(*distance, name),
            None => self.globals.borrow().get(name),
        }
    }
}

impl ExprVisitor<Result<Value>> for Interpreter {
    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Value> {
        let left_value = self.evaluate(left)?;
        let right_value = self.evaluate(right)?;

        Ok(match operator.kind {
            TokenKind::Minus => {
                Value::Number(left_value.to_number(operator)? - right_value.to_number(operator)?)
            }
            TokenKind::Slash => {
                Value::Number(left_value.to_number(operator)? / right_value.to_number(operator)?)
            }
            TokenKind::Star => {
                Value::Number(left_value.to_number(operator)? * right_value.to_number(operator)?)
            }
            TokenKind::Plus => match (left_value, right_value) {
                (Value::Number(l), Value::Number(r)) => Value::Number(l + r),
                (Value::String(l), Value::String(r)) => Value::String(format!("{}{}", l, r)),
                _ => {
                    return Err(LoxError::RuntimeError {
                        // TODO get rid of clone
                        token: operator.clone(),
                        message: "Operands must be two numbers or two strings.".into(),
                    });
                }
            },
            TokenKind::Greater => {
                Value::Boolean(left_value.to_number(operator)? > right_value.to_number(operator)?)
            }
            TokenKind::GreaterEqual => {
                Value::Boolean(left_value.to_number(operator)? >= right_value.to_number(operator)?)
            }
            TokenKind::Less => {
                Value::Boolean(left_value.to_number(operator)? < right_value.to_number(operator)?)
            }
            TokenKind::LessEqual => {
                Value::Boolean(left_value.to_number(operator)? <= right_value.to_number(operator)?)
            }
            TokenKind::BangEqual => Value::Boolean(!left_value.is_equal(&right_value)),
            TokenKind::EqualEqual => Value::Boolean(left_value.is_equal(&right_value)),
            _ => unreachable!(),
        })
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<Value> {
        let right_value = self.evaluate(right)?;

        Ok(match operator.kind {
            TokenKind::Minus => Value::Number(-right_value.to_number(operator)?),
            TokenKind::Bang => Value::Boolean(!right_value.is_truthy()),
            _ => unreachable!(),
        })
    }

    fn visit_group_expr(&mut self, expr: &Expr) -> Result<Value> {
        self.evaluate(expr)
    }

    fn visit_literal_expr(&mut self, literal: &Value) -> Result<Value> {
        Ok(literal.clone())
    }

    fn visit_variable_expr(&mut self, name: &Token) -> Result<Value> {
        self.lookup_variable(name)
    }

    fn visit_assign_expr(&mut self, name: &Token, expr: &Expr) -> Result<Value> {
        let value = self.evaluate(expr)?;

        match self.locals.get(&name.id) {
            None => self.globals.borrow_mut().assign(name, &value)?,
            Some(distance) => self
                .environment
                .borrow_mut()
                .assign_at(*distance, name, &value)?,
        };

        Ok(value)
    }

    fn visit_logicial_expr(
        &mut self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Value> {
        let left_value = self.evaluate(left)?;
        match operator.kind {
            TokenKind::Or => {
                if left_value.is_truthy() {
                    return Ok(left_value);
                }
            }
            TokenKind::And => {
                if !left_value.is_truthy() {
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
    ) -> Result<Value> {
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

    fn visit_get_expr(&mut self, object: &Expr, name: &Token) -> Result<Value> {
        match self.evaluate(object)? {
            Value::Instance(instance) => instance.get(name),
            _ => Err(LoxError::RuntimeError {
                message: "Only instances have properties.".into(),
                token: name.clone(),
            }),
        }
    }

    fn visit_set_expr(&mut self, object: &Expr, name: &Token, value: &Expr) -> Result<Value> {
        match self.evaluate(object)? {
            Value::Instance(mut instance) => {
                let value = self.evaluate(value)?;
                instance.set(name, &value);
                Ok(value)
            }
            _ => Err(LoxError::RuntimeError {
                message: "Only instances have fields.".into(),
                token: name.clone(),
            }),
        }
    }

    fn visit_this_expr(&mut self, keyword: &Token) -> Result<Value> {
        self.lookup_variable(keyword)
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
            None => Value::Nil,
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
        if self.evaluate(condition)?.is_truthy() {
            self.execute(then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(else_branch)?;
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<()> {
        while self.evaluate(condition)?.is_truthy() {
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
        let function = Value::Function(
            name.clone().into(),
            parameters.to_vec(),
            block.to_vec(),
            self.environment.clone(),
            false,
        );

        self.environment.borrow_mut().define(&name.lexeme, function);

        Ok(())
    }

    fn visit_return_stmt(&mut self, _keyword: &Token, value: Option<&Expr>) -> Result<()> {
        Err(LoxError::ReturnJump(match value {
            Some(v) => self.evaluate(v)?,
            None => Value::Nil,
        }))
    }

    fn visit_class_stmt(&mut self, name: &Token, methods: &[Stmt]) -> Result<()> {
        self.environment
            .borrow_mut()
            .define(&name.lexeme, Value::Nil);

        let methods = methods
            .iter()
            .fold(HashMap::new(), |mut acc, method| match method {
                Stmt::Function(name, parameters, block) => {
                    acc.insert(
                        name.lexeme.to_string(),
                        Value::Function(
                            name.clone().into(),
                            parameters.clone(),
                            block.clone(),
                            self.environment.clone(),
                            name.lexeme == "init",
                        ),
                    );

                    acc
                }
                _ => unreachable!(),
            });

        let class = Value::Class(LoxClass::new(name.lexeme.clone(), methods));

        self.environment.borrow_mut().assign(name, &class)?;

        Ok(())
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

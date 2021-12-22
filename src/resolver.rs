use core::slice::Iter;
use std::collections::HashMap;

use crate::{
    error::{LoxError, ResolverErrorDetails, Result},
    expr::{Expr, ExprVisitor},
    interpreter::Interpreter,
    stmt::{Stmt, StmtVisitor},
    token::Token,
    value::Value,
};

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Stack<HashMap<String, bool>>,
    current_function_kind: Option<FunctionKind>,
    current_class_kind: Option<ClassKind>,
    errors: Vec<ResolverErrorDetails>,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Stack::new(),
            current_function_kind: None,
            current_class_kind: None,
            errors: vec![],
        }
    }

    pub fn resolve(mut self, statements: &[Stmt]) -> Result<()> {
        self.resolve_statements(statements)?;

        match self.errors.len() {
            0 => Ok(()),
            _ => Err(LoxError::ResolutionError(self.errors)),
        }
    }

    fn resolve_statements(&mut self, statements: &[Stmt]) -> Result<()> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }

        Ok(())
    }

    fn resolve_statement(&mut self, stmt: &Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn resolve_expression(&mut self, expr: &Expr) -> Result<()> {
        expr.accept(self)
    }

    fn resolve_local(&mut self, name: &Token) {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(name, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn resolve_function(
        &mut self,
        kind: FunctionKind,
        parameters: &[Token],
        body: &[Stmt],
    ) -> Result<()> {
        let enclosing_function_kind = self.current_function_kind;
        self.current_function_kind = Some(kind);

        self.begin_scope();
        for parameter in parameters {
            self.declare(parameter);
            self.define(parameter);
        }

        self.resolve_statements(body)?;
        self.end_scope();
        self.current_function_kind = enclosing_function_kind;

        Ok(())
    }

    #[allow(clippy::needless_return)]
    fn declare(&mut self, name: &Token) {
        match self.scopes.peek_mut() {
            None => return,
            Some(scope) => {
                if scope.contains_key(&name.lexeme) {
                    self.errors.push(ResolverErrorDetails {
                        message: "Already a variable with this name in this scope.".into(),
                        token: name.clone(),
                    });
                }

                scope.insert(name.lexeme.to_string(), false);
            }
        };
    }

    #[allow(clippy::needless_return)]
    fn define(&mut self, name: &Token) {
        match self.scopes.peek_mut() {
            None => return,
            Some(scope) => scope.insert(name.lexeme.to_string(), true),
        };
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}

impl<'a> ExprVisitor<Result<()>> for Resolver<'a> {
    fn visit_binary_expr(&mut self, left: &Expr, _operator: &Token, right: &Expr) -> Result<()> {
        self.resolve_expression(left)?;
        self.resolve_expression(right)?;
        Ok(())
    }

    fn visit_unary_expr(&mut self, _operator: &Token, right: &Expr) -> Result<()> {
        self.resolve_expression(right)
    }

    fn visit_group_expr(&mut self, expr: &Expr) -> Result<()> {
        self.resolve_expression(expr)
    }

    fn visit_literal_expr(&mut self, _literal: &Value) -> Result<()> {
        Ok(())
    }

    fn visit_variable_expr(&mut self, name: &Token) -> Result<()> {
        if let Some(false) = self.scopes.peek().and_then(|scope| scope.get(&name.lexeme)) {
            self.errors.push(ResolverErrorDetails {
                token: name.clone(),
                message: "Can't read local variable in its own initializer.".into(),
            });
        }

        self.resolve_local(name);
        Ok(())
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<()> {
        self.resolve_expression(value)?;
        self.resolve_local(name);
        Ok(())
    }

    fn visit_logicial_expr(&mut self, left: &Expr, _operator: &Token, right: &Expr) -> Result<()> {
        self.resolve_expression(left)?;
        self.resolve_expression(right)?;
        Ok(())
    }

    fn visit_call_expr(&mut self, callee: &Expr, arguments: &[Expr], _paren: &Token) -> Result<()> {
        self.resolve_expression(callee)?;
        for argument in arguments {
            self.resolve_expression(argument)?;
        }

        Ok(())
    }

    fn visit_get_expr(&mut self, object: &Expr, _name: &Token) -> Result<()> {
        self.resolve_expression(object)?;
        Ok(())
    }

    fn visit_set_expr(&mut self, object: &Expr, _name: &Token, value: &Expr) -> Result<()> {
        self.resolve_expression(value)?;
        self.resolve_expression(object)?;
        Ok(())
    }

    fn visit_this_expr(&mut self, keyword: &Token) -> Result<()> {
        match self.current_class_kind {
            Some(_) => self.resolve_local(keyword),
            None => self.errors.push(ResolverErrorDetails {
                message: "Can't use 'this' outside of a class.".into(),
                token: keyword.clone(),
            }),
        };

        Ok(())
    }
}

impl<'a> StmtVisitor<Result<()>> for Resolver<'a> {
    fn visit_expression_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.resolve_expression(expr)
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> Result<()> {
        self.resolve_expression(expr)
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: Option<&Expr>) -> Result<()> {
        self.declare(name);
        if let Some(init) = initializer {
            self.resolve_expression(init)?;
        }
        self.define(name);

        Ok(())
    }

    fn visit_block_stmt(&mut self, statements: &[Stmt]) -> Result<()> {
        self.begin_scope();
        self.resolve_statements(statements)?;
        self.end_scope();

        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) -> Result<()> {
        self.resolve_expression(condition)?;
        self.resolve_statement(then_branch)?;
        if let Some(else_branch) = else_branch {
            self.resolve_statement(else_branch)?;
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<()> {
        self.resolve_expression(condition)?;
        self.resolve_statement(body)?;
        Ok(())
    }

    fn visit_function_stmt(
        &mut self,
        name: &Token,
        parameters: &[Token],
        body: &[Stmt],
    ) -> Result<()> {
        self.declare(name);
        self.define(name);
        self.resolve_function(FunctionKind::Function, parameters, body)?;

        Ok(())
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: Option<&Expr>) -> Result<()> {
        if self.current_function_kind.is_none() {
            self.errors.push(ResolverErrorDetails {
                message: "Can't return from top level code.".into(),
                token: keyword.clone(),
            });
        }

        if let Some(value) = value {
            if let Some(FunctionKind::Initializer) = self.current_function_kind {
                self.errors.push(ResolverErrorDetails {
                    message: "Can't return a value from an initializer.".into(),
                    token: keyword.clone(),
                });
            }

            self.resolve_expression(value)?;
        }

        Ok(())
    }

    fn visit_class_stmt(&mut self, name: &Token, methods: &[Stmt]) -> Result<()> {
        let enclosing_class_kind = self.current_class_kind;
        self.current_class_kind = Some(ClassKind::Class);

        self.declare(name);
        self.begin_scope();
        self.scopes
            .peek_mut()
            .expect("Unexpected global scope")
            .insert("this".into(), true);

        for method in methods {
            match method {
                Stmt::Function(name, parameters, body) => {
                    let kind = match name.lexeme == "init" {
                        true => FunctionKind::Initializer,
                        false => FunctionKind::Method,
                    };

                    self.resolve_function(kind, parameters, body)?;
                }
                _ => unreachable!(),
            };
        }

        self.end_scope();
        self.define(name);
        self.current_class_kind = enclosing_class_kind;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum FunctionKind {
    Function,
    Method,
    Initializer,
}

#[derive(Debug, Clone, Copy)]
enum ClassKind {
    Class,
}

struct Stack<T>(Vec<T>);

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn push(&mut self, value: T) {
        self.0.push(value)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.0.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        match self.0.len() {
            0 => None,
            len => Some(&self.0[len - 1]),
        }
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        match self.0.len() {
            0 => None,
            len => Some(&mut self.0[len - 1]),
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> Iter<T> {
        self.0.iter()
    }
}

use crate::{expr::Expr, token::Token};

#[derive(Debug)]
pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>),
    Var(Token, Option<Expr>),
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        match self {
            Stmt::Expression(expr) => visitor.visit_expression_stmt(expr),
            Stmt::Print(expr) => visitor.visit_print_stmt(expr),
            Stmt::Var(name, initializer) => visitor.visit_var_stmt(name, initializer.as_ref()),
        }
    }
}

pub trait StmtVisitor<T> {
    fn visit_expression_stmt(&mut self, expr: &Expr) -> T;
    fn visit_print_stmt(&mut self, expr: &Expr) -> T;
    fn visit_var_stmt(&mut self, name: &Token, initializer: Option<&Expr>) -> T;
}

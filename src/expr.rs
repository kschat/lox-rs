use crate::token::{Token, TokenLiteral};

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(TokenLiteral),
    Variable(Token),
    Assign(Token, Box<Expr>),
}

impl Expr {
    pub fn accept<T>(&self, visitor: &mut dyn ExprVisitor<T>) -> T {
        match self {
            Expr::Binary(left, operator, right) => visitor.visit_binary_expr(left, operator, right),
            Expr::Unary(operator, right) => visitor.visit_unary_expr(operator, right),
            Expr::Grouping(expr) => visitor.visit_group_expr(expr),
            Expr::Literal(literal) => visitor.visit_literal_expr(literal),
            Expr::Variable(name) => visitor.visit_variable_expr(name),
            Expr::Assign(name, value) => visitor.visit_assign_expr(name, value),
        }
    }
}

pub trait ExprVisitor<T> {
    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> T;
    fn visit_group_expr(&mut self, expr: &Expr) -> T;
    fn visit_literal_expr(&mut self, literal: &TokenLiteral) -> T;
    fn visit_variable_expr(&mut self, name: &Token) -> T;
    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> T;
}

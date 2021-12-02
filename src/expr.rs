use crate::token::{Token, TokenLiteral};

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(TokenLiteral),
    Variable(Token),
    Assign(Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>, Token),
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
            Expr::Logical(left, operator, right) => {
                visitor.visit_logicial_expr(left, operator, right)
            }
            Expr::Call(callee, arguments, paren) => {
                visitor.visit_call_expr(callee, arguments, paren)
            }
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
    fn visit_logicial_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_call_expr(&mut self, callee: &Expr, arguments: &[Expr], paren: &Token) -> T;
}

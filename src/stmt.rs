use crate::{expr::Expr, token::Token};

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var(Token, Option<Expr>),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Function(Token, Vec<Token>, Vec<Stmt>),
    Return(Token, Option<Expr>),
    Class(Token, Vec<Stmt>),
}

impl Stmt {
    pub fn accept<T>(&self, visitor: &mut dyn StmtVisitor<T>) -> T {
        match self {
            Stmt::Expression(expr) => visitor.visit_expression_stmt(expr),
            Stmt::Print(expr) => visitor.visit_print_stmt(expr),
            Stmt::Var(name, initializer) => visitor.visit_var_stmt(name, initializer.as_ref()),
            Stmt::Block(statements) => visitor.visit_block_stmt(statements),
            Stmt::If(condition, then_branch, else_branch) => {
                visitor.visit_if_stmt(condition, then_branch, else_branch.as_deref())
            }
            Stmt::While(condition, body) => visitor.visit_while_stmt(condition, body),
            Stmt::Function(name, parameters, body) => {
                visitor.visit_function_stmt(name, parameters, body)
            }
            Stmt::Return(keyword, value) => visitor.visit_return_stmt(keyword, value.as_ref()),
            Stmt::Class(name, methods) => visitor.visit_class_stmt(name, methods),
        }
    }
}

pub trait StmtVisitor<T> {
    fn visit_expression_stmt(&mut self, expr: &Expr) -> T;
    fn visit_print_stmt(&mut self, expr: &Expr) -> T;
    fn visit_var_stmt(&mut self, name: &Token, initializer: Option<&Expr>) -> T;
    fn visit_block_stmt(&mut self, statements: &[Stmt]) -> T;
    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: Option<&Stmt>,
    ) -> T;
    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> T;
    fn visit_function_stmt(&mut self, name: &Token, parameters: &[Token], body: &[Stmt]) -> T;
    fn visit_return_stmt(&mut self, keyword: &Token, value: Option<&Expr>) -> T;
    fn visit_class_stmt(&mut self, name: &Token, methods: &[Stmt]) -> T;
}

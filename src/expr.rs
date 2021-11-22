use crate::token::{Token, TokenLiteral};

pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(TokenLiteral),
}

impl Expr {
    pub fn accept<T>(&self, visitor: &dyn Visitor<T>) -> T {
        match self {
            Expr::Binary(left, operator, right) => visitor.visit_binary_expr(left, operator, right),
            Expr::Unary(operator, right) => visitor.visit_unary_expr(operator, right),
            Expr::Grouping(expr) => visitor.visit_group_expr(expr),
            Expr::Literal(literal) => visitor.visit_literal_expr(literal),
        }
    }
}

pub trait Visitor<T> {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> T;
    fn visit_group_expr(&self, expr: &Expr) -> T;
    fn visit_literal_expr(&self, literal: &TokenLiteral) -> T;
}

struct AstPrinter;

impl AstPrinter {
    fn print(&self, expr: Expr) -> String {
        expr.accept(self)
    }

    fn parenthesize(&self, name: &str, exprs: &[&Expr]) -> String {
        exprs.iter().fold(format!("({}", name), |acc, expr| {
            format!("{} {}", acc, expr.accept(self))
        }) + ")"
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> String {
        self.parenthesize(&operator.lexeme, &[left, right])
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> String {
        self.parenthesize(&operator.lexeme, &[right])
    }

    fn visit_group_expr(&self, expr: &Expr) -> String {
        self.parenthesize("group", &[expr])
    }

    fn visit_literal_expr(&self, literal: &TokenLiteral) -> String {
        literal.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token_kind::TokenKind;

    #[test]
    fn test_printer() {
        let expr = Expr::Binary(
            Box::new(Expr::Unary(
                Token {
                    kind: TokenKind::Minus,
                    lexeme: "-".to_string(),
                    literal: None,
                    line: 1,
                },
                Box::new(Expr::Literal(TokenLiteral::Number(123_f64))),
            )),
            Token {
                kind: TokenKind::Star,
                lexeme: "*".to_string(),
                literal: None,
                line: 1,
            },
            Box::new(Expr::Grouping(Box::new(Expr::Literal(
                TokenLiteral::Number(45.76),
            )))),
        );

        assert_eq!(AstPrinter.print(expr), "(* (- 123) (group 45.76))");
    }
}

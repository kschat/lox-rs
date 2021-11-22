use crate::{
    error::{LoxError, Result},
    expr::Expr,
    token::{Token, TokenLiteral},
    token_kind::TokenKind,
};

/// Grammar:
///
/// expression  -> equality ;
/// equality    -> comparison ( ( "==" | "!=" ) comparison )* ;
/// comparison  -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
/// term        -> factor ( ( "-" | "+" ) factor )* ;
/// factor      -> unary ( ( "/" | "*" ) unary )* ;
/// unary       -> ( "!" | "-" ) unary | primary ;
/// primary     -> NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    error_handler: Box<dyn FnMut(&Token, &str) + 'a>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            error_handler: Box::new(|_, _| panic!("Unhandled error")),
        }
    }

    pub fn on_error<F>(&mut self, handler: F)
    where
        F: FnMut(&Token, &str) + 'a,
    {
        self.error_handler = Box::new(handler);
    }

    pub fn parse(mut self) -> Option<Expr> {
        self.expression().ok()
    }

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.matches(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            // TODO get rid of clone
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while self.matches(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            // TODO get rid of clone
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.matches(&[TokenKind::Minus, TokenKind::Plus]) {
            // TODO get rid of clone
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.matches(&[TokenKind::Slash, TokenKind::Star]) {
            // TODO get rid of clone
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.matches(&[TokenKind::Bang, TokenKind::Minus]) {
            // TODO get rid of clone
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, right.into()));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.matches(&[TokenKind::True]) {
            return Ok(Expr::Literal(TokenLiteral::Boolean(true)));
        }

        if self.matches(&[TokenKind::False]) {
            return Ok(Expr::Literal(TokenLiteral::Boolean(false)));
        }

        if self.matches(&[TokenKind::Nil]) {
            return Ok(Expr::Literal(TokenLiteral::Nil));
        }

        if self.matches(&[TokenKind::String, TokenKind::Number]) {
            // TODO get rid of clone
            let literal = self.previous().clone().literal.unwrap();

            return Ok(Expr::Literal(literal));
        }

        if self.matches(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.try_consume(TokenKind::RightParen, "Expected ')' after expression.")?;

            return Ok(Expr::Grouping(expr.into()));
        }

        // TODO get rid of clone
        Err(self.report_error(&self.peek().clone(), "Expected expression."))
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Eof {
                return;
            }

            match self.peek().kind {
                TokenKind::Class
                | TokenKind::Fun
                | TokenKind::If
                | TokenKind::Print
                | TokenKind::Return
                | TokenKind::Var
                | TokenKind::While => return,
                _ => self.advance(),
            };
        }
    }

    fn matches(&mut self, kinds: &[TokenKind]) -> bool {
        let found_match = kinds.iter().any(|kind| self.check(*kind));
        if found_match {
            self.advance();
            return true;
        }

        false
    }

    fn try_consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token> {
        if self.check(kind) {
            return Ok(self.advance());
        }

        // TODO get rid of clone
        Err(self.report_error(&self.peek().clone(), message))
    }

    fn report_error(&mut self, token: &Token, message: &str) -> LoxError {
        (self.error_handler)(token, message);

        LoxError::ParseError
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().kind == kind
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}

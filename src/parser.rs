use crate::{
    error::{LoxError, ParserErrorDetails, Result},
    expr::Expr,
    stmt::Stmt,
    token::{Token, TokenLiteral},
    token_kind::TokenKind,
};

/// Result used internally to interupt parsing until synchronization can occur
type ParserResult<T> = Result<T, ParserErrorDetails>;

/// Grammar:
///
/// program             -> declaration* EOF ;
///
/// declaration         -> varDecl | statement
/// varDecl             -> "var" IDENTIFIER ("=" expression)? ";" ;
///
/// statement           -> expressionStatement | printStatement ;
/// expressionStatement -> expression ";" ;
/// printStatement      -> "print" expression ";" ;
///
/// expression          -> assignment ;
/// assignment          -> IDENTIFIER "=" assignment | equality
/// equality            -> comparison ( ( "==" | "!=" ) comparison )* ;
/// comparison          -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
/// term                -> factor ( ( "-" | "+" ) factor )* ;
/// factor              -> unary ( ( "/" | "*" ) unary )* ;
/// unary               -> ( "!" | "-" ) unary | primary ;
/// primary             -> NUMBER | STRING | "nil" | "true" | "false"
///                      | "(" expression ")" | IDENTIFIER ;
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    parsing_errors: Vec<ParserErrorDetails>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            parsing_errors: vec![],
        }
    }

    pub fn parse(mut self) -> Result<Vec<Stmt>> {
        let mut statements: Vec<Stmt> = vec![];
        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(error) => self.parsing_errors.push(error),
            }
        }

        match self.parsing_errors.len() {
            0 => Ok(statements),
            _ => Err(LoxError::ParseError {
                statements,
                details: self.parsing_errors,
            }),
        }
    }

    fn declaration(&mut self) -> ParserResult<Stmt> {
        self.try_declaration().map_err(|error| {
            self.synchronize();
            error
        })
    }

    fn try_declaration(&mut self) -> ParserResult<Stmt> {
        if self.matches(&[TokenKind::Var]) {
            return self.var_declaration();
        }

        self.statement()
    }

    fn var_declaration(&mut self) -> ParserResult<Stmt> {
        // TODO get rid of clone
        let identifier = self
            .try_consume(TokenKind::Identifier, "Expected variable name.")?
            .clone();

        let initializer = match self.matches(&[TokenKind::Equal]) {
            true => Some(self.expression()?),
            false => None,
        };

        self.try_consume(
            TokenKind::Semicolon,
            "Expected ';' after variable declaration.",
        )?;

        Ok(Stmt::Var(identifier, initializer))
    }

    fn statement(&mut self) -> ParserResult<Stmt> {
        if self.matches(&[TokenKind::Print]) {
            return self.print_statement();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> ParserResult<Stmt> {
        let value = self.expression()?;
        self.try_consume(TokenKind::Semicolon, "Expected ';' after value.")?;

        Ok(Stmt::Print(value.into()))
    }

    fn expression_statement(&mut self) -> ParserResult<Stmt> {
        let value = self.expression()?;
        self.try_consume(TokenKind::Semicolon, "Expected ';' after expression.")?;

        Ok(Stmt::Expression(value.into()))
    }

    fn expression(&mut self) -> ParserResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Expr> {
        let expr = self.equality()?;

        if self.matches(&[TokenKind::Equal]) {
            // TODO get rid of clone
            let equal = self.previous().clone();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign(name, value.into()));
            }

            self.report_error(&equal, "Invalid assignment target.");
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ParserResult<Expr> {
        let mut expr = self.comparison()?;

        while self.matches(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            // TODO get rid of clone
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParserResult<Expr> {
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

    fn term(&mut self) -> ParserResult<Expr> {
        let mut expr = self.factor()?;

        while self.matches(&[TokenKind::Minus, TokenKind::Plus]) {
            // TODO get rid of clone
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParserResult<Expr> {
        let mut expr = self.unary()?;

        while self.matches(&[TokenKind::Slash, TokenKind::Star]) {
            // TODO get rid of clone
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParserResult<Expr> {
        if self.matches(&[TokenKind::Bang, TokenKind::Minus]) {
            // TODO get rid of clone
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, right.into()));
        }

        self.primary()
    }

    fn primary(&mut self) -> ParserResult<Expr> {
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
            // TODO get rid of clone & unwrap
            let literal = self.previous().clone().literal.unwrap();

            return Ok(Expr::Literal(literal));
        }

        if self.matches(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.try_consume(TokenKind::RightParen, "Expected ')' after expression.")?;

            return Ok(Expr::Grouping(expr.into()));
        }

        if self.matches(&[TokenKind::Identifier]) {
            // TODO get rid of clone
            return Ok(Expr::Variable(self.previous().clone()));
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

    fn try_consume(&mut self, kind: TokenKind, message: &str) -> ParserResult<&Token> {
        if self.check(kind) {
            return Ok(self.advance());
        }

        // TODO get rid of clone
        Err(self.report_error(&self.peek().clone(), message))
    }

    fn report_error(&mut self, token: &Token, message: &str) -> ParserErrorDetails {
        ParserErrorDetails {
            message: message.into(),
            token: token.clone(),
        }
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

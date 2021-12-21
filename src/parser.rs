use crate::{
    error::{LoxError, ParserErrorDetails, Result},
    expr::Expr,
    stmt::Stmt,
    token::Token,
    token_kind::TokenKind,
    value::Value,
};

const MAX_ARGUMENT_COUNT: usize = 255;

/// Result used internally to interupt parsing until synchronization can occur
type ParserResult<T> = Result<T, ParserErrorDetails>;

/// Grammar:
///
/// program             -> declaration* EOF ;
///
/// declaration         -> classDeclaration | varDeclaration
///                      | functionDeclaration | statement ;
/// classDeclaration    -> "class" IDENTIFIER "{" function* "}" ;
/// varDeclaration      -> "var" IDENTIFIER ( "=" expression )? ";" ;
/// functionDeclaration -> "fun" function ;
/// function            -> IDENTIFIER "(" parameters? ")" block ;
/// parameters          -> IDENTIFIER ( "," IDENTIFIER )* ;
///
/// statement           -> expressionStatement | printStatement | block
///                      | ifStatement | whileStatement | returnStatment ;
/// ifStatement         -> "if" "(" expression ")" statement
///                      ( "else" statement )? ;
/// whileStatement      -> "while" "(" expression ")" statement ;
/// forStatement        -> "for" "("
///                      ( varDeclaration | expressionStatement | ";" )
///                      expression? ";" expression?  ")" statement ;
/// expressionStatement -> expression ";" ;
/// printStatement      -> "print" expression ";" ;
/// block               -> "{" declaration* "}" ;
/// returnStatment      -> "return" expression? ";" ;
///
/// expression          -> assignment ;
/// assignment          -> ( call "." )? IDENTIFIER "=" assignment | logicOr ;
/// logicOr             -> logicAnd ( "or" logicAnd )* ;
/// logicAnd            -> equality ( "and" equality )* ;
/// equality            -> comparison ( ( "==" | "!=" ) comparison )* ;
/// comparison          -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
/// term                -> factor ( ( "-" | "+" ) factor )* ;
/// factor              -> unary ( ( "/" | "*" ) unary )* ;
/// unary               -> ( "!" | "-" ) unary | call ;
/// call                -> primary ( "(" arguments? ")" | "." IDENTIFIER )* ;
/// arguments           -> expression ( "," expression )* ;
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
        if self.matches(&[TokenKind::Class]) {
            return self.class_declaration();
        }

        if self.matches(&[TokenKind::Var]) {
            return self.var_declaration();
        }

        if self.matches(&[TokenKind::Fun]) {
            return self.function("function");
        }

        self.statement()
    }

    fn class_declaration(&mut self) -> ParserResult<Stmt> {
        let name = self
            .try_consume(TokenKind::Identifier, "Expected class name.")?
            .clone();

        self.try_consume(TokenKind::LeftBrace, "Expected '{' before class body.")?;

        let mut methods = vec![];
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.try_consume(TokenKind::RightBrace, "Expected '}' after class body.")?;

        Ok(Stmt::Class(name, methods))
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

    fn function(&mut self, kind: &str) -> ParserResult<Stmt> {
        let name = self
            .try_consume(TokenKind::Identifier, &format!("Expected {} name.", kind))?
            .clone();

        self.try_consume(
            TokenKind::LeftParen,
            &format!("Expected '(' after {} name.", kind),
        )?;

        let parameters = match self.check(TokenKind::RightParen) {
            true => vec![],
            false => self.parameters()?,
        };

        self.try_consume(TokenKind::RightParen, "Expeced ')' after parameters.")?;

        self.try_consume(
            TokenKind::LeftBrace,
            &format!("Expected '{{' before {} body.", kind),
        )?;

        let body = self.block_statements()?;

        Ok(Stmt::Function(name, parameters, body))
    }

    fn parameters(&mut self) -> ParserResult<Vec<Token>> {
        let mut parameters = vec![];

        loop {
            if parameters.len() >= MAX_ARGUMENT_COUNT {
                self.report_warning(
                    self.peek().clone(),
                    &format!("Can't have more than {} arguments.", MAX_ARGUMENT_COUNT),
                );
            }

            parameters.push(
                self.try_consume(TokenKind::Identifier, "Expeced parameter name")?
                    .clone(),
            );

            if !self.matches(&[TokenKind::Comma]) {
                return Ok(parameters);
            }
        }
    }

    fn statement(&mut self) -> ParserResult<Stmt> {
        if self.matches(&[TokenKind::Print]) {
            return self.print_statement();
        }

        if self.matches(&[TokenKind::LeftBrace]) {
            return self.block();
        }

        if self.matches(&[TokenKind::If]) {
            return self.if_statement();
        }

        if self.matches(&[TokenKind::While]) {
            return self.while_statement();
        }

        if self.matches(&[TokenKind::For]) {
            return self.for_statement();
        }

        if self.matches(&[TokenKind::Return]) {
            return self.return_statement();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> ParserResult<Stmt> {
        let value = self.expression()?;
        self.try_consume(TokenKind::Semicolon, "Expected ';' after value.")?;

        Ok(Stmt::Print(value))
    }

    fn if_statement(&mut self) -> ParserResult<Stmt> {
        self.try_consume(TokenKind::LeftParen, "Expected '(' after if.")?;
        let condition = self.expression()?;
        self.try_consume(TokenKind::RightParen, "Expected ')' after condition.")?;

        let then_branch = self.statement()?;

        let else_branch = match self.matches(&[TokenKind::Else]) {
            true => Some(Box::new(self.statement()?)),
            _ => None,
        };

        Ok(Stmt::If(condition, then_branch.into(), else_branch))
    }

    fn while_statement(&mut self) -> ParserResult<Stmt> {
        self.try_consume(TokenKind::LeftParen, "Expected '(' after while.")?;
        let condition = self.expression()?;
        self.try_consume(TokenKind::RightParen, "Expected ')' after condition.")?;

        let body = self.statement()?;

        Ok(Stmt::While(condition, body.into()))
    }

    fn for_statement(&mut self) -> ParserResult<Stmt> {
        self.try_consume(TokenKind::LeftParen, "Expected '(' after for.")?;
        let initializer = if self.matches(&[TokenKind::Var]) {
            Some(self.var_declaration()?)
        } else if self.matches(&[TokenKind::Semicolon]) {
            None
        } else {
            Some(self.expression_statement()?)
        };

        let condition = match self.check(TokenKind::Semicolon) {
            true => Expr::Literal(Value::Boolean(true)),
            false => self.expression()?,
        };

        self.try_consume(TokenKind::Semicolon, "Expected ';' after loop condition.")?;

        let increment = match self.check(TokenKind::RightParen) {
            true => None,
            false => Some(self.expression()?),
        };

        self.try_consume(TokenKind::RightParen, "Expected ')' after for clauses.")?;

        let body = match (increment, self.statement()?) {
            (Some(inc), body) => Stmt::Block(vec![body, Stmt::Expression(inc)]),
            (_, body) => body,
        };

        let while_statement = Stmt::While(condition, body.into());

        Ok(match initializer {
            Some(init) => Stmt::Block(vec![init, while_statement]),
            None => while_statement,
        })
    }

    fn return_statement(&mut self) -> ParserResult<Stmt> {
        // TODO get rid of clone
        let keyword = self.previous().clone();
        let value = match self.check(TokenKind::Semicolon) {
            false => Some(self.expression()?),
            true => None,
        };

        self.try_consume(TokenKind::Semicolon, "Expected ';' after return.")?;

        Ok(Stmt::Return(keyword, value))
    }

    fn expression_statement(&mut self) -> ParserResult<Stmt> {
        let value = self.expression()?;
        self.try_consume(TokenKind::Semicolon, "Expected ';' after expression.")?;

        Ok(Stmt::Expression(value))
    }

    fn block(&mut self) -> ParserResult<Stmt> {
        Ok(Stmt::Block(self.block_statements()?))
    }

    fn block_statements(&mut self) -> ParserResult<Vec<Stmt>> {
        let mut statements = vec![];
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.try_consume(TokenKind::RightBrace, "Expected '}' after block.")?;

        Ok(statements)
    }

    fn expression(&mut self) -> ParserResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Expr> {
        let expr = self.or()?;

        if self.matches(&[TokenKind::Equal]) {
            // TODO get rid of clone
            let equal = self.previous().clone();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign(name, value.into()));
            }

            if let Expr::Get(object, name) = expr {
                return Ok(Expr::Set(object, name, value.into()));
            }

            self.parser_error(equal, "Invalid assignment target.");
        }

        Ok(expr)
    }

    fn or(&mut self) -> ParserResult<Expr> {
        let mut expr = self.and()?;

        while self.matches(&[TokenKind::Or]) {
            // TODO get rid of clone
            let operator = self.previous().clone();
            let right = self.and()?;

            expr = Expr::Logical(expr.into(), operator, right.into());
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParserResult<Expr> {
        let mut expr = self.equality()?;

        while self.matches(&[TokenKind::And]) {
            // TODO get rid of clone
            let operator = self.previous().clone();
            let right = self.equality()?;

            expr = Expr::Logical(expr.into(), operator, right.into());
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

        self.call()
    }

    fn call(&mut self) -> ParserResult<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.matches(&[TokenKind::LeftParen]) {
                expr = self.finish_call(expr)?;
                continue;
            }

            if self.matches(&[TokenKind::Dot]) {
                let name = self
                    .try_consume(TokenKind::Identifier, "Expected property name after '.'.")?
                    .clone();

                expr = Expr::Get(expr.into(), name);

                continue;
            }

            return Ok(expr);
        }
    }

    fn finish_call(&mut self, callee: Expr) -> ParserResult<Expr> {
        let arguments = match self.check(TokenKind::RightParen) {
            true => vec![],
            false => self.arguments()?,
        };

        let right_paren =
            self.try_consume(TokenKind::RightParen, "Expected ')' after arguments.")?;

        // TODO get rid of clone
        Ok(Expr::Call(callee.into(), arguments, right_paren.clone()))
    }

    fn arguments(&mut self) -> ParserResult<Vec<Expr>> {
        let mut args = vec![self.expression()?];

        while self.matches(&[TokenKind::Comma]) {
            if args.len() >= MAX_ARGUMENT_COUNT {
                self.report_warning(
                    self.peek().clone(),
                    &format!("Can't have more than {} arguments.", MAX_ARGUMENT_COUNT),
                );
            }

            args.push(self.expression()?);
        }

        Ok(args)
    }

    fn primary(&mut self) -> ParserResult<Expr> {
        if self.matches(&[TokenKind::True]) {
            return Ok(Expr::Literal(Value::Boolean(true)));
        }

        if self.matches(&[TokenKind::False]) {
            return Ok(Expr::Literal(Value::Boolean(false)));
        }

        if self.matches(&[TokenKind::Nil]) {
            return Ok(Expr::Literal(Value::Nil));
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

        if self.matches(&[TokenKind::This]) {
            return Ok(Expr::This(self.previous().clone()));
        }

        if self.matches(&[TokenKind::Identifier]) {
            // TODO get rid of clone
            return Ok(Expr::Variable(self.previous().clone()));
        }

        // TODO get rid of clone
        Err(self.parser_error(self.peek().clone(), "Expected expression."))
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
        Err(self.parser_error(self.peek().clone(), message))
    }

    fn parser_error(&mut self, token: Token, message: &str) -> ParserErrorDetails {
        ParserErrorDetails {
            message: message.into(),
            token,
        }
    }

    fn report_warning(&mut self, token: Token, message: &str) {
        let error = self.parser_error(token, message);
        self.parsing_errors.push(error);
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

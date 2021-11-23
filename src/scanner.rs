use crate::{
    error::{LoxError, Result, ScannerErrorDetails},
    token::{Token, TokenLiteral},
    token_kind::TokenKind,
};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    scanning_errors: Vec<ScannerErrorDetails>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            scanning_errors: vec![],
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token {
            kind: TokenKind::Eof,
            lexeme: "".into(),
            literal: None,
            line: self.line,
        });

        match self.scanning_errors.len() {
            0 => Ok(self.tokens),
            _ => Err(LoxError::ScanningError {
                tokens: self.tokens,
                details: self.scanning_errors,
            }),
        }
    }

    fn scan_token(&mut self) {
        match self.advance() {
            // Single-character tokens
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.add_token(TokenKind::Dot),
            '-' => self.add_token(TokenKind::Minus),
            '+' => self.add_token(TokenKind::Plus),
            ';' => self.add_token(TokenKind::Semicolon),
            '*' => self.add_token(TokenKind::Star),

            // One or two character tokens
            '!' if self.match_char('=') => self.add_token(TokenKind::BangEqual),
            '!' => self.add_token(TokenKind::Bang),

            '=' if self.match_char('=') => self.add_token(TokenKind::EqualEqual),
            '=' => self.add_token(TokenKind::Equal),

            '<' if self.match_char('=') => self.add_token(TokenKind::LessEqual),
            '<' => self.add_token(TokenKind::Less),

            '>' if self.match_char('=') => self.add_token(TokenKind::GreaterEqual),
            '>' => self.add_token(TokenKind::Greater),

            '/' if self.match_char('/') => {
                while self.peek() != '\n' && !self.is_at_end() {
                    self.advance();
                }
            }
            '/' => self.add_token(TokenKind::Slash),

            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,

            // Literals and keywords
            '"' => self.parse_string(),

            c if Scanner::is_digit(c) => self.parse_number(),

            c if Scanner::is_alpha(c) => self.parse_identifier(),

            c => self.report_error(self.line, &format!("Unexpected character '{}'.", c)),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.char_at(self.current);
        self.current += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.char_at(self.current)
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        self.char_at(self.current + 1)
    }

    fn add_token(&mut self, kind: TokenKind) {
        self.tokens.push(self.create_token(kind, None));
    }

    fn add_token_literal<T: Into<TokenLiteral>>(&mut self, kind: TokenKind, literal: T) {
        self.tokens
            .push(self.create_token(kind, Some(literal.into())));
    }

    fn create_token(&self, kind: TokenKind, literal: Option<TokenLiteral>) -> Token {
        let lexeme = self.str_at(self.start, self.current).to_string();
        Token {
            kind,
            lexeme,
            literal,
            line: self.line,
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.char_at(self.current) != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn parse_string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            self.report_error(self.line, "Unterminated string.");

            return;
        }

        // the closing "
        self.advance();

        // trim surrounding quotes
        let value = self.str_at(self.start + 1, self.current - 1).to_string();
        self.add_token_literal(TokenKind::String, value);
    }

    fn parse_number(&mut self) {
        while Scanner::is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && Scanner::is_digit(self.peek_next()) {
            // consume .
            self.advance();
        }

        while Scanner::is_digit(self.peek()) {
            self.advance();
        }

        let value = self
            .str_at(self.start, self.current)
            .to_string()
            .parse::<f64>()
            .unwrap();

        self.add_token_literal(TokenKind::Number, value);
    }

    fn parse_identifier(&mut self) {
        while Scanner::is_alpha_numeric(self.peek()) {
            self.advance();
        }

        self.add_token(match self.str_at(self.start, self.current) {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "for" => TokenKind::For,
            "fun" => TokenKind::Fun,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "print" => TokenKind::Print,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "var" => TokenKind::Var,
            "while" => TokenKind::While,
            _ => TokenKind::Identifier,
        });
    }

    fn char_at(&self, index: usize) -> char {
        self.source
            .chars()
            .nth(index)
            .expect("Unexpected end of input")
    }

    fn str_at(&self, start: usize, end: usize) -> &str {
        &self.source[start..end]
    }

    fn report_error(&mut self, line: usize, message: &str) {
        self.scanning_errors.push(ScannerErrorDetails {
            line,
            message: message.into(),
        });
    }

    fn is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }

    fn is_alpha(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_alpha_numeric(c: char) -> bool {
        Scanner::is_alpha(c) || Scanner::is_digit(c)
    }
}

use std::{
    env, fs,
    io::{self, BufRead, Write},
    path::Path,
    process,
};

use expr::AstPrinter;
use parser::Parser;
use token::Token;
use token_kind::TokenKind;

use crate::error::Result;
use crate::scanner::Scanner;

mod error;
mod expr;
mod parser;
mod scanner;
mod token;
mod token_kind;

struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Self { had_error: false }
    }

    fn run_file<T: AsRef<Path>>(&mut self, path: T) -> Result<()> {
        let source = fs::read_to_string(path.as_ref())?;
        self.run(source);

        if self.had_error {
            process::exit(65);
        }

        Ok(())
    }

    fn run_prompt(&mut self) -> Result<()> {
        let stdin = io::stdin();
        print!("> ");
        io::stdout().flush()?;

        for line in stdin.lock().lines() {
            match line {
                Ok(l) => {
                    self.run(l);
                    self.had_error = false;
                }
                Err(_) => break,
            }

            print!("> ");
            io::stdout().flush()?;
        }

        Ok(())
    }

    fn run(&mut self, source: String) {
        let scanner = {
            let mut s = Scanner::new(source);
            s.on_error(|line, message| self.scanner_error(line, "", &message));
            s
        };

        let tokens = scanner.scan_tokens();
        let parser = {
            let mut p = Parser::new(tokens);
            p.on_error(|token, message| self.parser_error(token, message));
            p
        };

        if let Some(expression) = parser.parse() {
            if self.had_error {
                return;
            }

            println!("{}", AstPrinter.print(expression));
        }
    }

    fn scanner_error(&mut self, line: usize, _at: &str, message: &str) {
        self.report_error(line, "", message);
    }

    fn parser_error(&mut self, token: &Token, message: &str) {
        let at = match token.kind {
            TokenKind::Eof => " at end".to_string(),
            _ => format!(" at '{}'", token.lexeme),
        };

        self.report_error(token.line, &at, message)
    }

    fn report_error(&mut self, line: usize, at: &str, message: &str) {
        eprintln!("[line {}] Error{}: {}", line, at, message);
        self.had_error = true;
    }
}

fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let mut lox = Lox::new();

    match args.len() {
        0 => lox.run_prompt()?,
        1 => lox.run_file(&args[0])?,
        _ => {
            println!("Usage: lox-rs [script]");
            process::exit(64);
        }
    }

    Ok(())
}

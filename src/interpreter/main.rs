use std::{
    env, fs,
    io::{self, BufRead, Write},
    path::Path,
    process,
};

use error::{LoxError, ParserErrorDetails, ResolverErrorDetails, ScannerErrorDetails};
use interpreter::Interpreter;
use parser::Parser;
use resolver::Resolver;
use token_kind::TokenKind;

use crate::error::Result;
use crate::scanner::Scanner;

mod callable;
mod environment;
mod error;
mod expr;
mod interpreter;
mod native_functions;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;
mod token_kind;
mod value;

struct Lox {
    had_error: bool,
    had_runtime_error: bool,
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            had_error: false,
            had_runtime_error: false,
            interpreter: Interpreter::new(),
        }
    }

    fn run_file<T: AsRef<Path>>(&mut self, path: T) -> Result<()> {
        let source = fs::read_to_string(path.as_ref())?;
        self.run(source);

        if self.had_error {
            process::exit(65);
        }

        if self.had_runtime_error {
            process::exit(70);
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
        let tokens = match Scanner::new(source).scan_tokens() {
            Ok(tokens) => tokens,
            Err(LoxError::ScanningError { tokens, details }) => {
                self.report_scanning_error(&details);
                tokens
            }
            Err(error) => panic!("Unexpected error: {}", error),
        };

        let statements = match Parser::new(tokens).parse() {
            Ok(statements) => statements,
            Err(LoxError::ParseError {
                statements,
                details,
            }) => {
                self.report_parse_error(&details);
                statements
            }
            Err(error) => panic!("Unexpected error: {}", error),
        };

        if self.had_error {
            return;
        }

        match Resolver::new(&mut self.interpreter).resolve(&statements) {
            Err(LoxError::ResolutionError(details)) => self.report_resolution_error(&details),
            Err(error) => panic!("Unexpected error: {}", error),
            _ => (),
        };

        if self.had_error {
            return;
        }

        if let Err(errors) = self.interpreter.interpret(statements) {
            for error in errors {
                self.runtime_error(&error);
            }
        }
    }

    fn report_scanning_error(&mut self, details: &[ScannerErrorDetails]) {
        for detail in details {
            self.report_error(detail.line, "", &detail.message)
        }
    }

    fn report_parse_error(&mut self, details: &[ParserErrorDetails]) {
        for detail in details {
            let at = match detail.token.kind {
                TokenKind::Eof => " at end".to_string(),
                _ => format!(" at '{}'", detail.token.lexeme),
            };

            self.report_error(detail.token.line, &at, &detail.message)
        }
    }

    fn report_resolution_error(&mut self, details: &[ResolverErrorDetails]) {
        for detail in details {
            let at = match detail.token.kind {
                TokenKind::Eof => " at end".to_string(),
                _ => format!(" at '{}'", detail.token.lexeme),
            };

            self.report_error(detail.token.line, &at, &detail.message)
        }
    }

    fn runtime_error(&mut self, error: &LoxError) {
        let message = match error {
            LoxError::RuntimeError { message, token } => {
                format!("{}\n[line {}]", message, token.line)
            }
            error => format!("{}", error),
        };

        eprintln!("{}", message);
        self.had_runtime_error = true;
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

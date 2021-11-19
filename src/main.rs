use std::{
    env, fs,
    io::{self, BufRead, Write},
    path::Path,
    process,
};

use crate::scanner::Scanner;

mod scanner;
mod token;
mod token_kind;

type Result<T> = std::result::Result<T, anyhow::Error>;

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
        let mut scanner = Scanner::new(source);
        scanner.on_error(|line, message| self.report_error(line, "", &message));

        for token in scanner.scan_tokens() {
            println!("{}", token);
        }
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

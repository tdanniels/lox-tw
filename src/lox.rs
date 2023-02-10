use crate::scanner::Scanner;

use std::fs;
use std::io;
use std::io::Write;
use std::process;

use anyhow::Result;

pub struct Lox {
    had_error: bool,
}

impl Lox {
    pub fn new() -> Self {
        Self { had_error: false }
    }

    pub fn run_file(&mut self, path: &str) -> Result<()> {
        let bytes = fs::read(path)?;
        self.run(&String::from_utf8(bytes)?);
        if self.had_error {
            process::exit(65);
        }
        Ok(())
    }

    pub fn run_prompt(&mut self) -> Result<()> {
        let mut line = String::new();
        loop {
            print!("> ");
            io::stdout().flush()?;
            match io::stdin().read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    self.run(&line);
                    self.had_error = false
                }
                Err(error) => eprintln!("IO error: {error}"),
            }
            line.clear();
        }
        Ok(())
    }

    fn run(&mut self, source: &str) {
        let mut scanner = Scanner::new(source, |l, m| self.error(l, m));
        let tokens = scanner.scan_tokens();

        for token in tokens {
            println!("{token}");
        }
    }

    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    fn report(&mut self, line: usize, where_: &str, message: &str) {
        eprintln!("[line {line}] Error{where_}: {message}");
        self.had_error = true;
    }
}

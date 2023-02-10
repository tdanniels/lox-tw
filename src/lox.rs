use crate::parser::Parser;
use crate::pretty_printer::AstPrinter;
use crate::scanner::Scanner;
use crate::token::Token;
use crate::token_type::TokenType;

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

        // Don't leave a dangling prompt line on exit.
        println!();
        Ok(())
    }

    fn run(&mut self, source: &str) {
        let tokens = {
            let mut scanner = Scanner::new(source, |l, m| self.line_error(l, m));
            scanner.scan_tokens()
        };

        let expr = {
            let mut parser = Parser::new(&tokens, |t, m| self.token_error(t, m));
            parser.parse().expect("Unexpected parse error.")
        };

        if self.had_error {
            return;
        }

        println!("{}", AstPrinter::print(&expr.unwrap()))
    }

    pub fn line_error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    fn report(&mut self, line: usize, where_: &str, message: &str) {
        eprintln!("[line {line}] Error{where_}: {message}");
        self.had_error = true;
    }

    pub fn token_error(&mut self, token: &Token, message: &str) {
        if token.type_ == TokenType::Eof {
            self.report(token.line, " at end", message);
        } else {
            self.report(
                token.line,
                &(" at '".to_owned() + &token.lexeme + "'"),
                message,
            );
        }
    }
}

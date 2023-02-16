use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::runtime_error::RuntimeError;
use crate::scanner::Scanner;
use crate::token::Token;
use crate::token_type::TokenType;

use std::cell::RefCell;
use std::fs;
use std::io::{self, Write};
use std::process;

use anyhow::Result;

pub struct Lox {
    lox: LoxInternal,
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            lox: LoxInternal::new(),
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(&mut self, path: &str) -> Result<()> {
        self.lox.run_file(path, &mut self.interpreter)
    }

    pub fn run_prompt(&mut self) -> Result<()> {
        self.lox.run_prompt(&mut self.interpreter)
    }
}

pub struct LoxInternal {
    had_error: RefCell<bool>,
    had_runtime_error: RefCell<bool>,
}

impl LoxInternal {
    fn new() -> Self {
        Self {
            had_error: false.into(),
            had_runtime_error: false.into(),
        }
    }

    fn run_file(&mut self, path: &str, interpreter: &mut Interpreter) -> Result<()> {
        let bytes = fs::read(path)?;
        self.run(&String::from_utf8(bytes)?, interpreter);
        if *self.had_error.borrow() {
            process::exit(65);
        }
        if *self.had_runtime_error.borrow() {
            process::exit(70);
        }
        Ok(())
    }

    fn run_prompt(&self, interpreter: &mut Interpreter) -> Result<()> {
        let mut line = String::new();
        loop {
            print!("> ");
            io::stdout().flush()?;
            match io::stdin().read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    self.run(&line, interpreter);
                    *self.had_error.borrow_mut() = false
                }
                Err(error) => eprintln!("IO error: {error}"),
            }
            line.clear();
        }

        // Don't leave a dangling prompt line on exit.
        println!();
        Ok(())
    }

    fn run(&self, source: &str, interpreter: &mut Interpreter) {
        let tokens = Scanner::new(source, |l, m| self.line_error(l, m)).scan_tokens();

        let expression = Parser::new(&tokens, |t, m| self.token_error(t, m))
            .parse()
            .expect("Unexpected parse error.");

        if *self.had_error.borrow() {
            return;
        }

        interpreter.interpret(&expression.unwrap(), |e| self.runtime_error(e));
    }

    fn line_error(&self, line: usize, message: &str) {
        self.report(line, "", message);
    }

    fn report(&self, line: usize, where_: &str, message: &str) {
        eprintln!("[line {line}] Error{where_}: {message}");
        *self.had_error.borrow_mut() = true;
    }

    fn token_error(&self, token: &Token, message: &str) {
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

    fn runtime_error(&self, error: &RuntimeError) {
        eprintln!("{}\n[line {}]", error.message, error.token.line);
        *self.had_runtime_error.borrow_mut() = true;
    }
}

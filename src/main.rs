mod expr;
mod generate_ast;
mod interpreter;
mod lox;
mod object;
mod parser;
mod pretty_printer;
mod runtime_error;
mod scanner;
mod stmt;
mod token;
mod token_type;

use crate::lox::Lox;

use std::env;
use std::process;

use anyhow::Result;

fn main() -> Result<()> {
    let mut lox = Lox::new();
    let args: Vec<_> = env::args().collect();

    match args.len() {
        1 => lox.run_prompt()?,
        2 => lox.run_file(&args[0])?,
        _ => {
            eprintln!("Usage: lox-tw [script]");
            process::exit(64);
        }
    }

    Ok(())
}

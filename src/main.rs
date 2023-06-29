mod environment;
mod expr;
mod generate_ast;
mod interpreter;
mod lox;
mod lox_callable;
mod lox_class;
mod lox_function;
mod lox_instance;
mod lox_result;
mod lox_return;
mod object;
mod parser;
mod pretty_printer;
mod resolver;
mod runtime_error;
mod scanner;
mod stmt;
mod token;
mod token_type;
mod unique_id;

use crate::lox::Lox;
use crate::lox_result::Result;

use std::env;
use std::process;

fn main() -> Result<()> {
    let mut lox = Lox::new();
    let args: Vec<_> = env::args().collect();

    match args.len() {
        1 => lox.run_prompt()?,
        2 => lox.run_file(&args[1])?,
        _ => {
            eprintln!("Usage: lox-tw [script]");
            process::exit(64);
        }
    }

    Ok(())
}

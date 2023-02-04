mod expr;
mod lox;
mod object;
mod pretty_printer;
mod scanner;
mod token;
mod token_type;

use crate::lox::Lox;

use std::env;
use std::process;

use anyhow::Result;

fn main() -> Result<()> {
    let mut lox = Lox::new();
    let args: Vec<_> = env::args().collect();

    if args.len() > 2 {
        eprintln!("Usage: lox-tw [script]");
        process::exit(64);
    } else if args.len() == 2 {
        lox.run_file(&args[0])?;
    } else {
        lox.run_prompt()?;
    }
    Ok(())
}

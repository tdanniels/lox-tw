use crate::token::Token;

use std::error::Error;
use std::fmt::{self, Display};

use gc::Gc;

#[derive(Clone, Debug)]
pub struct RuntimeError {
    pub token: Gc<Token>,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Gc<Token>, message: &str) -> Self {
        Self {
            token,
            message: message.to_string(),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for RuntimeError {}

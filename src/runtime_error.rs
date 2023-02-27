use std::error::Error;
use std::fmt::{self, Display};
use std::rc::Rc;

use crate::token::Token;

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Rc<Token>,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Rc<Token>, message: &str) -> Self {
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

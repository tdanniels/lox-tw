use crate::token::Token;

use thiserror::Error;

#[derive(Debug, Error)]
#[error("{message}")]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    pub fn new(token: Token, message: &str) -> Self {
        Self {
            token,
            message: message.to_string(),
        }
    }
}

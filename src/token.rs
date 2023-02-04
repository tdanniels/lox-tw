use crate::object::Object;
use crate::token_type::TokenType;

use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub type_: TokenType,
    pub lexeme: String,
    pub literal: Option<Object>,
    pub line: usize,
}

impl Token {
    pub fn new(
        type_: TokenType,
        lexeme: &str,
        literal: Option<Object>,
        line: usize,
    ) -> Self {
        Token {
            type_,
            lexeme: lexeme.to_owned(),
            literal,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.literal {
            Some(literal) => write!(f, "{} {} {}", self.type_, self.lexeme, literal),
            None => write!(f, "{} {}", self.type_, self.lexeme),
        }
    }
}

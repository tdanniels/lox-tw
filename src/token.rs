use crate::object::Object;
use crate::token_type::TokenType;

use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub type_: TokenType,
    pub lexeme: String,
    pub literal: Object,
    pub line: usize,
}

impl Token {
    pub fn new(type_: TokenType, lexeme: &str, literal: Object, line: usize) -> Self {
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
        write!(f, "{} {} {}", self.type_, self.lexeme, self.literal)
    }
}

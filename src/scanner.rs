use crate::object::Object;
use crate::token::Token;
use crate::token_type::TokenType::{self, self as TT};

use gc::Gc;
use phf::phf_map;

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TT::And,
    "class" => TT::Class,
    "else" => TT::Else,
    "false" => TT::False,
    "for" => TT::For,
    "fun" => TT::Fun,
    "if" => TT::If,
    "nil" => TT::Nil,
    "or" => TT::Or,
    "print" => TT::Print,
    "return" => TT::Return,
    "super" => TT::Super,
    "this" => TT::This,
    "true" => TT::True,
    "var" => TT::Var,
    "while" => TT::While
};

pub struct Scanner<F>
where
    F: FnMut(usize, &str),
{
    source: String,
    error_handler: F,
    tokens: Vec<Gc<Token>>,
    start: usize,
    current: usize,
    line: usize,
}

fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

fn is_alpha(c: u8) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == b'_'
}

fn is_alpha_numeric(c: u8) -> bool {
    is_alpha(c) || is_digit(c)
}

impl<F> Scanner<F>
where
    F: FnMut(usize, &str),
{
    /// Panics if `source` is not valid ASCII.
    pub fn new(source: &str, error_handler: F) -> Self {
        assert!(source.is_ascii());
        Scanner {
            source: source.to_owned(),
            error_handler,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> Vec<Gc<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens
            .push(Token::new(TT::Eof, "", Object::Nil, self.line).into());
        self.tokens
    }

    fn scan_token(&mut self) {
        let c: u8 = self.advance();
        match c {
            b'(' => self.add_token(TT::LeftParen),
            b')' => self.add_token(TT::RightParen),
            b'{' => self.add_token(TT::LeftBrace),
            b'}' => self.add_token(TT::RightBrace),
            b',' => self.add_token(TT::Comma),
            b'.' => self.add_token(TT::Dot),
            b'-' => self.add_token(TT::Minus),
            b'+' => self.add_token(TT::Plus),
            b';' => self.add_token(TT::Semicolon),
            b'*' => self.add_token(TT::Star),
            b'!' => {
                let m = self.match_(b'=');
                self.add_token(if m { TT::BangEqual } else { TT::Bang })
            }
            b'=' => {
                let m = self.match_(b'=');
                self.add_token(if m { TT::EqualEqual } else { TT::Equal })
            }
            b'<' => {
                let m = self.match_(b'=');
                self.add_token(if m { TT::LessEqual } else { TT::Less })
            }
            b'>' => {
                let m = self.match_(b'=');
                self.add_token(if m { TT::GreaterEqual } else { TT::Greater })
            }
            b'/' => {
                if self.match_(b'/') {
                    while self.peek() != b'\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TT::Slash);
                }
            }
            b' ' | b'\r' | b'\t' => {}
            b'\n' => self.line += 1,
            b'"' => self.string(),
            x if is_digit(x) => self.number(),
            x if is_alpha(x) => self.identifier(),
            _ => (self.error_handler)(self.line, "Unexpected character."),
        }
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let ident = TT::Identifier;
        let type_ = KEYWORDS.get(text).unwrap_or(&ident);
        self.add_token(type_.clone());
    }

    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == b'.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }
        self.add_token_literal(
            TT::Number,
            Object::Number(
                self.source[self.start..self.current]
                    .parse()
                    .expect("BUG: failed to parse Number."),
            ),
        );
    }

    fn string(&mut self) {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            (self.error_handler)(self.line, "Unterminated string.");
            return;
        }

        self.advance();

        let value = self.source[self.start + 1..self.current - 1].to_owned();
        self.add_token_literal(TT::String, Object::String(value));
    }

    fn match_(&mut self, expected: u8) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.as_bytes()[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            return b'\0';
        }
        return self.source.as_bytes()[self.current];
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.source.len() {
            return b'\0';
        }
        self.source.as_bytes()[self.current + 1]
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> u8 {
        self.current += 1;
        return self.source.as_bytes()[self.current - 1];
    }

    fn add_token(&mut self, type_: TokenType) {
        self.add_token_literal(type_, Object::Nil);
    }

    fn add_token_literal(&mut self, type_: TokenType, literal: Object) {
        let text = &self.source.as_bytes()[self.start..self.current];
        self.tokens.push(
            Token::new(
                type_,
                std::str::from_utf8(text).expect("Invalid UTF-8"),
                literal,
                self.line,
            )
            .into(),
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid_tokens() {
        let mut error_count = 0usize;
        let source = r#"var a = 1; var b = "2";
                        print a + 2.5; print b;"#;
        let tokens = Scanner::new(source, |_, _| {
            error_count += 1;
        })
        .scan_tokens();
        assert_eq!(error_count, 0);
        assert_eq!(
            tokens,
            vec![
                Token::new(TT::Var, "var", Object::Nil, 1).into(),
                Token::new(TT::Identifier, "a", Object::Nil, 1).into(),
                Token::new(TT::Equal, "=", Object::Nil, 1).into(),
                Token::new(TT::Number, "1", Object::Number(1.0), 1).into(),
                Token::new(TT::Semicolon, ";", Object::Nil, 1).into(),
                Token::new(TT::Var, "var", Object::Nil, 1).into(),
                Token::new(TT::Identifier, "b", Object::Nil, 1).into(),
                Token::new(TT::Equal, "=", Object::Nil, 1).into(),
                Token::new(TT::String, "\"2\"", Object::String("2".to_string()), 1).into(),
                Token::new(TT::Semicolon, ";", Object::Nil, 1).into(),
                Token::new(TT::Print, "print", Object::Nil, 2).into(),
                Token::new(TT::Identifier, "a", Object::Nil, 2).into(),
                Token::new(TT::Plus, "+", Object::Nil, 2).into(),
                Token::new(TT::Number, "2.5", Object::Number(2.5), 2).into(),
                Token::new(TT::Semicolon, ";", Object::Nil, 2).into(),
                Token::new(TT::Print, "print", Object::Nil, 2).into(),
                Token::new(TT::Identifier, "b", Object::Nil, 2).into(),
                Token::new(TT::Semicolon, ";", Object::Nil, 2).into(),
                Token::new(TT::Eof, "", Object::Nil, 2).into(),
            ]
        );
    }

    #[test]
    fn unclosed_string() {
        let mut error_count = 0usize;
        let source = "var a = \"foo;";
        Scanner::new(source, |_, _| {
            error_count += 1;
        })
        .scan_tokens();
        assert_eq!(error_count, 1);
    }

    #[test]
    fn invalid_characters() {
        let mut error_count = 0usize;
        let source = "var a = #\nvar b = @";
        Scanner::new(source, |_, _| {
            error_count += 1;
        })
        .scan_tokens();
        assert_eq!(error_count, 2);
    }
}

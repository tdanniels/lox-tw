use crate::expr::{self, Expr};
use crate::object::Object;
use crate::token::Token;
use crate::token_type::TokenType::{self, self as TT};

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("parse error")]
struct ParseError;

pub struct Parser<F>
where
    F: FnMut(&Token, &str),
{
    tokens: Vec<Token>,
    current: usize,
    error_handler: F,
}

impl<F> Parser<F>
where
    F: FnMut(&Token, &str),
{
    pub fn new(tokens: &[Token], error_handler: F) -> Self {
        Self {
            tokens: tokens.to_vec(),
            current: 0,
            error_handler,
        }
    }

    pub fn parse(&mut self) -> Result<Option<Box<Expr>>> {
        match self.expression() {
            Ok(expr) => Ok(Some(expr)),
            Err(err) => match err.downcast_ref::<ParseError>() {
                Some(ParseError) => Ok(None), // Error already handled by error_handler.
                _ => Err(err),
            },
        }
    }

    fn expression(&mut self) -> Result<Box<Expr>> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.comparison()?;

        while self.match_(&[TT::BangEqual, TT::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = expr::Binary::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.term()?;

        while self.match_(&[TT::Greater, TT::GreaterEqual, TT::Less, TT::LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = expr::Binary::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.factor()?;

        while self.match_(&[TT::Minus, TT::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = expr::Binary::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Box<Expr>> {
        let mut expr = self.unary()?;

        while self.match_(&[TT::Slash, TT::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = expr::Binary::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Box<Expr>> {
        if self.match_(&[TT::Bang, TT::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(expr::Unary::make(operator, right));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Box<Expr>> {
        if self.match_(&[TT::False]) {
            return Ok(expr::Literal::make(Object::Boolean(false)));
        }
        if self.match_(&[TT::True]) {
            return Ok(expr::Literal::make(Object::Boolean(true)));
        }
        if self.match_(&[TT::Nil]) {
            return Ok(expr::Literal::make(Object::Nil));
        }

        if self.match_(&[TT::Number, TT::String]) {
            return Ok(expr::Literal::make(self.previous().clone().literal));
        }

        if self.match_(&[TT::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TT::RightParen, "Expect ')' after expression.")?;
            return Ok(expr::Grouping::make(expr));
        }

        let token = self.peek().clone();
        Err(self.error(&token, "Expect expression.").into())
    }

    fn match_(&mut self, types: &[TokenType]) -> bool {
        for type_ in types {
            if self.check(*type_) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, type_: TokenType, message: &str) -> Result<&Token> {
        if self.check(type_) {
            return Ok(self.advance());
        }

        let token = self.peek().clone();
        Err(self.error(&token, message).into())
    }

    fn check(&self, type_: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().type_ == type_
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().type_ == TT::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn error(&mut self, token: &Token, message: &str) -> ParseError {
        (self.error_handler)(token, message);
        ParseError
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().type_ == TT::Semicolon {
                return;
            }

            match self.peek().type_ {
                TT::Class
                | TT::Fun
                | TT::Var
                | TT::For
                | TT::If
                | TT::While
                | TT::Print
                | TT::Return => {
                    return;
                }
                _ => self.advance(),
            };
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pretty_printer::AstPrinter;

    #[test]
    fn simple_expr() {
        let mut error_count = 0usize;

        let tokens = vec![
            Token::new(TT::LeftParen, "(", Object::Nil, 1),
            Token::new(TT::Number, "1", Object::Number(1.0), 1),
            Token::new(TT::Plus, "+", Object::Nil, 1),
            Token::new(TT::Number, "2", Object::Number(2.0), 1),
            Token::new(TT::Minus, "-", Object::Nil, 1),
            Token::new(TT::Number, "0.5", Object::Number(0.5), 1),
            Token::new(TT::RightParen, ")", Object::Nil, 1),
            Token::new(TT::Star, "*", Object::Nil, 1),
            Token::new(TT::Minus, "-", Object::Nil, 1),
            Token::new(TT::Number, "4", Object::Number(4.0), 1),
            Token::new(TT::Eof, "", Object::Nil, 1),
        ];

        let mut parser = Parser::new(&tokens, |_, _| {
            error_count += 1;
        });

        let expr = parser.parse().unwrap().unwrap();

        assert_eq!(error_count, 0);
        assert_eq!(
            AstPrinter::print(&expr),
            "(* (group (- (+ 1 2) 0.5)) (- 4))"
        );
    }
}

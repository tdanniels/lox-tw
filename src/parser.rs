use crate::expr::{self, Expr};
use crate::object::Object;
use crate::token::Token;
use crate::token_type::TokenType::{self, self as TT};

use std::cell::RefCell;

use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("parse error")]
struct ParseError;

pub struct Parser<'a, F>
where
    F: FnMut(&'a Token, &str),
{
    tokens: &'a [Token],
    current: RefCell<usize>,
    error_handler: RefCell<F>,
}

impl<'a, F> Parser<'a, F>
where
    F: FnMut(&'a Token, &str) + 'a,
{
    pub fn new(tokens: &'a [Token], error_handler: F) -> Self {
        Self {
            tokens,
            current: 0.into(),
            error_handler: error_handler.into(),
        }
    }

    pub fn parse(self) -> Result<Option<Box<Expr<'a>>>> {
        match self.expression() {
            Ok(expr) => Ok(Some(expr)),
            Err(err) => match err.downcast_ref::<ParseError>() {
                Some(ParseError) => Ok(None), // Error already handled by error_handler.
                _ => Err(err),
            },
        }
    }

    fn expression(&self) -> Result<Box<Expr<'a>>> {
        self.equality()
    }

    fn equality(&self) -> Result<Box<Expr<'a>>> {
        let mut expr = self.comparison()?;

        while self.match_(&[TT::BangEqual, TT::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = expr::Binary::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&self) -> Result<Box<Expr<'a>>> {
        let mut expr = self.term()?;

        while self.match_(&[TT::Greater, TT::GreaterEqual, TT::Less, TT::LessEqual]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = expr::Binary::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&self) -> Result<Box<Expr<'a>>> {
        let mut expr = self.factor()?;

        while self.match_(&[TT::Minus, TT::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = expr::Binary::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&self) -> Result<Box<Expr<'a>>> {
        let mut expr = self.unary()?;

        while self.match_(&[TT::Slash, TT::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = expr::Binary::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&self) -> Result<Box<Expr<'a>>> {
        if self.match_(&[TT::Bang, TT::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(expr::Unary::make(operator, right));
        }

        self.primary()
    }

    fn primary(&self) -> Result<Box<Expr<'a>>> {
        if self.match_(&[TT::False]) {
            return Ok(expr::Literal::make(&Object::Boolean(false)));
        }
        if self.match_(&[TT::True]) {
            return Ok(expr::Literal::make(&Object::Boolean(true)));
        }
        if self.match_(&[TT::Nil]) {
            return Ok(expr::Literal::make(&Object::Nil));
        }

        if self.match_(&[TT::Number, TT::String]) {
            return Ok(expr::Literal::make(&self.previous().literal));
        }

        if self.match_(&[TT::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TT::RightParen, "Expect ')' after expression.")?;
            return Ok(expr::Grouping::make(expr));
        }

        let token = self.peek();
        Err(self.error(token, "Expect expression.").into())
    }

    fn match_(&self, types: &[TokenType]) -> bool {
        for type_ in types {
            if self.check(*type_) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&self, type_: TokenType, message: &str) -> Result<&Token> {
        if self.check(type_) {
            return Ok(self.advance());
        }

        let token = self.peek();
        Err(self.error(token, message).into())
    }

    fn check(&self, type_: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().type_ == type_
    }

    fn advance(&self) -> &'a Token {
        if !self.is_at_end() {
            *self.current.borrow_mut() += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().type_ == TT::Eof
    }

    fn peek(&self) -> &'a Token {
        &self.tokens[*self.current.borrow()]
    }

    fn previous(&self) -> &'a Token {
        &self.tokens[*self.current.borrow() - 1]
    }

    fn error(&self, token: &'a Token, message: &str) -> ParseError {
        (self.error_handler.borrow_mut())(token, message);
        ParseError
    }

    #[allow(unused)]
    fn synchronize(&self) {
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

    use std::cell::RefCell;

    #[test]
    fn simple_expr() {
        let error_count = RefCell::new(0usize);

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

        let expr = Parser::new(&tokens, |_, _| {
            *error_count.borrow_mut() += 1;
        })
        .parse()
        .unwrap()
        .unwrap();

        assert_eq!(*error_count.borrow(), 0);
        assert_eq!(
            AstPrinter::print(&expr),
            "(* (group (- (+ 1 2) 0.5)) (- 4))"
        );
    }
}

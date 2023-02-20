use crate::expr::{self, Expr};
use crate::object::Object;
use crate::stmt::{self, Stmt};
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

    pub fn parse(self) -> Result<Vec<Stmt<'a>>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(declaration_result) = self.declaration() {
                statements.push(declaration_result?);
            }
        }
        Ok(statements)
    }

    fn expression(&self) -> Result<Box<Expr<'a>>> {
        self.assignment()
    }

    fn declaration(&self) -> Option<Result<Stmt<'a>>> {
        let stmt_result = if self.match_(&[TT::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };
        match stmt_result {
            Err(error) => {
                return match error.downcast_ref::<ParseError>() {
                    Some(_) => {
                        self.synchronize();
                        None
                    }
                    None => Some(Err(error)),
                }
            }
            Ok(res) => Some(Ok(res)),
        }
    }

    fn statement(&self) -> Result<Stmt<'a>> {
        if self.match_(&[TT::Print]) {
            return self.print_statement();
        }
        if self.match_(&[TT::LeftBrace]) {
            return Ok(stmt::Block::var(self.block()?));
        }
        self.expression_statement()
    }

    fn print_statement(&self) -> Result<Stmt<'a>> {
        let value = self.expression()?;
        self.consume(TT::Semicolon, "Expect ';' after value.")?;
        Ok(stmt::Print::var(value))
    }

    fn var_declaration(&self) -> Result<Stmt<'a>> {
        let name = self.consume(TT::Identifier, "Expect variable name.")?;

        let initializer = if self.match_(&[TT::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TT::Semicolon, "Expect ';' after variable declaration.")?;
        Ok(stmt::Var::var(name, initializer))
    }

    fn expression_statement(&self) -> Result<Stmt<'a>> {
        let expr = self.expression()?;
        self.consume(TT::Semicolon, "Expect ';' after expression.")?;
        Ok(stmt::Expression::var(expr))
    }

    fn block(&self) -> Result<Vec<Stmt<'a>>> {
        let mut statements = Vec::new();

        while !self.check(TT::RightBrace) && !self.is_at_end() {
            if let Some(declaration) = self.declaration() {
                statements.push(declaration?);
            }
        }

        self.consume(TT::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn assignment(&self) -> Result<Box<Expr<'a>>> {
        let expr = self.equality()?;

        if self.match_(&[TT::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(var) = *expr {
                let name = var.name;
                return Ok(expr::Assign::make(name, value));
            }

            self.error(equals, "Invalid assignment target.");
        }

        Ok(expr)
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

        if self.match_(&[TT::Identifier]) {
            return Ok(expr::Variable::make(self.previous()));
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

    fn consume(&self, type_: TokenType, message: &str) -> Result<&'a Token> {
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
            Token::new(TT::Semicolon, ";", Object::Nil, 1),
            Token::new(TT::Eof, "", Object::Nil, 1),
        ];

        let statements = Parser::new(&tokens, |_, _| {
            *error_count.borrow_mut() += 1;
        })
        .parse()
        .unwrap();

        assert_eq!(*error_count.borrow(), 0);

        if let Stmt::Expression(expr_statement) = &statements[0] {
            assert_eq!(
                AstPrinter::print(&expr_statement.expression),
                "(* (group (- (+ 1 2) 0.5)) (- 4))"
            );
        } else {
            panic!("Expected an expression statement");
        }
    }
}

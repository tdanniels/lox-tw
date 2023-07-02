use crate::expr::{self, Expr};
use crate::lox_result::Result;
use crate::object::Object;
use crate::stmt::{self, Stmt};
use crate::token::Token;
use crate::token_type::TokenType::{self, self as TT};

use std::cell::RefCell;
use std::error::Error;
use std::fmt::{self, Debug, Display};

use gc::Gc;

#[derive(Debug)]
struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parse error")
    }
}

impl Error for ParseError {}

pub struct Parser<F>
where
    F: FnMut(Gc<Token>, &str),
{
    tokens: Vec<Gc<Token>>,
    current: RefCell<usize>,
    error_handler: RefCell<F>,
}

impl<F> Parser<F>
where
    F: FnMut(Gc<Token>, &str),
{
    pub fn new(tokens: Vec<Gc<Token>>, error_handler: F) -> Self {
        Self {
            tokens,
            current: 0.into(),
            error_handler: error_handler.into(),
        }
    }

    pub fn parse(self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(declaration_result) = self.declaration() {
                statements.push(declaration_result?);
            }
        }
        Ok(statements)
    }

    fn expression(&self) -> Result<Expr> {
        self.assignment()
    }

    fn declaration(&self) -> Option<Result<Stmt>> {
        let stmt_result = if self.match_(&[TT::Var]) {
            self.var_declaration()
        } else if self.match_(&[TT::Class]) {
            self.class_declaration()
        } else if self.match_(&[TT::Fun]) {
            self.function("function")
                .map(|f| Stmt::Function(Gc::new(f)))
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

    fn class_declaration(&self) -> Result<Stmt> {
        let name = self.consume(TT::Identifier, "Expect class name.")?;

        let superclass = if self.match_(&[TT::Less]) {
            self.consume(TT::Identifier, "Expect superclass name.")?;
            Some(expr::Variable::new(self.previous()).into())
        } else {
            None
        };

        self.consume(TT::LeftBrace, "Expect '{' before class body.")?;

        let mut methods = Vec::new();
        while !self.check(TT::RightBrace) && !self.is_at_end() {
            methods.push(Gc::new(self.function("method")?));
        }

        self.consume(TT::RightBrace, "Expect '}' after class body.")?;

        Ok(stmt::Class::make(name, superclass, methods))
    }

    fn statement(&self) -> Result<Stmt> {
        if self.match_(&[TT::For]) {
            return self.for_statement();
        }
        if self.match_(&[TT::If]) {
            return self.if_statement();
        }
        if self.match_(&[TT::Print]) {
            return self.print_statement();
        }
        if self.match_(&[TT::Return]) {
            return self.return_statement();
        }
        if self.match_(&[TT::While]) {
            return self.while_statement();
        }
        if self.match_(&[TT::LeftBrace]) {
            return Ok(stmt::Block::make(self.block()?));
        }
        self.expression_statement()
    }

    fn for_statement(&self) -> Result<Stmt> {
        self.consume(TT::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_(&[TT::Semicolon]) {
            None
        } else if self.match_(&[TT::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let mut condition = if self.check(TT::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TT::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if self.check(TT::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TT::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(incr) = increment {
            body = stmt::Block::make(vec![body, stmt::Expression::make(incr)]);
        }

        if condition.is_none() {
            condition = Some(expr::Literal::make(Object::Boolean(true)));
        }

        body = stmt::While::make(condition.unwrap(), body);

        if let Some(init) = initializer {
            body = stmt::Block::make(vec![init, body]);
        }

        Ok(body)
    }

    fn if_statement(&self) -> Result<Stmt> {
        self.consume(TT::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TT::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;
        let else_branch = if self.match_(&[TT::Else]) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(stmt::If::make(condition, then_branch, else_branch))
    }

    fn print_statement(&self) -> Result<Stmt> {
        let value = self.expression()?;
        self.consume(TT::Semicolon, "Expect ';' after value.")?;
        Ok(stmt::Print::make(value))
    }

    fn return_statement(&self) -> Result<Stmt> {
        let keyword = self.previous();
        let value = if self.check(TT::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(TT::Semicolon, "Expect ';' after return value.")?;

        Ok(stmt::Return::make(keyword, value))
    }

    fn var_declaration(&self) -> Result<Stmt> {
        let name = self.consume(TT::Identifier, "Expect variable name.")?;

        let initializer = if self.match_(&[TT::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TT::Semicolon, "Expect ';' after variable declaration.")?;
        Ok(stmt::Var::make(name, initializer))
    }

    fn while_statement(&self) -> Result<Stmt> {
        self.consume(TT::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TT::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;

        Ok(stmt::While::make(condition, body))
    }

    fn expression_statement(&self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TT::Semicolon, "Expect ';' after expression.")?;
        Ok(stmt::Expression::make(expr))
    }

    fn function(&self, kind: &str) -> Result<stmt::Function> {
        let name = self.consume(TT::Identifier, &format!("Expect {kind} name."))?;
        self.consume(TT::LeftParen, &format!("Expect '(' after {kind} name."))?;
        let mut parameters = Vec::new();
        if !self.check(TT::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    self.error(&self.peek(), "Can't have more than 255 parameters.");
                }

                parameters.push(self.consume(TT::Identifier, "Expect parameter name.")?);

                if !self.match_(&[TT::Comma]) {
                    break;
                }
            }
        }
        self.consume(TT::RightParen, "Expect ')' after parameters.")?;

        self.consume(TT::LeftBrace, &format!("Expect '{{' before {kind} body."))?;
        let body = self.block()?;
        Ok(stmt::Function::new(name, parameters, body))
    }

    fn block(&self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.check(TT::RightBrace) && !self.is_at_end() {
            if let Some(declaration) = self.declaration() {
                statements.push(declaration?);
            }
        }

        self.consume(TT::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn assignment(&self) -> Result<Expr> {
        let expr = self.or()?;

        if self.match_(&[TT::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(var) = &expr {
                let name = var.name.clone();
                return Ok(expr::Assign::make(name, value));
            } else if let Expr::Get(get) = &expr {
                return Ok(expr::Set::make(get.object.clone(), get.name.clone(), value));
            }

            self.error(&equals, "Invalid assignment target.");
        }

        Ok(expr)
    }

    fn or(&self) -> Result<Expr> {
        let mut expr = self.and()?;

        while self.match_(&[TT::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = expr::Logical::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn and(&self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while self.match_(&[TT::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = expr::Logical::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn equality(&self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.match_(&[TT::BangEqual, TT::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = expr::Binary::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&self) -> Result<Expr> {
        let mut expr = self.term()?;

        while self.match_(&[TT::Greater, TT::GreaterEqual, TT::Less, TT::LessEqual]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = expr::Binary::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.match_(&[TT::Minus, TT::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = expr::Binary::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.match_(&[TT::Slash, TT::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = expr::Binary::make(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&self) -> Result<Expr> {
        if self.match_(&[TT::Bang, TT::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(expr::Unary::make(operator, right));
        }

        self.call()
    }

    fn finish_call(&self, callee: Expr) -> Result<Expr> {
        let mut arguments = Vec::new();

        if !self.check(TT::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    self.error(&self.peek(), "Can't have more than 255 arguments.");
                }

                arguments.push(self.expression()?);
                if !self.match_(&[TT::Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(TT::RightParen, "Expect ')' after arguments.")?;

        Ok(expr::Call::make(callee, paren, arguments))
    }

    fn call(&self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_(&[TT::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_(&[TT::Dot]) {
                let name =
                    self.consume(TT::Identifier, "Expect property name after '.'.")?;
                expr = expr::Get::make(expr, name);
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&self) -> Result<Expr> {
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
            return Ok(expr::Literal::make(self.previous().literal.clone()));
        }

        if self.match_(&[TT::Super]) {
            let keyword = self.previous();
            self.consume(TT::Dot, "Expect '.' after 'super'.")?;
            let method = self.consume(TT::Identifier, "Expect superclass method name.")?;
            return Ok(expr::Super::make(keyword, method));
        }

        if self.match_(&[TT::This]) {
            return Ok(expr::This::make(self.previous()));
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
        Err(self.error(&token, "Expect expression.").into())
    }

    fn match_(&self, types: &[TokenType]) -> bool {
        for type_ in types {
            if self.check(type_.clone()) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&self, type_: TokenType, message: &str) -> Result<Gc<Token>> {
        if self.check(type_) {
            return Ok(self.advance());
        }

        let token = self.peek();
        Err(self.error(&token, message).into())
    }

    fn check(&self, type_: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().type_ == type_
    }

    fn advance(&self) -> Gc<Token> {
        if !self.is_at_end() {
            *self.current.borrow_mut() += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().type_ == TT::Eof
    }

    fn peek(&self) -> Gc<Token> {
        self.tokens[*self.current.borrow()].clone()
    }

    fn previous(&self) -> Gc<Token> {
        self.tokens[*self.current.borrow() - 1].clone()
    }

    fn error(&self, token: &Token, message: &str) -> ParseError {
        (self.error_handler.borrow_mut())(Gc::new(token.clone()), message);
        ParseError
    }

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
            Token::new(TT::LeftParen, "(", Object::Nil, 1).into(),
            Token::new(TT::Number, "1", Object::Number(1.0), 1).into(),
            Token::new(TT::Plus, "+", Object::Nil, 1).into(),
            Token::new(TT::Number, "2", Object::Number(2.0), 1).into(),
            Token::new(TT::Minus, "-", Object::Nil, 1).into(),
            Token::new(TT::Number, "0.5", Object::Number(0.5), 1).into(),
            Token::new(TT::RightParen, ")", Object::Nil, 1).into(),
            Token::new(TT::Star, "*", Object::Nil, 1).into(),
            Token::new(TT::Minus, "-", Object::Nil, 1).into(),
            Token::new(TT::Number, "4", Object::Number(4.0), 1).into(),
            Token::new(TT::Semicolon, ";", Object::Nil, 1).into(),
            Token::new(TT::Eof, "", Object::Nil, 1).into(),
        ];

        let statements = Parser::new(tokens, |_, _| {
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

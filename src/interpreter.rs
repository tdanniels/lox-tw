use crate::expr::{self, Expr};
use crate::object::Object::{
    self, Boolean as OBoolean, Nil as ONil, Number as ONumber, String as OString,
};
use crate::runtime_error::RuntimeError;
use crate::stmt::{self, Stmt};
use crate::token::Token;
use crate::token_type::TokenType as TT;

use anyhow::Result;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret<F>(&mut self, statements: &[Box<Stmt>], mut error_handler: F)
    where
        F: FnMut(&RuntimeError),
    {
        for statement in statements {
            match self.execute(statement) {
                Ok(_) => {}
                Err(error) => {
                    (error_handler)(
                        error
                            .downcast_ref::<RuntimeError>()
                            .expect("Unexpected error"),
                    );
                    return;
                }
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Expression(s) => self.visit_expression_stmt(s),
            Stmt::Print(s) => self.visit_print_stmt(s),
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object> {
        match expr {
            Expr::Binary(ex) => self.visit_binary_expr(ex),
            Expr::Grouping(ex) => self.visit_grouping_expr(ex),
            Expr::Literal(ex) => Ok(self.visit_literal_expr(ex)),
            Expr::Unary(ex) => self.visit_unary_expr(ex),
        }
    }

    fn visit_expression_stmt(&mut self, stmt: &stmt::Expression) -> Result<()> {
        self.evaluate(&stmt.expression)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: &stmt::Print) -> Result<()> {
        let value = self.evaluate(&stmt.expression)?;
        println!("{value}");
        Ok(())
    }

    fn visit_binary_expr(&mut self, expr: &expr::Binary) -> Result<Object> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        match expr.operator.type_ {
            TT::BangEqual => Ok(OBoolean(!is_equal(&left, &right))),
            TT::EqualEqual => Ok(OBoolean(is_equal(&left, &right))),
            TT::Greater => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(OBoolean(l > r))
            }
            TT::GreaterEqual => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(OBoolean(l >= r))
            }
            TT::Less => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(OBoolean(l < r))
            }
            TT::LessEqual => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(OBoolean(l <= r))
            }
            TT::Minus => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(ONumber(l - r))
            }
            TT::Plus => match (left, right) {
                (ONumber(l), ONumber(r)) => Ok(ONumber(l + r)),
                (OString(l), OString(r)) => Ok(OString(l + r.as_str())),
                _ => Err(RuntimeError::new(
                    expr.operator.clone(),
                    "Operands must be two numbers or two strings.",
                )
                .into()),
            },
            TT::Slash => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(ONumber(l / r))
            }
            TT::Star => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(ONumber(l * r))
            }
            _ => unreachable!(),
        }
    }

    fn visit_grouping_expr(&mut self, expr: &expr::Grouping) -> Result<Object> {
        self.evaluate(&expr.expression)
    }

    fn visit_literal_expr(&mut self, expr: &expr::Literal) -> Object {
        expr.value.clone()
    }

    fn visit_unary_expr(&mut self, expr: &expr::Unary) -> Result<Object> {
        let right = self.evaluate(&expr.right)?;

        match expr.operator.type_ {
            TT::Bang => Ok(OBoolean(!is_truthy(&right))),
            TT::Minus => {
                let r = check_number_operand(expr.operator, &right)?;
                Ok(ONumber(-r))
            }
            _ => unreachable!(),
        }
    }
}

fn check_number_operand(operator: &Token, operand: &Object) -> Result<f64> {
    if let ONumber(l) = operand {
        Ok(*l)
    } else {
        Err(RuntimeError::new(operator.clone(), "Operand must be a number.").into())
    }
}

fn check_number_operands(
    operator: &Token,
    left: &Object,
    right: &Object,
) -> Result<(f64, f64)> {
    if let (ONumber(l), ONumber(r)) = (left, right) {
        Ok((*l, *r))
    } else {
        Err(RuntimeError::new(operator.clone(), "Operands must be numbers.").into())
    }
}

fn is_truthy(object: &Object) -> bool {
    match object {
        ONil => false,
        OBoolean(b) => *b,
        _ => true,
    }
}

fn is_equal(a: &Object, b: &Object) -> bool {
    match (a, b) {
        // Mimic the behaviour of Java's NaN.equals(NaN) even though
        // it disagrees with IEEE-754.
        (ONumber(x), ONumber(y)) if x.is_nan() && y.is_nan() => true,
        _ => a == b,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::Parser;

    use std::cell::RefCell;

    #[test]
    fn evaluate() -> Result<()> {
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

        let mut interpreter = Interpreter::new();

        if let Stmt::Expression(expr_statement) = &*statements[0] {
            let res = interpreter.evaluate(&expr_statement.expression)?;
            assert_eq!(res, Object::Number(-10.0));
        } else {
            panic!("Expected an expression statement");
        }
        Ok(())
    }
}

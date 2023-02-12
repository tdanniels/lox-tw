use crate::expr::{self, Expr};
use crate::object::Object::{
    self, Boolean as OBoolean, Nil as ONil, Number as ONumber, String as OString,
};
use crate::runtime_error::RuntimeError;
use crate::token::Token;
use crate::token_type::TokenType as TT;

use anyhow::Result;

pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn interpret<F>(&mut self, expression: &Expr, mut error_handler: F)
    where
        F: FnMut(&RuntimeError),
    {
        match self.evaluate(expression) {
            Ok(value) => println!("{value}"),
            Err(error) => (error_handler)(
                error
                    .downcast_ref::<RuntimeError>()
                    .expect("Unexpected error"),
            ),
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

    fn visit_binary_expr(&mut self, expr: &expr::Binary) -> Result<Object> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        match expr.operator.type_ {
            TT::BangEqual => Ok(OBoolean(!Self::is_equal(&left, &right))),
            TT::EqualEqual => Ok(OBoolean(Self::is_equal(&left, &right))),
            TT::Greater => {
                Self::check_number_operands(&expr.operator, &left, &right)?;
                Ok(OBoolean(f64::try_from(left)? > f64::try_from(right)?))
            }
            TT::GreaterEqual => {
                Self::check_number_operands(&expr.operator, &left, &right)?;
                Ok(OBoolean(f64::try_from(left)? >= f64::try_from(right)?))
            }
            TT::Less => {
                Self::check_number_operands(&expr.operator, &left, &right)?;
                Ok(OBoolean(f64::try_from(left)? < f64::try_from(right)?))
            }
            TT::LessEqual => {
                Self::check_number_operands(&expr.operator, &left, &right)?;
                Ok(OBoolean(f64::try_from(left)? <= f64::try_from(right)?))
            }
            TT::Minus => {
                Self::check_number_operands(&expr.operator, &left, &right)?;
                Ok(ONumber(f64::try_from(left)? - f64::try_from(right)?))
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
                Self::check_number_operands(&expr.operator, &left, &right)?;
                Ok(ONumber(f64::try_from(left)? / f64::try_from(right)?))
            }
            TT::Star => {
                Self::check_number_operands(&expr.operator, &left, &right)?;
                Ok(ONumber(f64::try_from(left)? * f64::try_from(right)?))
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
            TT::Bang => Ok(OBoolean(!Self::is_truthy(&right))),
            TT::Minus => {
                Self::check_number_operand(&expr.operator, &right)?;
                Ok(ONumber(-f64::try_from(right)?))
            }
            _ => unreachable!(),
        }
    }

    fn check_number_operand(operator: &Token, operand: &Object) -> Result<()> {
        if let ONumber(_) = operand {
            Ok(())
        } else {
            Err(RuntimeError::new(operator.clone(), "Operand must be a number.").into())
        }
    }

    fn check_number_operands(operator: &Token, left: &Object, right: &Object) -> Result<()> {
        if let (ONumber(_), ONumber(_)) = (left, right) {
            Ok(())
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn evaluate() -> Result<()> {
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

        let expr = {
            let mut parser = Parser::new(&tokens, |_, _| {
                error_count += 1;
            });
            let expr = parser.parse().unwrap().unwrap();
            assert_eq!(error_count, 0);
            expr
        };

        let mut interpreter = Interpreter::new();
        let res = interpreter.evaluate(&expr)?;

        assert_eq!(res, Object::Number(-10.0));
        Ok(())
    }
}

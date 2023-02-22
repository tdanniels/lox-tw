use crate::environment::Environment;
use crate::expr::{self, Expr};
use crate::object::Object::{
    self, Boolean as OBoolean, Nil as ONil, Number as ONumber, String as OString,
};
use crate::runtime_error::RuntimeError;
use crate::stmt::{self, Stmt};
use crate::token::Token;
use crate::token_type::TokenType as TT;

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use anyhow::Result;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
    writer: Rc<RefCell<dyn io::Write>>,
}

impl Interpreter {
    pub fn new(writer: Rc<RefCell<dyn io::Write>>) -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new(None))),
            writer,
        }
    }

    pub fn interpret<F>(&mut self, statements: &[Stmt], mut error_handler: F)
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
            Stmt::Block(s) => self.visit_block_stmt(s),
            Stmt::Expression(s) => self.visit_expression_stmt(s),
            Stmt::If(s) => self.visit_if_stmt(s),
            Stmt::Print(s) => self.visit_print_stmt(s),
            Stmt::Var(s) => self.visit_var_stmt(s),
            Stmt::While(s) => self.visit_while_statement(s),
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Rc<RefCell<Environment>>,
    ) -> Result<()> {
        let previous = Rc::clone(&self.environment);
        self.environment = environment;

        for statement in statements {
            let result = self.execute(statement);
            if result.is_err() {
                self.environment = previous;
                return result;
            }
        }

        self.environment = previous;
        Ok(())
    }

    fn visit_block_stmt(&mut self, stmt: &stmt::Block) -> Result<()> {
        self.execute_block(
            &stmt.statements,
            Rc::new(RefCell::new(Environment::new(Some(Rc::clone(
                &self.environment,
            ))))),
        )?;
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Rc<Object>> {
        match expr {
            Expr::Assign(ex) => self.visit_assign_expr(ex),
            Expr::Binary(ex) => self.visit_binary_expr(ex),
            Expr::Grouping(ex) => self.visit_grouping_expr(ex),
            Expr::Literal(ex) => Ok(Rc::new(self.visit_literal_expr(ex))),
            Expr::Logical(ex) => self.visit_logical_expr(ex),
            Expr::Unary(ex) => self.visit_unary_expr(ex),
            Expr::Variable(ex) => self.visit_variable_expr(ex),
        }
    }

    fn visit_expression_stmt(&mut self, stmt: &stmt::Expression) -> Result<()> {
        self.evaluate(&stmt.expression)?;
        Ok(())
    }

    fn visit_if_stmt(&mut self, stmt: &stmt::If) -> Result<()> {
        if is_truthy(&*self.evaluate(&stmt.condition)?) {
            self.execute(&stmt.then_branch)?;
        } else if let Some(else_branch) = &stmt.else_branch {
            self.execute(else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: &stmt::Print) -> Result<()> {
        let value = self.evaluate(&stmt.expression)?;
        writeln!(self.writer.borrow_mut(), "{value}")?;
        Ok(())
    }

    fn visit_var_stmt(&mut self, stmt: &stmt::Var) -> Result<()> {
        let value = if let Some(initializer) = &stmt.initializer {
            self.evaluate(initializer)?
        } else {
            Rc::new(Object::Nil)
        };

        self.environment
            .borrow_mut()
            .define(&stmt.name.lexeme, value);
        Ok(())
    }

    fn visit_while_statement(&mut self, stmt: &stmt::While) -> Result<()> {
        while is_truthy(&*self.evaluate(&stmt.condition)?) {
            self.execute(&stmt.body)?;
        }
        Ok(())
    }

    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Result<Rc<Object>> {
        let value = self.evaluate(&expr.value)?;
        self.environment
            .borrow_mut()
            .assign(expr.name, Rc::clone(&value))?;
        Ok(value)
    }

    fn visit_binary_expr(&mut self, expr: &expr::Binary) -> Result<Rc<Object>> {
        let left = self.evaluate(&expr.left)?;
        let right = self.evaluate(&expr.right)?;

        match expr.operator.type_ {
            TT::BangEqual => Ok(Rc::new(OBoolean(!is_equal(&left, &right)))),
            TT::EqualEqual => Ok(Rc::new(OBoolean(is_equal(&left, &right)))),
            TT::Greater => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(Rc::new(OBoolean(l > r)))
            }
            TT::GreaterEqual => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(Rc::new(OBoolean(l >= r)))
            }
            TT::Less => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(Rc::new(OBoolean(l < r)))
            }
            TT::LessEqual => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(Rc::new(OBoolean(l <= r)))
            }
            TT::Minus => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(Rc::new(ONumber(l - r)))
            }
            TT::Plus => match (left.as_ref(), right.as_ref()) {
                (ONumber(l), ONumber(r)) => Ok(Rc::new(ONumber(l + r))),
                (OString(l), OString(r)) => Ok(Rc::new(OString(l.to_owned() + r.as_str()))),
                _ => Err(RuntimeError::new(
                    expr.operator.clone(),
                    "Operands must be two numbers or two strings.",
                )
                .into()),
            },
            TT::Slash => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(Rc::new(ONumber(l / r)))
            }
            TT::Star => {
                let (l, r) = check_number_operands(expr.operator, &left, &right)?;
                Ok(Rc::new(ONumber(l * r)))
            }
            _ => unreachable!(),
        }
    }

    fn visit_grouping_expr(&mut self, expr: &expr::Grouping) -> Result<Rc<Object>> {
        self.evaluate(&expr.expression)
    }

    fn visit_literal_expr(&mut self, expr: &expr::Literal) -> Object {
        expr.value.clone()
    }

    fn visit_logical_expr(&mut self, expr: &expr::Logical) -> Result<Rc<Object>> {
        let left = self.evaluate(&expr.left)?;

        match expr.operator.type_ {
            TT::Or => {
                if is_truthy(&left) {
                    return Ok(left);
                }
            }
            TT::And => {
                if !is_truthy(&left) {
                    return Ok(left);
                }
            }
            _ => unreachable!(),
        }

        self.evaluate(&expr.right)
    }

    fn visit_unary_expr(&mut self, expr: &expr::Unary) -> Result<Rc<Object>> {
        let right = self.evaluate(&expr.right)?;

        match expr.operator.type_ {
            TT::Bang => Ok(Rc::new(OBoolean(!is_truthy(&right)))),
            TT::Minus => {
                let r = check_number_operand(expr.operator, &right)?;
                Ok(Rc::new(ONumber(-r)))
            }
            _ => unreachable!(),
        }
    }

    fn visit_variable_expr(&mut self, expr: &expr::Variable) -> Result<Rc<Object>> {
        self.environment.borrow().get(expr.name)
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
    use crate::scanner::Scanner;

    use std::cell::RefCell;
    use std::str;

    fn positive_interpreter_test(source: &str, expected_output: &str) -> Result<()> {
        let error_count = RefCell::new(0usize);

        let tokens =
            Scanner::new(source, |_, _| *error_count.borrow_mut() += 1).scan_tokens();

        let statements = Parser::new(&tokens, |_, _| {
            *error_count.borrow_mut() += 1;
        })
        .parse()
        .unwrap();

        assert_eq!(*error_count.borrow(), 0);

        let output = Rc::new(RefCell::new(Vec::new()));
        let mut interpreter = Interpreter::new(output.clone());
        interpreter.interpret(&statements, |_| *error_count.borrow_mut() += 1);

        assert_eq!(*error_count.borrow(), 0);

        // First compare the stringified output/expected output in order to
        // get an error message in terms of strings if they don't match.
        assert_eq!(str::from_utf8(&output.borrow())?, expected_output);

        // This should always pass if the above assertion passed, but let's
        // be thorough.
        assert_eq!(*output.borrow(), expected_output.as_bytes());

        Ok(())
    }

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

        let mut interpreter = Interpreter::new(Rc::new(RefCell::new(std::io::stdout())));

        if let Stmt::Expression(expr_statement) = &statements[0] {
            let res = interpreter.evaluate(&expr_statement.expression)?;
            assert_eq!(*res, Object::Number(-10.0));
        } else {
            panic!("Expected an expression statement");
        }
        Ok(())
    }

    #[test]
    fn lexical_scope() -> Result<()> {
        let source = r"
            var a = 3; print a;
            {
                var a = 5; print a;
                {
                    var a = 7; print a;
                }
                print a;
            }
            print a;
            {
                a = 1; print a;
            }
            print a;
        ";
        let expected_output = "3\n5\n7\n5\n3\n1\n1\n";
        positive_interpreter_test(source, expected_output)
    }

    #[test]
    fn if_else() -> Result<()> {
        let source = r#"
            if (true) print "foo"; else print "bar";
            if (false) print "foo"; else print "bar";
        "#;
        let expected_output = "foo\nbar\n";
        positive_interpreter_test(source, expected_output)
    }

    #[test]
    fn and_or() -> Result<()> {
        let source = r#"
            var a = "a" or "x"; print a;
            var b = nil or "b"; print b;
            var c = false and 3; print c;
            var d = true and "d"; print d;
        "#;
        let expected_output = "a\nb\nfalse\nd\n";
        positive_interpreter_test(source, expected_output)
    }

    #[test]
    fn while_for() -> Result<()> {
        let source = r"
            var i = 0;
            while (i < 5) { print i; i = i + 1; }

            var a = 0;
            var temp;
            for (var b = 1; a < 60; b = temp + b) { print a; temp = a; a = b; }
        ";
        let expected_output = "0\n1\n2\n3\n4\n0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n55\n";
        positive_interpreter_test(source, expected_output)
    }
}

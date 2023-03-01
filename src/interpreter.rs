use crate::environment::Environment;
use crate::expr::{self, Expr};
use crate::lox_callable::{Clock, LoxCallable};
use crate::lox_function::LoxFunction;
use crate::lox_result::Result;
use crate::lox_return::Return;
use crate::object::Object::{
    self, Boolean as OBoolean, Callable as OCallable, Nil as ONil, Number as ONumber,
    String as OString,
};
use crate::runtime_error::RuntimeError;
use crate::stmt::{self, Stmt};
use crate::token::Token;
use crate::token_type::TokenType as TT;

use std::io::Write;

use gc::{Finalize, Gc, GcCell, Trace};

#[derive(Clone, Finalize, Trace)]
pub enum InterpreterOutput {
    StdOut,
    #[allow(unused)]
    ByteVec(Gc<GcCell<Vec<u8>>>),
}

#[derive(Finalize, Trace)]
pub struct Interpreter {
    pub globals: Gc<GcCell<Environment>>,
    environment: Gc<GcCell<Environment>>,
    output: InterpreterOutput,
}

impl Interpreter {
    pub fn new(output: InterpreterOutput) -> Self {
        let globals = Gc::new(GcCell::new(Environment::new(None)));

        globals.borrow_mut().define(
            "clock",
            Gc::new(OCallable(Gc::new(LoxCallable::Clock(Clock::new())))),
        );

        Self {
            globals: Gc::clone(&globals),
            environment: Gc::clone(&globals),
            output,
        }
    }

    pub fn interpret<F>(&mut self, statements: &[Stmt], mut error_handler: F)
    where
        F: FnMut(&RuntimeError),
    {
        for statement in statements {
            match self.execute(statement.clone()) {
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

    fn execute(&mut self, stmt: Stmt) -> Result<()> {
        match &stmt {
            Stmt::Block(s) => self.visit_block_stmt(s.clone()),
            Stmt::Expression(s) => self.visit_expression_stmt(s.clone()),
            Stmt::Function(s) => self.visit_function_stmt(s.clone()),
            Stmt::If(s) => self.visit_if_stmt(s.clone()),
            Stmt::Print(s) => self.visit_print_stmt(s.clone()),
            Stmt::Return(s) => self.visit_return_statement(s.clone()),
            Stmt::Var(s) => self.visit_var_stmt(s.clone()),
            Stmt::While(s) => self.visit_while_statement(s.clone()),
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Gc<GcCell<Environment>>,
    ) -> Result<()> {
        let previous = Gc::clone(&self.environment);
        self.environment = environment;

        for statement in statements {
            let result = self.execute(statement.clone());
            if result.is_err() {
                self.environment = previous;
                return result;
            }
        }

        self.environment = previous;
        Ok(())
    }

    fn visit_block_stmt(&mut self, stmt: Gc<stmt::Block>) -> Result<()> {
        self.execute_block(
            &stmt.statements,
            Gc::new(GcCell::new(Environment::new(Some(Gc::clone(
                &self.environment,
            ))))),
        )?;
        Ok(())
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Gc<Object>> {
        match &expr {
            Expr::Assign(ex) => self.visit_assign_expr(ex.clone()),
            Expr::Binary(ex) => self.visit_binary_expr(ex.clone()),
            Expr::Call(ex) => self.visit_call_expr(ex.clone()),
            Expr::Grouping(ex) => self.visit_grouping_expr(ex.clone()),
            Expr::Literal(ex) => self.visit_literal_expr(ex.clone()),
            Expr::Logical(ex) => self.visit_logical_expr(ex.clone()),
            Expr::Unary(ex) => self.visit_unary_expr(ex.clone()),
            Expr::Variable(ex) => self.visit_variable_expr(ex.clone()),
        }
    }

    fn visit_expression_stmt(&mut self, stmt: Gc<stmt::Expression>) -> Result<()> {
        self.evaluate(stmt.expression.clone())?;
        Ok(())
    }

    fn visit_function_stmt(&mut self, stmt: Gc<stmt::Function>) -> Result<()> {
        let function = Gc::new(LoxCallable::Function(LoxFunction::new(
            stmt.clone(),
            self.environment.clone(),
        )));
        self.environment
            .borrow_mut()
            .define(&stmt.name.lexeme, Gc::new(OCallable(function)));
        Ok(())
    }

    fn visit_if_stmt(&mut self, stmt: Gc<stmt::If>) -> Result<()> {
        if is_truthy(&*self.evaluate(stmt.condition.clone())?) {
            self.execute(stmt.then_branch.clone())?;
        } else if let Some(else_branch) = stmt.else_branch.clone() {
            self.execute(else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: Gc<stmt::Print>) -> Result<()> {
        let value = self.evaluate(stmt.expression.clone())?;
        match &self.output {
            InterpreterOutput::ByteVec(v) => writeln!(v.borrow_mut(), "{value}")?,
            InterpreterOutput::StdOut => println!("{value}"),
        }
        Ok(())
    }

    fn visit_return_statement(&mut self, stmt: Gc<stmt::Return>) -> Result<()> {
        let value = match &stmt.value {
            Some(expr) => self.evaluate(expr.clone())?,
            None => Gc::new(ONil),
        };

        Err(Return::new(value).into())
    }

    fn visit_var_stmt(&mut self, stmt: Gc<stmt::Var>) -> Result<()> {
        let value = if let Some(initializer) = stmt.initializer.clone() {
            self.evaluate(initializer)?
        } else {
            Gc::new(ONil)
        };

        self.environment
            .borrow_mut()
            .define(&stmt.name.lexeme, value);
        Ok(())
    }

    fn visit_while_statement(&mut self, stmt: Gc<stmt::While>) -> Result<()> {
        while is_truthy(&*self.evaluate(stmt.condition.clone())?) {
            self.execute(stmt.body.clone())?;
        }
        Ok(())
    }

    fn visit_assign_expr(&mut self, expr: Gc<expr::Assign>) -> Result<Gc<Object>> {
        let value = self.evaluate(expr.value.clone())?;
        self.environment
            .borrow_mut()
            .assign(&expr.name, Gc::clone(&value))?;
        Ok(value)
    }

    fn visit_binary_expr(&mut self, expr: Gc<expr::Binary>) -> Result<Gc<Object>> {
        let left = self.evaluate(expr.left.clone())?;
        let right = self.evaluate(expr.right.clone())?;

        let obj = match expr.operator.type_ {
            TT::BangEqual => OBoolean(!is_equal(&left, &right)),
            TT::EqualEqual => OBoolean(is_equal(&left, &right)),
            TT::Greater => {
                let (l, r) = check_number_operands(&expr.operator, &left, &right)?;
                OBoolean(l > r)
            }
            TT::GreaterEqual => {
                let (l, r) = check_number_operands(&expr.operator, &left, &right)?;
                OBoolean(l >= r)
            }
            TT::Less => {
                let (l, r) = check_number_operands(&expr.operator, &left, &right)?;
                OBoolean(l < r)
            }
            TT::LessEqual => {
                let (l, r) = check_number_operands(&expr.operator, &left, &right)?;
                OBoolean(l <= r)
            }
            TT::Minus => {
                let (l, r) = check_number_operands(&expr.operator, &left, &right)?;
                ONumber(l - r)
            }
            TT::Plus => match (left.as_ref(), right.as_ref()) {
                (ONumber(l), ONumber(r)) => ONumber(l + r),
                (OString(l), OString(r)) => OString(l.to_owned() + r.as_str()),
                _ => {
                    return Err(RuntimeError::new(
                        expr.operator.clone(),
                        "Operands must be two numbers or two strings.",
                    )
                    .into())
                }
            },
            TT::Slash => {
                let (l, r) = check_number_operands(&expr.operator, &left, &right)?;
                ONumber(l / r)
            }
            TT::Star => {
                let (l, r) = check_number_operands(&expr.operator, &left, &right)?;
                ONumber(l * r)
            }
            _ => unreachable!(),
        };
        Ok(Gc::new(obj))
    }

    fn visit_call_expr(&mut self, expr: Gc<expr::Call>) -> Result<Gc<Object>> {
        let callee = self.evaluate(expr.callee.clone())?;

        let arguments = {
            let mut arguments = Vec::new();
            for argument in expr.arguments.clone() {
                arguments.push(self.evaluate(argument)?);
            }
            arguments
        };

        if let OCallable(function) = &*callee {
            if arguments.len() != function.arity() {
                Err(RuntimeError::new(
                    expr.paren.clone(),
                    &format!(
                        "Expected {} arguments but got {}.",
                        function.arity(),
                        arguments.len()
                    ),
                )
                .into())
            } else {
                Ok(function.call(self, &arguments)?)
            }
        } else {
            Err(RuntimeError::new(
                expr.paren.clone(),
                "Can only call functions and classes.",
            )
            .into())
        }
    }

    fn visit_grouping_expr(&mut self, expr: Gc<expr::Grouping>) -> Result<Gc<Object>> {
        self.evaluate(expr.expression.clone())
    }

    fn visit_literal_expr(&mut self, expr: Gc<expr::Literal>) -> Result<Gc<Object>> {
        Ok(expr.value.clone())
    }

    fn visit_logical_expr(&mut self, expr: Gc<expr::Logical>) -> Result<Gc<Object>> {
        let left = self.evaluate(expr.left.clone())?;

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

        self.evaluate(expr.right.clone())
    }

    fn visit_unary_expr(&mut self, expr: Gc<expr::Unary>) -> Result<Gc<Object>> {
        let right = self.evaluate(expr.right.clone())?;

        match expr.operator.type_ {
            TT::Bang => Ok(Gc::new(OBoolean(!is_truthy(&right)))),
            TT::Minus => {
                let r = check_number_operand(&expr.operator, &right)?;
                Ok(Gc::new(ONumber(-r)))
            }
            _ => unreachable!(),
        }
    }

    fn visit_variable_expr(&mut self, expr: Gc<expr::Variable>) -> Result<Gc<Object>> {
        self.environment.borrow().get(&expr.name)
    }
}

fn check_number_operand(operator: &Token, operand: &Object) -> Result<f64> {
    if let ONumber(l) = operand {
        Ok(*l)
    } else {
        Err(RuntimeError::new(Gc::new(operator.clone()), "Operand must be a number.").into())
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
        Err(RuntimeError::new(Gc::new(operator.clone()), "Operands must be numbers.").into())
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

    use std::str;

    use gc::{Gc, GcCell};

    fn positive_interpreter_test(source: &str, expected_output: &str) -> Result<()> {
        let error_count = GcCell::new(0usize);

        let tokens =
            Scanner::new(source, |_, _| *error_count.borrow_mut() += 1).scan_tokens();

        let statements = Parser::new(tokens, |_, _| {
            *error_count.borrow_mut() += 1;
        })
        .parse()
        .unwrap();

        assert_eq!(*error_count.borrow(), 0);

        let output = Gc::new(GcCell::new(Vec::new()));
        let mut interpreter = Interpreter::new(InterpreterOutput::ByteVec(output.clone()));
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
        let error_count = GcCell::new(0usize);

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

        let mut interpreter = Interpreter::new(InterpreterOutput::StdOut);

        if let Stmt::Expression(expr_statement) = &statements[0] {
            let res = interpreter.evaluate(expr_statement.expression.clone())?;
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

    #[test]
    fn basic_fun() -> Result<()> {
        let source = r#"
            fun say_hi(first, last) {
                print "Hi, " + first + " " + last + "!";
            }

            say_hi("Foo", "Bar");
        "#;
        let expected_output = "Hi, Foo Bar!\n";
        positive_interpreter_test(source, expected_output)
    }

    #[test]
    fn fib() -> Result<()> {
        let source = r"
            fun fib(n) {
                if (n <= 1) return n;
                return fib(n - 2) + fib(n - 1);
            }

            for (var i = 0; i < 10; i = i + 1) {
                print fib(i);
            }
        ";
        let expected_output = "0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n";
        positive_interpreter_test(source, expected_output)
    }

    #[test]
    fn counter_closure() -> Result<()> {
        let source = r"
            fun make_counter() {
                var i = 0;
                fun count() {
                    i = i + 1;
                    print i;
                }
                return count;
            }

            var counter = make_counter();
            counter();
            counter();
            counter();
        ";
        let expected_output = "1\n2\n3\n";
        positive_interpreter_test(source, expected_output)
    }
}

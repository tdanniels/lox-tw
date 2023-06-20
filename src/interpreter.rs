use crate::environment::Environment;
use crate::expr::{self, Expr};
use crate::lox_callable::{Clock, LoxCallable};
use crate::lox_class::LoxClass;
use crate::lox_function::LoxFunction;
use crate::lox_result::Result;
use crate::lox_return::Return;
use crate::object::Object::{
    self, Boolean as OBoolean, Callable as OCallable, Class as OClass,
    Instance as OInstance, Nil as ONil, Number as ONumber, String as OString,
};
use crate::runtime_error::RuntimeError;
use crate::stmt::{self, Stmt};
use crate::token::Token;
use crate::token_type::TokenType as TT;

use std::collections::HashMap;
use std::io::Write;

use gc::{Finalize, Gc, GcCell, Trace};

#[derive(Clone, Finalize, Trace)]
pub enum InterpreterOutput {
    StdOut,
    #[allow(unused)]
    ByteVec(Gc<GcCell<Vec<u8>>>),
}

pub struct Interpreter {
    globals: Environment,
    locals: HashMap<usize, usize>,
    environment: Environment,
    output: InterpreterOutput,
}

impl Interpreter {
    pub fn new(output: InterpreterOutput) -> Self {
        let globals = Environment::new(None);

        globals.define(
            "clock",
            Gc::new(OCallable(Gc::new(LoxCallable::Clock(Clock::new())))),
        );

        Self {
            globals: globals.clone(),
            locals: HashMap::new(),
            environment: globals,
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
            Stmt::Class(s) => self.visit_class_stmt(s.clone()),
            Stmt::Expression(s) => self.visit_expression_stmt(s.clone()),
            Stmt::Function(s) => self.visit_function_stmt(s.clone()),
            Stmt::If(s) => self.visit_if_stmt(s.clone()),
            Stmt::Print(s) => self.visit_print_stmt(s.clone()),
            Stmt::Return(s) => self.visit_return_stmt(s.clone()),
            Stmt::Var(s) => self.visit_var_stmt(s.clone()),
            Stmt::While(s) => self.visit_while_stmt(s.clone()),
        }
    }

    pub fn resolve(&mut self, expr_id: usize, depth: usize) {
        self.locals.insert(expr_id, depth);
    }

    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        environment: Environment,
    ) -> Result<()> {
        let previous = self.environment.clone();
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
            Environment::new(Some(self.environment.clone())),
        )?;
        Ok(())
    }

    fn visit_class_stmt(&mut self, stmt: Gc<stmt::Class>) -> Result<()> {
        self.environment.define(&stmt.name.lexeme, ONil.into());

        let mut methods = HashMap::new();
        for method in &stmt.methods {
            let function = LoxFunction::new(
                method.clone(),
                self.environment.clone(),
                method.name.lexeme == "init",
            );
            methods.insert(method.name.lexeme.clone(), function);
        }

        let class = LoxClass::new(&stmt.name.lexeme, methods);
        self.environment.assign(&stmt.name, OClass(class).into())?;
        Ok(())
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Gc<Object>> {
        match &expr {
            Expr::Assign(ex) => self.visit_assign_expr(ex.clone()),
            Expr::Binary(ex) => self.visit_binary_expr(ex.clone()),
            Expr::Call(ex) => self.visit_call_expr(ex.clone()),
            Expr::Get(ex) => self.visit_get_expr(ex.clone()),
            Expr::Grouping(ex) => self.visit_grouping_expr(ex.clone()),
            Expr::Literal(ex) => self.visit_literal_expr(ex.clone()),
            Expr::Logical(ex) => self.visit_logical_expr(ex.clone()),
            Expr::Set(ex) => self.visit_set_expr(ex.clone()),
            Expr::This(ex) => self.visit_this_expr(ex.clone()),
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
            false,
        )));
        self.environment
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

    fn visit_return_stmt(&mut self, stmt: Gc<stmt::Return>) -> Result<()> {
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

        self.environment.define(&stmt.name.lexeme, value);
        Ok(())
    }

    fn visit_while_stmt(&mut self, stmt: Gc<stmt::While>) -> Result<()> {
        while is_truthy(&*self.evaluate(stmt.condition.clone())?) {
            self.execute(stmt.body.clone())?;
        }
        Ok(())
    }

    fn visit_assign_expr(&mut self, expr: Gc<expr::Assign>) -> Result<Gc<Object>> {
        let value = self.evaluate(expr.value.clone())?;

        if let Some(distance) = self.locals.get(&expr.id()) {
            self.environment
                .assign_at(*distance, &expr.name, Gc::clone(&value));
        } else {
            self.globals.assign(&expr.name, Gc::clone(&value))?;
        }

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
        let callee = {
            let callee = self.evaluate(expr.callee.clone())?;

            if let OClass(class) = &*callee {
                // TODO: it would be nice to drop this special case. This probably requires
                // converting LoxCallable into a trait.
                OCallable(LoxCallable::Class(class.clone()).into()).into()
            } else {
                callee
            }
        };

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

    fn visit_get_expr(&mut self, expr: Gc<expr::Get>) -> Result<Gc<Object>> {
        let object = self.evaluate(expr.object.clone())?;
        if let OInstance(instance) = &*object {
            return instance.get(&expr.name);
        }
        Err(RuntimeError::new(expr.name.clone(), "Only instances have properties.").into())
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

    fn visit_set_expr(&mut self, expr: Gc<expr::Set>) -> Result<Gc<Object>> {
        let object = self.evaluate(expr.object.clone())?;

        if let OInstance(instance) = &*object {
            let value = self.evaluate(expr.value.clone())?;
            instance.set(&expr.name, value.clone());
            Ok(value)
        } else {
            Err(RuntimeError::new(expr.name.clone(), "Only instances have fields.").into())
        }
    }

    fn visit_this_expr(&self, expr: Gc<expr::This>) -> Result<Gc<Object>> {
        self.look_up_variable(&expr.keyword, expr.id())
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
        self.look_up_variable(&expr.name, expr.id())
    }

    fn look_up_variable(&self, name: &Token, expr_id: usize) -> Result<Gc<Object>> {
        if let Some(distance) = self.locals.get(&expr_id) {
            Ok(self.environment.get_at(*distance, &name.lexeme))
        } else {
            self.globals.get(name)
        }
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
    use crate::resolver::Resolver;
    use crate::scanner::Scanner;

    use std::str;

    use gc::{Gc, GcCell};

    fn interpreter_test(
        source: &str,
        expected_output: &str,
        expected_error_count: usize,
        expected_error_message: Option<&str>,
    ) -> Result<()> {
        let mut error_count = 0usize;
        let mut error = None;

        let tokens = Scanner::new(source, |_, _| error_count += 1).scan_tokens();

        let statements = Parser::new(tokens, |_, _| {
            error_count += 1;
        })
        .parse()
        .unwrap();

        // Interpreter tests should always parse.
        assert_eq!(error_count, 0);

        let output = Gc::new(GcCell::new(Vec::new()));
        let mut interpreter = Interpreter::new(InterpreterOutput::ByteVec(output.clone()));

        Resolver::new(&mut interpreter, |_, _| {
            error_count += 1;
        })
        .resolve(&statements)
        .unwrap();

        // Interpreter tests should always resolve.
        assert_eq!(error_count, 0);

        interpreter.interpret(&statements, |err| {
            error_count += 1;
            error = Some(err.clone());
        });

        assert_eq!(error_count, expected_error_count);

        // First compare the stringified output/expected output in order to
        // get an error message in terms of strings if they don't match.
        assert_eq!(str::from_utf8(&output.borrow())?, expected_output);

        // This should always pass if the above assertion passed, but let's
        // be thorough.
        assert_eq!(*output.borrow(), expected_output.as_bytes());

        if let Some(expected_error_output) = expected_error_message {
            assert_eq!(&error.unwrap().message, expected_error_output);
        }

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

        Resolver::new(&mut interpreter, |_, _| {
            *error_count.borrow_mut() += 1;
        })
        .resolve(&statements)
        .unwrap();

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
        interpreter_test(source, expected_output, 0, None)
    }

    #[test]
    fn if_else() -> Result<()> {
        let source = r#"
            if (true) print "foo"; else print "bar";
            if (false) print "foo"; else print "bar";
        "#;
        let expected_output = "foo\nbar\n";
        interpreter_test(source, expected_output, 0, None)
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
        interpreter_test(source, expected_output, 0, None)
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
        interpreter_test(source, expected_output, 0, None)
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
        interpreter_test(source, expected_output, 0, None)
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
        interpreter_test(source, expected_output, 0, None)
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
        interpreter_test(source, expected_output, 0, None)
    }

    #[test]
    fn undefined_variable_in_fun() -> Result<()> {
        let source = r"
            fun foo() { a = 1; }
            foo();
        ";
        let expected_output = "";
        let expected_error_message = Some("Undefined variable a.");
        interpreter_test(source, expected_output, 1, expected_error_message)
    }

    #[test]
    fn static_scope() -> Result<()> {
        let source = r#"
            var a = "global";
            {
                fun show_a() {
                    print a;
                }

                show_a();
                var a = "block";
                show_a();
            }
        "#;
        let expected_output = "global\nglobal\n";
        interpreter_test(source, expected_output, 0, None)
    }

    #[test]
    fn simple_method_call() -> Result<()> {
        let source = r#"
            class Printer {
                print_twice(x) {
                    print x;
                    print x;
                }
            }
            Printer().print_twice(54);
        "#;
        let expected_output = "54\n54\n";
        interpreter_test(source, expected_output, 0, None)
    }

    #[test]
    fn simple_this_usage() -> Result<()> {
        let source = r#"
            class ThisXPrinter {
                print_this_x() {
                    print this.x;
                }
            }

            var p = ThisXPrinter();
            p.x = "A";

            p.print_this_x();

            var p2 = p;
            p2.x = "B";

            p.print_this_x();
            p2.print_this_x();

            p.x = "C";

            p.print_this_x();
            p2.print_this_x();

        "#;
        let expected_output = "A\nB\nB\nC\nC\n";
        interpreter_test(source, expected_output, 0, None)
    }

    #[test]
    fn simple_initializer() -> Result<()> {
        let source = r#"
            class Foo {
                init(x, y, z) {
                    this.x = x;
                    this.y = y;
                    this.z = z;
                }

                do_print() {
                    print this.x;
                    print this.y;
                    print this.z;
                }
            }

            var foo = Foo(3, 5, 9);
            foo.do_print();
        "#;
        let expected_output = "3\n5\n9\n";
        interpreter_test(source, expected_output, 0, None)
    }
}

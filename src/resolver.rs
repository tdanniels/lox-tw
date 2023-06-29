use crate::expr::{self, Expr};
use crate::interpreter::Interpreter;
use crate::lox_result::Result;
use crate::stmt::{self, Stmt};
use crate::token::Token;

use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Clone, Copy, PartialEq)]
enum ClassType {
    None,
    Class,
    SubClass,
}

pub struct Resolver<'a, F>
where
    F: FnMut(&Token, &str),
{
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<&'a str, bool>>,
    current_function: FunctionType,
    current_class: ClassType,
    error_handler: RefCell<F>,
}

impl<'a, F> Resolver<'a, F>
where
    F: FnMut(&Token, &str),
{
    pub fn new(interpreter: &'a mut Interpreter, error_handler: F) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
            error_handler: error_handler.into(),
        }
    }

    fn error(&self, token: &Token, message: &str) {
        (self.error_handler.borrow_mut())(token, message);
    }

    fn visit_block_stmt(&mut self, stmt: &'a stmt::Block) -> Result<()> {
        self.begin_scope();
        self.resolve_stmts(&stmt.statements)?;
        self.end_scope();

        Ok(())
    }

    fn visit_class_stmt(&mut self, stmt: &'a stmt::Class) -> Result<()> {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;

        self.declare(&stmt.name);
        self.define(&stmt.name);

        if let Some(superclass) = &stmt.superclass {
            if stmt.name.lexeme == superclass.name.lexeme {
                self.error(&superclass.name, "A class can't inherit from itself.");
            }
            self.current_class = ClassType::SubClass;
            self.visit_variable_expr(superclass)?;
        }

        if stmt.superclass.is_some() {
            self.begin_scope();
            self.scopes.last_mut().unwrap().insert("super", true);
        }

        self.begin_scope();
        self.scopes.last_mut().unwrap().insert("this", true);

        for method in &stmt.methods {
            let declaration = if method.name.lexeme == "init" {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            };
            self.resolve_function(method, declaration)?;
        }

        self.end_scope();

        if stmt.superclass.is_some() {
            self.end_scope();
        }

        self.current_class = enclosing_class;

        Ok(())
    }

    fn visit_expression_stmt(&mut self, stmt: &stmt::Expression) -> Result<()> {
        self.resolve_expr(&stmt.expression)?;
        Ok(())
    }

    fn visit_function_stmt(&mut self, stmt: &'a stmt::Function) -> Result<()> {
        self.declare(&stmt.name);
        self.define(&stmt.name);

        self.resolve_function(stmt, FunctionType::Function)?;
        Ok(())
    }

    fn visit_if_stmt(&mut self, stmt: &'a stmt::If) -> Result<()> {
        self.resolve_expr(&stmt.condition)?;
        self.resolve_stmt(&stmt.then_branch)?;
        if let Some(else_branch) = &stmt.else_branch {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, stmt: &stmt::Print) -> Result<()> {
        self.resolve_expr(&stmt.expression)?;
        Ok(())
    }

    fn visit_return_stmt(&mut self, stmt: &stmt::Return) -> Result<()> {
        if self.current_function == FunctionType::None {
            self.error(&stmt.keyword, "Can't return from top-level code.");
        }

        if let Some(value) = &stmt.value {
            if self.current_function == FunctionType::Initializer {
                self.error(&stmt.keyword, "Can't return a value from an initializer.");
            }

            self.resolve_expr(value)?;
        }

        Ok(())
    }

    fn visit_var_stmt(&mut self, stmt: &'a stmt::Var) -> Result<()> {
        self.declare(&stmt.name);
        if let Some(initializer) = &stmt.initializer {
            self.resolve_expr(initializer)?
        }
        self.define(&stmt.name);

        Ok(())
    }

    fn visit_while_stmt(&mut self, stmt: &'a stmt::While) -> Result<()> {
        self.resolve_expr(&stmt.condition)?;
        self.resolve_stmt(&stmt.body)?;
        Ok(())
    }

    fn visit_assign_expr(&mut self, expr: &expr::Assign) -> Result<()> {
        self.resolve_expr(&expr.value)?;
        self.resolve_local(expr.id(), &expr.name)?;
        Ok(())
    }

    fn visit_binary_expr(&mut self, expr: &expr::Binary) -> Result<()> {
        self.resolve_expr(&expr.left)?;
        self.resolve_expr(&expr.right)?;
        Ok(())
    }

    fn visit_call_expr(&mut self, expr: &expr::Call) -> Result<()> {
        self.resolve_expr(&expr.callee)?;

        for argument in &expr.arguments {
            self.resolve_expr(argument)?;
        }

        Ok(())
    }

    fn visit_get_expr(&mut self, expr: &expr::Get) -> Result<()> {
        self.resolve_expr(&expr.object)?;
        Ok(())
    }

    fn visit_grouping_expr(&mut self, expr: &expr::Grouping) -> Result<()> {
        self.resolve_expr(&expr.expression)?;
        Ok(())
    }

    fn visit_literal_expr(&mut self, _expr: &expr::Literal) -> Result<()> {
        Ok(())
    }

    fn visit_logical_expr(&mut self, expr: &expr::Logical) -> Result<()> {
        self.resolve_expr(&expr.left)?;
        self.resolve_expr(&expr.right)?;
        Ok(())
    }

    fn visit_set_expr(&mut self, expr: &expr::Set) -> Result<()> {
        self.resolve_expr(&expr.value)?;
        self.resolve_expr(&expr.object)?;
        Ok(())
    }

    fn visit_super_expr(&mut self, expr: &expr::Super) -> Result<()> {
        if self.current_class == ClassType::None {
            self.error(&expr.keyword, "Can't use 'super' outside of a class.");
        } else if self.current_class != ClassType::SubClass {
            self.error(
                &expr.keyword,
                "Can't use 'super' in a class with no superclass.",
            );
        }

        self.resolve_local(expr.id(), &expr.keyword)?;
        Ok(())
    }

    fn visit_this_expr(&mut self, expr: &expr::This) -> Result<()> {
        if self.current_class == ClassType::None {
            self.error(&expr.keyword, "Can't use 'this' outside of a class.");
            return Ok(());
        }

        self.resolve_local(expr.id(), &expr.keyword)?;
        Ok(())
    }

    fn visit_unary_expr(&mut self, expr: &expr::Unary) -> Result<()> {
        self.resolve_expr(&expr.right)?;
        Ok(())
    }

    fn visit_variable_expr(&mut self, expr: &expr::Variable) -> Result<()> {
        if self
            .scopes
            .last()
            .map_or(false, |s| s.get(&expr.name.lexeme.as_str()) == Some(&false))
        {
            self.error(
                &expr.name,
                "Can't read local variable in its own initializer.",
            );
        }

        self.resolve_local(expr.id(), &expr.name)?;
        Ok(())
    }

    pub fn resolve(&mut self, statements: &'a [stmt::Stmt]) -> Result<()> {
        self.resolve_stmts(statements)
    }

    fn resolve_stmts(&mut self, statements: &'a [stmt::Stmt]) -> Result<()> {
        for statement in statements {
            self.resolve_stmt(statement)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, statement: &'a stmt::Stmt) -> Result<()> {
        match statement {
            Stmt::Block(s) => self.visit_block_stmt(s),
            Stmt::Class(s) => self.visit_class_stmt(s),
            Stmt::Expression(s) => self.visit_expression_stmt(s),
            Stmt::Function(s) => self.visit_function_stmt(s),
            Stmt::If(s) => self.visit_if_stmt(s),
            Stmt::Print(s) => self.visit_print_stmt(s),
            Stmt::Return(s) => self.visit_return_stmt(s),
            Stmt::Var(s) => self.visit_var_stmt(s),
            Stmt::While(s) => self.visit_while_stmt(s),
        }
    }

    fn resolve_expr(&mut self, expr: &expr::Expr) -> Result<()> {
        match expr {
            Expr::Assign(ex) => self.visit_assign_expr(ex),
            Expr::Binary(ex) => self.visit_binary_expr(ex),
            Expr::Call(ex) => self.visit_call_expr(ex),
            Expr::Get(ex) => self.visit_get_expr(ex),
            Expr::Grouping(ex) => self.visit_grouping_expr(ex),
            Expr::Literal(ex) => self.visit_literal_expr(ex),
            Expr::Logical(ex) => self.visit_logical_expr(ex),
            Expr::Set(ex) => self.visit_set_expr(ex),
            Expr::Super(ex) => self.visit_super_expr(ex),
            Expr::This(ex) => self.visit_this_expr(ex),
            Expr::Unary(ex) => self.visit_unary_expr(ex),
            Expr::Variable(ex) => self.visit_variable_expr(ex),
        }
    }

    fn resolve_function(
        &mut self,
        function: &'a stmt::Function,
        type_: FunctionType,
    ) -> Result<()> {
        let enclosing_function = self.current_function;
        self.current_function = type_;

        self.begin_scope();
        for param in &function.params {
            self.declare(param);
            self.define(param);
        }
        self.resolve_stmts(&function.body)?;
        self.end_scope();
        self.current_function = enclosing_function;

        Ok(())
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop().expect("Scope stack underflow.");
    }

    fn declare(&mut self, name: &'a Token) {
        if let Some(scope) = self.scopes.last() {
            if scope.contains_key(name.lexeme.as_str()) {
                self.error(name, "Already a variable with this name in this scope.");
            }
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(&name.lexeme, false);
        }
    }

    fn define(&mut self, name: &'a Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(&name.lexeme, true);
        }
    }

    fn resolve_local(&mut self, expr_id: usize, name: &Token) -> Result<()> {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name.lexeme.as_str()) {
                self.interpreter.resolve(expr_id, i);
                break;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::interpreter::{Interpreter, InterpreterOutput};
    use crate::parser::Parser;
    use crate::scanner::Scanner;

    use gc::{Gc, GcCell};

    fn resolver_test(
        source: &str,
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

        // Resolver tests should always parse.
        assert_eq!(error_count, 0);

        let output = Gc::new(GcCell::new(Vec::new()));
        let mut interpreter = Interpreter::new(InterpreterOutput::ByteVec(output));

        Resolver::new(&mut interpreter, |_, err| {
            error_count += 1;
            error = Some(err.to_owned());
        })
        .resolve(&statements)
        .unwrap();

        assert_eq!(error_count, expected_error_count);

        if let Some(expected_error_output) = expected_error_message {
            assert_eq!(error.unwrap(), expected_error_output);
        }

        Ok(())
    }

    #[test]
    fn this_outside_class() -> Result<()> {
        let source = r"
            print this;
        ";
        let expected_error_message = Some("Can't use 'this' outside of a class.");
        resolver_test(source, 1, expected_error_message)
    }
}

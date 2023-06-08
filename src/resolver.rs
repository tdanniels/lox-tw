use crate::expr::{self, Expr};
use crate::interpreter::Interpreter;
use crate::lox_result::Result;
use crate::stmt::{self, Stmt};
use crate::token::Token;

use std::cell::RefCell;
use std::collections::HashMap;

use gc::Gc;

#[derive(Clone, Copy, PartialEq)]
enum FunctionType {
    None,
    Function,
    Method,
}

pub struct Resolver<'a, F>
where
    F: FnMut(Gc<Token>, &str),
{
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<&'a str, bool>>,
    current_function: FunctionType,
    error_handler: RefCell<F>,
}

impl<'a, F> Resolver<'a, F>
where
    F: FnMut(Gc<Token>, &str),
{
    pub fn new(interpreter: &'a mut Interpreter, error_handler: F) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            error_handler: error_handler.into(),
        }
    }
    fn visit_block_stmt(&mut self, stmt: &'a stmt::Block) -> Result<()> {
        self.begin_scope();
        self.resolve_stmts(&stmt.statements)?;
        self.end_scope();

        Ok(())
    }

    fn visit_class_stmt(&mut self, stmt: &'a stmt::Class) -> Result<()> {
        self.declare(&stmt.name);
        self.define(&stmt.name);

        for method in &stmt.methods {
            let declaration = FunctionType::Method;
            self.resolve_function(method, declaration)?;
        }

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
            (self.error_handler.borrow_mut())(
                stmt.keyword.clone(),
                "Can't return from top-level code.",
            );
        }

        if let Some(value) = &stmt.value {
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
            (self.error_handler.borrow_mut())(
                expr.name.clone(),
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
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name.lexeme.as_str()) {
                (self.error_handler.borrow_mut())(
                    Gc::new(name.clone()),
                    "Already a variable with this name in this scope.",
                );
            }
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

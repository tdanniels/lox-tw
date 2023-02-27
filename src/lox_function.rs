use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::lox_callable::LoxCallable;
use crate::lox_result::Result;
use crate::object::Object;
use crate::stmt;
use crate::unique_id::unique_id;

use std::cell::RefCell;
use std::fmt;
use std::iter::zip;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct LoxFunction {
    declaration: Rc<stmt::Function>,
    id: u128,
}

impl LoxFunction {
    pub fn new(declaration: Rc<stmt::Function>) -> Self {
        Self {
            declaration,
            id: unique_id(),
        }
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Rc<Object>],
    ) -> Result<Rc<Object>> {
        let mut environment = Environment::new(Some(interpreter.globals.clone()));
        for (param, arg) in zip(self.declaration.params.iter(), arguments.iter()) {
            environment.define(&param.lexeme, arg.clone());
        }

        interpreter
            .execute_block(&self.declaration.body, Rc::new(RefCell::new(environment)))?;
        Ok(Rc::new(Object::Nil))
    }

    fn id(&self) -> u128 {
        self.id
    }
}

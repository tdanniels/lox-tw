use crate::environment::Environment;
use crate::interpreter::Interpreter;
use crate::lox_instance::LoxInstance;
use crate::lox_result::Result;
use crate::lox_return::Return;
use crate::object::Object;
use crate::stmt;
use crate::unique_id::unique_u128;

use std::fmt;
use std::iter::zip;

use gc::{Finalize, Gc, Trace};

#[derive(Clone, Debug, Finalize, Trace)]
pub struct LoxFunction {
    is_initializer: bool,
    closure: Environment,
    declaration: Gc<stmt::Function>,
    id: u128,
}

impl LoxFunction {
    pub fn new(
        declaration: Gc<stmt::Function>,
        closure: Environment,
        is_initializer: bool,
    ) -> Self {
        Self {
            is_initializer,
            closure,
            declaration,
            id: unique_u128(),
        }
    }

    pub fn bind(&self, instance: Gc<LoxInstance>) -> LoxFunction {
        let environment = Environment::new(Some(self.closure.clone()));
        environment.define("this", Object::Instance(instance).into());
        LoxFunction::new(self.declaration.clone(), environment, self.is_initializer)
    }

    pub fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Gc<Object>],
    ) -> Result<Gc<Object>> {
        let environment = Environment::new(Some(self.closure.clone()));
        for (param, arg) in zip(self.declaration.params.iter(), arguments.iter()) {
            environment.define(&param.lexeme, arg.clone());
        }

        if let Err(err) = interpreter.execute_block(&self.declaration.body, environment) {
            if let Some(return_value) = err.downcast_ref::<Return>() {
                if self.is_initializer {
                    return Ok(self.closure.get_at(0, "this"));
                }
                return Ok(return_value.value.clone());
            } else {
                return Err(err);
            }
        }

        if self.is_initializer {
            return Ok(self.closure.get_at(0, "this"));
        }

        Ok(Gc::new(Object::Nil))
    }

    pub fn id(&self) -> u128 {
        self.id
    }
}

impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

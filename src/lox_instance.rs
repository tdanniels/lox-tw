use crate::lox_callable::LoxCallable;
use crate::lox_class::LoxClass;
use crate::lox_result::Result;
use crate::object::Object;
use crate::runtime_error::RuntimeError;
use crate::token::Token;

use std::{collections::HashMap, fmt};

use gc::{Finalize, Gc, GcCell, Trace};

#[derive(Clone, Debug, Finalize, PartialEq, Trace)]
pub struct LoxInstance {
    class: LoxClass,
    fields: Gc<GcCell<HashMap<String, Gc<Object>>>>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self {
            class,
            fields: GcCell::new(HashMap::new()).into(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Gc<Object>> {
        if let Some(field) = self.fields.borrow().get(&name.lexeme) {
            return Ok(field.clone());
        }

        if let Some(method) = self.class.find_method(&name.lexeme) {
            return Ok(
                Object::Callable(LoxCallable::Function(method.bind(self.clone()))).into(),
            );
        }

        Err(RuntimeError::new(
            name.clone().into(),
            &format!("Undefined property {}.", &name.lexeme),
        )
        .into())
    }

    pub fn set(&self, name: &Token, value: Gc<Object>) {
        self.fields.borrow_mut().insert(name.lexeme.clone(), value);
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} instance", self.class)
    }
}

use crate::lox_result::Result;
use crate::runtime_error::RuntimeError;
use crate::{object::Object, token::Token};

use std::collections::HashMap;

use gc::{Finalize, Gc, GcCell, Trace};

#[derive(Clone, Debug, Finalize, Trace)]
pub struct Environment {
    enclosing: Option<Gc<GcCell<Environment>>>,
    values: HashMap<String, Gc<Object>>,
}

impl Environment {
    pub fn new(enclosing: Option<Gc<GcCell<Environment>>>) -> Self {
        Self {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Gc<Object>> {
        self.values
            .get(&name.lexeme)
            .map_or_else(
                || {
                    if let Some(enclosing) = &self.enclosing {
                        enclosing.borrow().get(name).ok()
                    } else {
                        None
                    }
                },
                |value| Some(Gc::clone(value)),
            )
            .ok_or(
                RuntimeError::new(
                    Gc::new(name.clone()),
                    &format!("Undefined variable '{}'.", name.lexeme),
                )
                .into(),
            )
    }

    pub fn assign(&mut self, name: &Token, value: Gc<Object>) -> Result<()> {
        if let Some(v) = self.values.get_mut(&name.lexeme) {
            *v = value;
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)?;
            return Ok(());
        }

        Err(RuntimeError::new(
            Gc::new(name.clone()),
            &format!("Undefined variable {}.", name.lexeme),
        )
        .into())
    }

    pub fn define(&mut self, name: &str, value: Gc<Object>) {
        self.values.insert(name.to_owned(), Gc::clone(&value));
    }
}

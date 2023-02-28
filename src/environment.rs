use crate::lox_result::Result;
use crate::runtime_error::RuntimeError;
use crate::{object::Object, token::Token};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Rc<Object>>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            enclosing,
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Rc<Object>> {
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
                |value| Some(Rc::clone(value)),
            )
            .ok_or(
                RuntimeError::new(
                    Rc::new(name.clone()),
                    &format!("Undefined variable '{}'.", name.lexeme),
                )
                .into(),
            )
    }

    pub fn assign(&mut self, name: &Token, value: Rc<Object>) -> Result<()> {
        if let Some(v) = self.values.get_mut(&name.lexeme) {
            *v = value;
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().assign(name, value)?;
            return Ok(());
        }

        Err(RuntimeError::new(
            Rc::new(name.clone()),
            &format!("Undefined variable {}.", name.lexeme),
        )
        .into())
    }

    pub fn define(&mut self, name: &str, value: Rc<Object>) {
        self.values.insert(name.to_owned(), Rc::clone(&value));
    }
}

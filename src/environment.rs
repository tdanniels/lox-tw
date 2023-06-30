use crate::lox_result::Result;
use crate::runtime_error::RuntimeError;
use crate::{object::Object, token::Token};

use std::collections::HashMap;

use gc::{Finalize, Gc, GcCell, Trace};

#[derive(Clone, Debug, Finalize, Trace)]
pub struct Environment(Gc<GcCell<EnvironmentInternal>>);

impl Environment {
    pub fn new(enclosing: Option<Environment>) -> Self {
        Self(Gc::new(GcCell::new(EnvironmentInternal::new(enclosing))))
    }

    pub fn enclosing(&self) -> Option<Self> {
        self.0.borrow().enclosing.clone()
    }

    pub fn get(&self, name: &Token) -> Result<Object> {
        self.0.borrow().get(name)
    }

    pub fn assign(&self, name: &Token, value: Object) -> Result<()> {
        self.0.borrow_mut().assign(name, value)
    }

    pub fn define(&self, name: &str, value: Object) {
        self.0.borrow_mut().define(name, value)
    }

    fn ancestor(&self, distance: usize) -> Self {
        if distance == 0 {
            self.clone()
        } else {
            self.0
                .borrow()
                .enclosing
                .as_ref()
                .unwrap()
                .ancestor(distance - 1)
        }
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Object {
        self.ancestor(distance).0.borrow().get_at(name, distance)
    }

    pub fn assign_at(&self, distance: usize, name: &Token, value: Object) {
        self.ancestor(distance)
            .0
            .borrow_mut()
            .assign_at(name, value);
    }
}

#[derive(Clone, Debug, Finalize, Trace)]
struct EnvironmentInternal {
    enclosing: Option<Environment>,
    values: HashMap<String, Object>,
}

impl EnvironmentInternal {
    fn new(enclosing: Option<Environment>) -> Self {
        Self {
            enclosing,
            values: HashMap::new(),
        }
    }

    fn get(&self, name: &Token) -> Result<Object> {
        self.values
            .get(&name.lexeme)
            .map_or_else(
                || {
                    if let Some(enclosing) = &self.enclosing {
                        enclosing.0.borrow().get(name).ok()
                    } else {
                        None
                    }
                },
                |value| Some(value.clone()),
            )
            .ok_or(
                RuntimeError::new(
                    Gc::new(name.clone()),
                    &format!("Undefined variable '{}'.", name.lexeme),
                )
                .into(),
            )
    }

    fn assign(&mut self, name: &Token, value: Object) -> Result<()> {
        if let Some(v) = self.values.get_mut(&name.lexeme) {
            *v = value;
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            enclosing.0.borrow_mut().assign(name, value)?;
            return Ok(());
        }

        Err(RuntimeError::new(
            Gc::new(name.clone()),
            &format!("Undefined variable {}.", name.lexeme),
        )
        .into())
    }

    fn define(&mut self, name: &str, value: Object) {
        self.values.insert(name.to_owned(), value);
    }

    fn get_at(&self, name: &str, distance: usize) -> Object {
        self.values
            .get(name)
            .unwrap_or_else(|| {
                panic!("Didn't find local variable {name} at distance {distance}")
            })
            .clone()
    }

    fn assign_at(&mut self, name: &Token, value: Object) {
        self.values.insert(name.lexeme.to_owned(), value);
    }
}

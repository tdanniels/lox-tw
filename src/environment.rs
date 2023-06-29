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

    pub fn get(&self, name: &Token) -> Result<Gc<Object>> {
        self.0.borrow().get(name)
    }

    pub fn assign(&self, name: &Token, value: Gc<Object>) -> Result<()> {
        self.0.borrow_mut().assign(name, value)
    }

    pub fn define(&self, name: &str, value: Gc<Object>) {
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

    pub fn get_at(&self, distance: usize, name: &str) -> Gc<Object> {
        self.ancestor(distance).0.borrow().get_at(name, distance)
    }

    pub fn assign_at(&self, distance: usize, name: &Token, value: Gc<Object>) {
        self.ancestor(distance)
            .0
            .borrow_mut()
            .assign_at(name, value);
    }
}

#[derive(Clone, Debug, Finalize, Trace)]
struct EnvironmentInternal {
    enclosing: Option<Environment>,
    values: HashMap<String, Gc<Object>>,
}

impl EnvironmentInternal {
    fn new(enclosing: Option<Environment>) -> Self {
        Self {
            enclosing,
            values: HashMap::new(),
        }
    }

    fn get(&self, name: &Token) -> Result<Gc<Object>> {
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

    fn assign(&mut self, name: &Token, value: Gc<Object>) -> Result<()> {
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

    fn define(&mut self, name: &str, value: Gc<Object>) {
        self.values.insert(name.to_owned(), Gc::clone(&value));
    }

    fn get_at(&self, name: &str, distance: usize) -> Gc<Object> {
        self.values
            .get(name)
            .unwrap_or_else(|| {
                panic!("Didn't find local variable {name} at distance {distance}")
            })
            .clone()
    }

    fn assign_at(&mut self, name: &Token, value: Gc<Object>) {
        self.values
            .insert(name.lexeme.to_owned(), Gc::clone(&value));
    }
}

use crate::interpreter::Interpreter;
use crate::lox_class::LoxClass;
use crate::lox_function::LoxFunction;
use crate::lox_result::Result;
use crate::object::Object;
use crate::unique_id::unique_u128;

use std::fmt::{self, Debug, Display};
use std::time::{SystemTime, UNIX_EPOCH};

use gc::{Finalize, Trace};

#[derive(Clone, Debug, Finalize, Trace)]
pub enum LoxCallable {
    Class(LoxClass),
    Clock(Clock),
    Function(LoxFunction),
}

impl LoxCallable {
    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::Class(c) => c.arity(),
            LoxCallable::Clock(c) => c.arity(),
            LoxCallable::Function(c) => c.arity(),
        }
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Object],
    ) -> Result<Object> {
        match self {
            LoxCallable::Class(c) => c.call(interpreter, arguments),
            LoxCallable::Clock(c) => c.call(interpreter, arguments),
            LoxCallable::Function(c) => c.call(interpreter, arguments),
        }
    }

    pub fn id(&self) -> u128 {
        match self {
            LoxCallable::Class(c) => c.id(),
            LoxCallable::Clock(c) => c.id(),
            LoxCallable::Function(c) => c.id(),
        }
    }
}

impl Display for LoxCallable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoxCallable::Class(c) => Display::fmt(c, f),
            LoxCallable::Clock(c) => Display::fmt(c, f),
            LoxCallable::Function(c) => Display::fmt(c, f),
        }
    }
}

impl PartialEq for LoxCallable {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[derive(Clone, Debug, Finalize, Trace)]
pub struct Clock {
    id: u128,
}

impl Clock {
    pub fn new() -> Self {
        Self { id: unique_u128() }
    }

    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &mut Interpreter, _arguments: &[Object]) -> Result<Object> {
        Ok(Object::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards.")
                .as_secs_f64(),
        ))
    }

    fn id(&self) -> u128 {
        self.id
    }
}

impl Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<global fn>")
    }
}

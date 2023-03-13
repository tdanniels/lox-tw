use crate::lox_callable::LoxCallable;
use crate::lox_class::LoxClass;
use crate::lox_instance::LoxInstance;

use std::fmt;

use gc::{Finalize, Gc, Trace};

#[derive(Clone, Debug, Finalize, Trace)]
pub enum Object {
    Boolean(bool),
    Callable(Gc<LoxCallable>),
    Class(LoxClass),
    Instance(LoxInstance),
    Nil,
    Number(f64),
    String(String),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Boolean(x) => write!(f, "{x}"),
            Object::Callable(x) => write!(f, "{x}"),
            Object::Class(x) => write!(f, "{x}"),
            Object::Instance(x) => write!(f, "{x}"),
            Object::Nil => write!(f, "nil"),
            Object::Number(x) => write!(f, "{x}"),
            Object::String(x) => write!(f, "{x}"),
        }
    }
}

// Doing this instead of deriving PartialEq for Object due to
// https://github.com/rust-lang/rust/issues/78808.
impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Boolean(a), Object::Boolean(b)) => a == b,
            (Object::Callable(a), Object::Callable(b)) => a == b,
            (Object::Class(a), Object::Class(b)) => a == b,
            (Object::Instance(a), Object::Instance(b)) => a == b,
            (Object::Nil, Object::Nil) => true,
            (Object::Number(a), Object::Number(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            _ => false,
        }
    }
}

use std::fmt;
use std::rc::Rc;

use crate::lox_callable::LoxCallable;

#[derive(Clone, Debug)]
pub enum Object {
    Boolean(bool),
    Callable(Rc<dyn LoxCallable>),
    Nil,
    Number(f64),
    String(String),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Boolean(x) => write!(f, "{x}"),
            Object::Callable(x) => write!(f, "{x}"),
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
            (Object::Nil, Object::Nil) => true,
            (Object::Number(a), Object::Number(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            _ => false,
        }
    }
}

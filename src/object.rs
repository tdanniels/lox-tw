use std::fmt;

use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    Boolean(bool),
    Nil,
    Number(f64),
    String(String),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Boolean(x) => write!(f, "{x}"),
            Object::Nil => write!(f, "nil"),
            Object::Number(x) => write!(f, "{x}"),
            Object::String(x) => write!(f, "\"{x}\""),
        }
    }
}

#[derive(Debug, Error)]
pub enum CastError {
    #[error("object is not a number")]
    ToNumber,
}

impl TryFrom<Object> for f64 {
    type Error = CastError;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        if let Object::Number(num) = value {
            Ok(num)
        } else {
            Err(CastError::ToNumber)
        }
    }
}

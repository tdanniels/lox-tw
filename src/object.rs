use std::fmt;

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
            Object::String(x) => write!(f, "{x}"),
        }
    }
}

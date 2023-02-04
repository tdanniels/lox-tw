use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Object {
    String(String),
    Number(f64)
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::String(x) => write!(f, "\"{}\"", x),
            Object::Number(x) => write!(f, "{}", x),
        }
    }
}

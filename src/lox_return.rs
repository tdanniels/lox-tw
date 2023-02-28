use crate::object::Object;

use std::error::Error;
use std::fmt::{self, Display};
use std::rc::Rc;

#[derive(Debug)]
pub struct Return {
    pub value: Rc<Object>,
}

impl Return {
    pub fn new(value: Rc<Object>) -> Self {
        Self { value }
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Return<{}>", self.value)
    }
}

impl Error for Return {}

use crate::object::Object;

use std::error::Error;
use std::fmt::{self, Display};

#[derive(Debug)]
pub struct Return {
    pub value: Object,
}

impl Return {
    pub fn new(value: Object) -> Self {
        Self { value }
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Return<{}>", self.value)
    }
}

impl Error for Return {}

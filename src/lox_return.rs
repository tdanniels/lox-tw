use crate::object::Object;

use std::error::Error;
use std::fmt::{self, Display};

use gc::{Finalize, Gc, Trace};

#[derive(Debug, Finalize, Trace)]
pub struct Return {
    pub value: Gc<Object>,
}

impl Return {
    pub fn new(value: Gc<Object>) -> Self {
        Self { value }
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Return<{}>", self.value)
    }
}

impl Error for Return {}

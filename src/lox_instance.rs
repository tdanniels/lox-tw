use crate::lox_class::LoxClass;

use std::fmt;

use gc::{Finalize, Trace};

#[derive(Clone, Debug, Finalize, PartialEq, Trace)]
pub struct LoxInstance {
    class: LoxClass,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self { class }
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} instance", self.class)
    }
}

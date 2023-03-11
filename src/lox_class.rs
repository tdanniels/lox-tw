use crate::unique_id::unique_u128;

use std::fmt;

use gc::{Finalize, Trace};

#[derive(Debug, Finalize, Trace)]
pub struct LoxClass {
    name: String,
    id: u128,
}

impl LoxClass {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            id: unique_u128(),
        }
    }

    pub fn id(&self) -> u128 {
        self.id
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for LoxClass {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

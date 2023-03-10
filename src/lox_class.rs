use crate::interpreter::Interpreter;
use crate::lox_instance::LoxInstance;
use crate::lox_result::Result;
use crate::object::Object;
use crate::unique_id::unique_u128;

use std::fmt;

use gc::{Finalize, Gc, Trace};

#[derive(Clone, Debug, Finalize, Trace)]
pub struct LoxClass(Gc<LoxClassInternal>);

impl LoxClass {
    pub fn new(name: &str) -> Self {
        Self(LoxClassInternal::new(name).into())
    }

    pub fn arity(&self) -> usize {
        self.0.arity()
    }

    pub fn call(
        &self,
        _interpreter: &mut Interpreter,
        _arguments: &[Gc<Object>],
    ) -> Result<Gc<Object>> {
        let instance = Object::Instance(LoxInstance::new(self.clone())).into();
        Ok(instance)
    }

    pub fn id(&self) -> u128 {
        self.0.id()
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for LoxClass {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

#[derive(Clone, Debug, Finalize, Trace)]
struct LoxClassInternal {
    name: String,
    id: u128,
}

impl LoxClassInternal {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            id: unique_u128(),
        }
    }

    fn arity(&self) -> usize {
        0
    }

    fn id(&self) -> u128 {
        self.id
    }
}

impl fmt::Display for LoxClassInternal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for LoxClassInternal {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

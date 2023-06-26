use crate::interpreter::Interpreter;
use crate::lox_function::LoxFunction;
use crate::lox_instance::LoxInstance;
use crate::lox_result::Result;
use crate::object::Object;
use crate::unique_id::unique_u128;

use std::collections::HashMap;
use std::fmt;

use gc::{Finalize, Gc, Trace};

#[derive(Clone, Debug, Finalize, Trace)]
pub struct LoxClass(Gc<LoxClassInternal>);

impl LoxClass {
    pub fn new(
        name: &str,
        superclass: Option<LoxClass>,
        methods: HashMap<String, LoxFunction>,
    ) -> Self {
        Self(LoxClassInternal::new(name, superclass, methods).into())
    }

    pub fn find_method(&self, name: &str) -> Option<LoxFunction> {
        self.0.find_method(name)
    }

    pub fn arity(&self) -> usize {
        self.0.arity()
    }

    // We implement call here instead of in LoxClassInternal because we need
    // self to be Gc-wrapped, since we clone it.
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Gc<Object>],
    ) -> Result<Gc<Object>> {
        let instance = Gc::new(LoxInstance::new(self.clone()));

        if let Some(initializer) = self.find_method("init") {
            initializer
                .bind(instance.clone())
                .call(interpreter, arguments)?;
        }

        Ok(Object::Instance(instance).into())
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
    superclass: Option<LoxClass>,
    methods: HashMap<String, LoxFunction>,
    id: u128,
}

impl LoxClassInternal {
    fn new(
        name: &str,
        superclass: Option<LoxClass>,
        methods: HashMap<String, LoxFunction>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            superclass,
            methods,
            id: unique_u128(),
        }
    }

    fn find_method(&self, name: &str) -> Option<LoxFunction> {
        self.methods.get(name).cloned().or_else(|| {
            if let Some(superclass) = &self.superclass {
                superclass.find_method(name)
            } else {
                None
            }
        })
    }

    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("init") {
            initializer.arity()
        } else {
            0
        }
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

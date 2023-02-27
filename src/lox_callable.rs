use crate::interpreter::Interpreter;
use crate::lox_result::Result;
use crate::object::Object;

use std::fmt::{Debug, Display};
use std::rc::Rc;

pub trait LoxCallable: CloneLoxCallable + Debug + Display {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &[Rc<Object>],
    ) -> Result<Rc<Object>>;
    fn id(&self) -> u128;
}

impl PartialEq for dyn LoxCallable {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

pub trait CloneLoxCallable {
    fn clone_lox_callable(&self) -> Box<dyn LoxCallable>;
}

impl<T> CloneLoxCallable for T
where
    T: LoxCallable + Clone + 'static,
{
    fn clone_lox_callable(&self) -> Box<dyn LoxCallable> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn LoxCallable> {
    fn clone(&self) -> Self {
        self.clone_lox_callable()
    }
}

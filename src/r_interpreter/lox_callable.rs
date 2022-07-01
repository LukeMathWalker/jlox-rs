use super::lox_value::{Function, LoxValue};
use super::tree_walker::RuntimeErrorOrReturn;
use super::{Interpreter, RuntimeError};
use std::cell::RefCell;
use std::iter::zip;
use std::rc::Rc;

pub(super) trait LoxCallable {
    fn arity(&self) -> u8;
    fn call(
        self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, RuntimeError>;
}

impl LoxCallable for Function {
    fn arity(&self) -> u8 {
        // Safe because the parser enforces that we do not have more than 255 parameters
        self.0.parameters_binding_ids.len() as u8
    }

    fn call(
        self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, RuntimeError> {
        for (parameter, argument) in zip(self.0.parameters_binding_ids, arguments) {
            interpreter
                .bindings
                .insert(parameter, Rc::new(RefCell::new(argument)));
        }
        for statement in self.0.body {
            if let Err(e) = interpreter._execute(statement) {
                return match e {
                    RuntimeErrorOrReturn::RuntimeError(e) => Err(e),
                    RuntimeErrorOrReturn::Return(v) => Ok(v.0),
                };
            }
        }
        Ok(LoxValue::Null)
    }
}

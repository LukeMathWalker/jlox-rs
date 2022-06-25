use crate::interpreter::lox_value::{Function, LoxValue};
use crate::interpreter::tree_walker::RuntimeErrorOrReturn;
use crate::{Interpreter, RuntimeError};
use std::iter::zip;
use std::rc::Rc;

pub(in crate::interpreter) trait LoxCallable {
    fn arity(&self) -> u8;
    fn call(
        self,
        interpreter: &Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, RuntimeError>;
}

impl LoxCallable for Function {
    fn arity(&self) -> u8 {
        // Safe because the parser enforces that we do not have more than 255 parameters
        self.declaration.parameters.len() as u8
    }

    fn call(
        self,
        interpreter: &Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, RuntimeError> {
        let mut scoped_interpreter = interpreter.fork(Rc::clone(&self.closure));

        for (parameter, argument) in zip(self.declaration.parameters, arguments) {
            (*scoped_interpreter.environment)
                .borrow_mut()
                .define(parameter.lexeme(), argument);
        }
        for statement in self.declaration.body {
            if let Err(e) = scoped_interpreter._execute(statement) {
                return match e {
                    RuntimeErrorOrReturn::RuntimeError(e) => Err(e),
                    RuntimeErrorOrReturn::Return(v) => Ok(v.0),
                };
            }
        }
        Ok(LoxValue::Null)
    }
}

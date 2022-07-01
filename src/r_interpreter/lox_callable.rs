use super::lox_value::{Function, LoxValue};
use super::tree_walker::RuntimeErrorOrReturn;
use super::{Interpreter, RuntimeError};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::zip;
use std::rc::Rc;

pub(super) trait LoxCallable {
    fn arity(&self) -> u8;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, RuntimeError>;
}

impl LoxCallable for Function {
    fn arity(&self) -> u8 {
        // Safe because the parser enforces that we do not have more than 255 parameters
        self.definition.parameters_binding_ids.len() as u8
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, RuntimeError> {
        let mut function_local_bindings = HashMap::new();
        for (parameter, argument) in zip(self.definition.parameters_binding_ids.iter(), arguments) {
            function_local_bindings.insert(*parameter, Rc::new(RefCell::new(argument)));
        }
        for (captured_binding_id, captured_value) in self.captured_environment.iter() {
            function_local_bindings.insert(*captured_binding_id, Rc::clone(captured_value));
        }
        // Let's temporarily mount the function-local bindings on the interpreter.
        let current_bindings =
            std::mem::replace(&mut interpreter.bindings, function_local_bindings);

        for statement in self.definition.body.clone() {
            if let Err(e) = interpreter._execute(statement) {
                // Let's restore the previous binding map.
                interpreter.bindings = current_bindings;
                return match e {
                    RuntimeErrorOrReturn::RuntimeError(e) => Err(e),
                    RuntimeErrorOrReturn::Return(v) => Ok(v.0),
                };
            }
        }

        // Let's restore the previous binding map.
        interpreter.bindings = current_bindings;
        Ok(LoxValue::Null)
    }
}

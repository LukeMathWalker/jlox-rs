use crate::interpreter::lox_value::LoxValue;
use crate::{Interpreter, RuntimeError};

pub(in crate::interpreter) trait LoxCallable {
    fn call(
        self,
        interpreter: &mut Interpreter,
        arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, RuntimeError>;
}

impl LoxCallable for LoxValue {
    fn call(
        self,
        _interpreter: &mut Interpreter,
        _arguments: Vec<LoxValue>,
    ) -> Result<LoxValue, RuntimeError> {
        todo!()
    }
}

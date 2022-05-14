use crate::interpreter::lox_value::LoxValue;
use crate::interpreter::tree_walker::RuntimeError;
use std::collections::HashMap;

pub(in crate::interpreter) struct Environment(HashMap<String, LoxValue>);

impl Environment {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn define(&mut self, variable_name: String, value: LoxValue) {
        self.0.insert(variable_name, value);
    }

    pub fn assign(&mut self, variable_name: String, value: LoxValue) -> Result<(), RuntimeError> {
        if self.0.contains_key(&variable_name) {
            self.0.insert(variable_name, value);
            Ok(())
        } else {
            Err(RuntimeError::undefined_variable(&variable_name))
        }
    }

    pub fn get_value(&self, variable_name: &str) -> Result<LoxValue, RuntimeError> {
        self.0
            .get(variable_name)
            .cloned()
            .ok_or_else(|| RuntimeError::undefined_variable(variable_name))
    }
}

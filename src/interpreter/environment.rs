use crate::interpreter::lox_value::LoxValue;
use crate::interpreter::tree_walker::RuntimeError;
use drop_bomb::DropBomb;
use std::collections::HashMap;

pub(in crate::interpreter) struct Environment {
    current_scope: Scope,
    parent_scopes: Vec<Scope>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            current_scope: Default::default(),
            parent_scopes: vec![],
        }
    }

    pub fn enter_scope(&mut self) -> ScopeGuard {
        let enclosing_scope = std::mem::replace(&mut self.current_scope, Scope::default());
        self.parent_scopes.push(enclosing_scope);
        ScopeGuard(DropBomb::new("You forgot to close a scope"))
    }

    pub fn exit_scope(&mut self, mut guard: ScopeGuard) {
        guard.0.defuse();
        let parent_scope = self.parent_scopes.pop().unwrap();
        self.current_scope = parent_scope;
    }

    pub fn define(&mut self, variable_name: String, value: LoxValue) {
        self.current_scope.define(variable_name, value);
    }

    pub fn assign(&mut self, variable_name: String, value: LoxValue) -> Result<(), RuntimeError> {
        if self.current_scope.assign(&variable_name, &value).is_ok() {
            return Ok(());
        }
        for scope in self.parent_scopes.iter_mut().rev() {
            if scope.assign(&variable_name, &value).is_ok() {
                return Ok(());
            }
        }
        Err(RuntimeError::undefined_variable(&variable_name))
    }

    pub fn get_value(&self, variable_name: &str) -> Result<LoxValue, RuntimeError> {
        if let Some(value) = self.current_scope.get_value(variable_name) {
            return Ok(value);
        }
        for scope in self.parent_scopes.iter().rev() {
            if let Some(value) = scope.get_value(variable_name) {
                return Ok(value);
            }
        }
        Err(RuntimeError::undefined_variable(variable_name))
    }
}

#[derive(Default, Debug)]
struct Scope(HashMap<String, LoxValue>);

impl Scope {
    pub fn define(&mut self, variable_name: String, value: LoxValue) {
        self.0.insert(variable_name, value);
    }

    pub fn assign(&mut self, variable_name: &str, value: &LoxValue) -> Result<(), ()> {
        if self.0.contains_key(variable_name) {
            self.0.insert(variable_name.to_owned(), value.to_owned());
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn get_value(&self, variable_name: &str) -> Option<LoxValue> {
        self.0.get(variable_name).cloned()
    }
}

/// `ScopeGuard` ensures, at runtime, that we never leave a scope unclosed.
/// The interpreter code has no way to defuse the drop bomb (the field is private outside of
/// this module) - the interpreter is forced to call [`Environment::exit_scope`], which gives us
/// a chance to change the currently active scope in the environment.
pub(in crate::interpreter) struct ScopeGuard(drop_bomb::DropBomb);

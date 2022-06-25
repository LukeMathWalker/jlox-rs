use crate::interpreter::lox_value::LoxValue;
use crate::interpreter::tree_walker::RuntimeError;
use drop_bomb::DropBomb;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, Default)]
pub struct Environment {
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

    pub(in crate::interpreter) fn enter_scope(&mut self) -> ScopeGuard {
        let enclosing_scope = std::mem::take(&mut self.current_scope);
        self.parent_scopes.push(enclosing_scope);
        ScopeGuard(DropBomb::new("You forgot to close a scope"))
    }

    pub(in crate::interpreter) fn exit_scope(&mut self, mut guard: ScopeGuard) {
        guard.0.defuse();
        let parent_scope = self.parent_scopes.pop().unwrap();
        self.current_scope = parent_scope;
    }

    pub(in crate::interpreter) fn define(&mut self, variable_name: String, value: LoxValue) {
        self.current_scope.define(variable_name, value);
    }

    pub(in crate::interpreter) fn assign(
        &mut self,
        variable_name: String,
        value: LoxValue,
    ) -> Result<(), RuntimeError> {
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

    pub(in crate::interpreter) fn get_value(
        &self,
        variable_name: &str,
    ) -> Result<LoxValue, RuntimeError> {
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

#[derive(Default, Debug, Clone)]
pub(in crate::interpreter) struct Scope(HashMap<String, Rc<RefCell<LoxValue>>>);

impl Scope {
    pub fn define(&mut self, variable_name: String, value: LoxValue) {
        self.0.insert(variable_name, Rc::new(RefCell::new(value)));
    }

    pub fn assign(&mut self, variable_name: &str, value: &LoxValue) -> Result<(), ()> {
        match self.0.get(variable_name) {
            None => Err(()),
            Some(slot) => {
                *slot.borrow_mut() = value.to_owned();
                Ok(())
            }
        }
    }

    pub fn get_value(&self, variable_name: &str) -> Option<LoxValue> {
        self.0.get(variable_name).map(|v| v.borrow().clone())
    }
}

/// `ScopeGuard` ensures, at runtime, that we never leave a scope unclosed.
/// The interpreter code has no way to defuse the drop bomb (the field is private outside of
/// this module) - the interpreter is forced to call [`Environment::exit_scope`], which gives us
/// a chance to change the currently active scope in the environment.
#[must_use = "Nested scopes must be closed!"]
pub(in crate::interpreter) struct ScopeGuard(drop_bomb::DropBomb);

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

    /// Return a reference to the top-level scope.
    pub fn globals(&self) -> &Scope {
        self.parent_scopes
            .first()
            .unwrap_or_else(|| &self.current_scope)
    }

    /// Create a new environment, starting from a pre-existing scope.
    ///
    /// [`Self::new_nested`] does not automatically create a new scope - you have to explicitly call
    /// [`Self::enter_scope`].
    pub fn new_nested(parent_scope: Scope) -> Self {
        Self {
            current_scope: parent_scope,
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

#[derive(Default, Debug, Clone)]
pub(in crate::interpreter) struct Scope(HashMap<String, LoxValue>);

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
#[must_use = "Nested scopes must be closed!"]
pub(in crate::interpreter) struct ScopeGuard(drop_bomb::DropBomb);

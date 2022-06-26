use crate::resolver::resolver::BindingStatus;
use crate::resolver::BindingId;
use drop_bomb::DropBomb;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Environment {
    current_scope: Scope,
    parent_scopes: Vec<Scope>,
    binding_id_cursor: u64,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            current_scope: Default::default(),
            parent_scopes: vec![],
            binding_id_cursor: 0,
        }
    }

    pub(in crate::resolver) fn enter_scope(&mut self) -> ScopeGuard {
        let enclosing_scope = std::mem::take(&mut self.current_scope);
        self.parent_scopes.push(enclosing_scope);
        ScopeGuard(DropBomb::new("You forgot to close a scope"))
    }

    pub(in crate::resolver) fn exit_scope(&mut self, mut guard: ScopeGuard) {
        guard.0.defuse();
        let parent_scope = self.parent_scopes.pop().unwrap();
        self.current_scope = parent_scope;
    }

    pub(in crate::resolver) fn define(&mut self, variable_name: String) -> BindingId {
        let new_binding_id = self.binding_id_cursor;
        self.binding_id_cursor += 1;
        self.current_scope.define(variable_name, new_binding_id);
        new_binding_id
    }

    pub(in crate::resolver) fn assign(
        &mut self,
        variable_name: String,
    ) -> Result<BindingId, anyhow::Error> {
        if let Ok(id) = self.current_scope.assign(&variable_name) {
            return Ok(id);
        }
        for scope in self.parent_scopes.iter_mut().rev() {
            if let Ok(id) = scope.assign(&variable_name) {
                return Ok(id);
            }
        }
        Err(anyhow::anyhow!(
            "Tried to assign a value to an undefined variable"
        ))
    }

    pub(in crate::resolver) fn get(
        &self,
        variable_name: &str,
    ) -> Result<(BindingId, BindingStatus), anyhow::Error> {
        if let Some(value) = self.current_scope.get(variable_name) {
            return Ok(value);
        }
        for scope in self.parent_scopes.iter().rev() {
            if let Some(value) = scope.get(variable_name) {
                return Ok(value);
            }
        }
        Err(anyhow::anyhow!("Tried to read an undefined variable"))
    }
}

#[derive(Default, Debug, Clone)]
pub(in crate::resolver) struct Scope(HashMap<String, (BindingId, BindingStatus)>);

impl Scope {
    pub fn define(&mut self, variable_name: String, binding_id: BindingId) {
        self.0
            .insert(variable_name, (binding_id, BindingStatus::Uninitialized));
    }

    pub fn assign(&mut self, variable_name: &str) -> Result<BindingId, ()> {
        match self.0.get_mut(variable_name) {
            None => Err(()),
            Some(slot) => {
                slot.1 = BindingStatus::Initialized;
                Ok(slot.0)
            }
        }
    }

    pub fn get(&self, variable_name: &str) -> Option<(BindingId, BindingStatus)> {
        self.0.get(variable_name).cloned()
    }
}

/// `ScopeGuard` ensures, at runtime, that we never leave a scope unclosed.
/// The interpreter code has no way to defuse the drop bomb (the field is private outside of
/// this module) - the interpreter is forced to call [`Environment::exit_scope`], which gives us
/// a chance to change the currently active scope in the environment.
#[must_use = "Nested scopes must be closed!"]
pub(in crate::resolver) struct ScopeGuard(drop_bomb::DropBomb);

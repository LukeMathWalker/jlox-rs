use crate::resolver::resolver::BindingStatus;
use crate::resolver::BindingId;
use drop_bomb::DropBomb;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Environment {
    current_scope: Scope,
    parent_scopes: Vec<Scope>,
    binding_id_cursor: u64,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            current_scope: Scope::default(),
            parent_scopes: vec![],
            binding_id_cursor: 0,
        }
    }

    pub(in crate::resolver) fn enter_scope(&mut self) -> ScopeGuard {
        let enclosing_scope = std::mem::take(&mut self.current_scope);
        self.parent_scopes.push(enclosing_scope);
        ScopeGuard(DropBomb::new("You forgot to close a scope"))
    }

    pub(in crate::resolver) fn exit_scope(&mut self, mut guard: ScopeGuard) -> Scope {
        guard.0.defuse();
        let parent_scope = self.parent_scopes.pop().unwrap();
        std::mem::replace(&mut self.current_scope, parent_scope)
    }

    pub(in crate::resolver) fn define(&mut self, variable_name: String) -> BindingId {
        let new_binding_id = self.binding_id_cursor;
        self.binding_id_cursor += 1;
        self.current_scope.define(variable_name, new_binding_id)
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
        &mut self,
        variable_name: &str,
    ) -> Result<(BindingId, BindingStatus), anyhow::Error> {
        if let Some(value) = self.current_scope.get(variable_name) {
            return Ok(value);
        }
        for scope in self.parent_scopes.iter_mut().rev() {
            if let Some(value) = scope.get(variable_name) {
                return Ok(value);
            }
        }
        Err(anyhow::anyhow!("Tried to read an undefined variable"))
    }
}

#[derive(Debug, Clone, Default)]
pub(in crate::resolver) struct Scope {
    bindings: HashMap<String, (BindingId, BindingStatus)>,
    // We keep track of all the resolution failures encountered
    // when we perform a variable lookup for this scope.
    // This can be used to determine what non-local variables are required
    // to successfully execute the code in this scope - e.g. what values from the
    // parent scopes have been captured by a closure.
    pub(in crate::resolver) failed_variable_lookups: HashSet<String>,
}

impl Scope {
    fn define(&mut self, variable_name: String, binding_id: u64) -> BindingId {
        self.bindings.insert(
            variable_name,
            (BindingId(binding_id), BindingStatus::Uninitialized),
        );
        BindingId(binding_id)
    }

    fn assign(&mut self, variable_name: &str) -> Result<BindingId, ()> {
        match self.bindings.get_mut(variable_name) {
            None => {
                self.failed_variable_lookups
                    .insert(variable_name.to_owned());
                Err(())
            }
            Some(slot) => {
                slot.1 = BindingStatus::Initialized;
                Ok(slot.0.clone())
            }
        }
    }

    fn get(&mut self, variable_name: &str) -> Option<(BindingId, BindingStatus)> {
        let v = self.bindings.get(variable_name).cloned();
        if v.is_none() {
            self.failed_variable_lookups
                .insert(variable_name.to_owned());
        }
        v
    }
}

/// `ScopeGuard` ensures, at runtime, that we never leave a scope unclosed.
/// The interpreter code has no way to defuse the drop bomb (the field is private outside of
/// this module) - the interpreter is forced to call [`Environment::exit_scope`], which gives us
/// a chance to change the currently active scope in the environment.
#[must_use = "Nested scopes must be closed!"]
pub(in crate::resolver) struct ScopeGuard(drop_bomb::DropBomb);

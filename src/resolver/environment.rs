use crate::resolver::resolver::BindingStatus;
use crate::resolver::BindingId;
use drop_bomb::DropBomb;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    current_scope: Scope,
    parent_scopes: Vec<Scope>,
    binding_id_cursor: u64,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            current_scope: Scope::new(ScopeType::Other),
            parent_scopes: vec![],
            binding_id_cursor: 0,
        }
    }

    pub(in crate::resolver) fn enter_scope(&mut self, type_: ScopeType) -> ScopeGuard {
        let enclosing_scope = std::mem::replace(&mut self.current_scope, Scope::new(type_));
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

#[derive(Debug, Clone)]
pub(in crate::resolver) struct Scope {
    bindings: HashMap<String, (BindingId, BindingStatus)>,
    type_: ScopeType,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ScopeType {
    Function,
    Other,
}

impl Scope {
    fn new(type_: ScopeType) -> Self {
        Self {
            bindings: Default::default(),
            type_,
        }
    }
    fn define(&mut self, variable_name: String, binding_id: u64) -> BindingId {
        let binding_id = match self.type_ {
            ScopeType::Function => BindingId::FunctionLocal(binding_id),
            ScopeType::Other => BindingId::Predetermined(binding_id),
        };
        self.bindings
            .insert(variable_name, (binding_id, BindingStatus::Uninitialized));
        binding_id
    }

    fn assign(&mut self, variable_name: &str) -> Result<BindingId, ()> {
        match self.bindings.get_mut(variable_name) {
            None => Err(()),
            Some(slot) => {
                slot.1 = BindingStatus::Initialized;
                Ok(slot.0)
            }
        }
    }

    fn get(&self, variable_name: &str) -> Option<(BindingId, BindingStatus)> {
        self.bindings.get(variable_name).cloned()
    }
}

/// `ScopeGuard` ensures, at runtime, that we never leave a scope unclosed.
/// The interpreter code has no way to defuse the drop bomb (the field is private outside of
/// this module) - the interpreter is forced to call [`Environment::exit_scope`], which gives us
/// a chance to change the currently active scope in the environment.
#[must_use = "Nested scopes must be closed!"]
pub(in crate::resolver) struct ScopeGuard(drop_bomb::DropBomb);

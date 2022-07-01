mod environment;
mod resolver;

pub mod resolved_ast;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BindingId(u64);

impl std::fmt::Display for BindingId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Binding({})", self.0)
    }
}

pub use resolver::Resolver;
use std::fmt::Formatter;

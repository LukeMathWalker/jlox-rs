mod environment;
mod resolver;

pub mod resolved_ast;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum BindingId {
    Predetermined(u64),
    FunctionLocal(u64),
}

impl std::fmt::Display for BindingId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BindingId::Predetermined(n) => write!(f, "Predetermined({n})"),
            BindingId::FunctionLocal(n) => write!(f, "FunctionLocal({n})"),
        }
    }
}

pub use resolver::Resolver;
use std::fmt::Formatter;

use crate::interpreter::environment::Environment;
use crate::parser::ast::FunctionDeclarationStatement;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(in crate::interpreter) enum LoxValue {
    Boolean(bool),
    Null,
    String(String),
    Number(f64),
    Function(Function),
}

impl LoxValue {
    pub fn is_truthy(&self) -> bool {
        if let Self::Null = self {
            false
        } else if let Self::Boolean(b) = self {
            *b
        } else {
            true
        }
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::String(s), Self::String(r)) => s == r,
            (Self::Boolean(s), Self::Boolean(r)) => s == r,
            (Self::Number(s), Self::Number(r)) => s == r,
            (_, _) => false,
        }
    }
}

impl Display for LoxValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxValue::Boolean(b) => b.fmt(f),
            LoxValue::Null => write!(f, "`nil`"),
            LoxValue::String(s) => s.fmt(f),
            LoxValue::Number(n) => n.fmt(f),
            LoxValue::Function(function) => function.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub(in crate::interpreter) struct Function {
    pub(in crate::interpreter) closure: Rc<RefCell<Environment>>,
    pub(in crate::interpreter) declaration: FunctionDeclarationStatement,
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.clone().lexeme())
    }
}

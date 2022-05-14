#[derive(Debug, Clone)]
pub(in crate::interpreter) enum LoxValue {
    Boolean(bool),
    Null,
    String(String),
    Number(f64),
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

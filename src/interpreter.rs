use crate::parser::ast::{BinaryExpression, LiteralExpression, UnaryExpression};
use crate::parser::{ast::Expression, Parser};
use crate::scanner::{Scanner, Token, TokenDiscriminant};

/// Interpret the jlox source code passed as input.
///
/// It returns `Err` if an error was encountered while interpreting the code.
/// The error type does not contain any information since `run` already takes care, internally,
/// to report the errors it has encountered (i.e. print error messages to stdout).
pub fn run(source: String) -> Result<(), ()> {
    let e = Parser::parse(Scanner::new(&source));
    if let Some(e) = e {
        match eval(e) {
            Ok(v) => {
                println!("{:?}", v);
                Ok(())
            }
            Err(e) => {
                println!("Runtime error!\n{:?}", e);
                Err(())
            }
        }
    } else {
        println!("Failed to parse the source code");
        Err(())
    }
}

fn eval(e: Expression) -> Result<LoxValue, RuntimeError> {
    match e {
        Expression::Binary(b) => {
            let BinaryExpression {
                left,
                operator,
                right,
            } = b;
            let left = eval(*left)?;
            let right = eval(*right)?;
            match operator.discriminant() {
                TokenDiscriminant::Minus => match (left, right) {
                    (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l - r)),
                    (_, _) => Err(RuntimeError::operands_must_be_numbers(operator)),
                },
                TokenDiscriminant::Plus => match (left, right) {
                    (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l + r)),
                    (LoxValue::String(l), LoxValue::String(r)) => Ok(LoxValue::String(l + &r)),
                    (_, _) => Err(RuntimeError::new(
                        operator,
                        "`+` operands must either be both numbers or both strings",
                    )),
                },
                TokenDiscriminant::Slash => match (left, right) {
                    (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l / r)),
                    (_, _) => Err(RuntimeError::operands_must_be_numbers(operator)),
                },
                TokenDiscriminant::Star => match (left, right) {
                    (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l * r)),
                    (_, _) => Err(RuntimeError::operands_must_be_numbers(operator)),
                },
                TokenDiscriminant::GreaterEqual => match (left, right) {
                    (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Boolean(l > r)),
                    (_, _) => Err(RuntimeError::operands_must_be_numbers(operator)),
                },
                TokenDiscriminant::Greater => match (left, right) {
                    (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Boolean(l >= r)),
                    (_, _) => Err(RuntimeError::operands_must_be_numbers(operator)),
                },
                TokenDiscriminant::Less => match (left, right) {
                    (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Boolean(l < r)),
                    (_, _) => Err(RuntimeError::operands_must_be_numbers(operator)),
                },
                TokenDiscriminant::LessEqual => match (left, right) {
                    (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Boolean(l <= r)),
                    (_, _) => Err(RuntimeError::operands_must_be_numbers(operator)),
                },
                TokenDiscriminant::EqualEqual => Ok(LoxValue::Boolean(left.is_equal(&right))),
                TokenDiscriminant::BangEqual => Ok(LoxValue::Boolean(!left.is_equal(&right))),
                _ => Err(RuntimeError::new(
                    operator,
                    "It is not a valid binary operator",
                )),
            }
        }
        Expression::Unary(u) => {
            let UnaryExpression { operand, operator } = u;
            let value = eval(*operand)?;
            match operator.discriminant() {
                TokenDiscriminant::Minus => match value {
                    LoxValue::Number(n) => Ok(LoxValue::Number(-n)),
                    _ => Err(RuntimeError::new(operator, "Operand must be a number")),
                },
                TokenDiscriminant::Bang => Ok(LoxValue::Boolean(!value.is_truthy())),
                _ => Err(RuntimeError::new(
                    operator,
                    "`!` and `-` are the only valid unary operators",
                )),
            }
        }
        Expression::Literal(l) => match l {
            LiteralExpression::Boolean(t) => {
                if t.discriminant() == TokenDiscriminant::True {
                    Ok(LoxValue::Boolean(true))
                } else {
                    Ok(LoxValue::Boolean(false))
                }
            }
            LiteralExpression::Null(_) => Ok(LoxValue::Null),
            LiteralExpression::String(s) => {
                // Avoidable .to_owned()
                let s = s.ty().to_owned().string().unwrap();
                Ok(LoxValue::String(s))
            }
            LiteralExpression::Number(n) => {
                // Avoidable .to_owned()
                let n = n.ty().to_owned().number().unwrap();
                Ok(LoxValue::Number(n))
            }
        },
        Expression::Grouping(g) => eval(*g.0),
    }
}

#[derive(Debug)]
enum LoxValue {
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

#[derive(Debug)]
struct RuntimeError {
    t: Token,
    msg: String,
}

impl RuntimeError {
    pub fn new(t: Token, msg: impl Into<String>) -> Self {
        Self { t, msg: msg.into() }
    }

    pub fn operands_must_be_numbers(operator: Token) -> Self {
        Self::new(operator, "Operands must be numbers")
    }
}

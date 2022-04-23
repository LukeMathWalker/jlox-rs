use crate::parser::ast::LiteralExpression;
use crate::parser::{ast::Expression, Parser};
use crate::scanner::{Scanner, TokenDiscriminant};

/// Interpret the jlox source code passed as input.
///
/// It returns `Err` if an error was encountered while interpreting the code.
/// The error type does not contain any information since `run` already takes care, internally,
/// to report the errors it has encountered (i.e. print error messages to stdout).
pub fn run(source: String) -> Result<(), ()> {
    let e = Parser::parse(Scanner::new(&source));
    if let Some(e) = e {
        eval(&e);
    }
    Ok(())
}

fn eval(e: &Expression) -> LoxValue {
    match e {
        Expression::Binary(b) => {
            todo!()
        }
        Expression::Unary(_) => {
            todo!()
        }
        Expression::Literal(l) => match l {
            LiteralExpression::Boolean(t) => {
                if t.discriminant() == TokenDiscriminant::True {
                    LoxValue::Boolean(true)
                } else {
                    LoxValue::Boolean(false)
                }
            }
            LiteralExpression::Null(_) => LoxValue::Null,
            LiteralExpression::String(s) => {
                todo!()
            }
            LiteralExpression::Number(_) => {
                todo!()
            }
        },
        Expression::Grouping(_) => {
            todo!()
        }
    }
}

enum LoxValue {
    Boolean(bool),
    Null,
    String(String),
    Number(f64),
}

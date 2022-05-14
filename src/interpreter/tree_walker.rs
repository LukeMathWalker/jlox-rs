use crate::interpreter::environment::Environment;
use crate::interpreter::lox_value::LoxValue;
use crate::parser::ast::{
    BinaryExpression, ExpressionStatement, LiteralExpression, PrintStatement, Statement,
    UnaryExpression, VariableDeclarationStatement,
};
use crate::parser::{ast::Expression, Parser};
use crate::scanner::{Scanner, Token, TokenDiscriminant};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    /// Scan, parse and then execute a Lox source file.
    ///
    /// It returns `Err` if an error was encountered while interpreting the code.
    /// The error type does not contain any information since `run` already takes care, internally,
    /// to report the errors it has encountered (i.e. print error messages to stdout).
    pub fn execute_raw(&mut self, source: &str) -> Result<(), ()> {
        if let Ok(statements) = Parser::parse(Scanner::new(&source)) {
            self.batch_execute(statements)
        } else {
            println!("Failed to parse the source code");
            Err(())
        }
    }

    /// Execute a single Lox statement.
    pub fn execute(&mut self, statement: Statement) -> Result<(), ()> {
        if let Err(e) = self._execute(statement) {
            println!("Runtime error!\nToken:{:?}\nMessage:{}", e.t, e.msg);
            return Err(());
        } else {
            Ok(())
        }
    }

    /// Execute a series of statements.
    /// It exits as soon as a runtime error is encountered.
    pub fn batch_execute(&mut self, statements: Vec<Statement>) -> Result<(), ()> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn _execute(&mut self, s: Statement) -> Result<(), RuntimeError> {
        match s {
            Statement::Expression(ExpressionStatement(e)) => {
                self.eval(e)?;
            }
            Statement::Print(PrintStatement(e)) => {
                let value = self.eval(e)?;
                println!("{:?}", value);
            }
            Statement::VariableDeclaration(VariableDeclarationStatement {
                initializer,
                identifier,
            }) => {
                let value = if let Some(initializer) = initializer {
                    self.eval(initializer)?
                } else {
                    LoxValue::Null
                };
                self.environment.define_binding(identifier.lexeme(), value);
            }
        }
        Ok(())
    }

    fn eval(&self, e: Expression) -> Result<LoxValue, RuntimeError> {
        match e {
            Expression::Binary(b) => {
                let BinaryExpression {
                    left,
                    operator,
                    right,
                } = b;
                let left = self.eval(*left)?;
                let right = self.eval(*right)?;
                match operator.discriminant() {
                    TokenDiscriminant::Minus => {
                        num_op(left, right, operator, |l, r| LoxValue::Number(l - r))
                    }
                    TokenDiscriminant::Plus => match (left, right) {
                        (LoxValue::Number(l), LoxValue::Number(r)) => Ok(LoxValue::Number(l + r)),
                        (LoxValue::String(l), LoxValue::String(r)) => Ok(LoxValue::String(l + &r)),
                        (_, _) => Err(RuntimeError::new(
                            operator,
                            "`+` operands must either be both numbers or both strings",
                        )),
                    },
                    TokenDiscriminant::Slash => {
                        num_op(left, right, operator, |l, r| LoxValue::Number(l / r))
                    }
                    TokenDiscriminant::Star => {
                        num_op(left, right, operator, |l, r| LoxValue::Number(l * r))
                    }
                    TokenDiscriminant::GreaterEqual => {
                        num_op(left, right, operator, |l, r| LoxValue::Boolean(l > r))
                    }
                    TokenDiscriminant::Greater => {
                        num_op(left, right, operator, |l, r| LoxValue::Boolean(l >= r))
                    }
                    TokenDiscriminant::Less => {
                        num_op(left, right, operator, |l, r| LoxValue::Boolean(l < r))
                    }
                    TokenDiscriminant::LessEqual => {
                        num_op(left, right, operator, |l, r| LoxValue::Boolean(l <= r))
                    }
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
                let value = self.eval(*operand)?;
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
            Expression::Grouping(g) => self.eval(*g.0),
            Expression::Variable(v) => {
                let name = v.identifier.lexeme();
                self.environment.get_value(&name)
            }
        }
    }
}

/// Short-hand for evaluating numerical operations.
fn num_op<F>(
    left: LoxValue,
    right: LoxValue,
    operator: Token,
    operation: F,
) -> Result<LoxValue, RuntimeError>
where
    F: Fn(f64, f64) -> LoxValue,
{
    match (left, right) {
        (LoxValue::Number(l), LoxValue::Number(r)) => Ok(operation(l, r)),
        (_, _) => Err(RuntimeError::operands_must_be_numbers(operator)),
    }
}

#[derive(Debug)]
pub(in crate::interpreter) struct RuntimeError {
    t: Option<Token>,
    msg: String,
}

impl RuntimeError {
    pub fn new(t: Token, msg: impl Into<String>) -> Self {
        Self {
            t: Some(t),
            msg: msg.into(),
        }
    }

    pub fn operands_must_be_numbers(operator: Token) -> Self {
        Self::new(operator, "Operands must be numbers")
    }

    pub fn undefined_variable(variable_name: &str) -> Self {
        Self {
            t: None,
            msg: format!("Undefined variable named {}", variable_name),
        }
    }
}

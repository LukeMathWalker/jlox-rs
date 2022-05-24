use crate::interpreter::environment::Environment;
use crate::interpreter::lox_callable::LoxCallable;
use crate::interpreter::lox_value::LoxValue;
use crate::parser::ast::{
    BinaryExpression, BlockStatement, ExpressionStatement, IfElseStatement, LiteralExpression,
    PrintStatement, Statement, UnaryExpression, VariableDeclarationStatement, WhileStatement,
};
use crate::parser::{ast::Expression, Parser};
use crate::scanner::{Scanner, Token, TokenDiscriminant};
use std::io::Write;

pub struct Interpreter<'a> {
    environment: Environment,
    output_stream: Box<dyn Write + 'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new<OutputStream>(output: OutputStream) -> Self
    where
        OutputStream: Write + 'a,
    {
        Self {
            environment: Environment::new(),
            output_stream: Box::new(output),
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
                writeln!(self.output_stream, "{value}").map_err(RuntimeError::failed_to_print)?;
                self.output_stream
                    .flush()
                    .map_err(RuntimeError::failed_to_flush)?;
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
                self.environment.define(identifier.lexeme(), value);
            }
            Statement::Block(BlockStatement(statements)) => {
                let guard = self.environment.enter_scope();
                let mut error = None;
                for statement in statements {
                    if let Err(e) = self._execute(*statement) {
                        error = Some(e);
                        break;
                    }
                }
                self.environment.exit_scope(guard);
                if let Some(e) = error {
                    return Err(e);
                }
            }
            Statement::IfElse(IfElseStatement {
                condition,
                if_branch,
                else_branch,
            }) => {
                if self.eval(condition)?.is_truthy() {
                    self._execute(*if_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self._execute(*else_branch)?;
                }
            }
            Statement::While(WhileStatement { condition, body }) => {
                while self.eval(condition.clone())?.is_truthy() {
                    self._execute(*body.clone())?;
                }
            }
        }
        Ok(())
    }

    fn eval(&mut self, e: Expression) -> Result<LoxValue, RuntimeError> {
        match e {
            Expression::Binary(b) => {
                let BinaryExpression {
                    left,
                    operator,
                    right,
                } = b;
                let left = self.eval(*left)?;

                // We handle short-circuiting operators first
                if let TokenDiscriminant::Or = operator.discriminant() {
                    return if left.is_truthy() {
                        Ok(left)
                    } else {
                        Ok(self.eval(*right)?)
                    };
                } else if let TokenDiscriminant::And = operator.discriminant() {
                    return if !left.is_truthy() {
                        Ok(left)
                    } else {
                        Ok(self.eval(*right)?)
                    };
                }

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
                LiteralExpression::Boolean(b) => Ok(LoxValue::Boolean(b)),
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
            Expression::VariableReference(v) => {
                let name = v.identifier.lexeme();
                self.environment.get_value(&name)
            }
            Expression::VariableAssignment(v) => {
                let name = v.identifier.lexeme();
                let value = self.eval(*v.value)?;
                self.environment.assign(name, value.clone())?;
                Ok(value)
            }
            Expression::Call(c) => {
                let callee = self.eval(*c.callee)?;
                let arguments = c
                    .arguments
                    .into_iter()
                    .map(|a| self.eval(*a))
                    .collect::<Result<Vec<_>, _>>()?;
                callee.call(self, arguments)
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
pub struct RuntimeError {
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

    pub fn failed_to_print(e: std::io::Error) -> Self {
        Self {
            t: None,
            msg: format!("Failed to execute a print statement.\n{}", e),
        }
    }

    pub fn failed_to_flush(e: std::io::Error) -> Self {
        Self {
            t: None,
            msg: format!("Failed to flush the output stream.\n{}", e),
        }
    }
}

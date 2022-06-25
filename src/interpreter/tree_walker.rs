use crate::interpreter::environment::Environment;
use crate::interpreter::lox_callable::LoxCallable;
use crate::interpreter::lox_value::{Function, LoxValue};
use crate::parser::ast::{
    BinaryExpression, BlockStatement, ExpressionStatement, IfElseStatement, LiteralExpression,
    PrintStatement, ReturnStatement, Statement, UnaryExpression, VariableDeclarationStatement,
    WhileStatement,
};
use crate::parser::{ast::Expression, Parser};
use crate::scanner::{Scanner, Token, TokenDiscriminant};
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::sync::Mutex;

pub struct Interpreter<'a, 'b> {
    pub(in crate::interpreter) environment: &'b mut Environment,
    output_stream: Rc<Mutex<dyn Write + 'a>>,
}

impl<'a, 'b> Interpreter<'a, 'b> {
    pub fn new<OutputStream>(output: OutputStream, environment: &'b mut Environment) -> Self
    where
        OutputStream: Write + 'a,
    {
        Self {
            environment,
            output_stream: Rc::new(Mutex::new(output)),
        }
    }
}

impl<'a, 'b> Interpreter<'a, 'b> {
    /// Create a new interpreter instance that inherits the global scope and shares the same
    /// output stream.
    ///
    /// This is used in the implementation of function calls.
    pub(in crate::interpreter) fn fork(&self, environment: &'b mut Environment) -> Self {
        Self {
            environment,
            output_stream: Rc::clone(&self.output_stream),
        }
    }

    /// Scan, parse and then execute a Lox source file.
    ///
    /// It returns `Err` if an error was encountered while interpreting the code.
    /// The error type does not contain any information since `run` already takes care, internally,
    /// to report the errors it has encountered (i.e. print error messages to stdout).
    pub fn execute_raw(&mut self, source: &str) -> Result<(), ExecuteRawError> {
        let statements =
            Parser::parse(Scanner::new(source)).map_err(ExecuteRawError::ParserError)?;
        self.batch_execute(statements)
            .map_err(ExecuteRawError::RuntimeError)
    }

    /// Execute a series of statements.
    /// It exits as soon as a runtime error is encountered.
    pub fn batch_execute(&mut self, statements: Vec<Statement>) -> Result<(), RuntimeError> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    /// Execute a single Lox statement.
    pub fn execute(&mut self, statement: Statement) -> Result<(), RuntimeError> {
        self._execute(statement).map_err(|e| match e {
            RuntimeErrorOrReturn::RuntimeError(e) => e,
            RuntimeErrorOrReturn::Return(_) => RuntimeError::unexpected_return(),
        })
    }

    pub(in crate::interpreter) fn _execute(
        &mut self,
        s: Statement,
    ) -> Result<(), RuntimeErrorOrReturn> {
        match s {
            Statement::Expression(ExpressionStatement(e)) => {
                self.eval(e)?;
            }
            Statement::Print(PrintStatement(e)) => {
                let value = self.eval(e)?;
                let mut stream = self.output_stream.lock().unwrap();
                writeln!(stream, "{value}").map_err(RuntimeError::failed_to_print)?;
                stream.flush().map_err(RuntimeError::failed_to_flush)?;
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
                    if let Err(e) = self._execute(statement) {
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
            Statement::FunctionDeclaration(statement) => {
                let function = Function {
                    closure: Rc::new(RefCell::new(self.environment.clone())),
                    declaration: statement,
                };
                self.environment.define(
                    function.declaration.name.clone().lexeme(),
                    LoxValue::Function(function),
                );
            }
            Statement::Return(ReturnStatement { value, .. }) => {
                let value = self.eval(value)?;
                return Err(Return(value).into());
            }
        }
        Ok(())
    }

    fn eval(&mut self, e: Expression) -> Result<LoxValue, RuntimeErrorOrReturn> {
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
                        )
                        .into()),
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
                    _ => {
                        Err(RuntimeError::new(operator, "It is not a valid binary operator").into())
                    }
                }
            }
            Expression::Unary(u) => {
                let UnaryExpression { operand, operator } = u;
                let value = self.eval(*operand)?;
                match operator.discriminant() {
                    TokenDiscriminant::Minus => match value {
                        LoxValue::Number(n) => Ok(LoxValue::Number(-n)),
                        _ => Err(RuntimeError::new(operator, "Operand must be a number").into()),
                    },
                    TokenDiscriminant::Bang => Ok(LoxValue::Boolean(!value.is_truthy())),
                    _ => Err(RuntimeError::new(
                        operator,
                        "`!` and `-` are the only valid unary operators",
                    )
                    .into()),
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
                Ok(self.environment.get_value(&name)?)
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
                    .map(|a| self.eval(a))
                    .collect::<Result<Vec<_>, _>>()?;
                match callee {
                    LoxValue::Function(callee) => {
                        // This is fine since the parser will reject functions with more than 255 arguments
                        let n_arguments = arguments.len() as u8;
                        if callee.arity() != n_arguments {
                            return Err(
                                RuntimeError::arity_mismatch(callee.arity(), n_arguments).into()
                            );
                        }
                        Ok(callee.call(self, arguments)?)
                    }
                    LoxValue::Boolean(_)
                    | LoxValue::Null
                    | LoxValue::String(_)
                    | LoxValue::Number(_) => Err(RuntimeError::not_callable(&callee).into()),
                }
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
) -> Result<LoxValue, RuntimeErrorOrReturn>
where
    F: Fn(f64, f64) -> LoxValue,
{
    match (left, right) {
        (LoxValue::Number(l), LoxValue::Number(r)) => Ok(operation(l, r)),
        (_, _) => Err(RuntimeError::operands_must_be_numbers(operator).into()),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExecuteRawError {
    #[error("Failed to parse the source code")]
    ParserError(Vec<Statement>),
    #[error(transparent)]
    RuntimeError(RuntimeError),
}

#[derive(Debug, thiserror::Error)]
pub(in crate::interpreter) enum RuntimeErrorOrReturn {
    #[error(transparent)]
    RuntimeError(#[from] RuntimeError),
    #[error(transparent)]
    Return(#[from] Return),
}

#[derive(Debug, thiserror::Error)]
#[error("An early return was encountered")]
pub(in crate::interpreter) struct Return(pub(in crate::interpreter) LoxValue);

#[derive(Debug, thiserror::Error)]
#[error("An error occurred at runtime. {msg}")]
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

    pub fn arity_mismatch(expected: u8, found: u8) -> Self {
        Self {
            t: None,
            msg: format!("Expect {expected} arguments, but got {found} arguments."),
        }
    }

    fn not_callable(v: &LoxValue) -> Self {
        Self {
            t: None,
            msg: format!("`{v}` is not callable."),
        }
    }

    fn unexpected_return() -> Self {
        Self {
            t: None,
            msg: "`return` was used in an illegal position".into(),
        }
    }
}

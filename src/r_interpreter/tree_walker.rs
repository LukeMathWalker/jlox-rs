use super::lox_callable::LoxCallable;
use super::lox_value::{Function, LoxValue};
use crate::parser::Parser;
use crate::resolver::resolved_ast::{
    BinaryExpression, BlockStatement, Expression, ExpressionStatement, IfElseStatement,
    LiteralExpression, PrintStatement, ReturnStatement, Statement, UnaryExpression,
    VariableDeclarationStatement, WhileStatement,
};
use crate::resolver::{BindingId, Resolver};
use crate::scanner::{Scanner, Token, TokenDiscriminant};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;
use std::sync::Mutex;

pub struct Interpreter<'a> {
    pub(super) bindings: HashMap<BindingId, Rc<RefCell<LoxValue>>>,
    output_stream: Rc<Mutex<dyn Write + 'a>>,
}

impl<'a> Interpreter<'a> {
    pub fn new<OutputStream>(output: OutputStream) -> Self
    where
        OutputStream: Write + 'a,
    {
        Self {
            bindings: HashMap::new(),
            output_stream: Rc::new(Mutex::new(output)),
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
        let statements = Resolver::new()
            .resolve(statements)
            .map_err(ExecuteRawError::NameResolutionError)?;
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

    pub(super) fn _execute(&mut self, s: Statement) -> Result<(), RuntimeErrorOrReturn> {
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
                binding_id,
            }) => {
                let value = if let Some(initializer) = initializer {
                    self.eval(initializer)?
                } else {
                    LoxValue::Null
                };
                self.bindings
                    .insert(binding_id, Rc::new(RefCell::new(value)));
            }
            Statement::Block(BlockStatement(statements)) => {
                for statement in statements {
                    self._execute(statement)?;
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
                let captured_environment =
                    HashMap::with_capacity(statement.captured_binding_ids.len());
                let name_binding_id = statement.name_binding_id;
                let function = Rc::new(RefCell::new(LoxValue::Function(Rc::new(RefCell::new(
                    Function {
                        definition: statement,
                        captured_environment,
                    },
                )))));
                self.bindings.insert(name_binding_id, Rc::clone(&function));
                if let LoxValue::Function(ref mut function) = *function.borrow_mut() {
                    let captured_environment = function
                        .borrow()
                        .definition
                        .captured_binding_ids
                        .iter()
                        .map(|binding_id| {
                            (
                                *binding_id,
                                Rc::clone(self.bindings.get(&binding_id).unwrap()),
                            )
                        })
                        .collect();
                    function.borrow_mut().captured_environment = captured_environment;
                };
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
                Ok(self
                    .bindings
                    .get(&v.binding_id)
                    .expect("Failed to look up the value of a variable reference via its binding id. This is an interpreter bug.")
                    .borrow().to_owned())
            }
            Expression::VariableAssignment(v) => {
                let value = self.eval(*v.value)?;
                self.bindings.entry(v.binding_id).and_modify(|variable| {
                    *variable.borrow_mut() = value.clone();
                }).or_insert_with(|| Rc::new(RefCell::new(value.clone())));
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
                        let arity = callee.borrow().arity();
                        if arity != n_arguments {
                            return Err(
                                RuntimeError::arity_mismatch(arity, n_arguments).into()
                            );
                        }
                        Ok(callee.borrow().call(self, arguments)?)
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
    ParserError(Vec<crate::parser::ast::Statement>),
    #[error(transparent)]
    NameResolutionError(anyhow::Error),
    #[error(transparent)]
    RuntimeError(RuntimeError),
}

#[derive(Debug, thiserror::Error)]
pub(super) enum RuntimeErrorOrReturn {
    #[error(transparent)]
    RuntimeError(#[from] RuntimeError),
    #[error(transparent)]
    Return(#[from] Return),
}

#[derive(Debug, thiserror::Error)]
#[error("An early return was encountered")]
pub(super) struct Return(pub(super) LoxValue);

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

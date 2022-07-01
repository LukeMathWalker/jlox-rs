use super::resolved_ast as r_ast;
use crate::parser::ast;
use crate::parser::ast::{Expression, Statement};
use crate::resolver::environment::Environment;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingStatus {
    Initialized,
    Uninitialized,
}

pub struct Resolver {
    environment: Environment,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn resolve(
        &mut self,
        statements: Vec<ast::Statement>,
    ) -> Result<Vec<r_ast::Statement>, anyhow::Error> {
        let mut resolved_statements = Vec::with_capacity(statements.len());
        for statement in statements {
            // TODO: collect all errors, do not short-circuit
            resolved_statements.push(self.resolve_statement(statement)?);
        }
        Ok(resolved_statements)
    }

    fn resolve_statement(
        &mut self,
        statement: ast::Statement,
    ) -> Result<r_ast::Statement, anyhow::Error> {
        let s = match statement {
            Statement::VariableDeclaration(v) => {
                let identifier = v.identifier.lexeme();
                let binding_id = self.environment.define(identifier.clone());
                let initializer = v
                    .initializer
                    .map(|init| self.resolve_expression(init))
                    .transpose()?;
                if initializer.is_some() {
                    self.environment.assign(identifier)?;
                }
                r_ast::Statement::VariableDeclaration(r_ast::VariableDeclarationStatement {
                    initializer,
                    binding_id,
                })
            }
            Statement::Expression(e) => r_ast::Statement::Expression(r_ast::ExpressionStatement(
                self.resolve_expression(e.0)?,
            )),
            Statement::Print(p) => {
                r_ast::Statement::Print(r_ast::PrintStatement(self.resolve_expression(p.0)?))
            }
            Statement::FunctionDeclaration(f) => {
                let name = f.name.lexeme();
                let name_binding_id = self.environment.define(name.clone());
                // We allow recursive functions, therefore we immediately mark the function name
                // binding as assigned.
                self.environment.assign(name)?;

                let parameters_binding_ids = f
                    .parameters
                    .into_iter()
                    .map(|p| {
                        let name = p.lexeme();
                        let parameter_binding_id = self.environment.define(name.clone());
                        // Function parameters are "assigned" when the function is called, they can
                        // never be unassigned.
                        self.environment.assign(name).map(|_| parameter_binding_id)
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let body = self.resolve(f.body)?;

                r_ast::Statement::FunctionDeclaration(r_ast::FunctionDeclarationStatement {
                    name_binding_id,
                    parameters_binding_ids,
                    body,
                })
            }
            Statement::Block(b) => {
                let scope_guard = self.environment.enter_scope();
                let outcome = self.resolve(b.0);
                self.environment.exit_scope(scope_guard);
                r_ast::Statement::Block(r_ast::BlockStatement(outcome?))
            }
            Statement::IfElse(ifelse) => {
                let condition = self.resolve_expression(ifelse.condition)?;
                let if_branch = Box::new(self.resolve_statement(*ifelse.if_branch)?);
                let else_branch = ifelse
                    .else_branch
                    .map(|else_branch| self.resolve_statement(*else_branch))
                    .transpose()?
                    .map(Box::new);
                r_ast::Statement::IfElse(r_ast::IfElseStatement {
                    condition,
                    if_branch,
                    else_branch,
                })
            }
            Statement::While(w) => {
                let condition = self.resolve_expression(w.condition)?;
                let body = Box::new(self.resolve_statement(*w.body)?);
                r_ast::Statement::While(r_ast::WhileStatement { condition, body })
            }
            Statement::Return(r) => {
                let value = self.resolve_expression(r.value)?;
                r_ast::Statement::Return(r_ast::ReturnStatement {
                    keyword: r.keyword,
                    value,
                })
            }
        };
        Ok(s)
    }

    fn resolve_expression(
        &mut self,
        expr: ast::Expression,
    ) -> Result<r_ast::Expression, anyhow::Error> {
        let e = match expr {
            Expression::Binary(b) => {
                let left = Box::new(self.resolve_expression(*b.left)?);
                let right = Box::new(self.resolve_expression(*b.right)?);
                r_ast::Expression::Binary(r_ast::BinaryExpression {
                    left,
                    operator: b.operator,
                    right,
                })
            }
            Expression::Unary(u) => {
                let operand = Box::new(self.resolve_expression(*u.operand)?);
                r_ast::Expression::Unary(r_ast::UnaryExpression {
                    operand,
                    operator: u.operator,
                })
            }
            Expression::Literal(l) => {
                let l = match l {
                    ast::LiteralExpression::Boolean(b) => r_ast::LiteralExpression::Boolean(b),
                    ast::LiteralExpression::Null(n) => r_ast::LiteralExpression::Null(n),
                    ast::LiteralExpression::String(s) => r_ast::LiteralExpression::String(s),
                    ast::LiteralExpression::Number(n) => r_ast::LiteralExpression::Number(n),
                };
                r_ast::Expression::Literal(l)
            }
            Expression::Grouping(g) => {
                let expr = self.resolve_expression(*g.0)?;
                r_ast::Expression::Grouping(r_ast::GroupingExpression(Box::new(expr)))
            }
            Expression::VariableReference(v) => {
                let (binding_id, binding_status) = self.environment.get(&v.identifier.lexeme())?;
                if binding_status == BindingStatus::Uninitialized {
                    return Err(anyhow::anyhow!("You cannot reference an uninitialized variable. This can also happen if you are referencing a variable in its own initializer."));
                }
                r_ast::Expression::VariableReference(r_ast::VariableReferenceExpression {
                    binding_id,
                })
            }
            Expression::VariableAssignment(a) => {
                let value = Box::new(self.resolve_expression(*a.value)?);
                let binding_id = self.environment.assign(a.identifier.lexeme())?;
                r_ast::Expression::VariableAssignment(r_ast::VariableAssignmentExpression {
                    binding_id,
                    value,
                })
            }
            Expression::Call(c) => {
                let callee = Box::new(self.resolve_expression(*c.callee)?);
                let arguments = c
                    .arguments
                    .into_iter()
                    .map(|arg| self.resolve_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                r_ast::Expression::Call(r_ast::CallExpression {
                    callee,
                    closing_parenthesis: c.closing_parenthesis,
                    arguments,
                })
            }
        };
        Ok(e)
    }
}

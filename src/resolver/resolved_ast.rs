use crate::resolver::BindingId;
use crate::scanner::Token;

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(ExpressionStatement),
    Print(PrintStatement),
    VariableDeclaration(VariableDeclarationStatement),
    FunctionDeclaration(FunctionDeclarationStatement),
    Block(BlockStatement),
    IfElse(IfElseStatement),
    While(WhileStatement),
    Return(ReturnStatement),
}

#[derive(Debug, Clone)]
pub struct ExpressionStatement(pub Expression);

#[derive(Debug, Clone)]
pub struct PrintStatement(pub Expression);

#[derive(Debug, Clone)]
pub struct BlockStatement(pub Vec<Statement>);

#[derive(Debug, Clone)]
pub struct VariableDeclarationStatement {
    pub initializer: Option<Expression>,
    pub binding_id: BindingId,
}

#[derive(Debug, Clone)]
pub struct FunctionDeclarationStatement {
    pub name_binding_id: BindingId,
    pub parameters_binding_ids: Vec<BindingId>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct IfElseStatement {
    pub condition: Expression,
    pub if_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Box<Statement>,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub keyword: Token,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Literal(LiteralExpression),
    Grouping(GroupingExpression),
    VariableReference(VariableReferenceExpression),
    VariableAssignment(VariableAssignmentExpression),
    Call(CallExpression),
}

impl Expression {
    pub fn binary(left: Expression, operator: Token, right: Expression) -> Self {
        Self::Binary(BinaryExpression {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    pub fn unary(operator: Token, operand: Expression) -> Self {
        Self::Unary(UnaryExpression {
            operator,
            operand: Box::new(operand),
        })
    }

    pub fn boolean(b: bool) -> Self {
        Self::Literal(LiteralExpression::Boolean(b))
    }

    pub fn string(t: Token) -> Self {
        Self::Literal(LiteralExpression::String(t))
    }

    pub fn number(t: Token) -> Self {
        Self::Literal(LiteralExpression::Number(t))
    }

    pub fn null(t: Token) -> Self {
        Self::Literal(LiteralExpression::Null(t))
    }

    pub fn grouping(e: Expression) -> Self {
        Self::Grouping(GroupingExpression(Box::new(e)))
    }

    pub fn variable_reference(i: BindingId) -> Self {
        Self::VariableReference(VariableReferenceExpression { binding_id: i })
    }

    pub fn variable_assignment(binding_id: BindingId, value: Expression) -> Self {
        Self::VariableAssignment(VariableAssignmentExpression {
            binding_id,
            value: Box::new(value),
        })
    }

    pub fn call(
        callee: Expression,
        closing_parenthesis: Token,
        arguments: Vec<Expression>,
    ) -> Self {
        Self::Call(CallExpression {
            callee: Box::new(callee),
            closing_parenthesis,
            arguments,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    // TODO: review if using a Token directly, here, is ideal
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub operand: Box<Expression>,
    // TODO: review if using a Token directly, here, is ideal
    pub operator: Token,
}

#[derive(Debug, Clone)]
pub struct VariableReferenceExpression {
    pub binding_id: BindingId,
}

#[derive(Debug, Clone)]
pub struct VariableAssignmentExpression {
    pub binding_id: BindingId,
    pub value: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum LiteralExpression {
    Boolean(bool),
    Null(Token),
    String(Token),
    Number(Token),
}

#[derive(Debug, Clone)]
pub struct GroupingExpression(pub Box<Expression>);

#[derive(Debug, Clone)]
pub struct CallExpression {
    pub callee: Box<Expression>,
    pub closing_parenthesis: Token,
    pub arguments: Vec<Expression>,
}

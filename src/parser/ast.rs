use crate::scanner::Token;

#[derive(Clone)]
pub enum Statement {
    Expression(ExpressionStatement),
    Print(PrintStatement),
    VariableDeclaration(VariableDeclarationStatement),
    Block(BlockStatement),
    IfElse(IfElseStatement),
    While(WhileStatement),
}

#[derive(Clone)]
pub struct ExpressionStatement(pub Expression);

#[derive(Clone)]
pub struct PrintStatement(pub Expression);

#[derive(Clone)]
pub struct BlockStatement(pub Vec<Box<Statement>>);

#[derive(Clone)]
pub struct VariableDeclarationStatement {
    pub initializer: Option<Expression>,
    pub identifier: Token,
}

#[derive(Clone)]
pub struct IfElseStatement {
    pub condition: Expression,
    pub if_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>,
}

#[derive(Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Box<Statement>,
}

#[derive(Clone)]
pub enum Expression {
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Literal(LiteralExpression),
    Grouping(GroupingExpression),
    VariableReference(VariableReferenceExpression),
    VariableAssignment(VariableAssignmentExpression),
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

    pub fn boolean(t: Token) -> Self {
        Self::Literal(LiteralExpression::Boolean(t))
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

    pub fn variable_reference(t: Token) -> Self {
        Self::VariableReference(VariableReferenceExpression { identifier: t })
    }

    pub fn variable_assignment(identifier: Token, value: Expression) -> Self {
        Self::VariableAssignment(VariableAssignmentExpression {
            identifier,
            value: Box::new(value),
        })
    }
}

#[derive(Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    // TODO: review if using a Token directly, here, is ideal
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Clone)]
pub struct UnaryExpression {
    pub operand: Box<Expression>,
    // TODO: review if using a Token directly, here, is ideal
    pub operator: Token,
}

#[derive(Clone)]
pub struct VariableReferenceExpression {
    // TODO: review if using a Token directly, here, is ideal
    pub identifier: Token,
}

#[derive(Clone)]
pub struct VariableAssignmentExpression {
    // TODO: review if using a Token directly, here, is ideal
    pub identifier: Token,
    pub value: Box<Expression>,
}

#[derive(Clone)]
pub enum LiteralExpression {
    Boolean(Token),
    Null(Token),
    String(Token),
    Number(Token),
}

#[derive(Clone)]
pub struct GroupingExpression(pub Box<Expression>);

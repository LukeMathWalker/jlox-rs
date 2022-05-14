use crate::scanner::Token;

pub enum Statement {
    Expression(ExpressionStatement),
    Print(PrintStatement),
    VariableDeclaration(VariableDeclarationStatement),
}

pub struct ExpressionStatement(pub Expression);

pub struct PrintStatement(pub Expression);

pub struct VariableDeclarationStatement {
    pub initializer: Option<Expression>,
    pub identifier: Token,
}

pub enum Expression {
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Literal(LiteralExpression),
    Grouping(GroupingExpression),
    Variable(VariableExpression),
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

    pub fn variable(t: Token) -> Self {
        Self::Variable(VariableExpression { identifier: t })
    }
}

pub struct BinaryExpression {
    pub left: Box<Expression>,
    // TODO: review if using a Token directly, here, is ideal
    pub operator: Token,
    pub right: Box<Expression>,
}

pub struct UnaryExpression {
    pub operand: Box<Expression>,
    // TODO: review if using a Token directly, here, is ideal
    pub operator: Token,
}

pub struct VariableExpression {
    // TODO: review if using a Token directly, here, is ideal
    pub identifier: Token,
}

pub enum LiteralExpression {
    Boolean(Token),
    Null(Token),
    String(Token),
    Number(Token),
}

pub struct GroupingExpression(pub Box<Expression>);

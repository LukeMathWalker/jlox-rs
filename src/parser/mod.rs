use crate::scanner::{Token, TokenType};
use std::iter::Peekable;

#[derive(PartialEq, Eq, Copy, Clone)]
enum ParsingMode {
    ErrorRecovery,
    Normal,
}

struct Parser<TokenIter>
where
    TokenIter: Iterator<Item = Token>,
{
    tokens: Peekable<TokenIter>,
    mode: ParsingMode,
}

impl<TokenIter> Parser<TokenIter>
where
    TokenIter: Iterator<Item = Token>,
{
    pub fn parse(tokens: TokenIter) -> Option<Expression> {
        let mut parser = Self {
            tokens: tokens.peekable(),
            mode: ParsingMode::Normal,
        };
        let expr = parser.expression();
        if expr.is_none() {
            parser.advance_until_recovery_point();
        }
        expr
    }

    fn expression(&mut self) -> Option<Expression> {
        self.equality()
    }

    fn equality(&mut self) -> Option<Expression> {
        let mut expr = self.comparison()?;

        while let Some(operator) =
            self.advance_on_match(&[TokenType::EqualEqual, TokenType::EqualEqual])
        {
            expr = Expression::binary(expr, operator, self.comparison()?);
        }
        Some(expr)
    }

    fn comparison(&mut self) -> Option<Expression> {
        let mut expr = self.term()?;

        while let Some(operator) = self.advance_on_match(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            expr = Expression::binary(expr, operator, self.term()?);
        }
        Some(expr)
    }

    fn term(&mut self) -> Option<Expression> {
        let mut expr = self.factor()?;

        while let Some(operator) = self.advance_on_match(&[TokenType::Minus, TokenType::Plus]) {
            expr = Expression::binary(expr, operator, self.factor()?);
        }
        Some(expr)
    }

    fn factor(&mut self) -> Option<Expression> {
        let mut expr = self.unary()?;

        while let Some(operator) = self.advance_on_match(&[TokenType::Slash, TokenType::Star]) {
            expr = Expression::binary(expr, operator, self.unary()?);
        }
        Some(expr)
    }

    fn unary(&mut self) -> Option<Expression> {
        if let Some(operator) = self.advance_on_match(&[TokenType::Bang, TokenType::Minus]) {
            Some(Expression::unary(operator, self.unary()?))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Option<Expression> {
        if let Some(t) = self.advance_on_match(&[TokenType::True, TokenType::False]) {
            Some(Expression::boolean(t))
        } else if let Some(t) = self.advance_on_match(&[TokenType::Nil]) {
            Some(Expression::null(t))
        } else if let Some(t) = self.advance_on_match(&[TokenType::Number]) {
            Some(Expression::number(t))
        } else if let Some(t) = self.advance_on_match(&[TokenType::String]) {
            Some(Expression::string(t))
        } else if let Some(t) = self.advance_on_match(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.expect(TokenType::RightParen)?;
            Some(Expression::grouping(expr))
        } else {
            self.mode = ParsingMode::ErrorRecovery;
            None
        }
    }

    fn advance_on_match(&mut self, token_types: &[TokenType]) -> Option<Token> {
        let upcoming = self.tokens.peek()?;
        if token_types.contains(&upcoming.ty()) {
            return self.advance();
        }
        None
    }

    fn advance_until_recovery_point(&mut self) {
        // Using a closure that returns `Option` to be able to use the `?` operator.
        // Looking forward to try blocks.
        let mut recover = || -> Option<()> {
            loop {
                let current = self.tokens.next()?;
                if current.ty() == TokenType::Semicolon {
                    break None;
                }
                let upcoming = self.tokens.peek()?;
                match upcoming.ty() {
                    TokenType::Class
                    | TokenType::Fun
                    | TokenType::Var
                    | TokenType::For
                    | TokenType::If
                    | TokenType::Print
                    | TokenType::Return
                    | TokenType::While => {
                        break None;
                    }
                    _ => {}
                }
            }
        };
        let _ = recover();
    }

    fn expect(&mut self, token_type: TokenType) -> Option<Token> {
        let t = self.advance_on_match(&[token_type]);
        if t.is_none() {
            self.mode = ParsingMode::ErrorRecovery;
        }
        t
    }

    fn advance(&mut self) -> Option<Token> {
        if self.mode == ParsingMode::Normal {
            self.tokens.next()
        } else {
            None
        }
    }
}

pub enum Expression {
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Literal(LiteralExpression),
    Grouping(GroupingExpression),
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
}

pub struct BinaryExpression {
    left: Box<Expression>,
    // TODO: review if using a Token directly, here, is ideal
    operator: Token,
    right: Box<Expression>,
}

pub struct UnaryExpression {
    operand: Box<Expression>,
    // TODO: review if using a Token directly, here, is ideal
    operator: Token,
}

pub enum LiteralExpression {
    Boolean(Token),
    Null(Token),
    String(Token),
    Number(Token),
}

pub struct GroupingExpression(Box<Expression>);

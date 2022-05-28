pub mod ast;

use crate::parser::ast::{
    BlockStatement, CallExpression, ExpressionStatement, FunctionDeclarationStatement,
    IfElseStatement, PrintStatement, Statement, VariableAssignmentExpression,
    VariableDeclarationStatement, VariableReferenceExpression, WhileStatement,
};
use crate::scanner::{Token, TokenDiscriminant, TokenType};
use ast::{Expression, LiteralExpression};
use std::fmt::Write;
use std::iter::Peekable;

#[derive(PartialEq, Eq, Copy, Clone)]
enum ParsingMode {
    ErrorRecovery,
    Normal,
}

pub struct Parser<TokenIter>
where
    TokenIter: Iterator<Item = Token>,
{
    tokens: Peekable<Source<TokenIter>>,
    mode: ParsingMode,
}

impl<TokenIter> Parser<TokenIter>
where
    TokenIter: Iterator<Item = Token>,
{
    pub fn parse(tokens: TokenIter) -> Result<Vec<Statement>, Vec<Statement>> {
        let mut parser = Self {
            tokens: Source(tokens).peekable(),
            mode: ParsingMode::Normal,
        };

        let mut has_errored = false;
        let mut statements = vec![];
        while !parser.is_at_end() {
            let statement = parser.declaration();
            match statement {
                Some(statement) => {
                    statements.push(statement);
                }
                None => {
                    parser.advance_until_recovery_point();
                    has_errored = true;
                }
            }
        }
        if has_errored {
            Err(statements)
        } else {
            Ok(statements)
        }
    }

    fn declaration(&mut self) -> Option<Statement> {
        if self.advance_on_match(&[TokenDiscriminant::Fun]).is_some() {
            self.function().map(Statement::FunctionDeclaration)
        } else if self.advance_on_match(&[TokenDiscriminant::Var]).is_some() {
            let identifier = self.expect(TokenDiscriminant::Identifier)?;
            let mut initializer = None;
            if self.advance_on_match(&[TokenDiscriminant::Equal]).is_some() {
                initializer = Some(self.expression()?);
            }
            self.expect(TokenDiscriminant::Semicolon)?;
            Some(Statement::VariableDeclaration(
                VariableDeclarationStatement {
                    initializer,
                    identifier,
                },
            ))
        } else {
            self.statement()
        }
    }

    fn function(&mut self) -> Option<FunctionDeclarationStatement> {
        let name = self.expect(TokenDiscriminant::Identifier)?;
        self.expect(TokenDiscriminant::LeftParen)?;

        // Function parameters
        let mut parameters = vec![];
        if self.peek()?.discriminant() != TokenDiscriminant::RightParen {
            loop {
                parameters.push(self.expect(TokenDiscriminant::Identifier)?);
                if self.peek()?.discriminant() != TokenDiscriminant::Comma {
                    break;
                }
            }
        }
        self.expect(TokenDiscriminant::RightParen)?;
        if parameters.len() >= 255 {
            // Ugly, we should set `has_errored` here.
            println!("You can't have more than 255 arguments");
            return None;
        }

        // Body
        self.expect(TokenDiscriminant::LeftBrace)?;
        let body = self.block_statement()?;

        Some(FunctionDeclarationStatement {
            name,
            parameters,
            body: body.0,
        })
    }

    fn statement(&mut self) -> Option<Statement> {
        if self.advance_on_match(&[TokenDiscriminant::Print]).is_some() {
            self.print_statement().map(Statement::Print)
        } else if self.advance_on_match(&[TokenDiscriminant::While]).is_some() {
            self.while_statement().map(Statement::While)
        } else if self.advance_on_match(&[TokenDiscriminant::For]).is_some() {
            self.for_statement()
        } else if self.advance_on_match(&[TokenDiscriminant::If]).is_some() {
            self.if_else_statement().map(Statement::IfElse)
        } else if self
            .advance_on_match(&[TokenDiscriminant::LeftBrace])
            .is_some()
        {
            self.block_statement().map(Statement::Block)
        } else {
            self.expression_statement().map(Statement::Expression)
        }
    }

    fn for_statement(&mut self) -> Option<Statement> {
        self.expect(TokenDiscriminant::LeftParen)?;
        let initializer = if self
            .advance_on_match(&[TokenDiscriminant::Semicolon])
            .is_some()
        {
            None
        } else if self
            .peek()
            .map(|t| t.discriminant() == TokenDiscriminant::Var)
            .unwrap_or(false)
        {
            Some(self.declaration()?)
        } else {
            Some(Statement::Expression(self.expression_statement()?))
        };
        let condition = if self.peek()?.discriminant() == TokenDiscriminant::Semicolon {
            None
        } else {
            Some(self.expression()?)
        };
        self.expect(TokenDiscriminant::Semicolon)?;
        let increment = if self.peek()?.discriminant() == TokenDiscriminant::RightParen {
            None
        } else {
            Some(self.expression()?)
        };
        self.expect(TokenDiscriminant::RightParen)?;
        let mut body = self.statement()?;

        // De-sugaring the for loop into an equivalent while loop
        if let Some(increment) = increment {
            body = Statement::Block(BlockStatement(vec![
                Box::new(body),
                Box::new(Statement::Expression(ExpressionStatement(increment))),
            ]))
        }

        body = Statement::While(WhileStatement {
            condition: condition.unwrap_or_else(|| Expression::boolean(true)),
            body: Box::new(body),
        });

        if let Some(initializer) = initializer {
            body = Statement::Block(BlockStatement(vec![Box::new(initializer), Box::new(body)]))
        }

        Some(body)
    }

    fn block_statement(&mut self) -> Option<BlockStatement> {
        let mut statements = vec![];

        loop {
            if self.is_at_end() {
                break;
            }
            if let Some(t) = self.peek() {
                if t.discriminant() == TokenDiscriminant::RightBrace {
                    break;
                }
            }
            statements.push(Box::new(self.declaration()?));
        }
        self.expect(TokenDiscriminant::RightBrace)?;
        Some(BlockStatement(statements))
    }

    fn while_statement(&mut self) -> Option<WhileStatement> {
        self.expect(TokenDiscriminant::LeftParen)?;
        let condition = self.expression()?;
        self.expect(TokenDiscriminant::RightParen)?;
        let body = self.statement()?;
        Some(WhileStatement {
            condition,
            body: Box::new(body),
        })
    }

    fn if_else_statement(&mut self) -> Option<IfElseStatement> {
        self.expect(TokenDiscriminant::LeftParen)?;
        let condition = self.expression()?;
        self.expect(TokenDiscriminant::RightParen)?;
        let if_branch = self.statement()?;
        let mut else_branch = None;
        if self.advance_on_match(&[TokenDiscriminant::Else]).is_some() {
            else_branch = Some(Box::new(self.statement()?));
        }
        Some(IfElseStatement {
            condition,
            if_branch: Box::new(if_branch),
            else_branch,
        })
    }

    fn print_statement(&mut self) -> Option<PrintStatement> {
        let expr = self.expression()?;
        self.expect(TokenDiscriminant::Semicolon)?;
        Some(PrintStatement(expr))
    }

    fn expression_statement(&mut self) -> Option<ExpressionStatement> {
        let expr = self.expression()?;
        self.expect(TokenDiscriminant::Semicolon)?;
        Some(ExpressionStatement(expr))
    }

    fn expression(&mut self) -> Option<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> Option<Expression> {
        let expr = self.or()?;

        if self.advance_on_match(&[TokenDiscriminant::Equal]).is_some() {
            let value = self.assignment()?;
            if let Expression::VariableReference(variable) = expr {
                let name = variable.identifier;
                Some(Expression::variable_assignment(name, value))
            } else {
                // Invalid assignment target!
                None
            }
        } else {
            Some(expr)
        }
    }

    fn or(&mut self) -> Option<Expression> {
        let mut expr = self.and()?;

        while let Some(operator) = self.advance_on_match(&[TokenDiscriminant::Or]) {
            expr = Expression::binary(expr, operator, self.and()?);
        }
        Some(expr)
    }

    fn and(&mut self) -> Option<Expression> {
        let mut expr = self.equality()?;

        while let Some(operator) = self.advance_on_match(&[TokenDiscriminant::And]) {
            expr = Expression::binary(expr, operator, self.equality()?);
        }
        Some(expr)
    }

    fn equality(&mut self) -> Option<Expression> {
        let mut expr = self.comparison()?;

        while let Some(operator) =
            self.advance_on_match(&[TokenDiscriminant::EqualEqual, TokenDiscriminant::EqualEqual])
        {
            expr = Expression::binary(expr, operator, self.comparison()?);
        }
        Some(expr)
    }

    fn comparison(&mut self) -> Option<Expression> {
        let mut expr = self.term()?;

        while let Some(operator) = self.advance_on_match(&[
            TokenDiscriminant::Greater,
            TokenDiscriminant::GreaterEqual,
            TokenDiscriminant::Less,
            TokenDiscriminant::LessEqual,
        ]) {
            expr = Expression::binary(expr, operator, self.term()?);
        }
        Some(expr)
    }

    fn term(&mut self) -> Option<Expression> {
        let mut expr = self.factor()?;

        while let Some(operator) =
            self.advance_on_match(&[TokenDiscriminant::Minus, TokenDiscriminant::Plus])
        {
            expr = Expression::binary(expr, operator, self.factor()?);
        }
        Some(expr)
    }

    fn factor(&mut self) -> Option<Expression> {
        let mut expr = self.unary()?;

        while let Some(operator) =
            self.advance_on_match(&[TokenDiscriminant::Slash, TokenDiscriminant::Star])
        {
            expr = Expression::binary(expr, operator, self.unary()?);
        }
        Some(expr)
    }

    fn unary(&mut self) -> Option<Expression> {
        if let Some(operator) =
            self.advance_on_match(&[TokenDiscriminant::Bang, TokenDiscriminant::Minus])
        {
            Some(Expression::unary(operator, self.unary()?))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Option<Expression> {
        let mut callee = self.primary()?;

        loop {
            if self
                .advance_on_match(&[TokenDiscriminant::LeftParen])
                .is_some()
            {
                callee = self.finish_call(callee)?;
            } else {
                break;
            }
        }
        Some(callee)
    }

    fn finish_call(&mut self, callee: Expression) -> Option<Expression> {
        let mut arguments = vec![];
        if self.peek()?.discriminant() != TokenDiscriminant::RightParen {
            loop {
                arguments.push(self.expression()?);
                if self.peek()?.discriminant() != TokenDiscriminant::Comma {
                    break;
                }
            }
        }
        let closing_parenthesis = self.expect(TokenDiscriminant::RightParen)?;
        if arguments.len() >= 255 {
            // Ugly, we should set `has_errored` here.
            println!("You can't have more than 255 arguments");
            return None;
        }
        Some(Expression::call(callee, closing_parenthesis, arguments))
    }

    fn primary(&mut self) -> Option<Expression> {
        if self.advance_on_match(&[TokenDiscriminant::True]).is_some() {
            Some(Expression::boolean(true))
        } else if self.advance_on_match(&[TokenDiscriminant::False]).is_some() {
            Some(Expression::boolean(false))
        } else if let Some(t) = self.advance_on_match(&[TokenDiscriminant::Nil]) {
            Some(Expression::null(t))
        } else if let Some(t) = self.advance_on_match(&[TokenDiscriminant::Number]) {
            Some(Expression::number(t))
        } else if let Some(t) = self.advance_on_match(&[TokenDiscriminant::String]) {
            Some(Expression::string(t))
        } else if let Some(t) = self.advance_on_match(&[TokenDiscriminant::Identifier]) {
            Some(Expression::variable_reference(t))
        } else if self
            .advance_on_match(&[TokenDiscriminant::LeftParen])
            .is_some()
        {
            let expr = self.expression()?;
            self.expect(TokenDiscriminant::RightParen)?;
            Some(Expression::grouping(expr))
        } else {
            self.mode = ParsingMode::ErrorRecovery;
            None
        }
    }

    fn advance_on_match(&mut self, token_types: &[TokenDiscriminant]) -> Option<Token> {
        let upcoming = self.tokens.peek()?;
        if token_types.contains(&upcoming.discriminant()) {
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
                if current.discriminant() == TokenDiscriminant::Semicolon {
                    break None;
                }
                let upcoming = self.tokens.peek()?;
                match upcoming.discriminant() {
                    TokenDiscriminant::Class
                    | TokenDiscriminant::Fun
                    | TokenDiscriminant::Var
                    | TokenDiscriminant::For
                    | TokenDiscriminant::If
                    | TokenDiscriminant::Print
                    | TokenDiscriminant::Return
                    | TokenDiscriminant::While => {
                        break None;
                    }
                    _ => {}
                }
            }
        };
        let _ = recover();
    }

    fn expect(&mut self, token_type: TokenDiscriminant) -> Option<Token> {
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

    fn peek(&mut self) -> Option<&Token> {
        if self.mode == ParsingMode::Normal {
            self.tokens.peek()
        } else {
            None
        }
    }

    fn is_at_end(&mut self) -> bool {
        self.tokens.peek().is_none()
    }
}

/// Our parser does not care about trivia tokens.
/// We give `Source` to our parser instead of the raw token stream: `Source` wraps the underlying
/// token stream and makes sure to skip all trivia tokens, making them invisible to the parser.
struct Source<TokenIter>(TokenIter)
where
    TokenIter: Iterator<Item = Token>;

impl<TokenIter> Iterator for Source<TokenIter>
where
    TokenIter: Iterator<Item = Token>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                None => break None,
                Some(t) if t.discriminant() == TokenDiscriminant::Trivia => continue,
                Some(t) => break Some(t),
            }
        }
    }
}

#[allow(unused)]
pub fn display_ast(s: &Statement) -> Result<String, std::fmt::Error> {
    let mut buffer = String::new();
    _display_statement(&mut buffer, s, 0)?;
    Ok(buffer)
}

fn _display_statement(w: &mut impl Write, s: &Statement, depth: u8) -> Result<(), std::fmt::Error> {
    // Can we avoid an allocation for the indentation string here?
    write!(w, "{}", " ".repeat(depth as usize))?;
    match s {
        Statement::Expression(ExpressionStatement(e)) => {
            writeln!(w, "Expression")?;
            _display_expression(w, &e, depth + 1)?;
        }
        Statement::Print(PrintStatement(e)) => {
            writeln!(w, "Print")?;
            _display_expression(w, &e, depth + 1)?;
        }
        Statement::VariableDeclaration(VariableDeclarationStatement {
            initializer,
            identifier,
        }) => {
            writeln!(w, "Variable Declaration")?;
            _display_token(w, &identifier, depth + 1)?;
            if let Some(e) = initializer {
                _display_expression(w, &e, depth + 1)?;
            }
        }
        Statement::Block(BlockStatement(statements)) => {
            writeln!(w, "Block")?;
            for statement in statements {
                _display_statement(w, statement, depth + 1)?;
            }
        }
        Statement::IfElse(IfElseStatement {
            condition,
            if_branch,
            else_branch,
        }) => {
            writeln!(w, "IfElse")?;
            _display_expression(w, condition, depth + 1)?;
            _display_statement(w, if_branch, depth + 1)?;
            if let Some(else_branch) = else_branch {
                _display_statement(w, else_branch, depth + 1)?;
            }
        }
        Statement::While(WhileStatement { condition, body }) => {
            writeln!(w, "While")?;
            _display_expression(w, condition, depth + 1)?;
            _display_statement(w, body, depth + 1)?;
        }
        Statement::FunctionDeclaration(FunctionDeclarationStatement {
            name,
            parameters,
            body,
        }) => {
            writeln!(w, "Function Declaration")?;
            _display_token(w, &name, depth + 1)?;
            _display_string(w, "Parameters", depth + 1)?;
            for parameter in parameters {
                _display_token(w, &parameter, depth + 2)?;
            }
            _display_string(w, "Body", depth + 1)?;
            for s in body {
                _display_statement(w, s, depth + 2)?;
            }
        }
    }
    Ok(())
}

fn _display_expression(
    w: &mut impl Write,
    e: &Expression,
    depth: u8,
) -> Result<(), std::fmt::Error> {
    // Can we avoid an allocation for the indentation string here?
    write!(w, "{}", " ".repeat(depth as usize))?;
    match e {
        Expression::Binary(b) => {
            writeln!(w, "Binary")?;
            _display_expression(w, &b.left, depth + 1)?;
            _display_token(w, &b.operator, depth + 1)?;
            _display_expression(w, &b.right, depth + 1)?;
        }
        Expression::Unary(u) => {
            writeln!(w, "Unary")?;
            _display_token(w, &u.operator, depth + 1)?;
            _display_expression(w, &u.operand, depth + 1)?;
        }
        Expression::Literal(l) => {
            writeln!(w, "Literal")?;
            match l {
                LiteralExpression::Null(t)
                | LiteralExpression::String(t)
                | LiteralExpression::Number(t) => {
                    _display_token(w, t, depth + 1)?;
                }
                LiteralExpression::Boolean(b) => {
                    let s = if *b { "True" } else { "False" };
                    _display_string(w, s, depth + 1)?;
                }
            }
        }
        Expression::Grouping(g) => {
            writeln!(w, "Grouping")?;
            _display_expression(w, &g.0, depth + 1)?;
        }
        Expression::VariableReference(VariableReferenceExpression { identifier }) => {
            writeln!(w, "Variable Reference")?;
            _display_token(w, identifier, depth + 1)?;
        }
        Expression::VariableAssignment(VariableAssignmentExpression { identifier, value }) => {
            writeln!(w, "Variable Assignment")?;
            _display_token(w, identifier, depth + 1)?;
            _display_expression(w, value, depth + 1)?;
        }
        Expression::Call(CallExpression {
            callee, arguments, ..
        }) => {
            writeln!(w, "Call")?;
            _display_expression(w, callee, depth + 1)?;
            _display_string(w, "Arguments", depth + 1)?;
            for argument in arguments {
                _display_expression(w, argument, depth + 2)?;
            }
        }
    }
    Ok(())
}

fn _display_token(w: &mut impl Write, t: &Token, depth: u8) -> std::fmt::Result {
    // Can we avoid an allocation for the indentation string here?
    write!(w, "{}", " ".repeat(depth as usize))?;
    write!(w, "{:?}", t.discriminant())?;
    match t.ty() {
        TokenType::String(s) => writeln!(w, " \"{}\"", s)?,
        TokenType::Number(n) => writeln!(w, " {}", n)?,
        _ => writeln!(w, "")?,
    }
    Ok(())
}

fn _display_string(w: &mut impl Write, s: &str, depth: u8) -> std::fmt::Result {
    // Can we avoid an allocation for the indentation string here?
    write!(w, "{}", " ".repeat(depth as usize))?;
    writeln!(w, "{}", s)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::parser::{display_ast, Parser};
    use crate::scanner::Scanner;
    use insta::assert_display_snapshot;

    fn parse(source: &str) -> String {
        if let Ok(statements) = Parser::parse(Scanner::new(source)) {
            display_ast(&statements[0]).unwrap()
        } else {
            panic!("Failed to parse the source code")
        }
    }

    #[test]
    fn parse_string_expression() {
        let ast = parse(r#""My name is Luça";"#);
        assert_display_snapshot!(ast, @r###"
        Expression
         Literal
          String "My name is Luça"
        "###)
    }

    #[test]
    fn parse_number() {
        let ast = parse(r#"12.65;"#);
        assert_display_snapshot!(ast, @r###"
        Expression
         Literal
          Number 12.65
        "###)
    }

    #[test]
    fn parse_binary() {
        let ast = parse(r#"12.65 + 2;"#);
        assert_display_snapshot!(ast, @r###"
        Expression
         Binary
          Literal
           Number 12.65
          Plus
          Literal
           Number 2
        "###)
    }

    #[test]
    fn parse_binary_without_parens() {
        let ast = parse(r#"12.65 + 2 * 3;"#);
        assert_display_snapshot!(ast, @r###"
        Expression
         Binary
          Literal
           Number 12.65
          Plus
          Binary
           Literal
            Number 2
           Star
           Literal
            Number 3
        "###)
    }

    #[test]
    fn parse_binary_with_parens() {
        let ast = parse(r#"(12.65 + 2) * 3;"#);
        assert_display_snapshot!(ast, @r###"
        Expression
         Binary
          Grouping
           Binary
            Literal
             Number 12.65
            Plus
            Literal
             Number 2
          Star
          Literal
           Number 3
        "###)
    }

    #[test]
    fn parse_complex_equality() {
        let ast = parse(r#"!((12 + 2) * 3) == 50 / 12;"#);
        assert_display_snapshot!(ast, @r###"
        Expression
         Binary
          Unary
           Bang
           Grouping
            Binary
             Grouping
              Binary
               Literal
                Number 12
               Plus
               Literal
                Number 2
             Star
             Literal
              Number 3
          EqualEqual
          Binary
           Literal
            Number 50
           Slash
           Literal
            Number 12
        "###)
    }

    #[test]
    fn parse_print_statement() {
        let ast = parse(r#"print 2+5;"#);
        assert_display_snapshot!(ast, @r###"
        Print
         Binary
          Literal
           Number 2
          Plus
          Literal
           Number 5
        "###)
    }

    #[test]
    fn parse_logical_statement() {
        let ast = parse(r#"true and 2+5 or true;"#);
        assert_display_snapshot!(ast, @r###"
        Expression
         Binary
          Binary
           Literal
            True
           And
           Binary
            Literal
             Number 2
            Plus
            Literal
             Number 5
          Or
          Literal
           True
        "###)
    }
}

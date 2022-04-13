use itertools::{Itertools, MultiPeek};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::str::{Chars, FromStr};

pub struct Scanner<'a> {
    source: MultiPeek<Chars<'a>>,
    tokens: Vec<Token>,
    current_token_buffer: Vec<char>,
    current_line: u64,
    keywords: HashMap<String, TokenType>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let keywords = HashMap::from_iter([
            ("and".into(), TokenType::And),
            ("class".into(), TokenType::Class),
            ("else".into(), TokenType::Else),
            ("false".into(), TokenType::False),
            ("for".into(), TokenType::For),
            ("fun".into(), TokenType::Fun),
            ("if".into(), TokenType::If),
            ("nil".into(), TokenType::Nil),
            ("or".into(), TokenType::Or),
            ("print".into(), TokenType::Print),
            ("return".into(), TokenType::Return),
            ("super".into(), TokenType::Super),
            ("this".into(), TokenType::This),
            ("true".into(), TokenType::True),
            ("var".into(), TokenType::Var),
            ("while".into(), TokenType::While),
        ]);
        Self {
            source: source.chars().multipeek(),
            tokens: Vec::new(),
            current_token_buffer: Vec::new(),
            current_line: 0,
            keywords,
        }
    }

    pub fn scan_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.scan_token();
        }
        self.tokens.push(Token {
            ty: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line: self.current_line,
        });
        self.tokens
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.finalize_current_token(TokenType::LeftParen),
            ')' => self.finalize_current_token(TokenType::RightParen),
            '{' => self.finalize_current_token(TokenType::LeftBrace),
            '}' => self.finalize_current_token(TokenType::RightBrace),
            ',' => self.finalize_current_token(TokenType::Comma),
            '.' => self.finalize_current_token(TokenType::Dot),
            '-' => self.finalize_current_token(TokenType::Minus),
            '+' => self.finalize_current_token(TokenType::Plus),
            ';' => self.finalize_current_token(TokenType::Semicolon),
            '*' => self.finalize_current_token(TokenType::Star),
            '!' => {
                if self.advance_on_match('=') {
                    self.finalize_current_token(TokenType::BangEqual)
                } else {
                    self.finalize_current_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.advance_on_match('=') {
                    self.finalize_current_token(TokenType::EqualEqual)
                } else {
                    self.finalize_current_token(TokenType::Equal)
                }
            }
            '<' => {
                if self.advance_on_match('=') {
                    self.finalize_current_token(TokenType::LessEqual)
                } else {
                    self.finalize_current_token(TokenType::Less)
                }
            }
            '>' => {
                if self.advance_on_match('=') {
                    self.finalize_current_token(TokenType::GreaterEqual)
                } else {
                    self.finalize_current_token(TokenType::Greater)
                }
            }
            '/' => {
                if self.advance_on_match('/') {
                    // Eat the entire comment, until we encounter a line break
                    self.advance_until('\n');
                    // Empty the token buffer - we don't care about comments.
                    self.current_token_buffer.clear();
                } else {
                    self.finalize_current_token(TokenType::Slash)
                }
            }
            ' ' | '\r' | '\t' => {
                // We ignore all trivia characters.
                self.current_token_buffer.clear();
            }
            '\n' => {
                self.current_token_buffer.clear();
            }
            '"' => {
                self.advance_until('"');
                if self.is_at_end() {
                    self.finalize_error_token(Some("Unterminated string"));
                    return;
                }
                // Eat the closing `"`
                self.advance();
                let lexeme = self.finalize_buffer_into_lexeme();
                let literal = lexeme.trim_matches('"').to_string();
                self.tokens.push(Token {
                    ty: TokenType::String,
                    lexeme,
                    literal: Some(Literal::String(literal)),
                    line: self.current_line,
                })
            }
            d if d.is_ascii_digit() => {
                self.advance_while_true(|c| c.is_ascii_digit());
                if self.peek() == Some(&'.') {
                    if let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            // Consume `.`
                            self.advance();
                            self.advance_while_true(|c| c.is_ascii_digit());
                        }
                    }
                }
                let lexeme = String::from_iter(self.current_token_buffer.drain(..));
                match f64::from_str(&lexeme) {
                    Ok(f) => self.tokens.push(Token {
                        ty: TokenType::Number,
                        lexeme,
                        literal: Some(Literal::Number(f)),
                        line: self.current_line,
                    }),
                    Err(_) => self.finalize_error_token(Some("Failed to parse number")),
                }
            }
            c => {
                if Self::is_alpha(&c) {
                    self.advance_while_true(|c| Self::is_alpha(c) || c.is_ascii_digit());
                    let lexeme = self.finalize_buffer_into_lexeme();
                    match self.keywords.get(&lexeme) {
                        None => self.tokens.push(Token {
                            ty: TokenType::Identifier,
                            lexeme,
                            literal: None,
                            line: self.current_line,
                        }),
                        Some(ty) => self.tokens.push(Token {
                            ty: *ty,
                            lexeme,
                            literal: None,
                            line: self.current_line,
                        }),
                    }
                } else {
                    self.finalize_error_token(None)
                }
            }
        }
    }

    fn is_alpha(c: &char) -> bool {
        c.is_ascii_alphanumeric() || c == &'_'
    }

    fn finalize_error_token(&mut self, error_msg: Option<&'static str>) {
        self.finalize_current_token(TokenType::SyntaxError { error_msg })
    }

    fn finalize_current_token(&mut self, ty: TokenType) {
        let lexeme = self.finalize_buffer_into_lexeme();
        self.tokens.push(Token {
            ty,
            lexeme,
            literal: None,
            line: self.current_line,
        })
    }

    fn finalize_buffer_into_lexeme(&mut self) -> String {
        String::from_iter(self.current_token_buffer.drain(..))
    }

    fn advance(&mut self) -> char {
        let char = self.source.next().unwrap();
        self.current_token_buffer.push(char);
        if char == '\n' {
            self.current_line += 1;
        }
        char
    }

    fn advance_on_match(&mut self, c: char) -> bool {
        if self.peek() == Some(&c) {
            self.advance();
            true
        } else {
            self.source.reset_peek();
            false
        }
    }

    fn advance_until(&mut self, c: char) {
        self.advance_while_true(|ch| ch != &c)
    }

    fn advance_while_true<F>(&mut self, f: F)
    where
        F: Fn(&char) -> bool,
    {
        loop {
            let next = self.peek();
            if let Some(next) = next {
                if f(next) {
                    self.advance();
                    continue;
                }
            }
            break;
        }
        self.source.reset_peek();
    }

    fn peek(&mut self) -> Option<&char> {
        self.source.peek()
    }

    fn is_at_end(&mut self) -> bool {
        let b = self.peek().is_none();
        self.source.reset_peek();
        b
    }
}

#[derive(Debug)]
enum Literal {
    String(String),
    Number(f64),
}

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(s) => s.fmt(f),
            Literal::Number(n) => n.fmt(f),
        }
    }
}

#[derive(Debug)]
pub struct Token {
    ty: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: u64,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let literal = self
            .literal
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or("".to_string());
        write!(
            f,
            "{} - {:?} {} {}",
            self.line, self.ty, self.lexeme, literal
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TokenType {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // End of file
    Eof,

    // Special token to signal that we encountered a token
    // that we couldn't successfully scan.
    // The scanner can choose to specify an error message to
    // help the user understand what it was attempting to do
    // before giving up.
    SyntaxError { error_msg: Option<&'static str> },
}

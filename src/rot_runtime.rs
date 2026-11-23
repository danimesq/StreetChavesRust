//! Source-preserving lexer and parser for the pinned Pascal ROT dialect.
//!
//! This module deliberately stops at parsing. It does not evaluate expressions,
//! mutate fighters, call game helpers, or consume RNG. The executable runtime is
//! a separate milestone and must preserve the Pascal semantics documented in
//! docs/rot-runtime.md.

use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SourceSpan {
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub start_offset: usize,
    pub end_offset: usize,
}

impl SourceSpan {
    fn point(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            end_line: line,
            end_column: column,
            start_offset: offset,
            end_offset: offset,
        }
    }

    fn join(first: Self, last: Self) -> Self {
        Self {
            line: first.line,
            column: first.column,
            end_line: last.end_line,
            end_column: last.end_column,
            start_offset: first.start_offset,
            end_offset: last.end_offset,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotError {
    pub source: String,
    pub span: SourceSpan,
    pub message: String,
}

impl RotError {
    fn new(source: &str, span: SourceSpan, message: impl Into<String>) -> Self {
        Self {
            source: source.to_owned(),
            span,
            message: message.into(),
        }
    }
}

impl fmt::Display for RotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.source, self.span.line, self.span.column, self.message
        )
    }
}

impl Error for RotError {}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Real(f64),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenKind {
    Identifier,
    Integer,
    Real,
    String,
    Symbol,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
struct Token {
    kind: TokenKind,
    text: String,
    literal: Option<Literal>,
    span: SourceSpan,
}

struct Lexer<'a> {
    source_name: &'a str,
    chars: Vec<char>,
    byte_offsets: Vec<usize>,
    index: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    fn new(source_name: &'a str, text: &str, first_line: usize) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let mut byte_offsets = Vec::with_capacity(chars.len() + 1);
        let mut byte_offset = 0;
        for ch in &chars {
            byte_offsets.push(byte_offset);
            byte_offset += ch.len_utf8();
        }
        byte_offsets.push(byte_offset);
        Self {
            source_name,
            chars,
            byte_offsets,
            index: 0,
            line: first_line,
            column: 1,
        }
    }

    fn current(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn starts_with(&self, first: char, second: char) -> bool {
        self.current() == Some(first) && self.next() == Some(second)
    }

    fn offset(&self) -> usize {
        self.byte_offsets[self.index]
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current()?;
        self.index += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn point(&self) -> SourceSpan {
        SourceSpan::point(self.line, self.column, self.offset())
    }

    fn error(&self, span: SourceSpan, message: impl Into<String>) -> RotError {
        RotError::new(self.source_name, span, message)
    }

    fn token(
        &self,
        kind: TokenKind,
        start_index: usize,
        start_span: SourceSpan,
        literal: Option<Literal>,
    ) -> Token {
        let text: String = self.chars[start_index..self.index].iter().collect();
        let mut span = start_span;
        span.end_line = self.line;
        span.end_column = self.column;
        span.end_offset = self.offset();
        Token {
            kind,
            text,
            literal,
            span,
        }
    }

    fn skip_brace_comment(&mut self, start: SourceSpan) -> Result<(), RotError> {
        self.advance();
        while let Some(ch) = self.current() {
            if ch == '}' {
                self.advance();
                return Ok(());
            }
            self.advance();
        }
        Err(self.error(start, "unterminated { comment"))
    }

    fn skip_paren_comment(&mut self, start: SourceSpan) -> Result<(), RotError> {
        self.advance();
        self.advance();
        while self.current().is_some() {
            if self.starts_with('*', ')') {
                self.advance();
                self.advance();
                return Ok(());
            }
            self.advance();
        }
        Err(self.error(start, "unterminated (* comment *)"))
    }

    fn lex_string(&mut self, start_index: usize, start: SourceSpan) -> Result<Token, RotError> {
        self.advance(); // opening quote
        let mut value = String::new();
        loop {
            match self.current() {
                None => return Err(self.error(start, "unterminated string literal")),
                Some('\'') => {
                    self.advance();
                    if self.current() == Some('\'') {
                        self.advance();
                        value.push('\'');
                    } else {
                        break;
                    }
                }
                Some(ch) => {
                    self.advance();
                    value.push(ch);
                }
            }
        }
        Ok(self.token(
            TokenKind::String,
            start_index,
            start,
            Some(Literal::String(value)),
        ))
    }

    fn lex_number(&mut self, start_index: usize, start: SourceSpan) -> Result<Token, RotError> {
        if self.current() == Some('$') {
            self.advance();
            let digits_start = self.index;
            while self.current().is_some_and(|ch| ch.is_ascii_hexdigit()) {
                self.advance();
            }
            if self.index == digits_start {
                return Err(self.error(start, "hex literal needs digits"));
            }
            let digits: String = self.chars[digits_start..self.index].iter().collect();
            let value = i64::from_str_radix(&digits, 16)
                .map_err(|_| self.error(start, "hex literal is outside the supported integer range"))?;
            return Ok(self.token(
                TokenKind::Integer,
                start_index,
                start,
                Some(Literal::Integer(value)),
            ));
        }

        while self.current().is_some_and(|ch| ch.is_ascii_digit()) {
            self.advance();
        }

        let mut is_real = false;
        if self.current() == Some('.')
            && self.next().is_some_and(|ch| ch.is_ascii_digit())
        {
            is_real = true;
            self.advance();
            while self.current().is_some_and(|ch| ch.is_ascii_digit()) {
                self.advance();
            }
        }

        if matches!(self.current(), Some('e' | 'E')) {
            is_real = true;
            self.advance();
            if matches!(self.current(), Some('+' | '-')) {
                self.advance();
            }
            let exponent_start = self.index;
            while self.current().is_some_and(|ch| ch.is_ascii_digit()) {
                self.advance();
            }
            if exponent_start == self.index {
                return Err(self.error(start, "malformed real exponent"));
            }
        }

        let raw: String = self.chars[start_index..self.index].iter().collect();
        if is_real {
            let value = raw
                .parse::<f64>()
                .map_err(|_| self.error(start, "invalid real literal"))?;
            Ok(self.token(
                TokenKind::Real,
                start_index,
                start,
                Some(Literal::Real(value)),
            ))
        } else {
            let value = raw
                .parse::<i64>()
                .map_err(|_| self.error(start, "integer literal is outside the supported range"))?;
            Ok(self.token(
                TokenKind::Integer,
                start_index,
                start,
                Some(Literal::Integer(value)),
            ))
        }
    }

    fn lex_identifier(&mut self, start_index: usize, start: SourceSpan) -> Token {
        self.advance();
        while self
            .current()
            .is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
        {
            self.advance();
        }
        self.token(TokenKind::Identifier, start_index, start, None)
    }

    fn run(mut self) -> Result<Vec<Token>, RotError> {
        let mut tokens = Vec::new();
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
                continue;
            }
            if self.starts_with('/', '/') {
                while self.current().is_some_and(|current| current != '\n') {
                    self.advance();
                }
                continue;
            }
            if ch == '{' {
                let start = self.point();
                self.skip_brace_comment(start)?;
                continue;
            }
            if self.starts_with('(', '*') {
                let start = self.point();
                self.skip_paren_comment(start)?;
                continue;
            }

            let start_index = self.index;
            let start = self.point();
            if ch == '\'' {
                tokens.push(self.lex_string(start_index, start)?);
                continue;
            }
            if ch == '$' || ch.is_ascii_digit() {
                tokens.push(self.lex_number(start_index, start)?);
                continue;
            }
            if ch.is_alphabetic() || ch == '_' {
                tokens.push(self.lex_identifier(start_index, start));
                continue;
            }

            if matches!(
                (ch, self.next()),
                (':', Some('='))
                    | ('<', Some('='))
                    | ('>', Some('='))
                    | ('<', Some('>'))
                    | ('.', Some('.'))
            ) {
                self.advance();
                self.advance();
                tokens.push(self.token(TokenKind::Symbol, start_index, start, None));
                continue;
            }

            if "+-*/=<>()[].,:;".contains(ch) {
                self.advance();
                tokens.push(self.token(TokenKind::Symbol, start_index, start, None));
                continue;
            }

            return Err(self.error(start, format!("unexpected character {ch:?}")));
        }

        let point = self.point();
        tokens.push(Token {
            kind: TokenKind::Eof,
            text: String::new(),
            literal: None,
            span: point,
        });
        Ok(tokens)
    }
}

pub fn lex(source_name: &str, text: &str) -> Result<Vec<(String, SourceSpan)>, RotError> {
    Lexer::new(source_name, text, 1)
        .run()
        .map(|tokens| tokens.into_iter().map(|token| (token.text, token.span)).collect())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PascalType {
    Byte,
    ShortInt,
    SmallInt,
    Word,
    Integer,
    Int64,
    Double,
    Boolean,
    Other(String),
}

impl PascalType {
    fn from_name(name: String) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "byte" => Self::Byte,
            "shortint" => Self::ShortInt,
            "smallint" => Self::SmallInt,
            "word" => Self::Word,
            "integer" | "longint" => Self::Integer,
            "int64" => Self::Int64,
            "double" => Self::Double,
            "boolean" => Self::Boolean,
            _ => Self::Other(name),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    pub names: Vec<String>,
    pub declared_type: PascalType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Not,
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    In,
    Add,
    Subtract,
    Or,
    Xor,
    Multiply,
    RealDivide,
    IntegerDivide,
    Modulo,
    And,
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Literal(Literal),
    Identifier(String),
    FighterField { base: char, name: String },
    Call { name: String, arguments: Vec<Expression> },
    Unary {
        operator: UnaryOperator,
        value: Box<Expression>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Group(Box<Expression>),
    Set(Vec<SetElement>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetElement {
    Value(Expression),
    Range {
        low: Expression,
        high: Expression,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForDirection {
    To,
    DownTo,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatementKind {
    Block(Vec<Statement>),
    Assign {
        target: Expression,
        value: Expression,
    },
    Call(Expression),
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    For {
        variable: String,
        direction: ForDirection,
        first: Expression,
        last: Expression,
        body: Box<Statement>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    RepeatUntil {
        body: Vec<Statement>,
        condition: Expression,
    },
    Case {
        selector: Expression,
        arms: Vec<CaseArm>,
        otherwise: Option<Vec<Statement>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseArm {
    pub labels: Vec<SetElement>,
    pub body: Statement,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RotProgram {
    pub source: String,
    pub section: String,
    pub declarations: Vec<VariableDeclaration>,
    pub body: Statement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RotSection {
    pub name: String,
    pub source: String,
    pub first_line: usize,
    pub text: String,
}

impl RotSection {
    pub fn parse(&self) -> Result<RotProgram, RotError> {
        Parser::try_new(&self.source, &self.text, self.first_line)?.parse(&self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RotDocument {
    pub source: String,
    sections: Vec<RotSection>,
}

fn store_section(
    sections: &mut Vec<RotSection>,
    source: &str,
    name: Option<String>,
    body: &mut String,
    first_line: usize,
) {
    if let Some(name) = name {
        sections.push(RotSection {
            name,
            source: source.to_owned(),
            first_line,
            text: std::mem::take(body),
        });
    }
}

impl RotDocument {
    pub fn from_str(source: &str, text: &str) -> Result<Self, RotError> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut document = Self {
            source: source.to_owned(),
            sections: Vec::new(),
        };
        let mut active_name: Option<String> = None;
        let mut active_body = String::new();
        let mut first_line = 1usize;

        for (index, line) in text.split_inclusive('\n').enumerate() {
            let line_without_newline = line.trim_end_matches(|ch| ch == '\n' || ch == '\r');
            let trimmed = line_without_newline.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() >= 2 {
                store_section(
                    &mut document.sections,
                    source,
                    active_name.take(),
                    &mut active_body,
                    first_line,
                );
                let name = trimmed[1..trimmed.len() - 1].trim();
                if name.is_empty() {
                    return Err(RotError::new(
                        source,
                        SourceSpan::point(index + 1, 1, 0),
                        "empty ROT section name",
                    ));
                }
                active_name = Some(name.to_owned());
                first_line = index + 2;
                continue;
            }

            if active_name.is_some() {
                if line_without_newline.trim_start().starts_with(';') {
                    if line.ends_with('\n') {
                        active_body.push('\n');
                    }
                } else {
                    active_body.push_str(line);
                }
            }
        }
        store_section(
            &mut document.sections,
            source,
            active_name,
            &mut active_body,
            first_line,
        );
        Ok(document)
    }

    /// Iterate sections in their original source-file order.
    pub fn sections(&self) -> impl Iterator<Item = &RotSection> {
        self.sections.iter()
    }

    pub fn section(&self, name: &str) -> Option<&RotSection> {
        self.sections
            .iter()
            .find(|section| section.name.eq_ignore_ascii_case(name))
    }

    /// Missing optional sections stay absent rather than receiving fabricated scripts.
    pub fn parse_optional(&self, name: &str) -> Result<Option<RotProgram>, RotError> {
        self.section(name).map(RotSection::parse).transpose()
    }
}

struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    position: usize,
}

impl<'a> Parser<'a> {
    fn try_new(
        source: &'a str,
        text: &str,
        first_line: usize,
    ) -> Result<Self, RotError> {
        Ok(Self {
            source,
            tokens: Lexer::new(source, text, first_line).run()?,
            position: 0,
        })
    }

    fn token(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn at(&self, text: &str) -> bool {
        self.token().text.eq_ignore_ascii_case(text)
    }

    fn at_identifier(&self) -> bool {
        self.token().kind == TokenKind::Identifier
    }

    fn bump(&mut self) -> Token {
        let token = self.token().clone();
        if token.kind != TokenKind::Eof {
            self.position += 1;
        }
        token
    }

    fn error(&self, message: impl Into<String>) -> RotError {
        RotError::new(self.source, self.token().span, message)
    }

    fn expect(&mut self, text: &str) -> Result<Token, RotError> {
        if !self.at(text) {
            return Err(self.error(format!(
                "expected {text:?}, found {:?}",
                self.token().text
            )));
        }
        Ok(self.bump())
    }

    fn expect_identifier(&mut self) -> Result<Token, RotError> {
        if !self.at_identifier() {
            return Err(self.error(format!(
                "expected identifier, found {:?}",
                self.token().text
            )));
        }
        Ok(self.bump())
    }

    fn eat(&mut self, text: &str) -> bool {
        if self.at(text) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn statement(
        &mut self,
        kind: StatementKind,
        start: SourceSpan,
        end: SourceSpan,
    ) -> Statement {
        Statement {
            kind,
            span: SourceSpan::join(start, end),
        }
    }

    fn expression(
        &mut self,
        kind: ExpressionKind,
        start: SourceSpan,
        end: SourceSpan,
    ) -> Expression {
        Expression {
            kind,
            span: SourceSpan::join(start, end),
        }
    }

    fn parse(mut self, section: &str) -> Result<RotProgram, RotError> {
        let mut declarations = Vec::new();
        while self.at("var") {
            declarations.extend(self.parse_variable_declarations()?);
        }
        let body = self.parse_statement_list(&[])?;
        if self.token().kind != TokenKind::Eof {
            return Err(self.error(format!(
                "unexpected trailing token {:?}",
                self.token().text
            )));
        }
        Ok(RotProgram {
            source: self.source.to_owned(),
            section: section.to_owned(),
            declarations,
            body,
        })
    }

    fn parse_variable_declarations(&mut self) -> Result<Vec<VariableDeclaration>, RotError> {
        self.expect("var")?;
        let mut names = vec![self.expect_identifier()?];
        while self.eat(",") {
            names.push(self.expect_identifier()?);
        }
        self.expect(":")?;
        let declared_type = self.expect_identifier()?;
        let end = self.expect(";")?;
        let span = SourceSpan::join(names[0].span, end.span);
        Ok(vec![VariableDeclaration {
            names: names.into_iter().map(|token| token.text).collect(),
            declared_type: PascalType::from_name(declared_type.text),
            span,
        }])
    }

    fn is_stop(&self, stop: &[&str]) -> bool {
        stop.iter().any(|word| self.at(word))
    }

    fn parse_statement_list(&mut self, stop: &[&str]) -> Result<Statement, RotError> {
        let start = self.token().span;
        let mut statements = Vec::new();

        while self.token().kind != TokenKind::Eof && !self.is_stop(stop) {
            if self.eat(";") {
                continue;
            }
            let before = self.position;
            statements.push(self.parse_statement()?);
            if self.eat(";") {
                continue;
            }
            if self.token().kind != TokenKind::Eof && !self.is_stop(stop) {
                return Err(self.error(format!(
                    "expected ';', found {:?}",
                    self.token().text
                )));
            }
            if before == self.position {
                return Err(self.error("parser made no progress"));
            }
        }

        let end = statements.last().map(|statement| statement.span).unwrap_or(start);
        Ok(self.statement(StatementKind::Block(statements), start, end))
    }

    fn empty_statement(&mut self) -> Statement {
        let span = self.token().span;
        self.statement(StatementKind::Block(Vec::new()), span, span)
    }

    fn parse_statement_or_empty(&mut self) -> Result<Statement, RotError> {
        if self.at("else") || self.at("end") || self.at("until") || self.at(";") {
            Ok(self.empty_statement())
        } else {
            self.parse_statement()
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, RotError> {
        let start = self.token().span;
        if self.eat("begin") {
            let block = self.parse_statement_list(&["end"])?;
            let end = self.expect("end")?.span;
            return Ok(self.statement(
                StatementKind::Block(match block.kind {
                    StatementKind::Block(items) => items,
                    _ => unreachable!(),
                }),
                start,
                end,
            ));
        }

        if self.eat("if") {
            let condition = self.parse_expression()?;
            self.expect("then")?;
            let then_branch = Box::new(self.parse_statement_or_empty()?);
            let else_branch = if self.eat("else") {
                Some(Box::new(self.parse_statement_or_empty()?))
            } else {
                None
            };
            let end = else_branch
                .as_ref()
                .map(|statement| statement.span)
                .unwrap_or(then_branch.span);
            return Ok(self.statement(
                StatementKind::If {
                    condition,
                    then_branch,
                    else_branch,
                },
                start,
                end,
            ));
        }

        if self.eat("for") {
            let variable = self.expect_identifier()?.text;
            self.expect(":=")?;
            let first = self.parse_expression()?;
            let direction = if self.eat("to") {
                ForDirection::To
            } else {
                self.expect("downto")?;
                ForDirection::DownTo
            };
            let last = self.parse_expression()?;
            self.expect("do")?;
            let body = Box::new(self.parse_statement_or_empty()?);
            return Ok(self.statement(
                StatementKind::For {
                    variable,
                    direction,
                    first,
                    last,
                    body: body.clone(),
                },
                start,
                body.span,
            ));
        }

        if self.eat("while") {
            let condition = self.parse_expression()?;
            self.expect("do")?;
            let body = Box::new(self.parse_statement_or_empty()?);
            return Ok(self.statement(
                StatementKind::While {
                    condition,
                    body: body.clone(),
                },
                start,
                body.span,
            ));
        }

        if self.eat("repeat") {
            let body = self.parse_statement_list(&["until"])?;
            self.expect("until")?;
            let condition = self.parse_expression()?;
            return Ok(self.statement(
                StatementKind::RepeatUntil {
                    body: match body.kind {
                        StatementKind::Block(items) => items,
                        _ => unreachable!(),
                    },
                    condition: condition.clone(),
                },
                start,
                condition.span,
            ));
        }

        if self.eat("case") {
            let selector = self.parse_expression()?;
            self.expect("of")?;
            let mut arms = Vec::new();
            let mut otherwise = None;

            while self.token().kind != TokenKind::Eof && !self.at("end") {
                if self.eat("else") {
                    let default = self.parse_statement_list(&["end"])?;
                    otherwise = match default.kind {
                        StatementKind::Block(items) => Some(items),
                        _ => unreachable!(),
                    };
                    break;
                }

                let arm_start = self.token().span;
                let mut labels = Vec::new();
                loop {
                    let low = self.parse_expression()?;
                    let label = if self.eat("..") {
                        let high = self.parse_expression()?;
                        SetElement::Range {
                            span: SourceSpan::join(low.span, high.span),
                            low,
                            high,
                        }
                    } else {
                        SetElement::Value(low)
                    };
                    labels.push(label);
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(":")?;
                let body = self.parse_statement_or_empty()?;
                let span = SourceSpan::join(arm_start, body.span);
                arms.push(CaseArm { labels, body, span });
                self.eat(";");
            }

            let end = self.expect("end")?.span;
            return Ok(self.statement(
                StatementKind::Case {
                    selector,
                    arms,
                    otherwise,
                },
                start,
                end,
            ));
        }

        let target = self.parse_factor()?;
        if self.eat(":=") {
            if !matches!(
                &target.kind,
                ExpressionKind::Identifier(_) | ExpressionKind::FighterField { .. }
            ) {
                return Err(RotError::new(
                    self.source,
                    target.span,
                    "invalid assignment target",
                ));
            }
            let value = self.parse_expression()?;
            return Ok(self.statement(
                StatementKind::Assign { target, value: value.clone() },
                start,
                value.span,
            ));
        }

        if matches!(&target.kind, ExpressionKind::Call { .. }) {
            return Ok(self.statement(StatementKind::Call(target.clone()), start, target.span));
        }

        Err(RotError::new(
            self.source,
            start,
            "statement must be an assignment or procedure call",
        ))
    }

    fn parse_expression(&mut self) -> Result<Expression, RotError> {
        let mut left = self.parse_simple_expression()?;
        while let Some(operator) = self.comparison_operator() {
            self.bump();
            let right = self.parse_simple_expression()?;
            let span = SourceSpan::join(left.span, right.span);
            left = self.expression(
                ExpressionKind::Binary {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                span,
            );
        }
        Ok(left)
    }

    fn parse_simple_expression(&mut self) -> Result<Expression, RotError> {
        let mut left = self.parse_term()?;
        while let Some(operator) = self.simple_operator() {
            self.bump();
            let right = self.parse_term()?;
            let span = SourceSpan::join(left.span, right.span);
            left = self.expression(
                ExpressionKind::Binary {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                span,
            );
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, RotError> {
        let mut left = self.parse_factor()?;
        while let Some(operator) = self.term_operator() {
            self.bump();
            let right = self.parse_factor()?;
            let span = SourceSpan::join(left.span, right.span);
            left = self.expression(
                ExpressionKind::Binary {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                span,
                span,
            );
        }
        Ok(left)
    }

    fn comparison_operator(&self) -> Option<BinaryOperator> {
        match self.token().text.to_ascii_lowercase().as_str() {
            "=" => Some(BinaryOperator::Equal),
            "<>" => Some(BinaryOperator::NotEqual),
            "<" => Some(BinaryOperator::Less),
            ">" => Some(BinaryOperator::Greater),
            "<=" => Some(BinaryOperator::LessEqual),
            ">=" => Some(BinaryOperator::GreaterEqual),
            "in" => Some(BinaryOperator::In),
            _ => None,
        }
    }

    fn simple_operator(&self) -> Option<BinaryOperator> {
        match self.token().text.to_ascii_lowercase().as_str() {
            "+" => Some(BinaryOperator::Add),
            "-" => Some(BinaryOperator::Subtract),
            "or" => Some(BinaryOperator::Or),
            "xor" => Some(BinaryOperator::Xor),
            _ => None,
        }
    }

    fn term_operator(&self) -> Option<BinaryOperator> {
        match self.token().text.to_ascii_lowercase().as_str() {
            "*" => Some(BinaryOperator::Multiply),
            "/" => Some(BinaryOperator::RealDivide),
            "div" => Some(BinaryOperator::IntegerDivide),
            "mod" => Some(BinaryOperator::Modulo),
            "and" => Some(BinaryOperator::And),
            "shl" => Some(BinaryOperator::ShiftLeft),
            "shr" => Some(BinaryOperator::ShiftRight),
            _ => None,
        }
    }

    fn parse_factor(&mut self) -> Result<Expression, RotError> {
        let start = self.token().span;
        let unary = match self.token().text.to_ascii_lowercase().as_str() {
            "not" => Some(UnaryOperator::Not),
            "+" => Some(UnaryOperator::Positive),
            "-" => Some(UnaryOperator::Negative),
            _ => None,
        };
        if let Some(operator) = unary {
            self.bump();
            let value = self.parse_factor()?;
            return Ok(self.expression(
                ExpressionKind::Unary {
                    operator,
                    value: Box::new(value.clone()),
                },
                start,
                value.span,
            ));
        }

        if self.eat("(") {
            let value = self.parse_expression()?;
            let end = self.expect(")")?.span;
            return Ok(self.expression(
                ExpressionKind::Group(Box::new(value)),
                start,
                end,
            ));
        }

        if self.eat("[") {
            let mut items = Vec::new();
            if !self.at("]") {
                loop {
                    let low = self.parse_expression()?;
                    if self.eat("..") {
                        let high = self.parse_expression()?;
                        items.push(SetElement::Range {
                            span: SourceSpan::join(low.span, high.span),
                            low,
                            high,
                        });
                    } else {
                        items.push(SetElement::Value(low));
                    }
                    if !self.eat(",") {
                        break;
                    }
                }
            }
            let end = self.expect("]")?.span;
            return Ok(self.expression(ExpressionKind::Set(items), start, end));
        }

        let token = self.token().clone();
        match token.kind {
            TokenKind::Integer | TokenKind::Real | TokenKind::String => {
                self.bump();
                let literal = token.literal.expect("literal token must carry a value");
                Ok(self.expression(
                    ExpressionKind::Literal(literal),
                    token.span,
                    token.span,
                ))
            }
            TokenKind::Identifier => {
                let name = self.bump();
                if name.text.eq_ignore_ascii_case("true")
                    || name.text.eq_ignore_ascii_case("false")
                {
                    let value = name.text.eq_ignore_ascii_case("true");
                    return Ok(self.expression(
                        ExpressionKind::Literal(Literal::Boolean(value)),
                        name.span,
                        name.span,
                    ));
                }

                if (name.text.eq_ignore_ascii_case("P")
                    || name.text.eq_ignore_ascii_case("O"))
                    && self.eat(".")
                {
                    let field = self.expect_identifier()?;
                    let base = name.text.chars().next().unwrap().to_ascii_uppercase();
                    return Ok(self.expression(
                        ExpressionKind::FighterField {
                            base,
                            name: field.text,
                        },
                        name.span,
                        field.span,
                    ));
                }

                if self.eat("(") {
                    let mut arguments = Vec::new();
                    if !self.at(")") {
                        loop {
                            arguments.push(self.parse_expression()?);
                            if !self.eat(",") {
                                break;
                            }
                        }
                    }
                    let end = self.expect(")")?.span;
                    return Ok(self.expression(
                        ExpressionKind::Call {
                            name: name.text,
                            arguments,
                        },
                        name.span,
                        end,
                    ));
                }

                Ok(self.expression(
                    ExpressionKind::Identifier(name.text),
                    name.span,
                    name.span,
                ))
            }
            _ => Err(self.error(format!(
                "unexpected expression token {:?}",
                self.token().text
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCES: &[(&str, &str)] = &[
        ("Comum.rot", include_str!("../research/pascal-rot/Comum.rot")),
        ("C01.rot", include_str!("../research/pascal-rot/C01.rot")),
        ("C02.rot", include_str!("../research/pascal-rot/C02.rot")),
        ("C03.rot", include_str!("../research/pascal-rot/C03.rot")),
        ("C04.rot", include_str!("../research/pascal-rot/C04.rot")),
        ("C05.rot", include_str!("../research/pascal-rot/C05.rot")),
        ("C06.rot", include_str!("../research/pascal-rot/C06.rot")),
        ("C07.rot", include_str!("../research/pascal-rot/C07.rot")),
        ("C08.rot", include_str!("../research/pascal-rot/C08.rot")),
        ("C09.rot", include_str!("../research/pascal-rot/C09.rot")),
        ("C10.rot", include_str!("../research/pascal-rot/C10.rot")),
        ("C11.rot", include_str!("../research/pascal-rot/C11.rot")),
        ("C12.rot", include_str!("../research/pascal-rot/C12.rot")),
        ("C13.rot", include_str!("../research/pascal-rot/C13.rot")),
        ("C14.rot", include_str!("../research/pascal-rot/C14.rot")),
        ("C15.rot", include_str!("../research/pascal-rot/C15.rot")),
    ];

    #[test]
    fn all_pinned_rot_sections_parse_without_evaluation() {
        for (filename, text) in SOURCES {
            let document = RotDocument::from_str(filename, text)
                .unwrap_or_else(|error| panic!("{filename}: {error}"));
            assert!(document.section("Acao").is_some(), "{filename} lacks [Acao]");
            for section in document.sections() {
                section
                    .parse()
                    .unwrap_or_else(|error| panic!("{filename} [{}]: {error}", section.name));
            }
            let action = document
                .parse_optional("Acao")
                .unwrap()
                .expect("action sections are required for the original 15 fighters");
            assert_eq!(action.section, "Acao");
        }
    }

    #[test]
    fn common_action_end_is_present_and_character_action_end_is_absent() {
        let common = RotDocument::from_str("Comum.rot", include_str!("../research/pascal-rot/Comum.rot")).unwrap();
        assert!(common.parse_optional("AcaoFim").unwrap().is_some());

        let c13 = RotDocument::from_str("C13.rot", include_str!("../research/pascal-rot/C13.rot")).unwrap();
        assert!(c13.parse_optional("AcaoFim").unwrap().is_none());
    }

    #[test]
    fn parses_nested_control_flow_sets_calls_and_pascal_literals() {
        let source = r#"
var I: Integer;
var X, Y: Double;
if (Codigo in [cmEsp1Fraco, cmEsp1Forte]) and (P.Acao in [acParado, acAndando]) then
begin
    for I := 1 to 5 do
        CriaEfeito(2, 'A', Jog, X + $140, Y - Random(10));
end
else
    while not P.Defendendo do
        repeat
            P.QuadroAcao := P.QuadroAcao + 1;
        until P.QuadroAcao >= 3;
"#;
        let section = RotSection {
            name: "Acao".to_owned(),
            source: "fixture.rot".to_owned(),
            first_line: 4,
            text: source.to_owned(),
        };
        let parsed = section.parse().unwrap();
        assert_eq!(parsed.declarations.len(), 2);
        assert_eq!(parsed.declarations[0].declared_type, PascalType::Integer);
        assert_eq!(parsed.declarations[1].declared_type, PascalType::Double);
        assert!(matches!(parsed.body.kind, StatementKind::Block(_)));
    }

    #[test]
    fn lexer_preserves_source_offsets_and_doubled_quotes() {
        let tokens = Lexer::new("fixture.rot", "P.X := 'it''s';", 12).run().unwrap();
        let string = tokens.iter().find(|token| token.kind == TokenKind::String).unwrap();
        assert_eq!(string.literal, Some(Literal::String("it's".to_owned())));
        assert_eq!(string.span.line, 12);
        assert_eq!(string.span.start_offset, 7);
        assert_eq!(string.span.end_offset, 14);
    }

    #[test]
    fn errors_report_source_line_and_column() {
        let section = RotSection {
            name: "Acao".to_owned(),
            source: "broken.rot".to_owned(),
            first_line: 30,
            text: "if P.Acao = then P.Acao := acParado;".to_owned(),
        };
        let error = section.parse().unwrap_err();
        assert_eq!(error.source, "broken.rot");
        assert_eq!(error.span.line, 30);
        assert!(error.span.column > 1);
    }
}

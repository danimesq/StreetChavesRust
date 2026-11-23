//! Source-preserving lexer, parser and expression evaluator for the pinned Pascal ROT dialect.
//!
//! The evaluator currently covers literals, typed local frames, scalar operators,
//! sets and pure math/conversion functions. It deliberately does not execute
//! statements, bind fighter/global state, invoke game helpers, or consume RNG.

use std::collections::BTreeMap;
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
            let value = i64::from_str_radix(&digits, 16).map_err(|_| {
                self.error(start, "hex literal is outside the supported integer range")
            })?;
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
        if self.current() == Some('.') && self.next().is_some_and(|ch| ch.is_ascii_digit()) {
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
    Lexer::new(source_name, text, 1).run().map(|tokens| {
        tokens
            .into_iter()
            .map(|token| (token.text, token.span))
            .collect()
    })
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
    FighterField {
        base: char,
        name: String,
    },
    Call {
        name: String,
        arguments: Vec<Expression>,
    },
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

#[derive(Debug, Clone, PartialEq)]
pub enum RotValue {
    Integer { value: i64, wide: bool },
    Real(f64),
    Boolean(bool),
    String(String),
    Set(Vec<RotSetItem>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RotSetItem {
    Value(i64),
    Range { low: i64, high: i64 },
}

#[derive(Debug, Clone)]
struct LocalSlot {
    declared_type: PascalType,
    value: RotValue,
    declaration_span: SourceSpan,
}

/// One call's local storage and the symbols explicitly supplied by its host.
///
/// Jog, Codigo and Desloc are initialized using the same byte narrowing and
/// player offset rules as TRoteiro.ExecutarCom. Each call receives fresh locals.
#[derive(Debug, Clone)]
pub struct RotExecutionFrame {
    source: String,
    section: String,
    locals: BTreeMap<String, LocalSlot>,
    symbols: BTreeMap<String, RotValue>,
}

impl RotExecutionFrame {
    pub fn new(program: &RotProgram, jog: u8, codigo: i32) -> Result<Self, RotError> {
        if !(1..=2).contains(&jog) {
            return Err(RotError::new(
                &program.source,
                program.body.span,
                "Jog must identify player 1 or 2",
            ));
        }

        let mut frame = Self {
            source: program.source.clone(),
            section: program.section.clone(),
            locals: BTreeMap::new(),
            symbols: BTreeMap::new(),
        };

        let predefined_span = program.body.span;
        frame.insert_local(
            "Jog",
            PascalType::Byte,
            RotValue::Integer {
                value: i64::from(jog),
                wide: false,
            },
            predefined_span,
        );
        frame.insert_local(
            "Codigo",
            PascalType::Byte,
            RotValue::Integer {
                value: i64::from(codigo as u8),
                wide: false,
            },
            predefined_span,
        );
        frame.insert_local(
            "Desloc",
            PascalType::Integer,
            RotValue::Integer {
                value: if jog == 2 { 50 } else { 0 },
                wide: false,
            },
            predefined_span,
        );

        for declaration in &program.declarations {
            let default = default_value(&declaration.declared_type).ok_or_else(|| {
                RotError::new(
                    &program.source,
                    declaration.span,
                    format!(
                        "unsupported local Pascal type {:?}",
                        declaration.declared_type
                    ),
                )
            })?;

            for name in &declaration.names {
                let key = name.to_ascii_lowercase();
                if frame.locals.contains_key(&key) {
                    return Err(RotError::new(
                        &program.source,
                        declaration.span,
                        format!("duplicate local variable {name}"),
                    ));
                }
                frame.insert_local(
                    name,
                    declaration.declared_type.clone(),
                    default.clone(),
                    declaration.span,
                );
            }
        }

        Ok(frame)
    }

    fn insert_local(
        &mut self,
        name: &str,
        declared_type: PascalType,
        value: RotValue,
        declaration_span: SourceSpan,
    ) {
        self.locals.insert(
            name.to_ascii_lowercase(),
            LocalSlot {
                declared_type,
                value,
                declaration_span,
            },
        );
    }

    /// Supply a constant or host-owned global value for identifier resolution.
    /// Locals take precedence, matching the Pascal interpreter's lookup order.
    pub fn set_symbol(&mut self, name: impl Into<String>, value: RotValue) {
        self.symbols.insert(name.into().to_ascii_lowercase(), value);
    }

    pub fn local(&self, name: &str) -> Option<&RotValue> {
        self.locals
            .get(&name.to_ascii_lowercase())
            .map(|slot| &slot.value)
    }

    /// Assign to a local using the pinned interpreter's declared-type conversion.
    pub fn set_local(&mut self, name: &str, value: RotValue) -> Result<(), RotError> {
        let key = name.to_ascii_lowercase();
        let slot = self.locals.get(&key).ok_or_else(|| {
            RotError::new(
                &self.source,
                SourceSpan::default(),
                format!("unknown local variable {name}"),
            )
        })?;
        let converted = convert_assignment(
            &self.source,
            slot.declaration_span,
            &slot.declared_type,
            value,
        )?;
        self.locals
            .get_mut(&key)
            .expect("local slot was checked")
            .value = converted;
        Ok(())
    }

    pub fn evaluate(&self, expression: &Expression) -> Result<RotValue, RotError> {
        evaluate_expression(self, expression)
    }

    /// Execute the scalar/local subset of a parsed ROT program.
    ///
    /// A host-side step limit prevents a malformed script from running forever.
    /// It is a safety boundary, not a Pascal language rule.
    pub fn execute(&mut self, program: &RotProgram) -> Result<(), RotError> {
        self.execute_with_step_limit(program, 1_000_000)
    }

    pub fn execute_with_step_limit(
        &mut self,
        program: &RotProgram,
        step_limit: usize,
    ) -> Result<(), RotError> {
        if program.source != self.source || program.section != self.section {
            return Err(self.error(
                program.body.span,
                "execution frame belongs to a different ROT source section",
            ));
        }
        let mut remaining = step_limit;
        self.execute_statement(&program.body, &mut remaining)
    }

    /// Execute a parsed ROT program against a live game host.
    ///
    /// Locals remain isolated per invocation; fighter fields, globals, RNG and
    /// engine helpers are read or mutated through the supplied host adapter.
    pub fn execute_with_host<H: RotExecutionHost>(
        &mut self,
        program: &RotProgram,
        host: &mut H,
    ) -> Result<(), RotError> {
        if program.source != self.source || program.section != self.section {
            return Err(self.error(
                program.body.span,
                "execution frame belongs to a different ROT source section",
            ));
        }
        let mut remaining = 1_000_000usize;
        execute_statement_with_host(self, &program.body, &mut remaining, host)
    }

    fn consume_step(&self, span: SourceSpan, remaining: &mut usize) -> Result<(), RotError> {
        if *remaining == 0 {
            return Err(self.error(span, "ROT execution step limit exceeded"));
        }
        *remaining -= 1;
        Ok(())
    }

    fn assign_local_at_span(
        &mut self,
        name: &str,
        value: RotValue,
        assignment_span: SourceSpan,
    ) -> Result<(), RotError> {
        let key = name.to_ascii_lowercase();
        let declared_type = self
            .locals
            .get(&key)
            .map(|slot| slot.declared_type.clone())
            .ok_or_else(|| self.error(assignment_span, format!("unknown local variable {name}")))?;
        let converted = convert_assignment(&self.source, assignment_span, &declared_type, value)?;
        self.locals
            .get_mut(&key)
            .expect("local slot was checked")
            .value = converted;
        Ok(())
    }

    fn execute_statement(
        &mut self,
        statement: &Statement,
        remaining: &mut usize,
    ) -> Result<(), RotError> {
        self.consume_step(statement.span, remaining)?;
        match &statement.kind {
            StatementKind::Block(statements) => {
                for child in statements {
                    self.execute_statement(child, remaining)?;
                }
                Ok(())
            }
            StatementKind::Assign { target, value } => {
                let ExpressionKind::Identifier(name) = &target.kind else {
                    return Err(self.error(
                        target.span,
                        "assignment target needs a bound local variable",
                    ));
                };
                let value = self.evaluate(value)?;
                self.assign_local_at_span(name, value, target.span)
            }
            StatementKind::Call(call) => {
                self.evaluate(call)?;
                Ok(())
            }
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_value = self.evaluate(condition)?;
                if require_boolean(self, condition.span, condition_value)? {
                    self.execute_statement(then_branch, remaining)
                } else if let Some(else_branch) = else_branch {
                    self.execute_statement(else_branch, remaining)
                } else {
                    Ok(())
                }
            }
            StatementKind::For {
                variable,
                direction,
                first,
                last,
                body,
            } => {
                // The bounds are evaluated once, in source order, before iteration.
                let first_value = self.evaluate(first)?;
                let (mut current, _) = require_integer(self, first.span, first_value)?;
                let last_value = self.evaluate(last)?;
                let (last, _) = require_integer(self, last.span, last_value)?;

                loop {
                    let in_range = match direction {
                        ForDirection::To => current <= last,
                        ForDirection::DownTo => current >= last,
                    };
                    if !in_range {
                        break;
                    }
                    self.consume_step(statement.span, remaining)?;
                    self.assign_local_at_span(variable, integer(current, false), statement.span)?;
                    self.execute_statement(body, remaining)?;
                    current = match direction {
                        ForDirection::To => current.wrapping_add(1),
                        ForDirection::DownTo => current.wrapping_sub(1),
                    };
                }
                Ok(())
            }
            StatementKind::While { condition, body } => loop {
                self.consume_step(statement.span, remaining)?;
                let condition_value = self.evaluate(condition)?;
                if !require_boolean(self, condition.span, condition_value)? {
                    break Ok(());
                }
                self.execute_statement(body, remaining)?;
            },
            StatementKind::RepeatUntil { body, condition } => loop {
                self.consume_step(statement.span, remaining)?;
                for child in body {
                    self.execute_statement(child, remaining)?;
                }
                let condition_value = self.evaluate(condition)?;
                if require_boolean(self, condition.span, condition_value)? {
                    break Ok(());
                }
            },
            StatementKind::Case {
                selector,
                arms,
                otherwise,
            } => {
                let selector_value = self.evaluate(selector)?;
                let (selector, _) = require_integer(self, selector.span, selector_value)?;
                for arm in arms {
                    let mut matched = false;
                    // Evaluate every label in an arm before deciding to enter it.
                    for label in &arm.labels {
                        let label_matches = match label {
                            SetElement::Value(value) => {
                                let value_result = self.evaluate(value)?;
                                let (value, _) = require_integer(self, value.span, value_result)?;
                                selector == value
                            }
                            SetElement::Range { low, high, span } => {
                                let low_result = self.evaluate(low)?;
                                let (low, _) = require_integer(self, *span, low_result)?;
                                let high_result = self.evaluate(high)?;
                                let (high, _) = require_integer(self, *span, high_result)?;
                                selector >= low && selector <= high
                            }
                        };
                        matched |= label_matches;
                    }
                    if matched {
                        return self.execute_statement(&arm.body, remaining);
                    }
                }
                if let Some(default) = otherwise {
                    for child in default {
                        self.execute_statement(child, remaining)?;
                    }
                }
                Ok(())
            }
        }
    }

    fn error(&self, span: SourceSpan, message: impl Into<String>) -> RotError {
        RotError::new(
            &format!("{} [{}]", self.source, self.section),
            span,
            message,
        )
    }

    fn lookup(&self, name: &str, span: SourceSpan) -> Result<RotValue, RotError> {
        let key = name.to_ascii_lowercase();
        if let Some(slot) = self.locals.get(&key) {
            return Ok(slot.value.clone());
        }
        if let Some(value) = self.symbols.get(&key) {
            return Ok(value.clone());
        }
        Err(self.error(span, format!("unknown ROT identifier {name}")))
    }
}


/// Live bindings exposed to a ROT action invocation.
///
/// The host owns authoritative fighter/global state and the shared fight RNG.
/// Unsupported fields and helpers must return errors rather than being treated
/// as inert values.
pub trait RotExecutionHost {
    fn read_fighter_field(&self, base: char, name: &str) -> Result<RotValue, String>;
    fn write_fighter_field(&mut self, base: char, name: &str, value: RotValue)
        -> Result<(), String>;

    fn read_identifier(&self, _name: &str) -> Option<RotValue> {
        None
    }

    fn write_identifier(&mut self, name: &str, _value: RotValue) -> Result<(), String> {
        Err(format!("global identifier {name} has no writable host binding"))
    }

    fn call_function(&mut self, name: &str, _arguments: &[RotValue])
        -> Result<RotValue, String>
    {
        Err(format!("ROT helper {name} has no host binding"))
    }
}

fn execute_statement_with_host<H: RotExecutionHost>(
    frame: &mut RotExecutionFrame,
    statement: &Statement,
    remaining: &mut usize,
    host: &mut H,
) -> Result<(), RotError> {
    frame.consume_step(statement.span, remaining)?;
    match &statement.kind {
        StatementKind::Block(statements) => {
            for child in statements {
                execute_statement_with_host(frame, child, remaining, host)?;
            }
            Ok(())
        }
        StatementKind::Assign { target, value } => {
            let value = evaluate_expression_with_host(frame, value, host)?;
            match &target.kind {
                ExpressionKind::Identifier(name) => {
                    if frame.locals.contains_key(&name.to_ascii_lowercase()) {
                        frame.assign_local_at_span(name, value, target.span)
                    } else {
                        host.write_identifier(name, value)
                            .map_err(|error| frame.error(target.span, error))
                    }
                }
                ExpressionKind::FighterField { base, name } => host
                    .write_fighter_field(*base, name, value)
                    .map_err(|error| frame.error(target.span, error)),
                _ => Err(frame.error(
                    target.span,
                    "assignment target must be a local, global or bound fighter field",
                )),
            }
        }
        StatementKind::Call(call) => {
            evaluate_expression_with_host(frame, call, host)?;
            Ok(())
        }
        StatementKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition_value = evaluate_expression_with_host(frame, condition, host)?;
            if require_boolean(frame, condition.span, condition_value)? {
                execute_statement_with_host(frame, then_branch, remaining, host)
            } else if let Some(else_branch) = else_branch {
                execute_statement_with_host(frame, else_branch, remaining, host)
            } else {
                Ok(())
            }
        }
        StatementKind::For {
            variable,
            direction,
            first,
            last,
            body,
        } => {
            // Pascal evaluates both bounds once, before the first iteration.
            let first_value = evaluate_expression_with_host(frame, first, host)?;
            let (mut current, _) = require_integer(frame, first.span, first_value)?;
            let last_value = evaluate_expression_with_host(frame, last, host)?;
            let (last, _) = require_integer(frame, last.span, last_value)?;

            loop {
                let in_range = match direction {
                    ForDirection::To => current <= last,
                    ForDirection::DownTo => current >= last,
                };
                if !in_range {
                    break;
                }
                frame.consume_step(statement.span, remaining)?;
                frame.assign_local_at_span(variable, integer(current, false), statement.span)?;
                execute_statement_with_host(frame, body, remaining, host)?;
                current = match direction {
                    ForDirection::To => current.wrapping_add(1),
                    ForDirection::DownTo => current.wrapping_sub(1),
                };
            }
            Ok(())
        }
        StatementKind::While { condition, body } => loop {
            frame.consume_step(statement.span, remaining)?;
            let condition_value = evaluate_expression_with_host(frame, condition, host)?;
            if !require_boolean(frame, condition.span, condition_value)? {
                break Ok(());
            }
            execute_statement_with_host(frame, body, remaining, host)?;
        },
        StatementKind::RepeatUntil { body, condition } => loop {
            frame.consume_step(statement.span, remaining)?;
            for child in body {
                execute_statement_with_host(frame, child, remaining, host)?;
            }
            let condition_value = evaluate_expression_with_host(frame, condition, host)?;
            if require_boolean(frame, condition.span, condition_value)? {
                break Ok(());
            }
        },
        StatementKind::Case {
            selector,
            arms,
            otherwise,
        } => {
            let selector_value = evaluate_expression_with_host(frame, selector, host)?;
            let (selector, _) = require_integer(frame, selector.span, selector_value)?;
            for arm in arms {
                let mut matched = false;
                for label in &arm.labels {
                    let label_matches = match label {
                        SetElement::Value(value) => {
                            let value_result = evaluate_expression_with_host(frame, value, host)?;
                            let (value, _) = require_integer(frame, value.span, value_result)?;
                            selector == value
                        }
                        SetElement::Range { low, high, span } => {
                            let low_result = evaluate_expression_with_host(frame, low, host)?;
                            let (low, _) = require_integer(frame, *span, low_result)?;
                            let high_result = evaluate_expression_with_host(frame, high, host)?;
                            let (high, _) = require_integer(frame, *span, high_result)?;
                            selector >= low && selector <= high
                        }
                    };
                    matched |= label_matches;
                }
                if matched {
                    return execute_statement_with_host(frame, &arm.body, remaining, host);
                }
            }
            if let Some(default) = otherwise {
                for child in default {
                    execute_statement_with_host(frame, child, remaining, host)?;
                }
            }
            Ok(())
        }
    }
}

fn evaluate_expression_with_host<H: RotExecutionHost>(
    frame: &RotExecutionFrame,
    expression: &Expression,
    host: &mut H,
) -> Result<RotValue, RotError> {
    match &expression.kind {
        ExpressionKind::Literal(literal) => Ok(match literal {
            Literal::Integer(value) => integer(
                *value,
                *value < i64::from(i32::MIN) || *value > i64::from(i32::MAX),
            ),
            Literal::Real(value) => RotValue::Real(*value),
            Literal::String(value) => RotValue::String(value.clone()),
            Literal::Boolean(value) => RotValue::Boolean(*value),
        }),
        ExpressionKind::Identifier(name) => {
            if let Some(value) = frame.local(name) {
                return Ok(value.clone());
            }
            if let Some(value) = host.read_identifier(name) {
                return Ok(value);
            }
            frame.lookup(name, expression.span)
        }
        ExpressionKind::FighterField { base, name } => host
            .read_fighter_field(*base, name)
            .map_err(|error| frame.error(expression.span, error)),
        ExpressionKind::Call { name, arguments } => {
            let mut values = Vec::with_capacity(arguments.len());
            for argument in arguments {
                values.push(evaluate_expression_with_host(frame, argument, host)?);
            }
            if matches!(
                name.to_ascii_lowercase().as_str(),
                "round" | "trunc" | "abs" | "cos" | "sin" | "shortint"
            ) {
                let literal_arguments = values
                    .into_iter()
                    .zip(arguments.iter())
                    .map(|(value, original)| {
                        let literal = match value {
                            RotValue::Integer { value, .. } => Literal::Integer(value),
                            RotValue::Real(value) => Literal::Real(value),
                            RotValue::Boolean(value) => Literal::Boolean(value),
                            RotValue::String(value) => Literal::String(value),
                            RotValue::Set(_) => {
                                return Err(frame.error(
                                    original.span,
                                    "pure ROT helper cannot accept a set value",
                                ))
                            }
                        };
                        Ok(Expression {
                            kind: ExpressionKind::Literal(literal),
                            span: original.span,
                        })
                    })
                    .collect::<Result<Vec<_>, RotError>>()?;
                evaluate_pure_function(frame, expression.span, name, &literal_arguments)
            } else {
                host.call_function(name, &values)
                    .map_err(|error| frame.error(expression.span, error))
            }
        }
        ExpressionKind::Unary { operator, value } => {
            let value = evaluate_expression_with_host(frame, value, host)?;
            match (operator, value) {
                (UnaryOperator::Not, RotValue::Boolean(value)) => Ok(RotValue::Boolean(!value)),
                (UnaryOperator::Not, RotValue::Integer { value, wide }) => {
                    Ok(integer(!value, wide))
                }
                (UnaryOperator::Positive, value @ RotValue::Integer { .. })
                | (UnaryOperator::Positive, value @ RotValue::Real(_)) => Ok(value),
                (UnaryOperator::Negative, RotValue::Integer { value, wide }) => {
                    Ok(integer(value.wrapping_neg(), wide))
                }
                (UnaryOperator::Negative, RotValue::Real(value)) => Ok(RotValue::Real(-value)),
                (UnaryOperator::Not, _) => {
                    Err(frame.error(expression.span, "not requires Boolean or integer"))
                }
                (_, _) => Err(frame.error(expression.span, "unary sign requires a number")),
            }
        }
        ExpressionKind::Binary {
            operator,
            left,
            right,
        } => evaluate_binary_with_host(frame, expression.span, *operator, left, right, host),
        ExpressionKind::Group(value) => evaluate_expression_with_host(frame, value, host),
        ExpressionKind::Set(elements) => {
            let mut values = Vec::with_capacity(elements.len());
            for element in elements {
                match element {
                    SetElement::Value(value) => {
                        let (value, _) = require_integer(
                            frame,
                            value.span,
                            evaluate_expression_with_host(frame, value, host)?,
                        )?;
                        values.push(RotSetItem::Value(value));
                    }
                    SetElement::Range { low, high, span } => {
                        let (low, _) = require_integer(
                            frame,
                            low.span,
                            evaluate_expression_with_host(frame, low, host)?,
                        )?;
                        let (high, _) = require_integer(
                            frame,
                            high.span,
                            evaluate_expression_with_host(frame, high, host)?,
                        )?;
                        values.push(RotSetItem::Range { low, high });
                        let _ = span;
                    }
                }
            }
            Ok(RotValue::Set(values))
        }
    }
}

fn evaluate_binary_with_host<H: RotExecutionHost>(
    frame: &RotExecutionFrame,
    span: SourceSpan,
    operator: BinaryOperator,
    left_expression: &Expression,
    right_expression: &Expression,
    host: &mut H,
) -> Result<RotValue, RotError> {
    let left = evaluate_expression_with_host(frame, left_expression, host)?;

    // Preserve Pascal Boolean short-circuiting so skipped helper calls consume
    // neither RNG values nor engine state.
    if operator == BinaryOperator::And {
        if let RotValue::Boolean(false) = left {
            return Ok(RotValue::Boolean(false));
        }
        if let RotValue::Boolean(true) = left {
            return Ok(RotValue::Boolean(require_boolean(
                frame,
                span,
                evaluate_expression_with_host(frame, right_expression, host)?,
            )?));
        }
    }
    if operator == BinaryOperator::Or {
        if let RotValue::Boolean(true) = left {
            return Ok(RotValue::Boolean(true));
        }
        if let RotValue::Boolean(false) = left {
            return Ok(RotValue::Boolean(require_boolean(
                frame,
                span,
                evaluate_expression_with_host(frame, right_expression, host)?,
            )?));
        }
    }

    if operator == BinaryOperator::In {
        let (needle, _) = require_integer(frame, left_expression.span, left)?;
        let set = evaluate_expression_with_host(frame, right_expression, host)?;
        let RotValue::Set(elements) = set else {
            return Err(frame.error(right_expression.span, "right operand of in must be a set"));
        };
        let found = elements.iter().any(|element| match element {
            RotSetItem::Value(value) => needle == *value,
            RotSetItem::Range { low, high } => needle >= *low && needle <= *high,
        });
        return Ok(RotValue::Boolean(found));
    }

    let right = evaluate_expression_with_host(frame, right_expression, host)?;
    match operator {
        BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::Less
        | BinaryOperator::Greater
        | BinaryOperator::LessEqual
        | BinaryOperator::GreaterEqual => evaluate_comparison(frame, span, operator, left, right),
        BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor => {
            evaluate_logic(frame, span, operator, left, right)
        }
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::RealDivide
        | BinaryOperator::IntegerDivide
        | BinaryOperator::Modulo
        | BinaryOperator::ShiftLeft
        | BinaryOperator::ShiftRight => evaluate_arithmetic(frame, span, operator, left, right),
        BinaryOperator::In => unreachable!("in was handled before evaluating the right operand"),
    }
}

fn integer(value: i64, wide: bool) -> RotValue {
    RotValue::Integer { value, wide }
}

fn default_value(declared_type: &PascalType) -> Option<RotValue> {
    match declared_type {
        PascalType::Byte
        | PascalType::ShortInt
        | PascalType::SmallInt
        | PascalType::Word
        | PascalType::Integer => Some(integer(0, false)),
        PascalType::Int64 => Some(integer(0, true)),
        PascalType::Double => Some(RotValue::Real(0.0)),
        PascalType::Boolean => Some(RotValue::Boolean(false)),
        PascalType::Other(_) => None,
    }
}

fn convert_assignment(
    source: &str,
    span: SourceSpan,
    declared_type: &PascalType,
    value: RotValue,
) -> Result<RotValue, RotError> {
    let mismatch = || {
        RotError::new(
            source,
            span,
            format!("value does not match Pascal type {declared_type:?}"),
        )
    };

    match declared_type {
        PascalType::Byte => match value {
            RotValue::Integer { value, .. } => Some(integer(i64::from(value as u8), false)),
            _ => None,
        },
        PascalType::ShortInt => match value {
            RotValue::Integer { value, .. } => Some(integer(i64::from(value as i8), false)),
            _ => None,
        },
        PascalType::SmallInt => match value {
            RotValue::Integer { value, .. } => Some(integer(i64::from(value as i16), false)),
            _ => None,
        },
        PascalType::Word => match value {
            RotValue::Integer { value, .. } => Some(integer(i64::from(value as u16), false)),
            _ => None,
        },
        PascalType::Integer => match value {
            RotValue::Integer { value, .. } => Some(integer(i64::from(value as i32), false)),
            _ => None,
        },
        PascalType::Int64 => match value {
            RotValue::Integer { value, .. } => Some(integer(value, true)),
            _ => None,
        },
        PascalType::Double => match value {
            RotValue::Real(value) => Some(RotValue::Real(value)),
            RotValue::Integer { value, .. } => Some(RotValue::Real(value as f64)),
            _ => None,
        },
        PascalType::Boolean => match value {
            RotValue::Boolean(value) => Some(RotValue::Boolean(value)),
            _ => None,
        },
        PascalType::Other(_) => None,
    }
    .ok_or_else(mismatch)
}

fn require_integer(
    frame: &RotExecutionFrame,
    span: SourceSpan,
    value: RotValue,
) -> Result<(i64, bool), RotError> {
    match value {
        RotValue::Integer { value, wide } => Ok((value, wide)),
        _ => Err(frame.error(span, "expected an integer value")),
    }
}

fn require_boolean(
    frame: &RotExecutionFrame,
    span: SourceSpan,
    value: RotValue,
) -> Result<bool, RotError> {
    match value {
        RotValue::Boolean(value) => Ok(value),
        _ => Err(frame.error(span, "expected a Boolean condition")),
    }
}

fn require_real(
    frame: &RotExecutionFrame,
    span: SourceSpan,
    value: RotValue,
) -> Result<f64, RotError> {
    match value {
        RotValue::Real(value) => Ok(value),
        RotValue::Integer { value, .. } => Ok(value as f64),
        _ => Err(frame.error(span, "expected a numeric value")),
    }
}

fn evaluate_expression(
    frame: &RotExecutionFrame,
    expression: &Expression,
) -> Result<RotValue, RotError> {
    match &expression.kind {
        ExpressionKind::Literal(literal) => Ok(match literal {
            Literal::Integer(value) => integer(
                *value,
                *value < i64::from(i32::MIN) || *value > i64::from(i32::MAX),
            ),
            Literal::Real(value) => RotValue::Real(*value),
            Literal::String(value) => RotValue::String(value.clone()),
            Literal::Boolean(value) => RotValue::Boolean(*value),
        }),
        ExpressionKind::Identifier(name) => frame.lookup(name, expression.span),
        ExpressionKind::FighterField { base, name } => Err(frame.error(
            expression.span,
            format!("{base}.{name} needs the fighter-state adapter"),
        )),
        ExpressionKind::Call { name, arguments } => {
            evaluate_pure_function(frame, expression.span, name, arguments)
        }
        ExpressionKind::Group(value) => evaluate_expression(frame, value),
        ExpressionKind::Set(elements) => {
            let mut values = Vec::with_capacity(elements.len());
            for element in elements {
                match element {
                    SetElement::Value(value) => {
                        let (value, _) =
                            require_integer(frame, value.span, evaluate_expression(frame, value)?)?;
                        values.push(RotSetItem::Value(value));
                    }
                    SetElement::Range { low, high, span } => {
                        let (low, _) =
                            require_integer(frame, low.span, evaluate_expression(frame, low)?)?;
                        let (high, _) =
                            require_integer(frame, high.span, evaluate_expression(frame, high)?)?;
                        values.push(RotSetItem::Range { low, high });
                        let _ = span;
                    }
                }
            }
            Ok(RotValue::Set(values))
        }
        ExpressionKind::Unary { operator, value } => {
            let value = evaluate_expression(frame, value)?;
            match (operator, value) {
                (UnaryOperator::Not, RotValue::Boolean(value)) => Ok(RotValue::Boolean(!value)),
                (UnaryOperator::Not, RotValue::Integer { value, wide }) => {
                    Ok(integer(!value, wide))
                }
                (UnaryOperator::Positive, value @ RotValue::Integer { .. })
                | (UnaryOperator::Positive, value @ RotValue::Real(_)) => Ok(value),
                (UnaryOperator::Negative, RotValue::Integer { value, wide }) => {
                    Ok(integer(value.wrapping_neg(), wide))
                }
                (UnaryOperator::Negative, RotValue::Real(value)) => Ok(RotValue::Real(-value)),
                (UnaryOperator::Not, _) => {
                    Err(frame.error(expression.span, "not requires Boolean or integer"))
                }
                (_, _) => Err(frame.error(expression.span, "unary sign requires a number")),
            }
        }
        ExpressionKind::Binary {
            operator,
            left,
            right,
        } => evaluate_binary(frame, expression.span, *operator, left, right),
    }
}

fn evaluate_binary(
    frame: &RotExecutionFrame,
    span: SourceSpan,
    operator: BinaryOperator,
    left_expression: &Expression,
    right_expression: &Expression,
) -> Result<RotValue, RotError> {
    let left = evaluate_expression(frame, left_expression)?;

    // Boolean and/or short-circuit. Integer and/or remain bitwise and evaluate
    // both operands, matching the pinned Pascal interpreter.
    if operator == BinaryOperator::And {
        if let RotValue::Boolean(false) = left {
            return Ok(RotValue::Boolean(false));
        }
        if let RotValue::Boolean(true) = left {
            return Ok(RotValue::Boolean(require_boolean(
                frame,
                span,
                evaluate_expression(frame, right_expression)?,
            )?));
        }
    }
    if operator == BinaryOperator::Or {
        if let RotValue::Boolean(true) = left {
            return Ok(RotValue::Boolean(true));
        }
        if let RotValue::Boolean(false) = left {
            return Ok(RotValue::Boolean(require_boolean(
                frame,
                span,
                evaluate_expression(frame, right_expression)?,
            )?));
        }
    }

    if operator == BinaryOperator::In {
        let (needle, _) = require_integer(frame, left_expression.span, left)?;
        let set = evaluate_expression(frame, right_expression)?;
        let RotValue::Set(elements) = set else {
            return Err(frame.error(right_expression.span, "right operand of in must be a set"));
        };
        let found = elements.iter().any(|element| match element {
            RotSetItem::Value(value) => needle == *value,
            RotSetItem::Range { low, high } => needle >= *low && needle <= *high,
        });
        return Ok(RotValue::Boolean(found));
    }

    let right = evaluate_expression(frame, right_expression)?;

    match operator {
        BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::Less
        | BinaryOperator::Greater
        | BinaryOperator::LessEqual
        | BinaryOperator::GreaterEqual => evaluate_comparison(frame, span, operator, left, right),
        BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor => {
            evaluate_logic(frame, span, operator, left, right)
        }
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::RealDivide
        | BinaryOperator::IntegerDivide
        | BinaryOperator::Modulo
        | BinaryOperator::ShiftLeft
        | BinaryOperator::ShiftRight => evaluate_arithmetic(frame, span, operator, left, right),
        BinaryOperator::In => unreachable!("in was handled before evaluating the right operand"),
    }
}

fn evaluate_logic(
    frame: &RotExecutionFrame,
    span: SourceSpan,
    operator: BinaryOperator,
    left: RotValue,
    right: RotValue,
) -> Result<RotValue, RotError> {
    match (left, right) {
        (RotValue::Boolean(left), RotValue::Boolean(right)) => {
            Ok(RotValue::Boolean(match operator {
                BinaryOperator::And => left && right,
                BinaryOperator::Or => left || right,
                BinaryOperator::Xor => left ^ right,
                _ => unreachable!(),
            }))
        }
        (
            RotValue::Integer {
                value: left,
                wide: left_wide,
            },
            RotValue::Integer {
                value: right,
                wide: right_wide,
            },
        ) => {
            let value = match operator {
                BinaryOperator::And => left & right,
                BinaryOperator::Or => left | right,
                BinaryOperator::Xor => left ^ right,
                _ => unreachable!(),
            };
            Ok(integer(value, left_wide || right_wide))
        }
        _ => Err(frame.error(
            span,
            "and/or/xor operands must both be Boolean or both be integers",
        )),
    }
}

fn evaluate_arithmetic(
    frame: &RotExecutionFrame,
    span: SourceSpan,
    operator: BinaryOperator,
    left: RotValue,
    right: RotValue,
) -> Result<RotValue, RotError> {
    if operator == BinaryOperator::RealDivide
        || matches!(left, RotValue::Real(_))
        || matches!(right, RotValue::Real(_))
    {
        if matches!(
            operator,
            BinaryOperator::IntegerDivide
                | BinaryOperator::Modulo
                | BinaryOperator::ShiftLeft
                | BinaryOperator::ShiftRight
        ) {
            return Err(frame.error(span, "div, mod, shl and shr require integer operands"));
        }
        let left = require_real(frame, span, left)?;
        let right = require_real(frame, span, right)?;
        if operator == BinaryOperator::RealDivide && right == 0.0 {
            return Err(frame.error(span, "division by zero"));
        }
        let value = match operator {
            BinaryOperator::Add => left + right,
            BinaryOperator::Subtract => left - right,
            BinaryOperator::Multiply => left * right,
            BinaryOperator::RealDivide => left / right,
            _ => return Err(frame.error(span, "invalid real arithmetic operator")),
        };
        return Ok(RotValue::Real(value));
    }

    let (left, left_wide) = require_integer(frame, span, left)?;
    let (right, right_wide) = require_integer(frame, span, right)?;
    let wide = left_wide || right_wide;

    let value = match operator {
        BinaryOperator::Add => left.wrapping_add(right),
        BinaryOperator::Subtract => left.wrapping_sub(right),
        BinaryOperator::Multiply => left.wrapping_mul(right),
        BinaryOperator::IntegerDivide => left
            .checked_div(right)
            .ok_or_else(|| frame.error(span, "integer division by zero or overflow"))?,
        BinaryOperator::Modulo => left
            .checked_rem(right)
            .ok_or_else(|| frame.error(span, "integer modulo by zero or overflow"))?,
        BinaryOperator::ShiftLeft => (left as u64).wrapping_shl(right as u32) as i64,
        BinaryOperator::ShiftRight => (left as u64).wrapping_shr(right as u32) as i64,
        _ => return Err(frame.error(span, "invalid integer arithmetic operator")),
    };
    Ok(integer(value, wide))
}

fn evaluate_comparison(
    frame: &RotExecutionFrame,
    span: SourceSpan,
    operator: BinaryOperator,
    left: RotValue,
    right: RotValue,
) -> Result<RotValue, RotError> {
    let ordering = match (left, right) {
        (RotValue::String(left), RotValue::String(right)) => match operator {
            BinaryOperator::Equal => Some(left == right),
            BinaryOperator::NotEqual => Some(left != right),
            _ => return Err(frame.error(span, "strings support only = and <> comparisons")),
        },
        (RotValue::Boolean(left), RotValue::Boolean(right)) => {
            Some(compare_ordering(operator, left.cmp(&right)))
        }
        (RotValue::Integer { value: left, .. }, RotValue::Integer { value: right, .. }) => {
            Some(compare_ordering(operator, left.cmp(&right)))
        }
        (
            left @ (RotValue::Integer { .. } | RotValue::Real(_)),
            right @ (RotValue::Integer { .. } | RotValue::Real(_)),
        ) => {
            let left = require_real(frame, span, left)?;
            let right = require_real(frame, span, right)?;
            Some(match operator {
                BinaryOperator::Equal => left == right,
                BinaryOperator::NotEqual => left != right,
                BinaryOperator::Less => left < right,
                BinaryOperator::Greater => left > right,
                BinaryOperator::LessEqual => left <= right,
                BinaryOperator::GreaterEqual => left >= right,
                _ => unreachable!(),
            })
        }
        (RotValue::String(_), _) | (_, RotValue::String(_)) => {
            return Err(frame.error(span, "cannot compare a string with a non-string value"));
        }
        (RotValue::Boolean(_), _) | (_, RotValue::Boolean(_)) => {
            return Err(frame.error(span, "cannot compare a Boolean with a numeric value"));
        }
        _ => return Err(frame.error(span, "unsupported comparison operands")),
    };
    Ok(RotValue::Boolean(ordering.unwrap_or(false)))
}

fn compare_ordering(operator: BinaryOperator, ordering: std::cmp::Ordering) -> bool {
    match operator {
        BinaryOperator::Equal => ordering.is_eq(),
        BinaryOperator::NotEqual => !ordering.is_eq(),
        BinaryOperator::Less => ordering.is_lt(),
        BinaryOperator::Greater => ordering.is_gt(),
        BinaryOperator::LessEqual => !ordering.is_gt(),
        BinaryOperator::GreaterEqual => !ordering.is_lt(),
        _ => unreachable!(),
    }
}

fn evaluate_pure_function(
    frame: &RotExecutionFrame,
    span: SourceSpan,
    name: &str,
    arguments: &[Expression],
) -> Result<RotValue, RotError> {
    let name_lower = name.to_ascii_lowercase();
    if !matches!(
        name_lower.as_str(),
        "round" | "trunc" | "abs" | "cos" | "sin" | "shortint"
    ) {
        return Err(frame.error(
            span,
            format!("{name} requires the runtime host/helper bridge"),
        ));
    }
    if arguments.len() != 1 {
        return Err(frame.error(span, format!("{name} expects exactly one argument")));
    }

    let value = evaluate_expression(frame, &arguments[0])?;
    match name_lower.as_str() {
        "round" => {
            let number = require_real(frame, arguments[0].span, value)?;
            Ok(integer(round_ties_even(frame, span, number)?, true))
        }
        "trunc" => {
            let number = require_real(frame, arguments[0].span, value)?;
            Ok(integer(float_to_i64(frame, span, number.trunc())?, true))
        }
        "abs" => match value {
            RotValue::Integer { value, wide } => value
                .checked_abs()
                .map(|value| integer(value, wide))
                .ok_or_else(|| frame.error(span, "integer absolute value overflow")),
            RotValue::Real(value) => Ok(RotValue::Real(value.abs())),
            _ => Err(frame.error(span, "Abs expects a numeric value")),
        },
        "cos" => Ok(RotValue::Real(
            require_real(frame, arguments[0].span, value)?.cos(),
        )),
        "sin" => Ok(RotValue::Real(
            require_real(frame, arguments[0].span, value)?.sin(),
        )),
        "shortint" => {
            let (value, _) = require_integer(frame, arguments[0].span, value)?;
            Ok(integer(i64::from(value as i8), false))
        }
        _ => unreachable!(),
    }
}

fn float_to_i64(frame: &RotExecutionFrame, span: SourceSpan, value: f64) -> Result<i64, RotError> {
    const LIMIT: f64 = 9_223_372_036_854_775_808.0;
    if !value.is_finite() || value < -LIMIT || value >= LIMIT {
        return Err(frame.error(span, "real value is outside the Int64 range"));
    }
    Ok(value as i64)
}

/// Free Pascal Round follows the active FPU rounding mode (ties-to-even in the
/// pinned FPC 3.2.2 profile); Rust's f64::round uses ties-away-from-zero.
fn round_ties_even(
    frame: &RotExecutionFrame,
    span: SourceSpan,
    value: f64,
) -> Result<i64, RotError> {
    const LIMIT: f64 = 9_223_372_036_854_775_808.0;
    if !value.is_finite() || value < -LIMIT || value >= LIMIT {
        return Err(frame.error(span, "Round result is outside the Int64 range"));
    }

    let floor = value.floor();
    let fraction = value - floor;
    let rounded = if fraction < 0.5 {
        floor
    } else if fraction > 0.5 {
        floor + 1.0
    } else {
        let lower = floor as i64;
        if lower & 1 == 0 {
            floor
        } else {
            floor + 1.0
        }
    };
    float_to_i64(frame, span, rounded)
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
    fn try_new(source: &'a str, text: &str, first_line: usize) -> Result<Self, RotError> {
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
            return Err(self.error(format!("expected {text:?}, found {:?}", self.token().text)));
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

    fn statement(&mut self, kind: StatementKind, start: SourceSpan, end: SourceSpan) -> Statement {
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
            return Err(self.error(format!("unexpected trailing token {:?}", self.token().text)));
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
                return Err(self.error(format!("expected ';', found {:?}", self.token().text)));
            }
            if before == self.position {
                return Err(self.error("parser made no progress"));
            }
        }

        let end = statements
            .last()
            .map(|statement| statement.span)
            .unwrap_or(start);
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
                StatementKind::Assign {
                    target,
                    value: value.clone(),
                },
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
            return Ok(self.expression(ExpressionKind::Group(Box::new(value)), start, end));
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
                Ok(self.expression(ExpressionKind::Literal(literal), token.span, token.span))
            }
            TokenKind::Identifier => {
                let name = self.bump();
                if name.text.eq_ignore_ascii_case("true") || name.text.eq_ignore_ascii_case("false")
                {
                    let value = name.text.eq_ignore_ascii_case("true");
                    return Ok(self.expression(
                        ExpressionKind::Literal(Literal::Boolean(value)),
                        name.span,
                        name.span,
                    ));
                }

                if (name.text.eq_ignore_ascii_case("P") || name.text.eq_ignore_ascii_case("O"))
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

                Ok(self.expression(ExpressionKind::Identifier(name.text), name.span, name.span))
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
        (
            "Comum.rot",
            include_str!("../research/pascal-rot/Comum.rot"),
        ),
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
            assert!(
                document.section("Acao").is_some(),
                "{filename} lacks [Acao]"
            );
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
        let common = RotDocument::from_str(
            "Comum.rot",
            include_str!("../research/pascal-rot/Comum.rot"),
        )
        .unwrap();
        assert!(common.parse_optional("AcaoFim").unwrap().is_some());

        let c13 = RotDocument::from_str("C13.rot", include_str!("../research/pascal-rot/C13.rot"))
            .unwrap();
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
        let tokens = Lexer::new("fixture.rot", "P.X := 'it''s';", 12)
            .run()
            .unwrap();
        let string = tokens
            .iter()
            .find(|token| token.kind == TokenKind::String)
            .unwrap();
        assert_eq!(string.literal, Some(Literal::String("it's".to_owned())));
        assert_eq!(string.span.line, 12);
        assert_eq!(string.span.start_offset, 7);
        assert_eq!(string.span.end_offset, 14);
    }

    fn expression_program(expression: &str) -> RotProgram {
        let section = RotSection {
            name: "Acao".to_owned(),
            source: "eval.rot".to_owned(),
            first_line: 1,
            text: format!("Result := {expression};"),
        };
        section.parse().unwrap()
    }

    fn only_assignment_value(program: &RotProgram) -> &Expression {
        let StatementKind::Block(statements) = &program.body.kind else {
            panic!("test expression did not parse into a block");
        };
        let StatementKind::Assign { value, .. } = &statements[0].kind else {
            panic!("test expression did not parse into an assignment");
        };
        value
    }

    #[test]
    fn evaluates_pascal_integer_real_comparison_and_set_operators() {
        let program = expression_program("5 + 2 * 3");
        let frame = RotExecutionFrame::new(&program, 1, 300).unwrap();
        assert_eq!(
            frame.evaluate(only_assignment_value(&program)).unwrap(),
            integer(11, false)
        );

        for (expression, expected) in [
            ("7 / 2", RotValue::Real(3.5)),
            ("-7 div 2", integer(-3, false)),
            ("-7 mod 2", integer(-1, false)),
            ("1 shl 3", integer(8, false)),
            ("$80 shr 1", integer(64, false)),
            ("not 0", integer(-1, false)),
            ("1.5 + 2", RotValue::Real(3.5)),
            ("3 in [1, 2..4]", RotValue::Boolean(true)),
            ("8 in [1, 2..4]", RotValue::Boolean(false)),
            ("'rot' = 'rot'", RotValue::Boolean(true)),
            ("'rot' <> 'pascal'", RotValue::Boolean(true)),
            ("true xor false", RotValue::Boolean(true)),
        ] {
            let program = expression_program(expression);
            let frame = RotExecutionFrame::new(&program, 1, 0).unwrap();
            assert_eq!(
                frame.evaluate(only_assignment_value(&program)).unwrap(),
                expected,
                "expression: {expression}"
            );
        }
    }

    #[test]
    fn typed_frames_initialize_locals_and_apply_pascal_assignment_narrowing() {
        let section = RotSection {
            name: "Acao".to_owned(),
            source: "frame.rot".to_owned(),
            first_line: 1,
            text: "var B: Byte; var S: ShortInt; var I: Integer; var L: Int64; var D: Double; var Flag: Boolean; Result := 0;".to_owned(),
        };
        let program = section.parse().unwrap();
        let mut player_two = RotExecutionFrame::new(&program, 2, 258).unwrap();

        assert_eq!(player_two.local("jog"), Some(&integer(2, false)));
        assert_eq!(player_two.local("codigo"), Some(&integer(2, false)));
        assert_eq!(player_two.local("desloc"), Some(&integer(50, false)));
        assert_eq!(player_two.local("b"), Some(&integer(0, false)));
        assert_eq!(player_two.local("l"), Some(&integer(0, true)));
        assert_eq!(player_two.local("d"), Some(&RotValue::Real(0.0)));
        assert_eq!(player_two.local("flag"), Some(&RotValue::Boolean(false)));

        player_two.set_local("B", integer(258, true)).unwrap();
        player_two.set_local("S", integer(130, false)).unwrap();
        player_two
            .set_local("I", integer(i64::from(i32::MAX) + 1, true))
            .unwrap();
        player_two.set_local("L", integer(7, false)).unwrap();
        player_two.set_local("D", integer(7, true)).unwrap();
        player_two
            .set_local("Flag", RotValue::Boolean(true))
            .unwrap();

        assert_eq!(player_two.local("B"), Some(&integer(2, false)));
        assert_eq!(player_two.local("S"), Some(&integer(-126, false)));
        assert_eq!(
            player_two.local("I"),
            Some(&integer(i64::from(i32::MIN), false))
        );
        assert_eq!(player_two.local("L"), Some(&integer(7, true)));
        assert_eq!(player_two.local("D"), Some(&RotValue::Real(7.0)));
        assert_eq!(player_two.local("Flag"), Some(&RotValue::Boolean(true)));

        assert!(player_two.set_local("Flag", integer(1, false)).is_err());
    }

    #[test]
    fn boolean_and_or_short_circuit_without_invoking_host_functions() {
        for (expression, expected) in [
            ("false and (Random(1) > 0)", RotValue::Boolean(false)),
            ("true or (Random(1) > 0)", RotValue::Boolean(true)),
        ] {
            let program = expression_program(expression);
            let frame = RotExecutionFrame::new(&program, 1, 0).unwrap();
            assert_eq!(
                frame.evaluate(only_assignment_value(&program)).unwrap(),
                expected
            );
        }

        let program = expression_program("true and (Random(1) > 0)");
        let frame = RotExecutionFrame::new(&program, 1, 0).unwrap();
        let error = frame.evaluate(only_assignment_value(&program)).unwrap_err();
        assert!(error.message.contains("runtime host/helper bridge"));
    }

    #[test]
    fn pure_pascal_functions_preserve_integer_width_and_rounding_profile() {
        for (expression, expected) in [
            ("Round(2.5)", integer(2, true)),
            ("Round(3.5)", integer(4, true)),
            ("Round(-2.5)", integer(-2, true)),
            ("Trunc(-2.9)", integer(-2, true)),
            ("Abs(-4)", integer(4, false)),
            ("ShortInt(130)", integer(-126, false)),
            ("Cos(0)", RotValue::Real(1.0)),
            ("Sin(0)", RotValue::Real(0.0)),
        ] {
            let program = expression_program(expression);
            let frame = RotExecutionFrame::new(&program, 1, 0).unwrap();
            assert_eq!(
                frame.evaluate(only_assignment_value(&program)).unwrap(),
                expected,
                "expression: {expression}"
            );
        }
    }

    #[test]
    fn evaluator_reports_unbound_state_helpers_and_invalid_operations() {
        for expression in [
            "P.Acao",
            "UnknownConstant",
            "CriaEfeito(2, 'A', Jog, 1, 2)",
            "1 div 0",
            "'text' < 'other'",
        ] {
            let program = expression_program(expression);
            let frame = RotExecutionFrame::new(&program, 1, 0).unwrap();
            assert!(
                frame.evaluate(only_assignment_value(&program)).is_err(),
                "expression should fail explicitly: {expression}"
            );
        }

        let program = expression_program("Codigo");
        let frame = RotExecutionFrame::new(&program, 1, 300).unwrap();
        assert_eq!(
            frame.evaluate(only_assignment_value(&program)).unwrap(),
            integer(44, false)
        );
    }

    #[test]
    fn executes_local_assignments_conditionals_and_typed_conversions() {
        let section = RotSection {
            name: "Acao".to_owned(),
            source: "execute.rot".to_owned(),
            first_line: 1,
            text: "var I: Integer; var D: Double; var Flag: Boolean; begin I := 1 + 2 * 3; D := I / 2; if I = 7 then Flag := True else Flag := False; end;".to_owned(),
        };
        let program = section.parse().unwrap();
        let mut frame = RotExecutionFrame::new(&program, 1, 0).unwrap();
        frame.execute(&program).unwrap();
        assert_eq!(frame.local("I"), Some(&integer(7, false)));
        assert_eq!(frame.local("D"), Some(&RotValue::Real(3.5)));
        assert_eq!(frame.local("Flag"), Some(&RotValue::Boolean(true)));
    }

    #[test]
    fn executes_for_while_repeat_and_case_in_source_order() {
        let section = RotSection {
            name: "Acao".to_owned(),
            source: "control.rot".to_owned(),
            first_line: 1,
            text: "var I: Integer; var Sum: Integer; var Choice: Integer; begin Sum := 0; for I := 1 to 3 do Sum := Sum + I; I := 0; while I < 2 do I := I + 1; repeat I := I - 1; until I = 0; case Sum of 5: Choice := 1; 6: Choice := 2; else Choice := 3; end; end;".to_owned(),
        };
        let program = section.parse().unwrap();
        let mut frame = RotExecutionFrame::new(&program, 1, 0).unwrap();
        frame.execute(&program).unwrap();
        assert_eq!(frame.local("Sum"), Some(&integer(6, false)));
        assert_eq!(frame.local("I"), Some(&integer(0, false)));
        assert_eq!(frame.local("Choice"), Some(&integer(2, false)));
    }

    #[test]
    fn statement_execution_rejects_unbound_fighter_fields_and_helpers() {
        for text in ["P.Acao := 2;", "CriaEfeito(2, 'A', Jog, 1, 2);"] {
            let section = RotSection {
                name: "Acao".to_owned(),
                source: "unsupported.rot".to_owned(),
                first_line: 7,
                text: text.to_owned(),
            };
            let program = section.parse().unwrap();
            let mut frame = RotExecutionFrame::new(&program, 1, 0).unwrap();
            assert!(frame.execute(&program).is_err(), "program: {text}");
        }
    }

    #[test]
    fn statement_execution_enforces_a_host_step_limit() {
        let section = RotSection {
            name: "Acao".to_owned(),
            source: "loop.rot".to_owned(),
            first_line: 12,
            text: "var I: Integer; while True do I := I + 1;".to_owned(),
        };
        let program = section.parse().unwrap();
        let mut frame = RotExecutionFrame::new(&program, 1, 0).unwrap();
        let error = frame.execute_with_step_limit(&program, 8).unwrap_err();
        assert!(error.message.contains("step limit"));
        assert_eq!(frame.local("I"), Some(&integer(3, false)));
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

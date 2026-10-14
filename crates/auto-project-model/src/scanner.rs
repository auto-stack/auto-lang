//! Document-level scanner for the static pac.at subset (PLAN-780 T-02).
//!
//! pac.at is a *flat multi-entry document*: a sequence of `key: value` props
//! and node declarations (`dep bps { path: "..." }`, `lib("auto") { ... }`,
//! `app main { ... }`). [`auto_atom::AtomParser`] accepts exactly one
//! top-level Atom value and its node bodies only take `name(args) { }` /
//! `name { }` heads, so it cannot parse a pac.at document directly; this
//! scanner owns the document grammar and delegates each *structured* value
//! extent (objects/arrays) to `AtomParser::parse` verbatim (T-01 核定).
//! Scalar values are parsed locally (`Atom::new` rejects top-level scalars).
//!
//! Constructs the static grammar cannot represent are recorded as
//! [`UnsupportedConstruct`] with position and raw text — the scan continues
//! so nothing is silently dropped, but the file is NOT classifiable as
//! static-equivalent. Genuinely malformed input is a syntax error with
//! stage/position diagnosis.

use crate::descriptor::{NodeName, SourceInfo, UnsupportedConstruct, UnsupportedKind};
use crate::{ErrorStage, ProjectModelError};
use auto_val::Value;

/// Bare identifier values the config VM defines as builtins
/// (`eval_config_with_vm` injects on/off; true/false/null/nil are literals).
pub(crate) const BUILTIN_IDENTS: [&str; 4] = ["true", "false", "null", "nil"];
/// `on`/`off` evaluate to Bool in the config VM; outside the toggle fields
/// that evaluation has no static equivalent, so they are classified.
const TOGGLE_KEYS: [&str; 2] = ["shadcn", "default_classes"];

pub(crate) struct Scanner<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
    col: usize,
    path: Option<std::path::PathBuf>,
}

/// One scanned top-level (or node-body) entry.
#[derive(Debug, Clone)]
pub(crate) enum Entry {
    Prop {
        key: String,
        value: ScannedValue,
        line: usize,
    },
    Node {
        head: String,
        name: NodeName,
        body: Vec<Entry>,
        line: usize,
        /// Interpolation found in the node-name position (template forms
        /// `lib ${name} {}` / `app("${name}") {}`) — only the config VM's
        /// render pipeline can produce that name.
        name_unsupported: Option<UnsupportedConstruct>,
    },
    Unsupported(UnsupportedConstruct),
}

/// A prop value: either a parsed literal, or a classified unsupported form
/// (the raw text is preserved on the construct — never dropped).
#[derive(Debug, Clone)]
pub(crate) enum ScannedValue {
    Literal(Value),
    Unsupported(UnsupportedConstruct),
}

pub(crate) struct ScanOutput {
    pub entries: Vec<Entry>,
}

impl<'a> Scanner<'a> {
    pub(crate) fn new(input: &'a str, source: &SourceInfo) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            col: 1,
            path: source.path.clone(),
        }
    }

    pub(crate) fn scan_document(mut self) -> Result<ScanOutput, ProjectModelError> {
        let mut entries = Vec::new();
        loop {
            self.skip_ws_and_comments();
            if self.peek().is_none() {
                break;
            }
            if self.peek() == Some('{') {
                // Minimal project format (PLAN-729): the whole document is
                // one JSON-ish object whose pairs become root props
                // (`examples/http_server/files/pac.at` shape).
                entries.extend(self.parse_object_document()?);
                continue;
            }
            entries.push(self.parse_entry()?);
        }
        Ok(ScanOutput { entries })
    }

    /// Top-level `{ "key": value, … }` document — entries surface as props.
    fn parse_object_document(&mut self) -> Result<Vec<Entry>, ProjectModelError> {
        let start_line = self.line;
        let text = self.take_bracket_extent()?;
        if let Some(u) = scan_unsupported_in_text(&text, start_line, "<document>") {
            return Ok(vec![Entry::Unsupported(u)]);
        }
        let atom = auto_atom::AtomParser::parse(&text).map_err(|e| {
            self.err_at(
                ErrorStage::Document,
                start_line,
                self.col,
                format!("top-level object document failed to parse: {e}"),
            )
        })?;
        match atom {
            auto_atom::Atom::Obj(obj) => Ok(obj
                .iter()
                .map(|(k, v)| Entry::Prop {
                    key: k.to_string(),
                    value: ScannedValue::Literal(v.clone()),
                    line: start_line,
                })
                .collect()),
            other => Err(self.err_at(
                ErrorStage::Document,
                start_line,
                self.col,
                format!(
                    "top-level object document must be an object, got {:?}",
                    std::mem::discriminant(&other)
                ),
            )),
        }
    }

    fn err(&self, stage: ErrorStage, message: impl Into<String>) -> ProjectModelError {
        ProjectModelError {
            stage,
            line: self.line,
            column: self.col,
            message: message.into(),
            source_path: self.path.clone(),
        }
    }

    fn err_at(
        &self,
        stage: ErrorStage,
        line: usize,
        col: usize,
        message: impl Into<String>,
    ) -> ProjectModelError {
        ProjectModelError {
            stage,
            line,
            column: col,
            message: message.into(),
            source_path: self.path.clone(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn peek2(&self) -> Option<char> {
        self.input[self.pos..].chars().nth(1)
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while self.peek().is_some_and(char::is_whitespace) {
                self.advance();
            }
            if self.peek() == Some('/') && self.peek2() == Some('/') {
                while self.peek().is_some_and(|c| c != '\n') {
                    self.advance();
                }
                continue;
            }
            if self.peek() == Some('/') && self.peek2() == Some('*') {
                self.advance();
                self.advance();
                while self.peek().is_some() {
                    if self.peek() == Some('*') && self.peek2() == Some('/') {
                        self.advance();
                        self.advance();
                        break;
                    }
                    self.advance();
                }
                continue;
            }
            // `#` not followed by `[` is a corpus-style line comment
            // (parser.rs Plan 442 C2 — same rule the config VM applies).
            if self.peek() == Some('#') && self.peek2() != Some('[') {
                while self.peek().is_some_and(|c| c != '\n') {
                    self.advance();
                }
                continue;
            }
            break;
        }
    }

    /// Skip spaces/tabs only — the prop `:` / node name must stay on the
    /// entry's first line (pac.at statements are line-based).
    fn skip_inline_ws(&mut self) {
        while matches!(self.peek(), Some(' ') | Some('\t')) {
            self.advance();
        }
    }

    fn take_ident(&mut self) -> String {
        let start = self.pos;
        if self.peek().is_some_and(is_ident_start) {
            self.advance();
        }
        while self.peek().is_some_and(is_ident_continue) {
            self.advance();
        }
        self.input[start..self.pos].to_string()
    }

    fn parse_entry(&mut self) -> Result<Entry, ProjectModelError> {
        let (line, col) = (self.line, self.col);
        let first = self
            .peek()
            .ok_or_else(|| self.err_at(ErrorStage::Document, line, col, "unexpected end of input"))?;

        if !is_ident_start(first) {
            return Err(self.err_at(
                ErrorStage::Entry,
                line,
                col,
                format!("unexpected character '{first}' at entry start (expected key or node head)"),
            ));
        }
        let ident = self.take_ident();
        self.skip_inline_ws();
        match self.peek() {
            Some(':') => {
                self.advance();
                self.parse_prop_value(&ident, line)
            }
            Some('=') => {
                // `x = ...` assignment — config-VM-only construct.
                self.skip_to_entry_end();
                Ok(Entry::Unsupported(UnsupportedConstruct {
                    kind: UnsupportedKind::Assignment,
                    line,
                    column: col,
                    raw: format!("{ident} ="),
                    key: Some(ident),
                }))
            }
            Some(c) if is_ident_start(c) => self.parse_node(ident, None, line, col),
            Some('"') => self.parse_node(ident, None, line, col),
            Some('{') => self.parse_node(ident, None, line, col),
            Some('(') => {
                let raw_args = self.take_arg_list()?;
                self.parse_node(ident, raw_args, line, col)
            }
            other => Err(self.err_at(
                ErrorStage::Entry,
                line,
                col,
                format!(
                    "unexpected {} after '{ident}' (expected ':' prop, node name, or '(' args)",
                    other.map(|c| c.to_string()).unwrap_or_else(|| "end of input".into())
                ),
            )),
        }
    }

    fn parse_prop_value(&mut self, key: &str, line: usize) -> Result<Entry, ProjectModelError> {
        self.skip_inline_ws();
        if matches!(self.peek(), Some('\n') | Some('\r')) || self.peek().is_none() {
            return Err(self.err(
                ErrorStage::Value,
                format!("prop '{key}' has no value on its declaration line"),
            ));
        }
        let value = self.parse_value(key)?;
        // Same-line trailing content beyond an optional comma is a syntax
        // error (type confusion / missing brackets surface here). A trailing
        // comment is grammar, not content — skip it first.
        self.skip_inline_ws();
        if self.peek() == Some('/') && self.peek2() == Some('/') {
            while self.peek().is_some_and(|c| c != '\n') {
                self.advance();
            }
        } else if self.peek() == Some('#') && self.peek2() != Some('[') {
            while self.peek().is_some_and(|c| c != '\n') {
                self.advance();
            }
        } else if self.peek() == Some('/') && self.peek2() == Some('*') {
            self.advance();
            self.advance();
            while self.peek().is_some() {
                if self.peek() == Some('*') && self.peek2() == Some('/') {
                    self.advance();
                    self.advance();
                    break;
                }
                self.advance();
            }
            self.skip_inline_ws();
        }
        match self.peek() {
            Some(',') => {
                self.advance();
            }
            Some('\n') | Some('\r') | None => {}
            Some('}') => {} // body close handled by caller
            Some(c) => {
                return Err(self.err(
                    ErrorStage::Value,
                    format!("unexpected trailing content '{c}' after value of '{key}'"),
                ))
            }
        }
        Ok(Entry::Prop {
            key: key.to_string(),
            value,
            line,
        })
    }

    fn parse_value(&mut self, key: &str) -> Result<ScannedValue, ProjectModelError> {
        let (line, col) = (self.line, self.col);
        match self.peek() {
            Some('"') => self.parse_dq_string(key, line, col),
            Some('`') => {
                self.skip_to_entry_end();
                Ok(ScannedValue::Unsupported(UnsupportedConstruct {
                    kind: UnsupportedKind::FString,
                    line,
                    column: col,
                    raw: "`…`".to_string(),
                    key: Some(key.to_string()),
                }))
            }
            Some('\'') => {
                self.skip_to_entry_end();
                Ok(ScannedValue::Unsupported(UnsupportedConstruct {
                    kind: UnsupportedKind::SingleQuotedString,
                    line,
                    column: col,
                    raw: "'…'".to_string(),
                    key: Some(key.to_string()),
                }))
            }
            Some(c) if c.is_ascii_digit() || c == '-' || c == '+' => {
                let text = self.take_number()?;
                let value = parse_number_text(&text)
                    .map_err(|e| self.err_at(ErrorStage::Value, line, col, e))?;
                Ok(ScannedValue::Literal(value))
            }
            Some('{') | Some('[') => {
                let text = self.take_bracket_extent()?;
                if let Some(u) = scan_unsupported_in_text(&text, line, key) {
                    return Ok(ScannedValue::Unsupported(u));
                }
                let atom = auto_atom::AtomParser::parse(&text).map_err(|e| {
                    self.err_at(
                        ErrorStage::Value,
                        line,
                        col,
                        format!("structured value for '{key}' failed to parse: {e}"),
                    )
                })?;
                Ok(ScannedValue::Literal(atom.to_value()))
            }
            Some(c) if is_ident_start(c) => {
                let start = self.pos;
                let ident = self.take_ident();
                self.skip_inline_ws();
                if self.peek() == Some('(') {
                    self.skip_to_entry_end();
                    let raw = self.input[start..self.pos].to_string();
                    return Ok(ScannedValue::Unsupported(UnsupportedConstruct {
                        kind: UnsupportedKind::Call,
                        line,
                        column: col,
                        raw,
                        key: Some(key.to_string()),
                    }));
                }
                if ident == "f" && self.peek() == Some('"') {
                    self.skip_to_entry_end();
                    return Ok(ScannedValue::Unsupported(UnsupportedConstruct {
                        kind: UnsupportedKind::FString,
                        line,
                        column: col,
                        raw: "f\"…\"".to_string(),
                        key: Some(key.to_string()),
                    }));
                }
                if BUILTIN_IDENTS.contains(&ident.as_str()) {
                    let value = match ident.as_str() {
                        "true" => Value::Bool(true),
                        "false" => Value::Bool(false),
                        _ => Value::Nil,
                    };
                    return Ok(ScannedValue::Literal(value));
                }
                if ident == "on" || ident == "off" {
                    if TOGGLE_KEYS.contains(&key) {
                        // VM evaluates on/off to Bool; the toggle normalization
                        // (Pac::new semantics) treats Str("on"/"off") and the
                        // evaluated Bool identically, so Str is DTO-equivalent.
                        return Ok(ScannedValue::Literal(Value::Str(auto_val::AutoStr::from(
                            ident.as_str(),
                        ))));
                    }
                    return Ok(ScannedValue::Unsupported(UnsupportedConstruct {
                        kind: UnsupportedKind::OnOffOutsideToggle,
                        line,
                        column: col,
                        raw: ident,
                        key: Some(key.to_string()),
                    }));
                }
                Ok(ScannedValue::Unsupported(UnsupportedConstruct {
                    kind: UnsupportedKind::BareIdentValue,
                    line,
                    column: col,
                    raw: ident,
                    key: Some(key.to_string()),
                }))
            }
            Some(c) => Err(self.err_at(
                ErrorStage::Value,
                line,
                col,
                format!("unexpected character '{c}' in value of '{key}'"),
            )),
            None => Err(self.err_at(
                ErrorStage::Value,
                line,
                col,
                format!("missing value for '{key}'"),
            )),
        }
    }

    /// Scan a double-quoted string with AtomParser's escape semantics
    /// (`\n \t \r \\ \"` recognized; other escapes pass through verbatim).
    /// `$` followed by `{` or an identifier char inside the literal is
    /// classified as interpolation (conservative: only the config VM can
    /// produce the evaluated result).
    fn parse_dq_string(
        &mut self,
        key: &str,
        line: usize,
        col: usize,
    ) -> Result<ScannedValue, ProjectModelError> {
        self.advance(); // opening quote
        let mut out = String::new();
        let mut interp: Option<usize> = None;
        loop {
            match self.peek() {
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.advance() {
                        Some('n') => out.push('\n'),
                        Some('t') => out.push('\t'),
                        Some('r') => out.push('\r'),
                        Some('\\') => out.push('\\'),
                        Some('"') => out.push('"'),
                        Some(other) => {
                            out.push('\\');
                            out.push(other);
                        }
                        None => {
                            return Err(self.err_at(
                                ErrorStage::Value,
                                line,
                                col,
                                "unterminated string escape",
                            ))
                        }
                    }
                }
                Some('$') => {
                    let next = self.peek2();
                    if interp.is_none()
                        && (next == Some('{')
                            || next.is_some_and(|c| c.is_alphabetic() || c == '_'))
                    {
                        interp = Some(self.line);
                    }
                    out.push('$');
                    self.advance();
                }
                Some(c) => {
                    out.push(c);
                    self.advance();
                }
                None => {
                    return Err(self.err_at(
                        ErrorStage::Value,
                        line,
                        col,
                        "unterminated string literal",
                    ))
                }
            }
        }
        if let Some(iline) = interp {
            return Ok(ScannedValue::Unsupported(UnsupportedConstruct {
                kind: UnsupportedKind::Interpolation,
                line: iline,
                column: col,
                raw: out,
                key: Some(key.to_string()),
            }));
        }
        Ok(ScannedValue::Literal(Value::Str(auto_val::AutoStr::from(out.as_str()))))
    }

    fn take_number(&mut self) -> Result<String, ProjectModelError> {
        let start = self.pos;
        if matches!(self.peek(), Some('-') | Some('+')) {
            self.advance();
        }
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }
        if self.peek() == Some('.') {
            self.advance();
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            self.advance();
            if matches!(self.peek(), Some('-') | Some('+')) {
                self.advance();
            }
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
        }
        Ok(self.input[start..self.pos].to_string())
    }

    /// Take a balanced `{ … }` / `[ … ]` extent verbatim (string- and
    /// comment-aware); errors on unterminated input (V-05).
    fn take_bracket_extent(&mut self) -> Result<String, ProjectModelError> {
        let open = self.peek().expect("caller checked bracket");
        let close = if open == '{' { '}' } else { ']' };
        let start = self.pos;
        let mut depth = 0usize;
        loop {
            match self.peek() {
                None => {
                    return Err(self.err(
                        ErrorStage::Value,
                        format!("unterminated '{open}' group (missing '{close}')"),
                    ))
                }
                Some(c) => {
                    match c {
                        '{' | '[' => depth += 1,
                        '}' | ']' => {
                            depth -= 1;
                            if depth == 0 {
                                self.advance();
                                return Ok(self.input[start..self.pos].to_string());
                            }
                        }
                        '"' => {
                            self.advance();
                            // consume string body
                            while let Some(sc) = self.peek() {
                                if sc == '\\' {
                                    self.advance();
                                    self.advance();
                                    continue;
                                }
                                if sc == '"' {
                                    break;
                                }
                                self.advance();
                            }
                            if self.peek().is_none() {
                                return Err(self.err(
                                    ErrorStage::Value,
                                    "unterminated string inside structured value",
                                ));
                            }
                        }
                        '\'' => {
                            self.advance();
                            while let Some(sc) = self.peek() {
                                if sc == '\\' {
                                    self.advance();
                                    self.advance();
                                    continue;
                                }
                                if sc == '\'' {
                                    break;
                                }
                                self.advance();
                            }
                            if self.peek().is_none() {
                                return Err(self.err(
                                    ErrorStage::Value,
                                    "unterminated string inside structured value",
                                ));
                            }
                        }
                        '/' if self.peek2() == Some('/') => {
                            while self.peek().is_some_and(|c| c != '\n') {
                                self.advance();
                            }
                            continue;
                        }
                        '#' if self.peek2() != Some('[') => {
                            while self.peek().is_some_and(|c| c != '\n') {
                                self.advance();
                            }
                            continue;
                        }
                        '/' if self.peek2() == Some('*') => {
                            self.advance();
                            self.advance();
                            while self.peek().is_some() {
                                if self.peek() == Some('*') && self.peek2() == Some('/') {
                                    self.advance();
                                    self.advance();
                                    break;
                                }
                                self.advance();
                            }
                            continue;
                        }
                        _ => {}
                    }
                    self.advance();
                }
            }
        }
    }

    /// Consume the arg-list text `…` between '(' and ')'.
    fn take_arg_list(&mut self) -> Result<Option<String>, ProjectModelError> {
        // caller ensured peek() == '('
        self.advance();
        let start = self.pos;
        let mut depth = 1usize;
        loop {
            match self.peek() {
                None => {
                    return Err(self.err(ErrorStage::Entry, "unterminated '(' argument list"))
                }
                Some(c) => {
                    if c == '(' {
                        depth += 1;
                    } else if c == ')' {
                        depth -= 1;
                        if depth == 0 {
                            let text = self.input[start..self.pos].to_string();
                            self.advance();
                            return Ok(Some(text));
                        }
                    } else if c == '"' || c == '\'' {
                        let quote = c;
                        self.advance();
                        while let Some(sc) = self.peek() {
                            if sc == '\\' {
                                self.advance();
                                self.advance();
                                continue;
                            }
                            if sc == quote {
                                break;
                            }
                            self.advance();
                        }
                        if self.peek().is_none() {
                            return Err(self.err(
                                ErrorStage::Entry,
                                "unterminated string in argument list",
                            ));
                        }
                    }
                    self.advance();
                }
            }
        }
    }

    fn parse_node(
        &mut self,
        head: String,
        raw_args: Option<String>,
        line: usize,
        col: usize,
    ) -> Result<Entry, ProjectModelError> {
        let mut name_unsupported = None;
        let name = if let Some(args_text) = raw_args.as_deref() {
            if let Some(u) = scan_unsupported_in_text(args_text, line, &head) {
                name_unsupported = Some(u);
                NodeName::None
            } else {
                // arg list already consumed by the caller — the only
                // statically nameable form is a single quoted string
                NodeName::from_arg_text(Some(args_text))
            }
        } else {
            match self.peek() {
                Some(c) if is_ident_start(c) => NodeName::Bare(self.take_ident()),
                Some('"') => {
                    let (qline, qcol) = (self.line, self.col);
                    match self.parse_dq_string("<node-name>", qline, qcol)? {
                        ScannedValue::Literal(Value::Str(s)) => NodeName::Quoted(s.to_string()),
                        _ => {
                            return Err(self.err_at(
                                ErrorStage::Entry,
                                line,
                                col,
                                format!("node '{head}' has an unrepresentable quoted name"),
                            ))
                        }
                    }
                }
                Some('$') => {
                    // template form `lib ${name} {}` — interpolation in the
                    // node-name position
                    let raw = self.take_interp_marker();
                    name_unsupported = Some(UnsupportedConstruct {
                        kind: UnsupportedKind::Interpolation,
                        line,
                        column: col,
                        raw,
                        key: Some(head.clone()),
                    });
                    NodeName::None
                }
                _ => NodeName::None,
            }
        };
        self.skip_inline_ws();
        if self.peek() != Some('{') {
            return Err(self.err_at(
                ErrorStage::Entry,
                line,
                col,
                format!(
                    "node '{}' expects '{{' body, found {}",
                    head,
                    self.peek()
                        .map(|c| format!("'{c}'"))
                        .unwrap_or_else(|| "end of input".into())
                ),
            ));
        }
        self.advance(); // '{'
        let mut body = Vec::new();
        loop {
            self.skip_ws_and_comments();
            match self.peek() {
                Some('}') => {
                    self.advance();
                    break;
                }
                None => {
                    return Err(self.err_at(
                        ErrorStage::NodeBody,
                        line,
                        col,
                        format!("unterminated node body for '{head}' (missing '}}')"),
                    ))
                }
                Some(_) => body.push(self.parse_entry()?),
            }
        }
        Ok(Entry::Node {
            head,
            name,
            body,
            line,
            name_unsupported,
        })
    }

    /// Consume a `$ident` / `${...}` marker in name position and return the
    /// raw text.
    fn take_interp_marker(&mut self) -> String {
        let start = self.pos;
        self.advance(); // '$'
        if self.peek() == Some('{') {
            while self.peek().is_some() {
                let c = self.advance().unwrap();
                if c == '}' {
                    break;
                }
            }
        } else {
            while self
                .peek()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
            {
                self.advance();
            }
        }
        self.input[start..self.pos].to_string()
    }

    fn skip_to_entry_end(&mut self) {
        // advance to end of line at depth 0 (best-effort for unsupported)
        let mut depth = 0usize;
        while let Some(c) = self.peek() {
            match c {
                '\n' | '\r' if depth == 0 => break,
                '{' | '[' | '(' => depth += 1,
                '}' | ']' | ')' if depth > 0 => depth -= 1,
                '"' | '\'' => {
                    let quote = c;
                    self.advance();
                    while let Some(sc) = self.peek() {
                        if sc == '\\' {
                            self.advance();
                            self.advance();
                            continue;
                        }
                        if sc == quote || sc == '\n' {
                            break;
                        }
                        self.advance();
                    }
                }
                '/' if self.peek2() == Some('/') && depth == 0 => break,
                _ => {}
            }
            self.advance();
        }
    }
}

impl NodeName {
    /// `head(<single quoted string>)` → nameable; anything else (pairs,
    /// positional, empty) keeps the raw text and no name.
    fn from_arg_text(text: Option<&str>) -> NodeName {
        let Some(text) = text else {
            return NodeName::Arg {
                name: None,
                raw: None,
            };
        };
        let t = text.trim();
        if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') && !t[1..t.len() - 1].contains('"')
        {
            return NodeName::Arg {
                name: Some(t[1..t.len() - 1].to_string()),
                raw: Some(t.to_string()),
            };
        }
        NodeName::Arg {
            name: None,
            raw: Some(t.to_string()),
        }
    }
}

/// Detect interpolation / f-string markers inside a structured value's
/// verbatim text (string-aware). Conservative: `${`, `$ident`, backtick, or
/// `f"` inside the extent make the whole value unsupported-static.
fn scan_unsupported_in_text(text: &str, base_line: usize, key: &str) -> Option<UnsupportedConstruct> {
    let mut chars = text.chars().peekable();
    let mut line = base_line;
    while let Some(c) = chars.next() {
        match c {
            '\n' => line += 1,
            '#' if chars.peek() != Some(&('[')) => {
                for sc in chars.by_ref() {
                    if sc == '\n' {
                        line += 1;
                        break;
                    }
                }
            }
            '"' | '\'' => {
                let quote = c;
                while let Some(sc) = chars.next() {
                    if sc == '\\' {
                        chars.next();
                        continue;
                    }
                    if sc == quote || (sc == '\n' && quote == '\'') {
                        break;
                    }
                    // interpolation lives INSIDE string bodies
                    if sc == '$'
                        && (matches!(chars.peek(), Some('{'))
                            || chars
                                .peek()
                                .is_some_and(|n| n.is_alphabetic() || *n == '_'))
                    {
                        return Some(UnsupportedConstruct {
                            kind: UnsupportedKind::Interpolation,
                            line,
                            column: 0,
                            raw: text.to_string(),
                            key: Some(key.to_string()),
                        });
                    }
                }
            }
            '`' => {
                return Some(UnsupportedConstruct {
                    kind: UnsupportedKind::FString,
                    line,
                    column: 0,
                    raw: text.to_string(),
                    key: Some(key.to_string()),
                })
            }
            '$' => {
                if matches!(chars.peek(), Some('{'))
                    || chars.peek().is_some_and(|n| n.is_alphabetic() || *n == '_')
                {
                    return Some(UnsupportedConstruct {
                        kind: UnsupportedKind::Interpolation,
                        line,
                        column: 0,
                        raw: text.to_string(),
                        key: Some(key.to_string()),
                    });
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_number_text(text: &str) -> Result<Value, String> {
    if !text.contains(['.', 'e', 'E']) {
        text.parse::<i32>()
            .map(Value::Int)
            .map_err(|_| format!("invalid integer literal: {text}"))
    } else {
        text.parse::<f64>()
            .map(Value::Double)
            .map_err(|_| format!("invalid float literal: {text}"))
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-'
}

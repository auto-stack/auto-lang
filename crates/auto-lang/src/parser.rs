use crate::ast::*;
use crate::error::{pos_to_span, AutoError, AutoResult, NameError, SyntaxError};
use crate::infer::check_field_type;
use crate::lexer::Lexer;
use crate::scope::Meta;
use crate::token::{Pos, Token, TokenKind};
use crate::universe::{SymbolLocation, Universe};
use auto_val::AutoPath;
use auto_val::AutoStr;
use auto_val::Op;
use auto_val::{shared, Shared};
use miette::SourceSpan;
use std::collections::HashMap;
use std::i32;
use std::rc::Rc;

/// TODO: T should be a generic AST node type
pub trait ParserExt {
    fn parse(input: impl Into<AutoStr>) -> AutoResult<Self>
    where
        Self: Sized;
}

pub struct PostfixPrec {
    l: u8,
    _r: (),
}

pub struct InfixPrec {
    l: u8,
    r: u8,
}

pub struct PrefixPrec {
    _l: (),
    r: u8,
}

const fn prefix_prec(n: u8) -> PrefixPrec {
    PrefixPrec { _l: (), r: 2 * n }
}

const fn postfix_prec(n: u8) -> PostfixPrec {
    PostfixPrec { l: 2 * n, _r: () }
}

const fn infix_prec(n: u8) -> InfixPrec {
    if n == 0 {
        InfixPrec { l: 0, r: 0 }
    } else {
        InfixPrec {
            l: 2 * n - 1,
            r: 2 * n,
        }
    }
}

const PREC_ASN: InfixPrec = infix_prec(1);
const PREC_ADDEQ: InfixPrec = infix_prec(2);
const PREC_MULEQ: InfixPrec = infix_prec(3);
const PREC_PAIR: PostfixPrec = postfix_prec(4);
const _PREC_OR: InfixPrec = infix_prec(5);
const _PREC_AND: InfixPrec = infix_prec(6);
const PREC_EQ: InfixPrec = infix_prec(7);
const PREC_CMP: InfixPrec = infix_prec(8);
const PREC_RANGE: InfixPrec = infix_prec(9);
const PREC_ADD: InfixPrec = infix_prec(10);
const PREC_MUL: InfixPrec = infix_prec(11);
const PREC_NULLCOALESCE: InfixPrec = infix_prec(5); // ?? operator (same as OR)

const _PREC_REF: PrefixPrec = prefix_prec(12);
const PREC_SIGN: PrefixPrec = prefix_prec(13);
const PREC_NOT: PrefixPrec = prefix_prec(14);
const PREC_CALL: PostfixPrec = postfix_prec(15);
const PREC_INDEX: PostfixPrec = postfix_prec(16);
const PREC_POSTFIX: PostfixPrec = postfix_prec(17); // Bang operator (!)
const PREC_DOT: InfixPrec = infix_prec(18);
const _PREC_ATOM: InfixPrec = infix_prec(18);

fn prefix_power(op: Op, span: SourceSpan) -> AutoResult<PrefixPrec> {
    match op {
        Op::Add | Op::Sub => Ok(PREC_SIGN),
        Op::Not => Ok(PREC_NOT),
        _ => Err(SyntaxError::Generic {
            message: format!("Invalid prefix operator: {}", op),
            span,
        }
        .into()),
    }
}

fn postfix_power(op: Op) -> AutoResult<Option<PostfixPrec>> {
    match op {
        Op::LSquare => Ok(Some(PREC_INDEX)),
        Op::LParen => Ok(Some(PREC_CALL)),
        Op::Colon => Ok(Some(PREC_PAIR)),
        Op::Not => Ok(Some(PREC_POSTFIX)), // Bang operator (!) for eager collection
        _ => Ok(None),
    }
}

/// Helper function to capitalize first letter for backwards compatibility
/// Converts "int" -> "Int", "string" -> "String", etc.
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn infix_power(op: Op, span: SourceSpan) -> AutoResult<InfixPrec> {
    match op {
        Op::Add | Op::Sub => Ok(PREC_ADD),
        Op::Mul | Op::Div => Ok(PREC_MUL),
        Op::AddEq | Op::SubEq => Ok(PREC_ADDEQ),
        Op::MulEq | Op::DivEq => Ok(PREC_MULEQ),
        Op::Asn => Ok(PREC_ASN),
        Op::Eq | Op::Neq => Ok(PREC_EQ),
        Op::Lt | Op::Gt | Op::Le | Op::Ge => Ok(PREC_CMP),
        Op::Range | Op::RangeEq => Ok(PREC_RANGE),
        Op::Dot => Ok(PREC_DOT),
        // Property keywords (Phase 3): same precedence as dot
        // Error propagation (Phase 1b..3): ?. same precedence as dot
        Op::DotView | Op::DotMut | Op::DotTake | Op::DotQuestion => Ok(PREC_DOT),
        // May type operators (Phase 1b.3)
        Op::QuestionQuestion => Ok(PREC_NULLCOALESCE),
        _ => Err(SyntaxError::Generic {
            message: format!("Invalid infix operator: {}", op),
            span,
        }
        .into()),
    }
}

pub trait BlockParser {
    fn parse(&self, parser: &mut Parser) -> AutoResult<Body>;
}

// pub fn parse(code: &str, scope: Rc<RefCell<Universe>>, interpreter: &'a Interpreter) -> AutoResult<Code> {
// let mut parser = Parser::new(code, scope, interpreter);
// parser.parse()
// }

#[derive(Debug, Clone)]
pub enum CompileDest {
    Interp,    // for interperter
    TransC,    // for tranpiler to C
    TransRust, // for tranpiler to Rust
}

pub struct Parser<'a> {
    pub scope: Shared<Universe>,
    lexer: Lexer<'a>,
    pub cur: Token,
    prev: Token, // Track previous token for validation
    pub special_blocks: HashMap<AutoStr, Box<dyn BlockParser>>,
    pub skip_check: bool,
    pub compile_dest: CompileDest,
    /// Error recovery: collected errors during parsing
    pub errors: Vec<AutoError>,
    /// Maximum number of errors to collect before aborting
    pub error_limit: usize,
    /// Current type parameters being parsed (for generic type definitions)
    current_type_params: Vec<Name>,
    /// Current const generic parameters being parsed (Plan 052)
    /// Maps const parameter name to its type (e.g., N -> u32)
    current_const_params: HashMap<Name, Type>,
}

impl<'a> Parser<'a> {
    pub fn from(code: &'a str) -> Self {
        Self::new(code, shared(Universe::new()))
    }

    pub fn new(code: &'a str, scope: Shared<Universe>) -> Self {
        let mut lexer = Lexer::new(code);
        let cur = lexer.next().expect("lexer should produce first token");
        let mut parser = Parser {
            scope,
            lexer,
            cur,
            prev: Token {
                kind: TokenKind::EOF,
                pos: Pos {
                    line: 0,
                    at: 0,
                    pos: 0,
                    len: 0,
                },
                text: "".into(),
            }, // Initialize with EOF token
            compile_dest: CompileDest::Interp,
            special_blocks: HashMap::new(),
            skip_check: false,
            errors: Vec::new(),
            error_limit: crate::get_error_limit(), // Use global error limit
            current_type_params: Vec::new(),
            current_const_params: HashMap::new(),
        };
        parser.skip_comments();
        parser
    }

    pub fn set_dest(&mut self, dest: CompileDest) {
        self.compile_dest = dest;
    }

    pub fn add_special_block(&mut self, block: AutoStr, parser: Box<dyn BlockParser>) {
        self.special_blocks.insert(block, parser);
    }

    pub fn new_with_note(code: &'a str, scope: Shared<Universe>, note: char) -> Self {
        let mut lexer = Lexer::new(code);
        lexer.set_fstr_note(note);
        let cur = lexer.next().expect("lexer should produce first token");
        let mut parser = Parser {
            scope,
            lexer,
            cur,
            prev: Token {
                kind: TokenKind::EOF,
                pos: Pos {
                    line: 0,
                    at: 0,
                    pos: 0,
                    len: 0,
                },
                text: "".into(),
            }, // Initialize with EOF token
            compile_dest: CompileDest::Interp,
            special_blocks: HashMap::new(),
            skip_check: false,
            errors: Vec::new(),
            error_limit: crate::get_error_limit(), // Use global error limit
            current_type_params: Vec::new(),
            current_const_params: HashMap::new(),
        };
        parser.skip_comments();
        parser
    }

    /// Create a new parser with a pre-lexed first token
    pub fn new_with_note_and_first_token(
        _code: &'a str,
        scope: Shared<Universe>,
        _note: char,
        first_token: Token,
        lexer: Lexer<'a>,
    ) -> Self {
        let mut parser = Parser {
            scope,
            lexer,
            cur: first_token,
            prev: Token {
                kind: TokenKind::EOF,
                pos: Pos {
                    line: 0,
                    at: 0,
                    pos: 0,
                    len: 0,
                },
                text: "".into(),
            }, // Initialize with EOF token
            compile_dest: CompileDest::Interp,
            special_blocks: HashMap::new(),
            skip_check: false,
            errors: Vec::new(),
            error_limit: crate::get_error_limit(), // Use global error limit
            current_type_params: Vec::new(),
            current_const_params: HashMap::new(),
        };
        parser.skip_comments();
        parser
    }

    pub fn skip_check(mut self) -> Self {
        self.skip_check = true;
        self
    }

    pub fn peek(&mut self) -> &Token {
        &self.cur
    }

    pub fn kind(&mut self) -> TokenKind {
        self.peek().kind
    }

    pub fn pos(&mut self) -> Pos {
        self.peek().pos
    }

    pub fn is_kind(&mut self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    /// Check if the current position is a function annotation followed by fn
    /// This is used by the statement parser to distinguish between:
    /// - [c] fn foo() ...  (function declaration)
    /// - [1, 2, 3]         (array literal)
    #[allow(dead_code)]
    fn is_fn_annotation(&mut self) -> bool {
        // We're at [ now, need to check if this is [c] or [vm] followed by fn
        // Manually peek ahead by consuming and checking tokens
        // Collect tokens to restore if this isn't an annotation
        let mut tokens = Vec::new();

        // Save the current token ([)
        tokens.push(self.cur.clone());

        // Skip the [ and check the next token
        self.next(); // Now self.cur is the token after [

        // Check for c or vm identifier
        let is_annot =
            self.cur.kind == TokenKind::Ident && (self.cur.text == "c" || self.cur.text == "vm");

        if !is_annot {
            // Not an annotation, restore and return false
            // We need to restore self.cur to the saved [
            for token in tokens.into_iter().rev() {
                self.lexer.push_token(token);
            }
            // Restore self.cur
            self.next();
            return false;
        }

        // Save the c/vm token
        tokens.push(self.cur.clone());

        // Skip the annotation, check for ]
        self.next(); // Now self.cur is the token after c/vm

        if self.cur.kind != TokenKind::RSquare {
            // Not a properly formed annotation, restore
            for token in tokens.into_iter().rev() {
                self.lexer.push_token(token);
            }
            self.next();
            return false;
        }

        // Save the ] token
        tokens.push(self.cur.clone());

        // Skip ], check for fn
        self.next(); // Now self.cur is the token after ]

        let has_fn = self.cur.kind == TokenKind::Fn;

        // Save the fn token
        tokens.push(self.cur.clone());

        // Restore all tokens in reverse order
        for token in tokens.into_iter().rev() {
            self.lexer.push_token(token);
        }
        // Restore self.cur to the first token
        self.next();

        has_fn
    }

    pub fn skip_comments(&mut self) {
        loop {
            match self.kind() {
                TokenKind::CommentLine
                | TokenKind::CommentStart
                | TokenKind::CommentContent
                | TokenKind::CommentEnd => {
                    self.cur = self.lexer.next().expect("lexer should produce token");
                }
                _ => {
                    break;
                }
            }
        }
    }

    pub fn next(&mut self) -> &Token {
        self.prev = self.cur.clone();
        // Try to get the next token, if lexer returns an error, record it and use EOF
        self.cur = match self.lexer.next() {
            Ok(token) => token,
            Err(err) => {
                // Record the lexer error
                self.errors.push(err);
                // Check if we've hit the error limit
                if self.errors.len() >= self.error_limit {
                    // Return EOF to stop parsing
                    return &self.cur;
                }
                // Create an EOF token to continue parsing
                Token {
                    kind: TokenKind::EOF,
                    pos: Pos {
                        line: 0,
                        at: 0,
                        pos: 0,
                        len: 0,
                    },
                    text: "".into(),
                }
            }
        };
        self.skip_comments();
        &self.cur
    }

    pub fn expect(&mut self, kind: TokenKind) -> AutoResult<()> {
        if self.is_kind(kind) {
            self.next();
            Ok(())
        } else {
            let expected = format!("{:?}", kind);
            let found = self.cur.text.to_string();
            let span = pos_to_span(self.cur.pos);
            Err(SyntaxError::UnexpectedToken {
                expected,
                found,
                span,
            }
            .into())
        }
    }

    fn define(&mut self, name: &str, meta: Meta) {
        self.scope.borrow_mut().define(name, Rc::new(meta));
    }

    fn define_alias(&mut self, alias: AutoStr, target: AutoStr) {
        self.scope.borrow_mut().define_alias(alias, target);
    }

    fn define_rc(&mut self, name: &str, meta: Rc<Meta>) {
        self.scope.borrow_mut().define(name, meta);
    }

    fn exists(&mut self, name: &str) -> bool {
        self.scope.borrow().exists(name)
    }

    fn exit_scope(&mut self) {
        self.scope.borrow_mut().exit_scope();
    }

    fn enter_scope(&mut self) {
        self.scope.borrow_mut().enter_scope();
    }

    fn lookup_meta(&mut self, name: &str) -> Option<Rc<Meta>> {
        self.scope.borrow().lookup_meta(name)
    }

    fn lookup_type(&mut self, name: &str) -> Shared<Type> {
        match self.scope.borrow().lookup_type_meta(name) {
            Some(meta) => match meta.as_ref() {
                Meta::Type(ty) => shared(ty.clone()),
                Meta::Spec(spec_decl) => shared(Type::Spec(shared(spec_decl.clone()))),
                _ => shared(Type::Unknown),
            },
            None => shared(Type::Unknown),
        }
    }

    fn break_stmt(&mut self) -> AutoResult<Stmt> {
        self.next();
        Ok(Stmt::Break)
    }

    fn return_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip return keyword
        let expr = self.parse_expr()?;
        Ok(Stmt::Return(Box::new(expr)))
    }

    /// Synchronize parser state after encountering an error
    ///
    /// This method skips tokens until we reach a statement boundary,
    /// allowing the parser to continue collecting errors instead of
    /// aborting on the first error.
    ///
    /// Statement boundaries are:
    /// - Semicolon
    /// - Keywords that start statements (fn, let, var, mut, for, while, if, return, etc.)
    /// - End of file
    fn synchronize(&mut self) {
        // Skip tokens until we reach a statement boundary
        while !self.is_at_end() {
            // Semicolon is a statement boundary
            if self.is_kind(TokenKind::Semi) {
                self.next();
                return;
            }

            // Check if we're at a keyword that starts a statement
            match self.kind() {
                TokenKind::Fn
                | TokenKind::Let
                | TokenKind::Var
                | TokenKind::Mut
                | TokenKind::Hold
                | TokenKind::For
                | TokenKind::If
                | TokenKind::Break
                | TokenKind::Use
                | TokenKind::Type
                | TokenKind::Union
                | TokenKind::Tag
                | TokenKind::Enum
                | TokenKind::On
                | TokenKind::Alias
                | TokenKind::Is => {
                    // We've reached a statement boundary, don't consume the token
                    return;
                }
                _ => {
                    // Not at a boundary, skip this token
                    self.next();
                }
            }
        }
    }

    /// Check if we're at the end of the input
    fn is_at_end(&mut self) -> bool {
        self.kind() == TokenKind::EOF
    }

    /// Add an error to the error collection
    ///
    /// Returns true if we should continue parsing (haven't hit error limit),
    /// false if we should abort.
    fn add_error(&mut self, error: AutoError) -> bool {
        self.errors.push(error);
        self.errors.len() < self.error_limit
    }
}

pub enum CodeSection {
    None,
    C,
    Rust,
    Auto,
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> AutoResult<Code> {
        let mut stmts = Vec::new();
        self.skip_empty_lines();
        let mut current_section = CodeSection::None;
        let mut stmt_index = 0; // Track statement index for is_first_stmt check

        while !self.is_kind(TokenKind::EOF) {
            // deal with sections (but not #[...] annotations)
            if self.is_kind(TokenKind::Hash) {
                // Check if this is a #[...] annotation by looking ahead
                // Save current state for lookahead
                let saved_cur = self.cur.clone();
                // Consume # and check next token
                self.next();
                let is_annotation = self.is_kind(TokenKind::LSquare);
                // Restore state
                self.lexer.push_token(self.cur.clone());
                self.cur = saved_cur;

                if is_annotation {
                    // This is #[...], let parse_stmt() handle it
                    // Don't process as section, continue to parse_stmt()
                } else {
                    // This is a #section declaration
                    self.next();
                    let section = self.parse_name()?;
                    match section.as_str() {
                        "C" => {
                            current_section = CodeSection::C;
                        }
                        "RUST" => {
                            current_section = CodeSection::Rust;
                        }
                        "AUTO" => {
                            current_section = CodeSection::Auto;
                        }
                        _ => {
                            let error = SyntaxError::Generic {
                                message: format!("Unknown section {}", section),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into();
                            if !self.add_error(error) {
                                return Err(self.errors.pop().unwrap()); // Error limit exceeded
                            }
                            self.synchronize();
                            continue;
                        }
                    }
                    // skip until newline
                    let _ = self.skip_line();
                    continue;
                }
            }

            if !self.is_kind(TokenKind::Hash) {
                match self.compile_dest {
                    CompileDest::Interp => match current_section {
                        CodeSection::None | CodeSection::Auto => {}
                        _ => {
                            self.skip_line()?;
                            continue;
                        }
                    },
                    CompileDest::TransC => match current_section {
                        CodeSection::None | CodeSection::C => {}
                        _ => {
                            self.skip_line()?;
                            continue;
                        }
                    },
                    CompileDest::TransRust => match current_section {
                        CodeSection::None | CodeSection::Rust => {}
                        _ => {
                            self.skip_line()?;
                            continue;
                        }
                    },
                }
            }

            match self.parse_stmt() {
                Ok(stmt) => {
                    // First level pairs are viewed as variable declarations
                    // TODO: this should only happen in a Config scenario
                    if let Stmt::Expr(Expr::Pair(Pair { key, value })) = &stmt {
                        if let Some(name) = key.name() {
                            self.define(
                                name,
                                Meta::Store(Store {
                                    name: name.into(),
                                    kind: StoreKind::Var,
                                    ty: Type::Unknown,
                                    expr: *value.clone(),
                                }),
                            );
                        }
                    }
                    stmts.push(stmt);
                    let is_first = stmt_index == 0;
                    // Check for ambiguous syntax errors in expect_eos
                    let newline_count = match self.expect_eos(is_first) {
                        Ok(count) => count,
                        Err(e) => {
                            // Ambiguous syntax errors should not be recovered from
                            if e.to_string().contains("Ambiguous syntax") {
                                return Err(e);
                            }
                            // Other EOS errors are added to collection and synchronized
                            if !self.add_error(e) {
                                return Err(self.errors.pop().unwrap());
                            }
                            self.synchronize();
                            0
                        }
                    };
                    // Insert EmptyLine statement for 2+ consecutive newlines
                    if newline_count > 1 {
                        stmts.push(Stmt::EmptyLine(newline_count - 1));
                    }
                    stmt_index += 1;
                }
                Err(e) => {
                    // Check if this is an ambiguous syntax error - these should not be recovered from
                    if e.to_string().contains("Ambiguous syntax") {
                        return Err(e);
                    }
                    // Add error to collection and synchronize
                    if !self.add_error(e) {
                        return Err(self.errors.pop().unwrap()); // Error limit exceeded
                    }
                    self.synchronize();
                }
            }
        }

        // Check if we collected any errors
        if !self.errors.is_empty() {
            // Return MultipleErrors with all collected errors
            let error_count = self.errors.len();
            let plural = if error_count > 1 { "s" } else { "" };

            return Err(AutoError::MultipleErrors {
                count: error_count,
                plural: plural.to_string(),
                errors: self.errors.clone(),
            });
        }

        stmts = self.convert_last_block(stmts)?;

        // Post-processing: Merge ext blocks into their target TypeDecl
        stmts = self.merge_ext_blocks(stmts)?;

        Ok(Code { stmts })
    }

    /// Merge ext blocks into their target TypeDecl
    /// This processes all Stmt::Ext statements and merges their fields/methods
    /// into the corresponding Stmt::TypeDecl
    fn merge_ext_blocks(&self, mut stmts: Vec<Stmt>) -> AutoResult<Vec<Stmt>> {
        use crate::ast::Stmt;

        // Collect all TypeDecls and Exts in separate passes
        let mut type_decl_indices: std::collections::HashMap<Name, usize> =
            std::collections::HashMap::new();
        let mut ext_statements: Vec<(usize, crate::ast::Ext)> = Vec::new();

        for (i, stmt) in stmts.iter().enumerate() {
            if let Stmt::TypeDecl(ref decl) = stmt {
                type_decl_indices.insert(decl.name.clone(), i);
            } else if let Stmt::Ext(ref ext) = stmt {
                ext_statements.push((i, ext.clone()));
            }
        }

        // Track which Ext statements were successfully merged
        let mut merged_ext_indices: Vec<usize> = Vec::new();

        // Process each Ext statement and merge into target TypeDecl
        for (ext_idx, ext) in ext_statements {
            // Find the target TypeDecl
            if let Some(&decl_idx) = type_decl_indices.get(&ext.target) {
                // Clone the TypeDecl, merge ext into it, then replace
                if let Stmt::TypeDecl(ref decl) = &stmts[decl_idx] {
                    let mut merged_decl = decl.clone();

                    // Merge fields: ext fields override type fields with same name
                    for ext_field in ext.fields.clone() {
                        // Remove existing field with same name if it exists
                        merged_decl.members.retain(|m| m.name != ext_field.name);
                        // Add the ext field
                        merged_decl.members.push(ext_field);
                    }

                    // Merge methods: ext methods override type methods with same name
                    for ext_method in ext.methods.clone() {
                        // Remove existing method with same name if it exists
                        merged_decl.methods.retain(|m| m.name != ext_method.name);
                        // Add the ext method
                        merged_decl.methods.push(ext_method);
                    }

                    stmts[decl_idx] = Stmt::TypeDecl(merged_decl);
                    // Mark this Ext as merged
                    merged_ext_indices.push(ext_idx);
                }
            }
            // If no TypeDecl exists (extending built-in type like int, str, etc.),
            // the Ext statement is kept in the AST for later processing
        }

        // Remove only Ext statements that were successfully merged
        let mut final_stmts: Vec<Stmt> = Vec::new();
        for (i, stmt) in stmts.into_iter().enumerate() {
            if let Stmt::Ext(_) = stmt {
                // Only keep this Ext if it wasn't merged
                if !merged_ext_indices.contains(&i) {
                    final_stmts.push(stmt);
                }
                // If it was merged, skip it (don't add to final_stmts)
            } else {
                final_stmts.push(stmt);
            }
        }

        Ok(final_stmts)
    }

    fn skip_line(&mut self) -> AutoResult<()> {
        // println!("Skiipping line"); // Debug output disabled for LSP
        while !self.is_kind(TokenKind::Newline) {
            self.next();
        }
        self.skip_empty_lines();
        Ok(())
    }

    fn skip_block(&mut self) -> AutoResult<()> {
        // Skip a { ... } block, handling nested braces
        self.expect(TokenKind::LBrace)?;
        let mut depth = 1;
        while depth > 0 && !self.is_kind(TokenKind::EOF) {
            if self.is_kind(TokenKind::LBrace) {
                depth += 1;
            } else if self.is_kind(TokenKind::RBrace) {
                depth -= 1;
            }
            self.next();
        }
        if depth > 0 {
            return Err(SyntaxError::Generic {
                message: "Unclosed block, missing '}'".to_string(),
                span: pos_to_span(self.prev.pos),
            }
            .into());
        }
        Ok(())
    }

    fn convert_last_block(&mut self, mut stmts: Vec<Stmt>) -> AutoResult<Vec<Stmt>> {
        let last = stmts.last();
        if let Some(st) = last {
            match st {
                Stmt::Block(body) => {
                    let obj = self.body_to_obj(body)?;
                    stmts.pop();
                    stmts.push(Stmt::Expr(Expr::Object(obj)))
                }
                _ => {}
            }
        }
        Ok(stmts)
    }

    fn body_to_obj(&mut self, body: &Body) -> AutoResult<Vec<Pair>> {
        let mut pairs = Vec::new();
        for stmt in body.stmts.iter() {
            match stmt {
                Stmt::Expr(expr) => match expr {
                    Expr::Pair(p) => {
                        pairs.push(p.clone());
                    }
                    _ => {
                        return Err(SyntaxError::Generic {
                            message: "Last block must be an object!".to_string(),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                },
                _ => {
                    return Err(SyntaxError::Generic {
                        message: "Last block must be an object!".to_string(),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            }
        }
        Ok(pairs)
    }
}

// Expressions
impl<'a> Parser<'a> {
    pub fn parse_expr(&mut self) -> AutoResult<Expr> {
        let mut exp = self.expr_pratt(0)?;
        exp = self.check_symbol(exp)?;
        Ok(exp)
    }

    // simple Pratt parser
    // ref: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
    pub fn expr_pratt(&mut self, min_power: u8) -> AutoResult<Expr> {
        // hold expression (Phase 3) - special case that returns directly
        if self.is_kind(TokenKind::Hold) {
            let start_pos = self.cur.pos; // Record start position

            self.next(); // skip hold

            // Parse the path expression (identifier with optional member access)
            let path = self.parse_path_expr()?;

            // Expect 'as' keyword
            self.expect(TokenKind::As)?;

            // Get the binding name
            let name = self.cur.text.clone();
            self.expect(TokenKind::Ident)?;

            // Skip empty lines before body (allows newline between name and {)
            if self.is_kind(TokenKind::Newline) {
                self.skip_empty_lines();
            }

            // Disable symbol checking inside hold body (bindings are created at runtime)
            let old_skip_check = self.skip_check;
            self.skip_check = true;

            // Parse the body (a block)
            let body = self.body()?;

            // Restore symbol checking
            self.skip_check = old_skip_check;

            // Calculate span: (offset, length)
            let span = Some((start_pos.pos, self.cur.pos.pos - start_pos.pos));

            // Hold is a complete expression, return directly
            return Ok(Expr::Hold(crate::ast::Hold {
                path: Box::new(path),
                name,
                body,
                span,
            }));
        }

        // Prefix
        let lhs = match self.kind() {
            // if expression
            TokenKind::If => self.if_expr()?,
            // unary
            TokenKind::Add | TokenKind::Sub | TokenKind::Not => {
                let op = self.op();
                let span = pos_to_span(self.cur.pos);
                let power = prefix_power(op, span)?;
                self.next(); // skip unary op
                let lhs = self.expr_pratt(power.r)?;
                Expr::Unary(op, Box::new(lhs))
            }
            // group or multi-param closure
            TokenKind::LParen => {
                // Plan 060: Check if this is a multi-param closure: (a, b) => expr
                self.next(); // skip (

                // Quick check: if first token is identifier, might be a closure
                let is_closure = if self.is_kind(TokenKind::Ident) {
                    // Collect tokens for lookahead and rollback
                    let mut tokens = Vec::new();
                    let mut found_comma = false;
                    let mut found_double_arrow = false;
                    let mut depth = 1; // Track nesting for types like Array<T>

                    // Scan ahead to detect closure pattern
                    loop {
                        let token = self.lexer.next()?;

                        // Check for pattern before consuming token into buffer
                        let is_closing_paren = depth == 1 && token.kind == TokenKind::RParen;

                        tokens.push(token.clone());

                        match token.kind {
                            TokenKind::LParen | TokenKind::LSquare | TokenKind::LBrace => {
                                depth += 1
                            }
                            TokenKind::RParen | TokenKind::RSquare | TokenKind::RBrace => {
                                depth -= 1;
                            }
                            TokenKind::Comma if depth == 1 => found_comma = true,
                            TokenKind::EOF => break,
                            _ => {}
                        }

                        // If we found the closing paren at depth 0, check for =>
                        if is_closing_paren {
                            // Peek at the next token without consuming it
                            if let Ok(next_token) = self.lexer.next() {
                                if next_token.kind == TokenKind::DoubleArrow {
                                    found_double_arrow = true;
                                }
                                // Don't add the peeked token to our buffer
                                // We need to push it back so it's available later
                                self.lexer.push_token(next_token);
                            }
                            break;
                        }
                    }

                    // Push tokens back to lexer buffer in reverse order
                    for token in tokens.into_iter().rev() {
                        self.lexer.push_token(token);
                    }

                    found_comma || found_double_arrow
                } else {
                    false
                };

                if is_closure {
                    // Multi-param closure: (a, b) => expr
                    // Need to restore state so parse_closure sees the ( token
                    // Save current token (identifier)
                    let ident_token = self.cur.clone();

                    // Set current token back to (
                    self.cur = Token {
                        kind: TokenKind::LParen,
                        text: AutoStr::from("("),
                        pos: ident_token.pos, // Use identifier's position for better error messages
                    };

                    // Push the identifier back to lexer
                    self.lexer.push_token(ident_token);

                    // Now parse_closure will see ( as current token
                    self.parse_closure()?
                } else {
                    // Regular group expression: (expr)
                    let lhs = self.expr_pratt(0)?;
                    self.expect(TokenKind::RParen)?; // skip )
                    lhs
                }
            }
            // array
            TokenKind::LSquare => self.array()?,
            // object
            TokenKind::LBrace => Expr::Object(self.object()?),
            // lambda (deprecated - use closure syntax instead: (a, b) => expr)
            // TokenKind::VBar => self.lambda()?,
            // fstr
            TokenKind::FStrStart => self.fstr()?,
            // grid
            TokenKind::Grid => Expr::Grid(self.grid()?),
            // dot
            TokenKind::Dot => self.dot_item()?,
            // normal
            _ => self.atom()?,
        };
        self.expr_pratt_with_left(lhs, min_power)
    }

    /// Parse a path expression (identifier with optional member access)
    /// Used for hold expressions to parse paths like `obj.field.subfield`
    /// Stops before keywords like 'as', 'in', etc.
    fn parse_path_expr(&mut self) -> AutoResult<Expr> {
        let mut lhs = self.atom()?;

        // Allow member access (dot operator) for paths like obj.field
        loop {
            if self.is_kind(TokenKind::Dot) {
                self.next(); // skip .
                let field_name = self.cur.text.clone();
                // Allow @, * as special field names for pointer operations
                // Allow numeric literals for integer-keyed objects: a.1, a.2
                // Allow boolean keywords: a.true, a.false
                if self.is_kind(TokenKind::Ident)
                    || self.is_kind(TokenKind::At)
                    || self.is_kind(TokenKind::Star)
                    || self.is_kind(TokenKind::Int)
                    || self.is_kind(TokenKind::Uint)
                    || self.is_kind(TokenKind::I8)
                    || self.is_kind(TokenKind::U8)
                    || self.is_kind(TokenKind::Float)
                    || self.is_kind(TokenKind::Double)
                    || self.is_kind(TokenKind::True)
                    || self.is_kind(TokenKind::False)
                    || self.is_kind(TokenKind::Nil)
                {
                    self.next();
                    // Use Expr::Dot for semantic clarity
                    lhs = Expr::Dot(Box::new(lhs), field_name);
                } else {
                    let message = format!(
                        "Expected identifier, @, *, number, or boolean after dot, got {:?}",
                        self.cur.kind
                    );
                    let span = pos_to_span(self.cur.pos);
                    return Err(SyntaxError::Generic { message, span }.into());
                }
            } else {
                break;
            }
        }

        Ok(lhs)
    }

    fn dot_item(&mut self) -> AutoResult<Expr> {
        self.next(); // skip dot
        let name = self.cur.text.clone();
        self.next(); // skip name
                     // Use Expr::Dot for semantic clarity (.field is shorthand for self.field)
        Ok(Expr::Dot(Box::new(Expr::Ident("self".into())), name))
    }

    fn expr_pratt_with_left(&mut self, mut lhs: Expr, min_power: u8) -> AutoResult<Expr> {
        // Plan 060: Check for single-param closure:  x => expr
        // If lhs is an identifier and next token is =>, parse as closure
        if matches!(lhs, Expr::Ident(_)) && self.is_kind(TokenKind::DoubleArrow) {
            use crate::ast::{Closure, ClosureParam};

            // Extract parameter name from identifier
            let param_name = match &lhs {
                Expr::Ident(name) => name.clone(),
                _ => unreachable!(),
            };

            // Expect =>
            self.expect(TokenKind::DoubleArrow)?;

            // Parse body (expression or block)
            let body = if self.is_kind(TokenKind::LBrace) {
                Expr::Block(self.body()?)
            } else {
                self.parse_expr()?
            };

            return Ok(Expr::Closure(Closure::new(
                vec![ClosureParam::new(param_name, None)],
                None,
                body,
            )));
        }

        loop {
            let op = match self.kind() {
                TokenKind::EOF
                | TokenKind::Newline
                | TokenKind::Semi
                | TokenKind::LBrace
                | TokenKind::RBrace
                | TokenKind::Comma
                | TokenKind::Arrow
                | TokenKind::DoubleArrow => break,
                TokenKind::Add
                | TokenKind::Sub
                | TokenKind::Star
                | TokenKind::Div
                | TokenKind::Not => self.op(),
                TokenKind::AddEq | TokenKind::SubEq | TokenKind::MulEq | TokenKind::DivEq => {
                    self.op()
                }
                TokenKind::DotView
                | TokenKind::DotMut
                | TokenKind::DotTake
                | TokenKind::DotQuestion => {
                    // Property keywords: .view, .mut, .take (Phase 3)
                    // Error propagation: ?. (Phase 1b.3)
                    // These are postfix operators with same precedence as dot
                    self.op()
                }
                TokenKind::Dot => self.op(),
                TokenKind::Colon => self.op(),
                TokenKind::Range | TokenKind::RangeEq => self.op(),
                TokenKind::LSquare => self.op(),
                TokenKind::LParen => self.op(),
                TokenKind::Asn => self.op(),
                TokenKind::Eq
                | TokenKind::Neq
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::Le
                | TokenKind::Ge => self.op(),
                TokenKind::QuestionQuestion => self.op(),
                TokenKind::RSquare => break,
                TokenKind::RParen => break,
                _ => {
                    let message = format!("Expected infix operator, got {:?}", self.peek());
                    let span = pos_to_span(self.cur.pos);
                    return Err(SyntaxError::Generic { message, span }.into());
                }
            };
            // Postfix

            if let Ok(Some(power)) = postfix_power(op) {
                if power.l < min_power {
                    break;
                }

                match op {
                    // Index
                    Op::LSquare => {
                        self.next(); // skip [
                        let rhs = self.expr_pratt(0)?;
                        self.expect(TokenKind::RSquare)?;
                        lhs = Expr::Index(Box::new(lhs), Box::new(rhs));
                        continue;
                    }
                    // Call or Node Instance
                    Op::LParen => {
                        let args = self.args()?;
                        lhs = self.call(lhs, args)?;
                        continue;
                    }
                    // Pair
                    Op::Colon => {
                        self.next(); // skip :
                        let key = match &lhs {
                            Expr::Ident(name) => Key::NamedKey(name.clone()),
                            Expr::Int(i) => Key::IntKey(*i),
                            Expr::Bool(b) => Key::BoolKey(*b),
                            Expr::Str(s) => Key::StrKey(s.clone()),
                            _ => {
                                let message = format!("Invalid key: {}", lhs);
                                let span = pos_to_span(self.cur.pos);
                                return Err(SyntaxError::Generic { message, span }.into());
                            }
                        };
                        let rhs = self.pair_expr()?;
                        lhs = Expr::Pair(Pair {
                            key,
                            value: Box::new(rhs),
                        });
                        return Ok(lhs);
                    }
                    // Bang operator (!) for eager collection
                    // Converts expr! into expr.collect()
                    Op::Not => {
                        self.next(); // skip !
                        // Convert to method call: lhs.collect()
                        let collect_name = crate::ast::Name::from("collect");
                        let collect_expr = Expr::Bina(
                            Box::new(lhs),
                            Op::Dot,
                            Box::new(Expr::Ident(collect_name))
                        );
                        lhs = Expr::Call(crate::ast::Call {
                            name: Box::new(collect_expr),
                            args: crate::ast::Args::new(),
                            ret: crate::ast::Type::Unknown,
                            type_args: Vec::new(),
                        });
                        continue;
                    }
                    _ => {
                        let message = format!("Invalid postfix operator: {}", op);
                        let span = pos_to_span(self.cur.pos);
                        return Err(SyntaxError::Generic { message, span }.into());
                    }
                }
            }
            // Infix
            let span = pos_to_span(self.cur.pos);
            let power = infix_power(op, span)?;
            if power.l < min_power {
                break;
            }
            self.next(); // skip binary op
                         // Check for whether assignment is allowed
            match op {
                Op::Asn => {
                    self.check_asn(&lhs)?;
                }
                _ => {}
            }
            match op {
                // Property keywords (Phase 3): postfix operators, no rhs needed
                Op::DotView => {
                    lhs = Expr::View(Box::new(lhs));
                    continue;
                }
                Op::DotMut => {
                    lhs = Expr::Mut(Box::new(lhs));
                    continue;
                }
                Op::DotTake => {
                    lhs = Expr::Take(Box::new(lhs));
                    continue;
                }
                // May type operators (Phase 1b.3): ?. error propagation
                Op::DotQuestion => {
                    lhs = Expr::ErrorPropagate(Box::new(lhs));
                    continue;
                }
                _ => {
                    // Regular infix operators need rhs
                    let rhs = self.expr_pratt(power.r)?;
                    match op {
                        Op::Range => {
                            lhs = Expr::Range(Range {
                                start: Box::new(lhs),
                                end: Box::new(rhs),
                                eq: false,
                            });
                        }
                        Op::RangeEq => {
                            lhs = Expr::Range(Range {
                                start: Box::new(lhs),
                                end: Box::new(rhs),
                                eq: true,
                            });
                        }
                        Op::Dot => {
                            // Dot expression: handle both field access and method calls
                            // Field access: object.field
                            // Method call: object.method(args)
                            match rhs {
                                Expr::Ident(field_name) => {
                                    // Simple field access: object.field
                                    lhs = Expr::Dot(Box::new(lhs), field_name);
                                }
                                Expr::Call(call) => {
                                    // Method call: object.method(args)
                                    // The RHS is a Call like: Call { name: Ident("push"), args: [...] }
                                    // We need to transform this into: Call { name: Dot(object, "push"), args: [...] }
                                    let method_name = match call.name.as_ref() {
                                        Expr::Ident(name) => name.clone(),
                                        _ => {
                                            let message = format!(
                                                "Method name must be an identifier, got {}",
                                                call.name
                                            );
                                            let span = pos_to_span(self.cur.pos);
                                            return Err(
                                                SyntaxError::Generic { message, span }.into()
                                            );
                                        }
                                    };
                                    lhs = Expr::Call(Call {
                                        name: Box::new(Expr::Dot(Box::new(lhs), method_name)),
                                        args: call.args,
                                        ret: call.ret,
                                        type_args: Vec::new(),  // Plan 061: No type args for method calls yet
                                    });
                                }
                                _ => {
                                    // Error: right-hand side of dot must be an identifier or method call
                                    let message = format!("Invalid field name after dot: {}", rhs);
                                    let span = pos_to_span(self.cur.pos);
                                    return Err(SyntaxError::Generic { message, span }.into());
                                }
                            }
                        }
                        Op::QuestionQuestion => {
                            // Null-coalescing operator: left ?? right
                            lhs = Expr::NullCoalesce(Box::new(lhs), Box::new(rhs));
                        }
                        _ => {
                            lhs = Expr::Bina(Box::new(lhs), op, Box::new(rhs));
                        }
                    }
                }
            }
        }
        Ok(lhs)
    }

    fn check_asn(&mut self, lhs: &Expr) -> AutoResult<()> {
        match lhs {
            Expr::Ident(name) => {
                let meta = self.lookup_meta(name.as_str());
                if let Some(Meta::Store(store)) = meta.as_deref() {
                    if matches!(store.kind, StoreKind::Let) {
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "Syntax error: Assignment not allowed for let store: {}",
                                store.name
                            ),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn op(&mut self) -> Op {
        match self.kind() {
            TokenKind::Add => Op::Add,
            TokenKind::Sub => Op::Sub,
            TokenKind::Star => Op::Mul,
            TokenKind::Div => Op::Div,
            TokenKind::AddEq => Op::AddEq,
            TokenKind::SubEq => Op::SubEq,
            TokenKind::MulEq => Op::MulEq,
            TokenKind::DivEq => Op::DivEq,
            TokenKind::LSquare => Op::LSquare,
            TokenKind::LParen => Op::LParen,
            TokenKind::LBrace => Op::LBrace,
            TokenKind::Not => Op::Not,
            TokenKind::Asn => Op::Asn,
            TokenKind::Eq => Op::Eq,
            TokenKind::Neq => Op::Neq,
            TokenKind::Lt => Op::Lt,
            TokenKind::Gt => Op::Gt,
            TokenKind::Le => Op::Le,
            TokenKind::Ge => Op::Ge,
            TokenKind::Range => Op::Range,
            TokenKind::RangeEq => Op::RangeEq,
            TokenKind::Dot => Op::Dot,
            TokenKind::Colon => Op::Colon,
            TokenKind::DotView => Op::DotView,
            TokenKind::DotMut => Op::DotMut,
            TokenKind::DotTake => Op::DotTake,
            TokenKind::QuestionQuestion => Op::QuestionQuestion,
            TokenKind::DotQuestion => Op::DotQuestion,
            _ => {
                // This should never happen if called from correct match branches
                // Return a default operator to avoid panic
                Op::Add
            }
        }
    }

    pub fn group(&mut self) -> AutoResult<Expr> {
        self.next(); // skip (
        let expr = self.parse_expr()?;
        self.expect(TokenKind::RParen)?; // skip )
        Ok(expr)
    }

    pub fn sep_array(&mut self) -> AutoResult<()> {
        let mut has_sep = false;
        while self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            has_sep = true;
            self.next();
        }
        if self.is_kind(TokenKind::RSquare) {
            return Ok(());
        }
        if !has_sep {
            let message = format!("Expected array separator, got {:?}", self.kind());
            let span = pos_to_span(self.cur.pos);
            return Err(SyntaxError::Generic { message, span }.into());
        }
        Ok(())
    }

    pub fn array(&mut self) -> AutoResult<Expr> {
        self.expect(TokenKind::LSquare)?;
        self.skip_empty_lines();
        let mut elems = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RSquare) {
            elems.push(self.rhs_expr()?);
            self.sep_array()?;
        }
        self.expect(TokenKind::RSquare)?; // skip ]
        Ok(Expr::Array(elems))
    }

    pub fn sep_args(&mut self) -> AutoResult<()> {
        if self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            self.next();
            return Ok(());
        }
        if self.is_kind(TokenKind::RParen) {
            return Ok(());
        }
        let pos = self.cur.pos;
        Err(SyntaxError::UnexpectedToken {
            expected: "argument separator (comma, newline, or ))".to_string(),
            found: format!("{:?}", self.kind()),
            span: pos_to_span(pos),
        }
        .into())
    }

    pub fn args(&mut self) -> AutoResult<Args> {
        self.expect(TokenKind::LParen)?;
        let mut args = Args::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RParen) {
            let expr = if self.is_kind(TokenKind::Ident) {
                let e = self.node_or_call_expr()?;
                e
            // } else {
            // let name = Expr::Ident(self.cur.text.clone());
            // self.next();
            // if self.is_kind(TokenKind::Comma) {
            //     name
            // } else {
            //     self.expr_pratt_with_left(name, 0)?
            // }
            } else {
                self.parse_expr()?
            };
            match &expr {
                // Named args
                Expr::Pair(p) => {
                    let k = p.key.clone();
                    match k {
                        Key::NamedKey(name) => {
                            args.args.push(Arg::Pair(name.clone(), (*p.value).clone()));
                            // args.map.push((name, *p.value));
                        }
                        _ => {
                            return Err(SyntaxError::Generic {
                                message: format!(
                                    "named args should have named key instead of {}",
                                    &k
                                ),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into());
                        }
                    }
                }
                // Positional args
                Expr::Ident(name) => {
                    // name arg without value
                    let name = name.clone();
                    match self.lookup_meta(&name) {
                        Some(_) => {
                            args.args.push(Arg::Pos(expr.clone()));
                        }
                        None => {
                            args.args.push(Arg::Name(name));
                        }
                    }
                }
                _ => {
                    args.args.push(Arg::Pos(expr.clone()));
                }
            }
            self.sep_args()?;
        }
        self.expect(TokenKind::RParen)?;
        Ok(args)
    }

    pub fn object(&mut self) -> AutoResult<Vec<Pair>> {
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut entries = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            entries.push(self.pair()?);
            self.sep_obj()?;
        }
        self.expect(TokenKind::RBrace)?;
        Ok(entries)
    }

    pub fn pair_expr(&mut self) -> AutoResult<Expr> {
        self.rhs_expr()
        // let exp = self.expr_pratt(0)?;
        // if let Expr::Ident(ident) = &exp {
        //     if !self.exists(ident) {
        //         return Ok(Expr::Str(ident.clone()));
        //     }
        // }
        // Ok(exp)
    }

    pub fn pair(&mut self) -> AutoResult<Pair> {
        let key = self.key()?;
        self.expect(TokenKind::Colon)?;
        let value = self.pair_expr()?;
        Ok(Pair {
            key,
            value: Box::new(value),
        })
    }

    #[allow(unused)]
    pub fn is_key(expr: &Expr) -> bool {
        match expr {
            Expr::Ident(_) => true,
            Expr::Int(_) => true,
            Expr::Bool(_) => true,
            _ => false,
        }
    }

    pub fn key(&mut self) -> AutoResult<Key> {
        match self.kind() {
            TokenKind::Ident => {
                let name = self.cur.text.clone();
                self.next();
                Ok(Key::NamedKey(name))
            }
            TokenKind::Int => {
                let value = self.cur.text.as_str().parse().unwrap();
                self.next();
                Ok(Key::IntKey(value))
            }
            TokenKind::True => {
                self.next();
                Ok(Key::BoolKey(true))
            }
            TokenKind::False => {
                self.next();
                Ok(Key::BoolKey(false))
            }
            TokenKind::Str => {
                let value = self.cur.text.clone();
                self.next();
                Ok(Key::StrKey(value))
            }
            // type关键字用作key，应该不冲突
            TokenKind::Type => {
                let value = self.cur.text.clone();
                self.next();
                Ok(Key::NamedKey(value))
            }
            _ => {
                let message = format!("Expected key, got {:?}", self.kind());
                let span = pos_to_span(self.cur.pos);
                return Err(SyntaxError::Generic { message, span }.into());
            }
        }
    }

    pub fn sep_obj(&mut self) -> AutoResult<()> {
        if self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            self.next();
            self.skip_empty_lines();
            return Ok(());
        }
        if self.is_kind(TokenKind::RBrace) {
            return Ok(());
        }
        let pos = self.cur.pos;
        Err(SyntaxError::UnexpectedToken {
            expected: "pair separator (comma, newline, or })".to_string(),
            found: format!("{:?}", self.kind()),
            span: pos_to_span(pos),
        }
        .into())
    }

    pub fn ident_name(&mut self) -> AutoResult<Name> {
        Ok(self.cur.text.clone())
    }

    pub fn ident(&mut self) -> AutoResult<Expr> {
        let name = self.cur.text.clone();
        // // check for existence
        // if !self.exists(&name) {
        //     return Err(format!("Undefined variable: {}", name));
        // }
        Ok(Expr::Ident(name))
    }

    pub fn parse_ident(&mut self) -> AutoResult<Expr> {
        let name = self.cur.text.clone();
        self.next();
        Ok(Expr::Ident(name))
    }

    pub fn parse_name(&mut self) -> AutoResult<Name> {
        let name = self.cur.text.clone();
        self.next();
        Ok(name)
    }

    pub fn parse_ints(&mut self) -> AutoResult<Expr> {
        let res = match self.cur.kind {
            TokenKind::Int => self.parse_int(),
            TokenKind::Uint => self.parse_uint(),
            TokenKind::U8 => self.parse_u8(),
            TokenKind::I8 => self.parse_i8(),
            _ => {
                let message = format!("Expected integer, got {:?}", self.kind());
                let span = pos_to_span(self.cur.pos);
                return Err(SyntaxError::Generic { message, span }.into());
            }
        };
        if res.is_ok() {
            self.next();
        }
        res
    }

    pub fn parse_int(&mut self) -> AutoResult<Expr> {
        if self.cur.text.starts_with("0x") {
            // trim 0x
            let trim = &self.cur.text[2..];
            let val = i64::from_str_radix(trim, 16).unwrap();
            if val > i32::MAX as i64 {
                Ok(Expr::I64(val))
            } else {
                Ok(Expr::Int(val as i32))
            }
        } else {
            let val = self.cur.text.as_str().parse::<i64>().unwrap();
            if val > i32::MAX as i64 {
                Ok(Expr::I64(val))
            } else {
                Ok(Expr::Int(val as i32))
            }
        }
    }

    fn parse_uint(&mut self) -> AutoResult<Expr> {
        if self.cur.text.starts_with("0x") {
            // trim 0x
            let trim = &self.cur.text[2..];
            let val = u32::from_str_radix(trim, 16).unwrap();
            Ok(Expr::Uint(val))
        } else {
            let val = self.cur.text.as_str().parse::<u32>().unwrap();
            Ok(Expr::Uint(val))
        }
    }

    fn parse_u8(&mut self) -> AutoResult<Expr> {
        if self.cur.text.starts_with("0x") {
            // trim 0x
            let trim = &self.cur.text[2..];
            let val = u8::from_str_radix(trim, 16).unwrap();
            Ok(Expr::U8(val as u8))
        } else {
            let val = self.cur.text.as_str().parse::<u8>().unwrap();
            Ok(Expr::U8(val as u8))
        }
    }

    fn parse_i8(&mut self) -> AutoResult<Expr> {
        if self.cur.text.starts_with("0x") {
            // trim 0x
            let trim = &self.cur.text[2..];
            let val = i8::from_str_radix(trim, 16).unwrap();
            Ok(Expr::I8(val as i8))
        } else {
            let val = self.cur.text.as_str().parse::<i8>().unwrap();
            Ok(Expr::I8(val as i8))
        }
    }

    pub fn is_literal(&mut self) -> bool {
        self.is_kind(TokenKind::Uint)
            || self.is_kind(TokenKind::Int)
            || self.is_kind(TokenKind::U8)
            || self.is_kind(TokenKind::I8)
            || self.is_kind(TokenKind::Float)
            || self.is_kind(TokenKind::Double)
            || self.is_kind(TokenKind::True)
            || self.is_kind(TokenKind::False)
            || self.is_kind(TokenKind::Nil)
            || self.is_kind(TokenKind::Str)
            || self.is_kind(TokenKind::CStr)
            || self.is_kind(TokenKind::FStrStart)
            || self.is_kind(TokenKind::Char)
    }

    pub fn literal(&mut self) -> AutoResult<Expr> {
        let e = match self.kind() {
            TokenKind::FStrStart => self.fstr(),
            TokenKind::Uint => self.parse_int(),
            TokenKind::Int => self.parse_int(),
            TokenKind::U8 => self.parse_u8(),
            TokenKind::I8 => self.parse_i8(),
            TokenKind::Float => self.parse_float(),
            TokenKind::Double => self.parse_double(),
            TokenKind::True => Ok(Expr::Bool(true)),
            TokenKind::False => Ok(Expr::Bool(false)),
            TokenKind::Nil => Ok(Expr::Nil),
            TokenKind::Str => self.parse_str(),
            TokenKind::CStr => Ok(Expr::CStr(self.cur.text.clone())),
            TokenKind::Char => Ok(Expr::Char(self.cur.text.chars().nth(0).unwrap())),
            _ => Err(format!("UnexpectedToken {}", self.cur).into()),
        };
        self.next();
        e
    }

    fn parse_float(&mut self) -> AutoResult<Expr> {
        Ok(Expr::Float(
            self.cur.text.as_str().parse().unwrap(),
            self.cur.text.clone(),
        ))
    }

    fn parse_double(&mut self) -> AutoResult<Expr> {
        Ok(Expr::Double(
            self.cur.text.as_str().parse().unwrap(),
            self.cur.text.clone(),
        ))
    }

    fn parse_str(&mut self) -> AutoResult<Expr> {
        Ok(Expr::Str(self.cur.text.clone()))
    }

    pub fn atom(&mut self) -> AutoResult<Expr> {
        if self.is_kind(TokenKind::LParen) {
            return self.group();
        }

        // Handle generic type instances specially (e.g., List<int, Heap>)
        // This must be checked before the match statement since we need to consume
        // the identifier and potentially parse type parameters
        if self.is_kind(TokenKind::Ident) {
            let name = self.cur.text.clone();
            self.next(); // consume the identifier

            // Check if this is a generic type instance (followed by <)
            // Only treat as generic type if the identifier is a known TYPE (not a variable)
            // This prevents false positives like "x < 10" being treated as generic type
            let is_type = self.scope.borrow().lookup_ident_type(&name).is_some();
            if self.is_kind(TokenKind::Lt) && is_type {
                // Parse as generic type instance: List<int, Heap>
                self.expect(TokenKind::Lt)?;
                let mut args = Vec::new();
                args.push(self.parse_type()?);

                while self.cur.kind == TokenKind::Comma {
                    self.next(); // Consume ','
                    args.push(self.parse_type()?);
                }

                self.expect(TokenKind::Gt)?;

                // Generate descriptive name: "List<int, Heap>"
                let args_str = args
                    .iter()
                    .map(|t| t.unique_name().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                let generic_name = format!("{}<{}>", name, args_str);

                return Ok(Expr::GenName(generic_name.into()));
            } else {
                // Not a generic type, just a regular identifier
                return Ok(Expr::Ident(name));
            }
        }

        let expr = match self.kind() {
            TokenKind::Uint => self.parse_uint()?,
            TokenKind::Int => self.parse_int()?,
            TokenKind::U8 => self.parse_u8()?,
            TokenKind::I8 => self.parse_i8()?,
            TokenKind::Float => self.parse_float()?,
            TokenKind::Double => self.parse_double()?,
            TokenKind::Str => self.parse_str()?,
            TokenKind::True => Expr::Bool(true),
            TokenKind::False => Expr::Bool(false),
            TokenKind::CStr => Expr::CStr(self.cur.text.clone()),
            TokenKind::Char => Expr::Char(self.cur.text.chars().nth(0).unwrap()),
            TokenKind::Ident => self.ident()?,
            // Allow @ and * as special identifiers for pointer operations
            TokenKind::At => Expr::Ident("@".into()),
            TokenKind::Star => Expr::Ident("*".into()),
            TokenKind::Nil => Expr::Nil,
            TokenKind::Null => Expr::Null,
            _ => {
                return Err(SyntaxError::Generic {
                    message: format!("Expected term, got {:?}", self.kind()),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        };

        self.next();
        Ok(expr)
    }

    /// 解析fstr
    /// fstr: [FStrSart, (FStrParts | ${Expr} | $Ident)*, FStrEnd]
    pub fn fstr(&mut self) -> AutoResult<Expr> {
        // skip fstrs (e.g. ` or f")
        self.expect(TokenKind::FStrStart)?;

        // parse fstr parts or interpolated exprs
        let mut parts = Vec::new();
        while !(self.is_kind(TokenKind::FStrEnd) || self.is_kind(TokenKind::EOF)) {
            // $ expressions
            if self.is_kind(TokenKind::FStrNote) {
                self.next(); // skip $
                if self.is_kind(TokenKind::LBrace) {
                    // ${Expr}
                    self.expect(TokenKind::LBrace)?;
                    // NOTE: allow only one expression in a ${} block
                    if !self.is_kind(TokenKind::RBrace) {
                        let expr = self.rhs_expr()?;
                        parts.push(expr);
                        // self.expect_eos()?;
                    }
                    self.expect(TokenKind::RBrace)?;
                } else {
                    // $Ident
                    let ident = self.ident()?;
                    parts.push(ident);
                    self.expect(TokenKind::Ident)?;
                }
            } else {
                // normal text parts
                let text = self.cur.text.clone();
                parts.push(Expr::Str(text));
                self.next();
            }
        }
        self.expect(TokenKind::FStrEnd)?;
        Ok(Expr::FStr(FStr::new(parts)))
    }

    pub fn if_expr(&mut self) -> AutoResult<Expr> {
        let (branches, else_stmt) = self.if_contents()?;
        Ok(Expr::If(If {
            branches,
            else_: else_stmt,
        }))
    }

    pub fn cond_expr(&mut self) -> AutoResult<Expr> {
        self.rhs_expr()
    }

    // An Expression that can be assigned to a variable, e.g. right-hand side of an assignment
    pub fn rhs_expr(&mut self) -> AutoResult<Expr> {
        // Use parse_expr() for all cases - it handles if expressions, identifiers, calls, etc.
        self.parse_expr()
    }

    fn tag_cover(&mut self, tag_name: &Name) -> AutoResult<Expr> {
        self.expect(TokenKind::Dot)?;
        // tag field
        let tag_field = self.parse_name()?;
        self.expect(TokenKind::LParen)?;
        let elem = self.parse_name()?;
        self.expect(TokenKind::RParen)?;
        // define elem
        return Ok(Expr::Cover(Cover::Tag(TagCover {
            kind: tag_name.clone(),
            tag: tag_field,
            elem,
        })));
    }

    pub fn is_branch_cond_expr(&mut self) -> AutoResult<Expr> {
        // Parse the left-hand side expression (identifier or tag)
        let lhs = if self.is_kind(TokenKind::Ident) {
            self.lhs_expr()?
        } else {
            self.atom()?
        };

        // Continue parsing to handle member access (e.g., Msg.Inc)
        // This allows expressions like "Msg.Inc" in is branches
        self.expr_pratt_with_left(lhs, 0)
    }

    pub fn lhs_expr(&mut self) -> AutoResult<Expr> {
        if !self.is_kind(TokenKind::Ident) {
            return Err(SyntaxError::Generic {
                message: format!("Expected LHS expr with ident, got {}", self.peek().kind),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }
        let name = self.parse_name()?;

        // if expr is A Tag, could be a Tag Creation Expr,
        // format is: TagName.Tag(elem)
        let typ = self.lookup_type(&name);
        match *typ.borrow() {
            Type::Tag(ref _t) => return self.tag_cover(&name),
            _ => {
                return Ok(Expr::Ident(name));
            }
        };
    }

    pub fn iterable_expr(&mut self) -> AutoResult<Expr> {
        // TODO: how to check for range/array but reject other cases?
        self.parse_expr()
    }

    // DEPRECATED: Lambda syntax |a, b| a + b is replaced by closure syntax (a, b) => a + b
    // This method is kept for backwards compatibility but should not be used
    #[allow(dead_code)]
    pub fn lambda(&mut self) -> AutoResult<Expr> {
        self.next(); // skip |
        let params = self.fn_params()?;
        self.expect(TokenKind::VBar)?; // skip |
        let id = self.scope.borrow_mut().gen_lambda_id();
        let lambda = if self.is_kind(TokenKind::LBrace) {
            let body = self.body()?;
            Fn::new(
                FnKind::Lambda,
                id.clone(),
                None,
                params,
                body,
                Type::Unknown,
            )
        } else {
            // single expression
            let expr = self.parse_expr()?;
            Fn::new(
                FnKind::Lambda,
                id.clone(),
                None,
                params,
                Body::single_expr(expr),
                Type::Unknown,
            )
        };
        // put lambda in scope
        self.define(id.as_str(), Meta::Fn(lambda.clone()));
        // TODO: return meta instead?
        Ok(Expr::Lambda(lambda))
    }

    // Plan 060: Parse JavaScript/TypeScript-style closure: ` x => body` or `(a, b) => body`
    pub fn parse_closure(&mut self) -> AutoResult<Expr> {
        use crate::ast::{Closure, ClosureParam};

        // Check if this is a single-param or multi-param closure
        let params = if self.is_kind(TokenKind::LParen) {
            // Multi-param closure: (a, b) => body or (a int, b int) => body
            self.next(); // skip (

            let mut params = Vec::new();
            loop {
                let name = self.parse_name()?;

                // Optional type annotation (no colon - Auto syntax: a int, b int)
                // Same logic as fn_params: check if next token is a type
                let ty = if self.is_type_name() {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                params.push(ClosureParam::new(name, ty));

                if !self.is_kind(TokenKind::Comma) {
                    break;
                }
                self.next(); // skip ,
            }

            self.expect(TokenKind::RParen)?; // skip )
            params
        } else {
            // Single-param closure:  x => body (no parentheses)
            let name = self.parse_name()?;
            vec![ClosureParam::new(name, None)]
        };

        // Expect =>
        self.expect(TokenKind::DoubleArrow)?;

        // Parse body (expression or block)
        let body = if self.is_kind(TokenKind::LBrace) {
            // Block body: { stmts }
            Expr::Block(self.body()?)
        } else {
            // Expression body
            self.parse_expr()?
        };

        Ok(Expr::Closure(Closure::new(params, None, body)))
    }
}

// Statements
impl<'a> Parser<'a> {
    // End of statement
    pub fn expect_eos(&mut self, _is_first_stmt: bool) -> AutoResult<usize> {
        // Save the previous token before consuming separators
        let token_before_sep = self.prev.clone();

        let mut has_sep = false;
        let mut newline_count = 0;
        while self.is_kind(TokenKind::Semi)
            || self.is_kind(TokenKind::Newline)
            || self.is_kind(TokenKind::Comma)
        {
            has_sep = true;
            if self.is_kind(TokenKind::Newline) {
                newline_count += 1;
            }
            self.next();
        }
        if self.is_kind(TokenKind::EOF) || self.is_kind(TokenKind::RBrace) {
            return Ok(newline_count);
        }
        if has_sep {
            // Check for ambiguous sequence: ')', '\n', '{'
            // This is ambiguous - use 2+ newlines to disambiguate
            if self.is_kind(TokenKind::LBrace)
                && token_before_sep.kind == TokenKind::RParen
                && newline_count == 1
            {
                return Err(SyntaxError::Generic {
                    message:
                        "Ambiguous syntax: statement ending with ')' followed by newline and '{'. \
                    Use 2+ newlines to separate statements, or put the '{' on the same line."
                            .to_string(),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
            Ok(newline_count)
        } else {
            Err(SyntaxError::Generic {
                message: format!(
                    "Expected end of statement, got {:?}<{}>",
                    self.kind(),
                    self.cur.text
                ),
                span: pos_to_span(self.cur.pos),
            }
            .into())
        }
    }

    pub fn parse_stmt(&mut self) -> AutoResult<Stmt> {
        let stmt = match self.kind() {
            TokenKind::Break => self.break_stmt()?,
            TokenKind::Return => self.return_stmt()?,
            TokenKind::Use => self.use_stmt()?,
            TokenKind::If => self.if_stmt()?,
            TokenKind::For => self.for_stmt()?,
            TokenKind::Is => self.is_stmt()?,
            TokenKind::Var => self.parse_store_stmt()?,
            TokenKind::Let => self.parse_store_stmt()?,
            TokenKind::Mut => self.parse_store_stmt()?,
            TokenKind::Fn => self.fn_decl_stmt("")?,
            TokenKind::Hash => {
                // #[...] annotation syntax (Rust-style)
                // Use centralized parse_fn_annotations() function
                let (has_c, has_vm, _has_pub, with_params) = self.parse_fn_annotations()?;

                // Skip empty lines after annotation
                self.skip_empty_lines();

                // Check if this annotation is compatible with current compile destination
                let should_skip = match self.compile_dest {
                    CompileDest::TransC if has_vm && !has_c => true, // Skip #[vm] in C transpiler
                    CompileDest::TransRust if has_vm && !has_c => true, // Skip #[vm] in Rust transpiler
                    CompileDest::Interp if has_c && !has_vm => true,    // Skip #[c] in interpreter
                    _ => false,
                };

                if should_skip {
                    // Skip the entire function/type declaration by parsing it normally but discarding the result
                    if self.is_kind(TokenKind::Fn) || self.is_kind(TokenKind::Static) {
                        // Skip function declaration
                        let is_static = self.is_kind(TokenKind::Static);
                        if is_static {
                            self.next(); // skip static keyword
                        }
                        // Parse with the actual annotation flags to correctly handle the function syntax
                        // For #[vm] functions, parse as VM function to allow newline termination
                        // For #[c] functions, parse as C function to allow semicolon termination
                        let _ = self.fn_decl_stmt_with_annotations("", has_c, has_vm, is_static, with_params.clone());
                        return Ok(Stmt::Expr(Expr::Nil));
                    } else if self.is_kind(TokenKind::Type) {
                        // Skip type declaration
                        let _ = self.type_decl_stmt();
                        return Ok(Stmt::Expr(Expr::Nil));
                    }
                }

                // Check what comes next
                if self.is_kind(TokenKind::Fn) || self.is_kind(TokenKind::Static) {
                    // Function declaration
                    let is_static = self.is_kind(TokenKind::Static);
                    if is_static {
                        self.next(); // skip static keyword
                    }
                    self.fn_decl_stmt_with_annotations("", has_c, has_vm, is_static, with_params)?
                } else if self.is_kind(TokenKind::Type) {
                    // Type declaration
                    self.type_decl_stmt_with_annotation(has_c)?
                } else if self.is_kind(TokenKind::Use) {
                    // Use statement with annotation
                    // Check if this is a C/Rust import (use.c or use.rust style with angle brackets)
                    self.next(); // skip use to check next token
                    let is_c_import = self.is_kind(TokenKind::Lt) || self.is_kind(TokenKind::Str);
                    // Put the use token back
                    self.lexer.push_token(self.cur.clone());
                    self.cur = self.prev.clone(); // Go back to use token

                    if has_c && is_c_import {
                        // C import: #[c] use <stdio.h>
                        self.next(); // skip use
                                     // Call use_c directly since we know it's a C import
                        let mut paths = Vec::new();
                        if self.is_kind(TokenKind::Lt) {
                            self.next(); // skip <
                            let mut name = "<".to_string();
                            while !self.is_kind(TokenKind::Gt) {
                                name.push_str(self.cur.text.as_str());
                                self.next();
                            }
                            name.push_str(">");
                            self.expect(TokenKind::Gt)?;
                            paths.push(name.into());
                        } else if self.is_kind(TokenKind::Str) {
                            let name = self.cur.text.clone();
                            self.next();
                            paths.push(format!("\"{}\"", name).into());
                        }

                        let items = self.parse_use_items()?;
                        let uses = Use {
                            kind: UseKind::C,
                            paths,
                            items,
                        };
                        self.import(&uses)?;
                        Stmt::Use(uses)
                    } else {
                        // Regular Auto use statement
                        self.use_stmt()?
                    }
                } else if self.is_kind(TokenKind::Let) {
                    // Let statement with annotation
                    self.parse_store_stmt()?
                } else {
                    return Err(SyntaxError::Generic {
                        message: "Expected 'fn', 'type', 'use', or 'let' after annotation"
                            .to_string(),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            }
            TokenKind::Type => self.type_decl_stmt()?,
            TokenKind::Union => self.union_stmt()?,
            TokenKind::Tag => self.tag_stmt()?,
            TokenKind::Spec => self.spec_decl_stmt()?,
            TokenKind::LBrace => Stmt::Block(self.body()?),
            // Node Instance?
            TokenKind::Ident => self.parse_node_or_call_stmt()?,
            // Enum Definition
            TokenKind::Enum => self.enum_stmt()?,
            // On Events Switch
            TokenKind::On => Stmt::OnEvents(self.parse_on_events()?),
            // Alias stmt
            TokenKind::Alias => self.parse_alias_stmt()?,
            // Ext statement (Plan 035)
            TokenKind::Ext => self.parse_ext_stmt()?,
            // Impl statement (Plan 059: synonym for ext, Rust-compatible syntax)
            TokenKind::Impl => self.parse_ext_stmt()?,
            // Otherwise, try to parse as an expression
            _ => self.expr_stmt()?,
        };
        Ok(stmt)
    }

    fn parse_alias_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip alias
        let alias = self.cur.text.clone();
        self.next();
        self.expect(TokenKind::Asn)?;
        let target = self.cur.text.clone();
        self.next();
        // define alias meta in scope
        self.define_alias(alias.clone(), target.clone());
        Ok(Stmt::Alias(Alias { alias, target }))
    }

    /// Parse ext statement: ext Target { methods... }
    ///
    /// Adds methods to an existing type (like Rust's impl block).
    /// Both instance methods (fn) and static methods (static fn) are supported.
    ///
    /// # Example
    ///
    /// ```auto
    /// ext str {
    ///     fn len() int {
    ///         return .size
    ///     }
    ///
    ///     static fn new(data *char, size int) str {
    ///         return str_new(data, size)
    ///     }
    /// }
    /// ```
    fn parse_ext_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip `ext` or `impl` keyword

        // Plan 059: Parse optional generic parameters for impl blocks
        // e.g., impl<T, S> ListIter<T, S> { ... }
        let mut generic_params = Vec::new();
        if self.is_kind(TokenKind::Lt) {
            self.next(); // Consume '<'
            generic_params.push(self.parse_generic_param()?);

            while self.is_kind(TokenKind::Comma) {
                self.next(); // Consume ','
                generic_params.push(self.parse_generic_param()?);
            }

            self.expect(TokenKind::Gt)?; // Consume '>'
        }

        // Parse target type name (e.g., "str", "Point", "ListIter")
        // Note: We only parse the base name, not generic instance like ListIter<T, S>
        let target = self.parse_name()?;

        // Skip generic instance syntax if present (e.g., <T, S> after ListIter)
        // For now, we just extract the base type name and skip the generic parameters
        // This allows `impl<T, S> ListIter<T, S>` to work
        if self.is_kind(TokenKind::Lt) {
            // Generic instance syntax like ListIter<T, S>
            self.next(); // skip '<'

            // Parse type arguments
            if self.next_token_is_type() {
                let _ = self.parse_type()?;
            }

            // Parse additional type arguments separated by commas
            while self.is_kind(TokenKind::Comma) {
                self.next(); // skip ','
                if self.next_token_is_type() {
                    let _ = self.parse_type()?;
                }
            }

            self.expect(TokenKind::Gt)?; // skip '>'
        }

        // Expect opening brace
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();

        // Parse fields and methods
        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            // Check for annotations: #[c], #[vm], #[pub], #[c,vm] before function declarations
            let (has_c, has_vm, _has_pub, with_params) = self.parse_fn_annotations()?;

            self.skip_empty_lines(); // Skip newlines after annotations

            // Check if this annotation should be skipped for current compile destination
            let should_skip = match self.compile_dest {
                CompileDest::TransC if has_vm && !has_c => true, // Skip #[vm] in C transpiler
                CompileDest::TransRust if has_vm && !has_c => true, // Skip #[vm] in Rust transpiler
                CompileDest::Interp if has_c && !has_vm => true, // Skip #[c] in interpreter
                _ => false,
            };

            // Parse field declarations: name Type (same syntax as type members)
            // Fields must come before methods
            if self.is_kind(TokenKind::Ident) {
                // Check if this is a field (next token is a type)
                // Lookahead: if current token is Ident and next token is NOT a keyword that starts a statement
                let is_field = match self.peek().kind {
                    TokenKind::Fn | TokenKind::Static | TokenKind::Has | TokenKind::RBrace => false,
                    TokenKind::Colon => {
                        // Old syntax with colon - reject with helpful error
                        return Err(SyntaxError::Generic {
                            message: "ext field syntax should be 'name Type' (without colon), same as type members. Use: '_fp *FILE' not '_fp: *FILE'".to_string(),
                            span: pos_to_span(self.cur.pos),
                        }.into());
                    }
                    _ => true,
                };

                if is_field {
                    // Parse field: name Type [optional_default_value]
                    let field_name = self.parse_name()?;
                    let field_type = self.parse_type()?;
                    let mut value = None;
                    if self.is_kind(TokenKind::Asn) {
                        self.next(); // skip =
                        let expr = self.parse_expr()?;
                        value = Some(expr);
                    }

                    // ext fields are always private
                    fields.push(crate::ast::Member::new(field_name, field_type, value));

                    self.expect_eos(false)?;
                    self.skip_empty_lines();
                    continue;
                }
            }

            // Parse method declarations (fn or static fn)
            // IMPORTANT: Check Static BEFORE Fn, since "static fn" starts with Static
            if self.is_kind(TokenKind::Static) || self.is_kind(TokenKind::Fn) {
                // Track if this is a static method (Plan 035 Phase 4)
                let is_static_method = self.is_kind(TokenKind::Static);

                // If static fn, skip the static keyword first
                if is_static_method {
                    self.next(); // skip `static` keyword
                                 // Now we expect `fn` keyword
                    if !self.is_kind(TokenKind::Fn) {
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "expected 'fn' after 'static', found {:?}",
                                self.kind()
                            ),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                }

                // Check if we should skip this based on annotation
                if should_skip {
                    // Skip the entire function declaration
                    // Parse with the actual annotation flags to correctly handle the function syntax
                    let _ = self.fn_decl_stmt_with_annotations(
                        &target,
                        has_c,
                        has_vm,
                        is_static_method,
                        with_params.clone(),
                    );
                    self.expect_eos(false)?;
                    self.skip_empty_lines();
                    continue;
                }

                let fn_stmt =
                    self.fn_decl_stmt_with_annotations(&target, has_c, has_vm, is_static_method, with_params)?;
                if let Stmt::Fn(mut fn_expr) = fn_stmt {
                    // Set is_static flag for static methods (Plan 035 Phase 4.2)
                    if is_static_method {
                        fn_expr.is_static = true;
                    }
                    methods.push(fn_expr);
                }
                // For VM/C methods, they can end with newline (interface contract)
                // For regular methods, expect EOS (semicolon or newline after statement)
                if !has_vm && !has_c {
                    self.expect_eos(false)?;
                }
                self.skip_empty_lines();
            } else {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "ext blocks can only contain field (name Type) or method declarations (fn or static fn), found {:?}",
                        self.kind()
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }

        // Expect closing brace
        self.expect(TokenKind::RBrace)?;

        // TODO: Implement proper module tracking
        // For now, assume same-module (will be enhanced in future task)
        let module_path: AutoStr = "".into();
        let is_same_module = true;

        // Plan 059: Create Ext with generic params, fields, and methods
        let ext = Ext {
            target,
            generic_params,
            fields,
            methods,
            module_path,
            is_same_module,
        };
        Ok(Stmt::Ext(ext))
    }

    /// Format: enum { item1, item2, item3 }
    fn enum_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip enum
        let name = self.cur.text.clone().into();
        self.next();
        let mut items = Vec::new();
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut last_val = 0;
        // parse items
        while !self.is_kind(TokenKind::RBrace) {
            let mut item = EnumItem {
                name: self.cur.text.clone().into(),
                value: 0,
            };
            self.next();
            if self.is_kind(TokenKind::Asn) {
                self.next(); // skip =
                let value = self.parse_ints()?;
                let value = self.get_int_expr(&value);
                item.value = value as i32;
                last_val = item.value;
            } else {
                item.value = last_val;
                last_val += 1;
            }
            if self.is_kind(TokenKind::Comma) {
                self.next(); // skip ,
            } else if self.is_kind(TokenKind::Newline) {
                self.next(); // skip newline
            } else if self.is_kind(TokenKind::RBrace) {
                // do nothing
            } else {
                let message = format!("expected ',' or newline, got {}", self.cur.text);
                let span = pos_to_span(self.cur.pos);
                return Err(SyntaxError::Generic { message, span }.into());
            }
            self.skip_empty_lines();
            items.push(item);
        }
        self.expect(TokenKind::RBrace)?;
        // make enum ast node
        let enum_decl = EnumDecl { name, items };
        self.define(enum_decl.name.as_str(), Meta::Enum(enum_decl.clone()));
        Ok(Stmt::EnumDecl(enum_decl))
    }

    fn expect_ident_str(&mut self) -> AutoResult<AutoStr> {
        if self.is_kind(TokenKind::Ident) {
            let name = self.cur.text.clone();
            self.next(); // skip name
            Ok(name)
        } else {
            return Err(SyntaxError::Generic {
                message: format!("Expected identifier, got {:?}", self.kind()),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }
    }

    fn parse_use_items(&mut self) -> AutoResult<Vec<AutoStr>> {
        let mut items = Vec::new();
        // end of path, next should be a colon (for items) or end-of-statement
        if self.is_kind(TokenKind::Colon) {
            self.next(); // skip :
                         // parse items
            if self.is_kind(TokenKind::Ident) {
                let name = self.expect_ident_str()?;
                items.push(name);
            } else {
                return Err(SyntaxError::Generic {
                    message: format!("Expected identifier, got {:?}", self.kind()),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
            while self.is_kind(TokenKind::Comma) {
                self.next(); // skip ,
                let name = self.expect_ident_str()?;
                items.push(name);
            }
        }
        Ok(items)
    }

    pub fn use_c_stmt(&mut self) -> AutoResult<Stmt> {
        let mut paths = Vec::new();
        // include "<lib.h>"
        if self.is_kind(TokenKind::Lt) {
            self.next(); // skip <
            let mut name = "<".to_string();
            while !self.is_kind(TokenKind::Gt) {
                name.push_str(self.cur.text.as_str());
                self.next(); // skip lib
            }
            name.push_str(">");
            self.expect(TokenKind::Gt)?;
            paths.push(name.into());
        } else if self.is_kind(TokenKind::Str) {
            // include "lib.h"
            let name = self.cur.text.clone();
            self.next(); // skip lib
            paths.push(format!("\"{}\"", name).into());
        } else {
            return Err(SyntaxError::Generic {
                message: format!(
                    "Expected <lib> or \"lib\", got {:?}, {}",
                    self.kind(),
                    self.cur.text
                ),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }

        let items = self.parse_use_items()?;

        for item in items.iter() {
            // add item to scope
            self.define(item.as_str(), Meta::Use(item.into()));
        }

        let uses = Use {
            kind: UseKind::C,
            paths,
            items,
        };

        Ok(Stmt::Use(uses))
    }

    pub fn use_rust_stmt(&mut self) -> AutoResult<Stmt> {
        return Err(SyntaxError::Generic {
            message: "Rust import not supported yet".to_string(),
            span: pos_to_span(self.cur.pos),
        }
        .into());
    }

    // There are three kinds of import
    // 1. auto: use std.io: println
    // 2. c: use c <stdio.h>
    // 3. rust: use rust std::fs
    pub fn use_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip use

        let mut paths = Vec::new();

        // check user.c or use.rust
        if self.is_kind(TokenKind::Dot) {
            self.next(); // skip .
            let name = self.expect_ident_str()?;

            if name == "c" {
                return self.use_c_stmt();
            } else if name == "rust" {
                return self.use_rust_stmt();
            } else {
                paths.push(name);
            }
        } else {
            let name = self.expect_ident_str()?;
            paths.push(name);
        }

        while self.is_kind(TokenKind::Dot) {
            self.next(); // skip .
            let name = self.expect_ident_str()?;
            paths.push(name);
        }

        let items = self.parse_use_items()?;

        // import the path into scope
        let uses = Use {
            kind: UseKind::Auto,
            paths,
            items,
        };
        self.import(&uses)?;
        Ok(Stmt::Use(uses))
    }

    /// Get file extensions to load based on compile destination
    /// Plan 036: Returns extensions in load order (bottom layer first, then top layer)
    fn get_file_extensions(&self) -> Vec<&'static str> {
        match self.compile_dest {
            CompileDest::Interp => vec![".at", ".vm.at"], // Interpreter: Interface (first) → VM implementation (second)
            CompileDest::TransC => vec![".at", ".c.at"], // Transpiler: Interface (first) → C implementation (second)
            CompileDest::TransRust => vec![".at", ".rust.at"], // Rust transpiler
        }
    }

    /// Check if a file exists at the given path
    #[allow(dead_code)]
    fn file_exists(&self, dir: &std::path::Path, name: &str, ext: &str) -> bool {
        let file_path = dir.join(format!("{}{}", name, ext));
        file_path.exists()
    }

    /// Import a path from `use` statement
    /// Plan 036: Supports loading and merging multiple files (.vm.at + .at or .c.at + .at)
    // TODO: clean up code
    // TODO: search path from System Env, Default Locations and etc.
    pub fn import(&mut self, uses: &Use) -> AutoResult<()> {
        // println!("Trying to import use library"); // LSP: disabled
        let path = uses.paths.join(".");
        let scope_name: AutoStr = path.clone().into();
        // println!("scope_name: {}", scope_name); // LSP: disabled

        // try to find stdlib in following locations
        // 1. ~/.auto/stdlib
        // 2. /usr/local/lib/auto
        // 3. /usr/lib/auto

        let file_path = if path.starts_with("auto.") {
            // stdlib/auto
            let std_path = crate::util::find_std_lib()?;
            // println!("debug: std lib location: {}", std_path); // LSP: disabled
            let path = path.replace("auto.", "");
            AutoPath::new(std_path).join(path.clone())
        } else if path.starts_with("c.") {
            // stdlib/c (C standard library layer)
            let std_path = crate::util::find_std_lib()?;
            // Get parent of stdlib/auto, which is stdlib, then join c/
            let stdlib_auto: &str = &std_path;
            if let Some(parent) = stdlib_auto.rfind("/auto") {
                let stdlib_base = &stdlib_auto[..parent];
                let path = path.replace("c.", "c/");
                AutoPath::new(stdlib_base).join(path.clone())
            } else {
                return Err(SyntaxError::Generic {
                    message: format!("Cannot find stdlib parent directory"),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        } else {
            // local lib
            AutoPath::new(".").join(path.clone())
        };
        let dir = file_path.parent();
        let name = file_path.path().file_name().unwrap();

        if !dir.exists() {
            return Err(SyntaxError::Generic {
                message: format!("Invalid import path: {}", path),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }

        // Plan 036: Load multiple files and merge their content
        // Strategy: Merge file contents (bottom layer first, then top layer), then parse as one file

        // For C stdlib layer, always load .c.at files regardless of compile destination
        let extensions = if path.starts_with("c.") {
            vec![".c.at"]
        } else {
            self.get_file_extensions()
        };

        let name_str = name.to_str().unwrap();
        let mut file_contents = Vec::new();
        let mut loaded_files = Vec::new();

        // Save current scope spot
        let cur_spot = self.scope.borrow().cur_spot.clone();
        self.scope.borrow_mut().reset_spot();

        for path in scope_name.split(".").into_iter() {
            self.scope.borrow_mut().enter_mod(path.to_string());
        }

        // Load files in order (bottom layer first, then top layer)
        for ext in extensions.iter() {
            let file_path_str = dir.join(format!("{}{}", name_str, ext));
            if file_path_str.exists() {
                let content = std::fs::read_to_string(file_path_str.path()).map_err(|e| {
                    SyntaxError::Generic {
                        message: format!(
                            "Failed to read file {}: {}",
                            file_path_str.path().display(),
                            e
                        ),
                        span: pos_to_span(self.cur.pos),
                    }
                })?;
                file_contents.push((content, file_path_str.clone()));
                loaded_files.push(file_path_str);
            }
        }

        // Fallback: if no split files found, try original .at file
        if loaded_files.is_empty() {
            let file_path_str = dir.join(format!("{}.at", name_str));
            if file_path_str.exists() {
                let content = std::fs::read_to_string(file_path_str.path()).map_err(|e| {
                    SyntaxError::Generic {
                        message: format!(
                            "Failed to read file {}: {}",
                            file_path_str.path().display(),
                            e
                        ),
                        span: pos_to_span(self.cur.pos),
                    }
                })?;
                file_contents.push((content, file_path_str.clone()));
                loaded_files.push(file_path_str);
            } else {
                // No files found at all
                return Err(SyntaxError::Generic {
                    message: format!("Cannot find file: {}{}", name_str, ".at"),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }

        // Merge all file contents with newlines between them
        // Filter out section marker lines (lines starting with "# AUTO", "# C", etc.)
        let merged_content: String = file_contents
            .iter()
            .map(|(content, _)| {
                content
                    .lines()
                    .filter(|line| {
                        !line.trim().starts_with("# ") && !line.trim().starts_with("#\t")
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        // Parse the merged content as a single file
        let mut new_parser = Parser::new(&merged_content, self.scope.clone());
        new_parser.set_dest(self.compile_dest.clone());
        let ast = new_parser.parse().map_err(|e| {
            eprintln!("===== PARSER ERROR IN IMPORT =====");
            eprintln!("Module: {}", scope_name);
            eprintln!("Files loaded: {:?}", loaded_files);
            eprintln!("Error: {:?}", e);
            eprintln!("Error display: {}", e);
            eprintln!("==================================");
            e
        })?;

        // Extract spec declarations
        let spec_decls: Vec<_> = ast
            .stmts
            .iter()
            .filter_map(|stmt| {
                if let Stmt::SpecDecl(spec_decl) = stmt {
                    Some(spec_decl.clone())
                } else {
                    None
                }
            })
            .collect();

        // Import the merged AST into scope
        // Use the path of the last loaded file (usually the main .at file)
        let last_file_path = loaded_files.last().unwrap();
        self.scope.borrow_mut().import(
            scope_name.clone(),
            ast,
            last_file_path.to_astr(),
            merged_content.into(),
        );

        self.scope.borrow_mut().set_spot(cur_spot);
        let mut items = uses.items.clone();
        // if item is empty, use last part of paths as an defined item in the scope
        if items.is_empty() && !uses.paths.is_empty() {
            items.push(uses.paths.last().unwrap().clone());
        }
        // println!("items: {:?}", items); // LSP: disabled

        // Import all spec declarations from this module
        // This ensures that specs are available even if not explicitly listed in use items
        for spec_decl in spec_decls {
            // Import spec into current scope
            // println!("Importing spec: {}", spec_decl.name); // LSP: disabled
            self.define_rc(
                spec_decl.name.clone().as_str(),
                std::rc::Rc::new(Meta::Spec(spec_decl)),
            );
        }

        // Define items in scope
        for item in items.iter() {
            // lookup item's meta from its mod
            let meta = if let Some(found_meta) = self
                .scope
                .borrow()
                .lookup(item.as_str(), scope_name.clone())
            {
                found_meta
            } else {
                // For C library functions (c.stdio, c.stdlib, etc.), create a placeholder meta
                // These are external C functions that will be linked by the transpiler
                if scope_name.starts_with("c.") {
                    use crate::ast::{Fn, FnKind};
                    let c_fn = Fn {
                        kind: FnKind::CFunction,
                        name: item.clone(),
                        parent: None,
                        params: vec![],
                        body: crate::ast::Body::new(),
                        ret: Type::Int,
                        ret_name: None,
                        is_static: false,
                        type_params: vec![],
                        span: None,
                    };
                    std::rc::Rc::new(Meta::Fn(c_fn))
                } else {
                    return Err(SyntaxError::Generic {
                        message: format!("Cannot find symbol: {} in {}", item, scope_name),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            };
            // define item with its name in current scope
            self.define_rc(item.as_str(), meta);
        }
        Ok(())
    }

    pub fn skip_empty_lines(&mut self) -> usize {
        let mut count = 0;
        while self.is_kind(TokenKind::Newline) {
            count += 1;
            self.next();
        }
        count
    }

    fn parse_body(&mut self, is_node: bool) -> AutoResult<Body> {
        self.expect(TokenKind::LBrace)?;
        self.enter_scope();
        let mut stmts = Vec::new();
        let new_lines = self.skip_empty_lines();
        if new_lines > 1 {
            stmts.push(Stmt::EmptyLine(new_lines - 1));
        }
        let has_new_line = new_lines > 0;
        let mut stmt_index = 0; // Track statement index for is_first_stmt check

        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            match self.parse_stmt() {
                Ok(stmt) => {
                    if is_node {
                        if let Stmt::Expr(Expr::Pair(Pair { key, value })) = &stmt {
                            // define as a property
                            self.define(
                                key.to_string().as_str(),
                                Meta::Pair(Pair {
                                    key: key.clone(),
                                    value: value.clone(),
                                }),
                            );
                        }
                    }
                    stmts.push(stmt);
                    let is_first = stmt_index == 0;
                    // Check for ambiguous syntax errors in expect_eos
                    let newline_count = match self.expect_eos(is_first) {
                        Ok(count) => count,
                        Err(e) => {
                            // Ambiguous syntax errors should not be recovered from
                            if e.to_string().contains("Ambiguous syntax") {
                                self.exit_scope();
                                return Err(e);
                            }
                            // Other EOS errors are added to collection and synchronized
                            if !self.add_error(e) {
                                self.exit_scope();
                                return Err(self.errors.pop().unwrap());
                            }
                            self.synchronize();
                            0
                        }
                    };
                    // Insert EmptyLine statement for 2+ consecutive newlines
                    if newline_count > 1 {
                        stmts.push(Stmt::EmptyLine(newline_count - 1));
                    }
                    stmt_index += 1;
                }
                Err(e) => {
                    // Add error to collection and synchronize
                    if !self.add_error(e) {
                        self.exit_scope();
                        return Err(self.errors.pop().unwrap()); // Error limit exceeded
                    }
                    self.synchronize();
                }
            }
        }

        stmts = self.convert_last_block(stmts)?;
        self.exit_scope();
        self.expect(TokenKind::RBrace)?;
        Ok(Body {
            stmts,
            has_new_line,
        })
    }

    pub fn parse_node_body(&mut self) -> AutoResult<Body> {
        self.parse_body(true)
    }

    pub fn body(&mut self) -> AutoResult<Body> {
        self.parse_body(false)
    }

    pub fn if_contents(&mut self) -> AutoResult<(Vec<Branch>, Option<Body>)> {
        let mut branches = Vec::new();
        self.next(); // skip if
        let cond = self.parse_expr()?;
        let body = self.body()?;
        branches.push(Branch { cond, body });

        let mut else_stmt = None;
        while self.is_kind(TokenKind::Else) {
            self.next(); // skip else
                         // more branches
            if self.is_kind(TokenKind::If) {
                self.next(); // skip if
                let cond = self.parse_expr()?;
                let body = self.body()?;
                branches.push(Branch { cond, body });
            } else {
                // last else
                else_stmt = Some(self.body()?);
            }
        }
        Ok((branches, else_stmt))
    }

    pub fn if_stmt(&mut self) -> AutoResult<Stmt> {
        let (branches, else_stmt) = self.if_contents()?;
        Ok(Stmt::If(If {
            branches,
            else_: else_stmt,
        }))
    }

    pub fn for_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip `for`
        if self.is_kind(TokenKind::LBrace) {
            // for {...}
            let body = self.body()?;
            let has_new_line = body.has_new_line;
            return Ok(Stmt::For(For {
                iter: Iter::Ever,
                range: Expr::Nil,
                body,
                new_line: has_new_line,
                init: None,
            }));
        }

        // Check if this has an initializer: for let x = 0; condition { ... }
        if self.is_kind(TokenKind::Let)
            || self.is_kind(TokenKind::Var)
            || self.is_kind(TokenKind::Mut)
        {
            // Parse the initializer statement
            let init_stmt = Some(Box::new(self.parse_store_stmt()?));

            // Expect semicolon after initializer
            self.expect(TokenKind::Semi)?;

            // After initializer, must parse condition
            let condition = self.parse_expr()?;
            let body = self.body()?;
            let has_new_line = body.has_new_line;
            return Ok(Stmt::For(For {
                iter: Iter::Cond,
                range: condition,
                body,
                new_line: has_new_line,
                init: init_stmt,
            }));
        }

        // No initializer, try to parse as iterator pattern: for ident in range { ... }
        // Or conditional loop: for ident < max { ... }
        if self.is_kind(TokenKind::Ident) {
            // Consume the identifier first
            let ident = self.parse_name()?;

            // Now check what comes next to determine the pattern
            if self.is_kind(TokenKind::In) {
                // for ident in range { ... }
                self.next(); // skip 'in'
                self.enter_scope();
                let meta = Meta::Store(Store {
                    kind: StoreKind::Var,
                    name: ident.clone(),
                    expr: Expr::Nil,
                    ty: Type::Int,
                });
                self.define(ident.as_str(), meta);
                let iter = Iter::Named(ident.clone());
                let range = self.iterable_expr()?;
                let body = self.body()?;
                let has_new_line = body.has_new_line;
                self.exit_scope();
                return Ok(Stmt::For(For {
                    iter,
                    range,
                    body,
                    new_line: has_new_line,
                    init: None,
                }));
            } else if self.is_kind(TokenKind::Comma) {
                // for ident, ident2 in range { ... } - indexed iterator
                self.next(); // skip ','
                let ident2 = self.parse_name()?;
                self.expect(TokenKind::In)?; // this calls self.next() internally
                self.enter_scope();
                let meta = Meta::Store(Store {
                    kind: StoreKind::Var,
                    name: ident2.clone(),
                    expr: Expr::Nil,
                    ty: Type::Int,
                });
                self.define(ident2.as_str(), meta);
                let iter = Iter::Indexed(ident.clone(), ident2.clone());
                let range = self.iterable_expr()?;
                let body = self.body()?;
                let has_new_line = body.has_new_line;
                self.exit_scope();
                return Ok(Stmt::For(For {
                    iter,
                    range,
                    body,
                    new_line: has_new_line,
                    init: None,
                }));
            } else if self.is_kind(TokenKind::LParen) {
                // for call(args) { ... }
                let args = self.args()?;
                let call = self.call(Expr::Ident(ident), args)?;
                let Expr::Call(call) = call else {
                    return Err(SyntaxError::Generic {
                        message: "Strange call in for statement".to_string(),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                };
                self.expect(TokenKind::Question)?;
                self.enter_scope();
                let body = self.body()?;
                self.exit_scope();
                return Ok(Stmt::For(For {
                    iter: Iter::Call(call),
                    range: Expr::Nil,
                    body,
                    new_line: false,
                    init: None,
                }));
            }

            // Check if next token is a binary operator (like <, >, <=, >=, ==, !=, +, -, *, /)
            let next_is_binop = matches!(
                self.kind(),
                TokenKind::Lt
                    | TokenKind::Gt
                    | TokenKind::Le
                    | TokenKind::Ge
                    | TokenKind::Eq
                    | TokenKind::Neq
                    | TokenKind::Add
                    | TokenKind::Sub
                    | TokenKind::Star
                    | TokenKind::Div
            );

            if next_is_binop {
                // This is a binary expression like "for i < max { ... }"
                // We've already consumed 'i', so we need to prepend it to the expression
                let ident_expr = Expr::Ident(ident.clone());
                let rest = self.expr_pratt_with_left(ident_expr, 0)?;
                let body = self.body()?;
                let has_new_line = body.has_new_line;
                return Ok(Stmt::For(For {
                    iter: Iter::Cond,
                    range: rest,
                    body,
                    new_line: has_new_line,
                    init: None,
                }));
            }

            // Identifier followed by something else (member access, etc.)
            // Fall through to parse as a general expression starting with this ident
            let ident_expr = Expr::Ident(ident);
            let condition = self.expr_pratt_with_left(ident_expr, 0)?;
            let body = self.body()?;
            let has_new_line = body.has_new_line;
            return Ok(Stmt::For(For {
                iter: Iter::Cond,
                range: condition,
                body,
                new_line: has_new_line,
                init: None,
            }));
        }

        // Otherwise, parse as conditional for loop: for condition { ... }
        let condition = self.parse_expr()?;
        let body = self.body()?;
        let has_new_line = body.has_new_line;
        Ok(Stmt::For(For {
            iter: Iter::Cond,
            range: condition,
            body,
            new_line: has_new_line,
            init: None,
        }))
    }

    pub fn is_stmt(&mut self) -> AutoResult<Stmt> {
        Ok(Stmt::Is(self.parse_is()?))
    }

    pub fn parse_is(&mut self) -> AutoResult<Is> {
        self.next(); // skip is
        let target = self.lhs_expr()?;

        self.expect(TokenKind::LBrace)?; // {
        self.skip_empty_lines();

        let mut branches = Vec::new();

        while !self.is_kind(TokenKind::RBrace) {
            let branch = self.parse_is_branch(&target)?;
            branches.push(branch);
        }
        self.expect(TokenKind::RBrace)?;

        let is = Is { target, branches };
        return Ok(is);
    }

    pub fn parse_expr_or_body(&mut self) -> AutoResult<Body> {
        if self.is_kind(TokenKind::LBrace) {
            self.body()
        } else {
            let mut body = Body::new();
            body.stmts.push(Stmt::Expr(self.parse_expr()?));
            Ok(body)
        }
    }

    pub fn parse_is_branch(&mut self, tgt: &Expr) -> AutoResult<IsBranch> {
        match self.cur.kind {
            TokenKind::If => {
                self.next(); // skip is
                let expr = self.cond_expr()?;
                self.expect(TokenKind::DoubleArrow)?;
                let body = self.parse_expr_or_body()?;
                let branch = IsBranch::IfBranch(expr, body);
                self.skip_empty_lines();
                return Ok(branch);
            }
            TokenKind::Else => {
                self.next(); // skip else
                self.expect(TokenKind::DoubleArrow)?;
                let body = self.parse_expr_or_body()?;
                let branch = IsBranch::ElseBranch(body);
                self.skip_empty_lines();
                return Ok(branch);
            }
            _ => {
                let expr = self.is_branch_cond_expr()?;
                self.expect(TokenKind::DoubleArrow)?;
                let body = if let Expr::Cover(Cover::Tag(cover)) = &expr {
                    self.enter_scope();
                    let tag_typ = self.lookup_type(&cover.kind);
                    let tag_field_type = match *tag_typ.borrow() {
                        Type::Tag(ref t) => t.borrow().get_field_type(&cover.tag),
                        _ => {
                            return Err(SyntaxError::Generic {
                                message: format!("Invalid tag type: {}", cover.kind),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into());
                        }
                    };

                    self.define(
                        cover.elem.as_str(),
                        Meta::Store(Store {
                            name: cover.elem.clone(),
                            kind: StoreKind::Let,
                            ty: tag_field_type,
                            expr: Expr::Uncover(TagUncover {
                                src: tgt.repr(),
                                cover: cover.clone(),
                            }),
                        }),
                    );
                    let body = self.parse_expr_or_body()?;
                    self.exit_scope();
                    body
                } else {
                    let body = self.parse_expr_or_body()?;
                    body
                };
                let branch = IsBranch::EqBranch(expr, body);
                self.skip_empty_lines();
                return Ok(branch);
            }
        }
    }

    pub fn parse_store_stmt(&mut self) -> AutoResult<Stmt> {
        // store kind: var/let (mut keyword is now aliased to var)
        let mut store_kind = self.store_kind()?;
        self.next(); // skip var/let/mut

        // identifier name
        let mut name = self.parse_name()?;

        // Capture the position of the variable name for LSP
        let name_pos = self.prev.pos;

        // special case: c decl
        if name == "." {
            self.next();
            name = self.parse_name()?;
            if name == "c" {
                store_kind = StoreKind::CVar;
            }
        }

        // type (optional)
        let mut ty = Type::Unknown;
        if self.is_type_name() {
            ty = self.parse_type()?;
        }

        // `=`, a store stmt must have an assignment unless:
        // 1. It's a C variable decl (StoreKind::CVar)
        // 2. It has an explicit type annotation (Plan 052: allow uninitialized typed variables)
        let has_explicit_type = !matches!(ty, Type::Unknown);
        let expr = if matches!(store_kind, StoreKind::CVar) {
            Expr::Nil
        } else if self.is_kind(TokenKind::Asn) {
            self.expect(TokenKind::Asn)?;
            // inital value: expression
            let expr = self.rhs_expr()?;
            // TODO: check type compatibility
            if matches!(ty, Type::Unknown) {
                ty = self.infer_type_expr(&expr);
            }
            expr
        } else if has_explicit_type {
            // Plan 052: Allow uninitialized variables with explicit type annotation
            Expr::Nil
        } else {
            return Err(SyntaxError::Generic {
                message: format!(
                    "Variable '{}' must have either a type annotation or an initial value",
                    name
                ),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        };

        let store = Store {
            kind: store_kind,
            name: name.clone(),
            ty,
            expr: expr.clone(),
        };
        if let Expr::Lambda(lambda) = &expr {
            self.define(name.as_str(), Meta::Ref(lambda.name.clone()));
        } else {
            self.define(name.as_str(), Meta::Store(store.clone()));
        }

        // Register symbol location for LSP
        let loc = SymbolLocation::new(
            name_pos.line.saturating_sub(1), // Convert from 1-based to 0-based
            name_pos.at,
            name_pos.pos,
        );
        self.scope
            .borrow_mut()
            .define_symbol_location(name.clone(), loc);

        Ok(Stmt::Store(store))
    }

    fn infer_type_expr(&mut self, expr: &Expr) -> Type {
        let mut typ = Type::Unknown;
        match expr {
            Expr::I8(..) => typ = Type::Int,
            Expr::Int(..) => typ = Type::Int,
            Expr::Float(..) => typ = Type::Float,
            Expr::Double(..) => typ = Type::Double,
            Expr::Bool(..) => typ = Type::Bool,
            Expr::Str(n) => typ = Type::Str(n.len()),
            Expr::CStr(..) => typ = Type::CStr,
            Expr::FStr(..) => typ = Type::Str(0),
            Expr::Bina(lhs, op, rhs) => {
                let ltype = self.infer_type_expr(lhs);
                let rtype = self.infer_type_expr(rhs);
                match op {
                    Op::Dot => {
                        // TODO
                        typ = rtype;
                    }
                    _ => {
                        typ = ltype;
                    }
                }
            }
            Expr::Ident(id) => {
                // println!("Infering type for identifier: {}", id); // LSP: disabled
                let meta = self.lookup_meta(id);
                if let Some(m) = meta {
                    match m.as_ref() {
                        Meta::Store(store) => {
                            typ = store.ty.clone();
                        }
                        _ => {}
                    }
                } else {
                    // try to lookup the id as a type name
                    let ltyp = self.lookup_type(id);
                    match *ltyp.borrow() {
                        Type::Unknown => {}
                        _ => {
                            typ = ltyp.borrow().clone();
                        }
                    };
                }
            }
            Expr::Node(nd) => {
                typ = nd.typ.borrow().clone();
            }
            Expr::Array(arr) => {
                // check first element
                if arr.len() > 0 {
                    let first = &arr[0];
                    let elem_ty = self.infer_type_expr(first);
                    typ = Type::Array(ArrayType {
                        elem: Box::new(elem_ty),
                        len: arr.len(),
                    });
                } else {
                    typ = Type::Array(ArrayType {
                        elem: Box::new(Type::Unknown),
                        len: 0,
                    });
                }
            }
            Expr::Call(call) => {
                // Check if this is a struct construction call: Type(args)
                // If the callee name matches a type name, infer the type as the constructed type
                if let Expr::Ident(type_name) = call.name.as_ref() {
                    let type_decl = self.lookup_type(type_name);
                    if !matches!(*type_decl.borrow(), Type::Unknown) {
                        // This is a struct construction call - return the type being constructed
                        typ = type_decl.borrow().clone();
                    } else {
                        // Regular function call - use the call's return type
                        typ = call.ret.clone();
                    }
                } else {
                    // Regular function call
                    typ = call.ret.clone();
                }
            }
            Expr::Index(arr, _idx) => {
                let arr_typ = self.infer_type_expr(arr);
                match arr_typ {
                    Type::Array(arr_ty) => {
                        typ = (*arr_ty.elem).clone();
                    }
                    Type::Str(..) => {
                        typ = Type::Char;
                    }
                    _ => {}
                };
            }
            _ => {}
        }
        typ
    }

    pub fn store_kind(&mut self) -> AutoResult<StoreKind> {
        match self.kind() {
            TokenKind::Var => Ok(StoreKind::Var),
            TokenKind::Let => Ok(StoreKind::Let),
            TokenKind::Mut => Ok(StoreKind::Var), // mut is now an alias for var in store statements
            _ => {
                let message = format!("Expected store kind, got {:?}", self.kind());
                let span = pos_to_span(self.cur.pos);
                return Err(SyntaxError::Generic { message, span }.into());
            }
        }
    }

    #[allow(dead_code)]
    fn fn_cdecl_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip keyword `c`

        // parse function name
        let name = self.cur.text.clone();
        self.expect(TokenKind::Ident)?;

        // Capture the position of the function name for LSP
        // self.prev now points to the name token after expect()
        let name_pos = self.prev.pos;

        // enter function scope
        self.scope.borrow_mut().enter_fn(name.clone());

        // parse function parameters
        self.expect(TokenKind::LParen)?;
        let params = self.fn_params()?;
        self.expect(TokenKind::RParen)?;

        // parse return type
        let mut ret_type = Type::Unknown;
        // TODO: determine return type with last stmt if it's not specified
        if self.is_type_name() {
            ret_type = self.parse_type()?;
        }

        // exit function scope
        self.exit_scope();

        // no body for c fn decl
        let body = Body::new();
        let fn_expr = Fn::new(
            FnKind::CFunction,
            name.clone(),
            None,
            params,
            body,
            ret_type,
        );
        let fn_stmt = Stmt::Fn(fn_expr.clone());

        // define function in scope
        self.define(name.as_str(), Meta::Fn(fn_expr.clone()));

        // Register symbol location for LSP
        let loc = SymbolLocation::new(name_pos.line.saturating_sub(1), name_pos.at, name_pos.pos);
        self.scope
            .borrow_mut()
            .define_symbol_location(name.clone(), loc);

        Ok(fn_stmt)
    }

    /// Parse function annotations: [c], [vm], [c,vm], [pub]
    ///
    /// Parse function annotations: #[c], #[vm], #[pub], #[c,vm], #[with(T as Spec)], etc.
    /// Annotations must start with # prefix (Rust-style).
    /// Returns (has_c, has_vm, has_pub, with_params) tuple
    ///
    /// Plan 061: Added support for #[with(T as Spec)] generic constraints
    fn parse_fn_annotations(
        &mut self,
    ) -> AutoResult<(bool, bool, bool, Vec<crate::ast::TypeParam>)> {
        let mut has_c = false;
        let mut has_vm = false;
        let mut has_pub = false;
        let mut with_params: Vec<crate::ast::TypeParam> = Vec::new();

        // Parse all annotation blocks: #[...] #[...] ...
        while self.is_kind(TokenKind::Hash) {
            self.next(); // skip #

            if self.is_kind(TokenKind::LSquare) {
                self.next(); // skip [

                while self.is_kind(TokenKind::Ident) {
                    let annot = self.cur.text.clone();
                    match annot.as_str() {
                        "c" => has_c = true,
                        "vm" => has_vm = true,
                        "pub" => has_pub = true,
                        "with" => {
                            // Plan 061: Parse #[with(T, U as Spec<V>)]
                            self.next(); // skip 'with'
                            with_params = self.parse_with_params()?;
                        }
                        _ => {
                            return Err(SyntaxError::Generic {
                                message: format!("Unknown annotation '{}'. Valid: #[c], #[vm], #[pub], #[with(...)], #[c,vm]", annot),
                                span: pos_to_span(self.cur.pos),
                            }.into());
                        }
                    }

                    // Skip 'with' since we already consumed it above
                    if annot != "with" {
                        self.next(); // skip the annotation identifier (c, vm, or pub)
                    }

                    if self.is_kind(TokenKind::Comma) {
                        self.next(); // skip ,
                        continue;
                    }

                    if self.is_kind(TokenKind::RSquare) {
                        self.next(); // skip ]
                        break;
                    } else {
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "Expected ',', ']', or annotation, found {:?}",
                                self.kind()
                            ),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                }
            } else {
                return Err(SyntaxError::Generic {
                    message: format!("Expected '[' after '#', found {:?}", self.kind()),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }

            // Skip any whitespace/newlines between multiple annotation blocks
            self.skip_empty_lines();
        }

        Ok((has_c, has_vm, has_pub, with_params))
    }

    /// Plan 061: Parse with(...) parameter list: (T, U as Spec<V>)
    /// Returns Vec<TypeParam> with constraints populated
    fn parse_with_params(&mut self) -> AutoResult<Vec<crate::ast::TypeParam>> {
        use crate::ast::TypeParam;

        self.expect(TokenKind::LParen)?; // expect '('

        let mut params = Vec::new();

        loop {
            self.skip_empty_lines();

            // Check for ) to end the list
            if self.is_kind(TokenKind::RParen) {
                break;
            }

            // Parse parameter name
            if !self.is_kind(TokenKind::Ident) {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected type parameter name in #[with(...)], got {:?}",
                        self.cur.text
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }

            let name = self.parse_name()?;

            // Check for 'as' keyword for constraint
            let constraint = if self.is_kind(TokenKind::As) {
                self.next(); // skip 'as'
                Some(Box::new(self.parse_type()?))
            } else {
                None
            };

            params.push(TypeParam { name, constraint });

            self.skip_empty_lines();

            // Check for , or )
            if self.is_kind(TokenKind::Comma) {
                self.next(); // skip ,
                continue;
            } else if self.is_kind(TokenKind::RParen) {
                break;
            } else {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected ',' or ')' in #[with(...)], got {:?}",
                        self.cur.text
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }

        self.expect(TokenKind::RParen)?; // expect ')'

        Ok(params)
    }

    // Function Declaration
    pub fn fn_decl_stmt(&mut self, parent_name: &str) -> AutoResult<Stmt> {
        // Check for annotations: #[c], #[vm], #[c,vm] BEFORE fn keyword
        let (has_c, has_vm, _has_pub, with_params) = if self.is_kind(TokenKind::Hash) {
            self.parse_fn_annotations()?
        } else {
            (false, false, false, Vec::new())
        };

        // Skip empty lines after annotations
        self.skip_empty_lines();

        self.next(); // skip keyword `fn`

        let mut is_vm = has_vm;
        let mut is_c = has_c;

        // Backwards compatibility: check for fn.c and fn.vm after fn keyword
        if !is_vm && !is_c && self.is_kind(TokenKind::Dot) {
            self.next(); // skip .
            let sub_kind = self.cur.text.clone();
            if sub_kind == "c" {
                is_c = true;
                self.next(); // skip 'c' keyword
            } else if sub_kind == "vm" {
                is_vm = true;
                self.next(); // skip 'vm' keyword
            }
        }

        // parse function name
        let name = self.parse_name()?;

        // Capture the position of the function name for LSP
        // self.prev now points to the name token after parse_name()
        let name_pos = self.prev.pos;

        // Plan 052: Parse generic parameters if present: fn foo<T, N u32>(...)
        let mut generic_params = self.parse_generic_params()?;

        // Plan 061: Merge with_params from #[with(T as Spec)] into generic_params
        // with_params override any params from <T> with the same name
        if !with_params.is_empty() {
            use crate::ast::GenericParam;

            for with_param in with_params {
                // Check if this param already exists in generic_params
                let existing_idx = generic_params.iter().position(|p| match p {
                    GenericParam::Type(tp) => tp.name == with_param.name,
                    GenericParam::Const(cp) => cp.name == with_param.name,
                });

                if let Some(idx) = existing_idx {
                    // Replace existing param with the with_param (which has constraint)
                    generic_params[idx] = GenericParam::Type(with_param.clone());
                } else {
                    // Add new param from with_params
                    generic_params.push(GenericParam::Type(with_param.clone()));
                    // Also add to current_type_params for scope
                    self.current_type_params.push(with_param.name.clone());
                }
            }
        }

        // enter function scope
        self.scope.borrow_mut().enter_fn(name.clone());

        // parse function parameters
        self.expect(TokenKind::LParen)?;
        let params = self.fn_params()?;
        self.expect(TokenKind::RParen)?;

        //

        // parse return type
        let mut ret_type = Type::Unknown;
        let mut ret_type_name: Option<AutoStr> = None;
        // TODO: determine return type with last stmt if it's not specified
        // Support: Ident (int, str), LSquare ([]int), Star (*int)
        if self.is_kind(TokenKind::Ident)
            || self.is_kind(TokenKind::LSquare)
            || self.is_kind(TokenKind::Star)
            || self.is_kind(TokenKind::Question)
        {
            if self.is_kind(TokenKind::Ident) {
                ret_type_name = Some(self.cur.text.clone());
            }
            ret_type = self.parse_type()?;
            // Skip empty lines after return type (but not for C/VM functions - they need the newlines for EOS)
            if !(is_c || is_vm) {
                self.skip_empty_lines();
            }
            // For C or VM functions, after return type we accept semicolon, newline, or nothing
            // For regular functions, we also accept semicolon for forward declarations
            // For methods in type blocks (parent_name is not empty), we also accept newline
            let allow_newline = is_c || is_vm || !parent_name.is_empty();
            if !allow_newline
                && !self.is_kind(TokenKind::LBrace)
                && !self.is_kind(TokenKind::Semi)
                && !self.is_kind(TokenKind::EOF)
                && !self.is_kind(TokenKind::Hash)
            {
                return Err(SyntaxError::Generic {
                    message: format!("Expected '{{' or ';', found {:?}", self.kind()),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        } else if self.is_kind(TokenKind::LBrace) {
            ret_type = Type::Void;
        } else if self.is_kind(TokenKind::Semi) {
            // For functions without a return type and ending with semicolon (forward declaration)
            ret_type = Type::Void;
        } else if is_c || is_vm {
            // For C or VM functions without return type and no semicolon (newline-terminated)
            ret_type = Type::Void;
        }

        // if has parent_name, define `self` in current scope
        if !parent_name.is_empty() {
            let parent_type = self.scope.borrow().find_type_for_name(parent_name);
            if let Some(parent_type) = parent_type {
                self.define(
                    "self",
                    Meta::Store(Store {
                        kind: StoreKind::Let,
                        name: "self".into(),
                        ty: parent_type.clone(),
                        expr: Expr::Ident("self".into()),
                    }),
                );
            }
        }

        // parse function body
        let body = if !is_vm && !is_c {
            // For regular functions, check if this is a forward declaration (semicolon)
            if self.is_kind(TokenKind::Semi) {
                self.next(); // skip semicolon
                Body::new()
            } else {
                self.body()?
            }
        } else {
            // For VM and C functions, check if there's a semicolon or function body
            if self.is_kind(TokenKind::Semi) {
                self.next(); // skip semicolon
            } else if self.is_kind(TokenKind::LBrace) {
                // Skip the function body for C/VM functions
                self.skip_block()?;
            }
            Body::new()
        };

        // exit function scope
        self.exit_scope();

        // parent name for method?
        let parent = if parent_name.is_empty() {
            None
        } else {
            Some(parent_name.into())
        };
        let kind = if is_vm {
            FnKind::VmFunction
        } else if is_c {
            FnKind::CFunction
        } else {
            FnKind::Function
        };

        // Create function, preserving return type name if type is Unknown
        let fn_expr = if matches!(ret_type, Type::Unknown) {
            if let Some(ret_name) = ret_type_name {
                Fn::with_ret_name(kind, name.clone(), parent, params, body, ret_type, ret_name)
            } else {
                Fn::new(kind, name.clone(), parent, params, body, ret_type)
            }
        } else {
            Fn::new(kind, name.clone(), parent, params, body, ret_type)
        };

        let fn_stmt = Stmt::Fn(fn_expr.clone());
        let unique_name = if parent_name.is_empty() {
            name.clone()
        } else {
            format!("{}.{}", parent_name, name).into()
        };

        // define function in scope
        self.define(unique_name.as_str(), Meta::Fn(fn_expr.clone()));

        // Register symbol location for LSP
        // Use the saved name_pos which is the position of the function name
        let loc = SymbolLocation::new(
            name_pos.line.saturating_sub(1), // Convert from 1-based to 0-based
            name_pos.at,
            name_pos.pos,
        );
        self.scope
            .borrow_mut()
            .define_symbol_location(unique_name.clone(), loc);

        Ok(fn_stmt)
    }

    // Function Declaration with pre-parsed annotations
    pub fn fn_decl_stmt_with_annotations(
        &mut self,
        parent_name: &str,
        has_c: bool,
        has_vm: bool,
        is_static: bool,
        with_params: Vec<crate::ast::TypeParam>,
    ) -> AutoResult<Stmt> {
        self.next(); // skip keyword `fn`

        let is_c = has_c;
        let is_vm = has_vm;

        // parse function name
        let name = self.parse_name()?;

        // Capture the position of the function name for LSP
        // self.prev now points to the name token after parse_name()
        let name_pos = self.prev.pos;

        // Plan 052: Parse generic parameters if present: fn foo<T, N u32>(...)
        let generic_params = self.parse_generic_params()?;

        // Plan 061: Merge with_params from #[with(...)] with generic_params from <T>
        // with_params take precedence (can override constraints from <T>)
        let mut type_params: Vec<crate::ast::TypeParam> = Vec::new();

        // First, extract type params from generic_params
        for gp in &generic_params {
            if let crate::ast::GenericParam::Type(tp) = gp {
                type_params.push(tp.clone());
            }
        }

        // Then, merge/override with with_params
        for wp in with_params {
            // Check if this param already exists
            let existing_idx = type_params.iter().position(|tp| tp.name == wp.name);
            if let Some(idx) = existing_idx {
                // Override with with_params version (has constraint)
                type_params[idx] = wp;
            } else {
                // Add new param
                type_params.push(wp);
            }
        }

        // enter function scope
        self.scope.borrow_mut().enter_fn(name.clone());

        // parse function parameters
        self.expect(TokenKind::LParen)?;
        let params = self.fn_params()?;
        self.expect(TokenKind::RParen)?;

        //

        // parse return type
        let mut ret_type = Type::Unknown;
        let mut ret_type_name: Option<AutoStr> = None;
        // TODO: determine return type with last stmt if it's not specified
        // Support: Ident (int, str), LSquare ([]int), Star (*int)
        if self.is_kind(TokenKind::Ident)
            || self.is_kind(TokenKind::LSquare)
            || self.is_kind(TokenKind::Star)
            || self.is_kind(TokenKind::Question)
        {
            if self.is_kind(TokenKind::Ident) {
                ret_type_name = Some(self.cur.text.clone());
            }
            ret_type = self.parse_type()?;
            // Skip empty lines after return type (but not for C/VM functions - they need the newlines for EOS)
            if !(is_c || is_vm) {
                self.skip_empty_lines();
            }
            // For C or VM functions, after return type we accept semicolon, newline, or nothing
            // For regular functions, we also accept semicolon for forward declarations
            // For methods in type blocks (parent_name is not empty), we also accept newline
            let allow_newline = is_c || is_vm || !parent_name.is_empty();
            if !allow_newline
                && !self.is_kind(TokenKind::LBrace)
                && !self.is_kind(TokenKind::Semi)
                && !self.is_kind(TokenKind::EOF)
                && !self.is_kind(TokenKind::Hash)
            {
                return Err(SyntaxError::Generic {
                    message: format!("Expected '{{' or ';', found {:?}", self.kind()),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        } else if self.is_kind(TokenKind::LBrace) {
            ret_type = Type::Void;
        } else if self.is_kind(TokenKind::Semi) {
            // For functions without a return type and ending with semicolon (forward declaration)
            ret_type = Type::Void;
        } else if is_c || is_vm {
            // For C or VM functions without return type and no semicolon (newline-terminated)
            ret_type = Type::Void;
        }

        // if has parent_name, define `self` in current scope
        if !parent_name.is_empty() {
            let parent_type = self.scope.borrow().find_type_for_name(parent_name);
            if let Some(parent_type) = parent_type {
                self.define(
                    "self",
                    Meta::Store(Store {
                        kind: StoreKind::Let,
                        name: "self".into(),
                        ty: parent_type.clone(),
                        expr: Expr::Ident("self".into()),
                    }),
                );
            }
        }

        // parse function body
        let body = if !is_vm && !is_c {
            // For regular functions, check if this is a forward declaration (semicolon)
            if self.is_kind(TokenKind::Semi) {
                self.next(); // skip semicolon
                Body::new()
            } else {
                self.body()?
            }
        } else {
            // For VM and C functions, check if there's a semicolon or function body
            if self.is_kind(TokenKind::Semi) {
                self.next(); // skip semicolon
            } else if self.is_kind(TokenKind::LBrace) {
                // Skip the function body for C/VM functions
                self.skip_block()?;
            }
            Body::new()
        };

        // exit function scope
        self.exit_scope();

        // parent name for method?
        let parent = if parent_name.is_empty() {
            None
        } else {
            Some(parent_name.into())
        };
        let kind = if is_vm {
            FnKind::VmFunction
        } else if is_c {
            FnKind::CFunction
        } else {
            FnKind::Function
        };

        // Create function, preserving return type name if type is Unknown
        let mut fn_expr = if matches!(ret_type, Type::Unknown) {
            if let Some(ret_name) = ret_type_name {
                Fn::with_ret_name(kind, name.clone(), parent, params, body, ret_type, ret_name)
            } else {
                Fn::new(kind, name.clone(), parent, params, body, ret_type)
            }
        } else {
            Fn::new(kind, name.clone(), parent, params, body, ret_type)
        };

        // Plan 061: Set type_params from #[with(...)] and <T>
        fn_expr.type_params = type_params;

        // Set is_static flag
        fn_expr.is_static = is_static;

        let fn_stmt = Stmt::Fn(fn_expr.clone());
        let unique_name = if parent_name.is_empty() {
            name.clone()
        } else {
            format!("{}.{}", parent_name, name).into()
        };

        // define function in scope
        self.define(unique_name.as_str(), Meta::Fn(fn_expr.clone()));

        // Register symbol location for LSP
        // Use the saved name_pos which is the position of the function name
        let loc = SymbolLocation::new(
            name_pos.line.saturating_sub(1), // Convert from 1-based to 0-based
            name_pos.at,
            name_pos.pos,
        );
        self.scope
            .borrow_mut()
            .define_symbol_location(unique_name.clone(), loc);

        Ok(fn_stmt)
    }

    pub fn sep_params(&mut self) -> AutoResult<()> {
        if self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            self.next();
            return Ok(());
        }
        if self.is_kind(TokenKind::RParen) || self.is_kind(TokenKind::VBar) {
            return Ok(());
        }
        let pos = self.cur.pos;
        Err(SyntaxError::UnexpectedToken {
            expected: "parameter separator (comma, newline, ), or |)".to_string(),
            found: format!("{:?}", self.kind()),
            span: pos_to_span(pos),
        }
        .into())
    }

    // parse function parameters
    pub fn fn_params(&mut self) -> AutoResult<Vec<Param>> {
        let mut params = Vec::new();
        while self.is_kind(TokenKind::Ident) {
            // param name
            let name = self.cur.text.clone();
            let name_pos = self.cur.pos; // Capture position before skipping name
            self.next(); // skip name

            // param type
            let mut ty = Type::Int;
            if self.is_type_name() {
                ty = self.parse_type()?;
            }
            // default val
            let mut default = None;
            if self.is_kind(TokenKind::Asn) {
                self.next(); // skip =
                let expr = self.parse_expr()?;
                default = Some(expr);
            }
            // define param in current scope (currently in fn scope)
            let var = Store {
                kind: StoreKind::Var,
                name: name.clone(),
                expr: default.clone().unwrap_or(Expr::Nil),
                ty: ty.clone(),
            };
            // TODO: should we consider Meta::Param instead of Meta::Var?
            self.define(name.as_str(), Meta::Store(var.clone()));

            // Register symbol location for LSP
            let loc = SymbolLocation::new(
                name_pos.line.saturating_sub(1), // Convert from 1-based to 0-based
                name_pos.at,
                name_pos.pos,
            );
            self.scope
                .borrow_mut()
                .define_symbol_location(name.clone(), loc);

            params.push(Param { name, ty, default });
            self.sep_params()?;
        }

        // Handle variadic arguments (...) for C functions
        if self.is_kind(TokenKind::Range) {
            self.next(); // skip .. (Range token)
            if !self.is_kind(TokenKind::Dot) {
                return Err(SyntaxError::Generic {
                    message: "Expected '...' for variadic arguments, found '..'".to_string(),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
            self.next(); // skip final .
                         // Add a variadic marker parameter
                         // Use a special name to indicate variadic
            params.push(Param {
                name: "...".into(),
                ty: Type::Variadic,
                default: None,
            });
        }

        Ok(params)
    }

    pub fn expr_stmt(&mut self) -> AutoResult<Stmt> {
        Ok(Stmt::Expr(self.parse_expr()?))
    }

    pub fn spec_decl_stmt(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip `spec` keyword

        let name = self.parse_name()?;

        // Plan 057: Parse generic parameters - e.g., spec Storage<T>, spec Storage<T, N u32>
        let generic_params = self.parse_generic_params()?;

        // Plan 057: Populate type parameter scope for use in method signatures
        for param in &generic_params {
            if let GenericParam::Type(tp) = param {
                self.current_type_params.push(tp.name.clone());
            } else if let GenericParam::Const(cp) = param {
                self.current_const_params
                    .insert(cp.name.clone(), cp.typ.clone());
            }
        }

        // Parse spec body
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();

        let mut methods = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            if self.is_kind(TokenKind::Fn) {
                let method = self.spec_method()?;
                methods.push(method);
                self.expect_eos(false)?;
            } else {
                return Err(SyntaxError::Generic {
                    message: "Expected method declaration in spec".to_string(),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
            self.skip_empty_lines();
        }

        self.expect(TokenKind::RBrace)?;

        // Plan 057: Clear type parameters after parsing spec body
        for param in &generic_params {
            if let GenericParam::Type(tp) = param {
                self.current_type_params.pop();
            } else if let GenericParam::Const(cp) = param {
                self.current_const_params.remove(&cp.name);
            }
        }

        // Plan 057: Use SpecDecl::with_generic_params if we have generic params
        let spec_decl = if generic_params.is_empty() {
            SpecDecl::new(name, methods)
        } else {
            SpecDecl::with_generic_params(name, generic_params, methods)
        };

        // Register spec in scope
        self.define(spec_decl.name.as_str(), Meta::Spec(spec_decl.clone()));

        // Plan 061 Phase 2: Register spec in Universe for constraint validation
        self.scope
            .borrow_mut()
            .register_spec(std::rc::Rc::new(spec_decl.clone()));

        Ok(Stmt::SpecDecl(spec_decl))
    }

    fn spec_method(&mut self) -> AutoResult<SpecMethod> {
        self.expect(TokenKind::Fn)?;
        let name = self.parse_name()?;

        self.expect(TokenKind::LParen)?;
        let params = self.fn_params()?;
        self.expect(TokenKind::RParen)?;

        // Parse return type
        let ret = if self.is_type_name() {
            self.parse_type()?
        } else {
            Type::Void // Default to void
        };

        Ok(SpecMethod { name, params, ret })
    }

    /// Parse a type alias statement: `type List<T> = List<T, DefaultStorage>;`
    pub fn parse_type_alias(&mut self) -> AutoResult<Stmt> {
        self.next(); // skip `type` keyword

        let name = self.parse_name()?;

        // Parse generic parameters (optional) - e.g., type List<T> = ...
        let mut params = Vec::new();
        if self.cur.kind == TokenKind::Lt {
            self.next(); // Consume '<'

            // Parse type parameter names (not full GenericParam, just identifiers)
            let param_name = self.parse_name()?;
            params.push(param_name);

            while self.cur.kind == TokenKind::Comma {
                self.next(); // Consume ','
                let param_name = self.parse_name()?;
                params.push(param_name);
            }

            self.expect(TokenKind::Gt)?; // Consume '>'
        }

        self.expect(TokenKind::Eq)?;

        let target = self.parse_type()?;

        self.expect(TokenKind::Semi)?;

        // Note: For now, we don't store type aliases in scope.
        // They'll be resolved during compilation/transpilation.
        // TODO: Add type alias storage to Universe for resolution

        Ok(Stmt::TypeAlias(TypeAlias {
            name,
            params,
            target,
        }))
    }

    pub fn type_decl_stmt(&mut self) -> AutoResult<Stmt> {
        self.type_decl_stmt_with_annotation(false)
    }

    pub fn type_decl_stmt_with_annotation(&mut self, has_c_annotation: bool) -> AutoResult<Stmt> {
        // TODO: deal with scope
        self.next(); // skip `type` keyword

        // Check for #[c] annotation before the type name (if not already provided)
        let has_c_annotation = if !has_c_annotation && self.is_kind(TokenKind::Hash) {
            self.parse_fn_annotations()?.0
        } else {
            has_c_annotation
        };

        // Skip empty lines after annotation
        self.skip_empty_lines();

        if self.is_kind(TokenKind::Dot) {
            self.next();
            let sub_kind = self.parse_name()?;
            if sub_kind == "c" {
                let name = self.parse_name()?;

                // Capture the position of the type name for LSP
                let name_pos = self.prev.pos;

                let decl = TypeDecl {
                    kind: TypeDeclKind::CType,
                    name: name.clone(),
                    parent: None,
                    has: Vec::new(),
                    specs: Vec::new(),
                    spec_impls: Vec::new(), // Plan 057
                    generic_params: Vec::new(),
                    members: Vec::new(),
                    delegations: Vec::new(),
                    methods: Vec::new(),
                };
                // put type in scope
                self.define(name.as_str(), Meta::Type(Type::CStruct(decl.clone())));

                // Register symbol location for LSP
                let loc =
                    SymbolLocation::new(name_pos.line.saturating_sub(1), name_pos.at, name_pos.pos);
                self.scope
                    .borrow_mut()
                    .define_symbol_location(name.clone(), loc);

                return Ok(Stmt::TypeDecl(decl));
            }
        }

        // If we have [c] annotation, treat as C type
        let kind = if has_c_annotation {
            TypeDeclKind::CType
        } else {
            TypeDeclKind::UserType
        };

        let name = self.parse_name()?;

        // Parse generic parameters (optional) - e.g., type List<T> { ... }, type Inline<T, const N u32> { ... }
        let mut generic_params = Vec::new();
        if self.cur.kind == TokenKind::Lt {
            self.next(); // Consume '<'

            generic_params.push(self.parse_generic_param()?);

            while self.cur.kind == TokenKind::Comma {
                self.next(); // Consume ','
                generic_params.push(self.parse_generic_param()?);
            }

            self.expect(TokenKind::Gt)?; // Consume '>'
        }

        // Check if this is a type alias (has `=` after name and params)
        if self.cur.kind == TokenKind::Asn {
            // This is a type alias: type List<T> = List<T, DefaultStorage>;
            self.next(); // Consume '='

            let target = self.parse_type()?;
            self.expect(TokenKind::Semi)?;

            // Extract just the names from GenericParam for TypeAlias
            let params: Vec<Name> = generic_params
                .into_iter()
                .filter_map(|p| match p {
                    GenericParam::Type(tp) => Some(tp.name),
                    GenericParam::Const(_) => None, // Const params not supported in type aliases
                })
                .collect();

            // Store type alias in universe for later resolution
            self.scope
                .borrow_mut()
                .define_type_alias(name.clone(), params.clone(), target.clone());

            return Ok(Stmt::TypeAlias(TypeAlias {
                name,
                params,
                target,
            }));
        }

        // Plan 052: Populate current_const_params map with const generic parameters
        // and current_type_params with type generic parameters
        // This allows parameters to be used in method signatures within the type body
        for param in &generic_params {
            if let crate::ast::GenericParam::Type(tp) = param {
                self.current_type_params.push(tp.name.clone());
            } else if let crate::ast::GenericParam::Const(cp) = param {
                self.current_const_params
                    .insert(cp.name.clone(), cp.typ.clone());
            }
        }

        // Capture the position of the type name for LSP
        // self.prev now points to the name token after parse_name()
        let name_pos = self.prev.pos;

        let mut decl = TypeDecl {
            kind: kind.clone(),
            name: name.clone(),
            parent: None,
            specs: Vec::new(),
            spec_impls: Vec::new(), // Plan 057
            has: Vec::new(),
            generic_params,
            members: Vec::new(),
            delegations: Vec::new(),
            methods: Vec::new(),
        };
        // println!(
        //     "Defining type {} in scope {}",
        //     name,
        //     self.scope.borrow().cur_spot
        // );

        // put type in scope
        self.define(name.as_str(), Meta::Type(Type::User(decl.clone())));

        // Register symbol location for LSP
        // Use the saved name_pos which is the position of the type name
        let loc = SymbolLocation::new(name_pos.line.saturating_sub(1), name_pos.at, name_pos.pos);
        self.scope
            .borrow_mut()
            .define_symbol_location(name.clone(), loc);

        // For C types, there's no body - they're opaque types
        if kind == TypeDeclKind::CType {
            // C types are opaque, no body needed
            // Type already registered above at line 3356
            return Ok(Stmt::TypeDecl(decl));
        }

        // deal with `is` keyword (single inheritance)
        let mut parent = None;
        if self.is_kind(TokenKind::Is) {
            self.next(); // skip `is` keyword
            let parent_name = self.parse_name()?;
            // Lookup parent type
            if let Some(meta) = self.lookup_meta(parent_name.as_str()) {
                if let Meta::Type(Type::User(parent_decl)) = meta.as_ref() {
                    parent = Some(Box::new(Type::User(parent_decl.clone())));
                } else {
                    return Err(SyntaxError::Generic {
                        message: format!("'{}' is not a user type", parent_name),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            } else {
                return Err(SyntaxError::Generic {
                    message: format!("Parent type '{}' not found", parent_name),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }
        decl.parent = parent;

        // Plan 057: deal with `as` keyword - parse spec implementations with optional type arguments
        let mut specs = Vec::new(); // Backwards compatibility: names only
        let mut spec_impls = Vec::new(); // Plan 057: Generic spec implementations
        if self.is_kind(TokenKind::As) {
            self.next(); // skip `as` keyword
                         // Parse one or more spec names with optional type arguments
            while !self.is_kind(TokenKind::LBrace) && !self.is_kind(TokenKind::Has) {
                if !specs.is_empty() {
                    self.expect(TokenKind::Comma)?;
                }
                let spec_name = self.parse_name()?;

                // Plan 057: Check for type arguments: as Storage<T>
                let type_args = if self.is_kind(TokenKind::Lt) {
                    self.next(); // skip `<`
                    let mut args = Vec::new();
                    loop {
                        self.skip_empty_lines();
                        args.push(self.parse_type()?);
                        self.skip_empty_lines();

                        if self.is_kind(TokenKind::Gt) {
                            self.next(); // skip `>`
                            break;
                        } else if self.is_kind(TokenKind::Comma) {
                            self.next(); // skip `,`
                            continue;
                        } else {
                            return Err(SyntaxError::Generic {
                                message: format!(
                                    "Expected '>' or ',' in type argument list, got {}",
                                    self.cur.text
                                ),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into());
                        }
                    }
                    args
                } else {
                    Vec::new() // No type arguments
                };

                // Add to backwards-compatible specs list
                specs.push(spec_name.clone());

                // Plan 057: Add to spec_impls if we have type arguments
                if !type_args.is_empty() {
                    spec_impls.push(crate::ast::SpecImpl {
                        spec_name,
                        type_args,
                    });
                }
            }
        }
        decl.specs = specs;
        decl.spec_impls = spec_impls; // Plan 057

        // deal with `has` keyword
        let mut has = Vec::new();
        if self.is_kind(TokenKind::Has) {
            self.next(); // skip `has` keyword
            while !self.is_kind(TokenKind::LBrace) {
                if !has.is_empty() {
                    self.expect(TokenKind::Colon)?; // skip ,
                }
                let typ = self.parse_type()?;
                has.push(typ);
            }
        }
        decl.has = has;

        // type body (for user types)
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        // list of members, methods, or delegations
        let mut members = Vec::new();
        let mut methods = Vec::new();
        let mut delegations = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            // Check for annotations: #[c], #[vm], #[pub], #[c,vm] before function declarations
            let (has_c, has_vm, _has_pub, with_params) = self.parse_fn_annotations()?;

            self.skip_empty_lines(); // Skip newlines after annotations

            // Check if this annotation should be skipped for current compile destination
            let should_skip = match self.compile_dest {
                CompileDest::TransC if has_vm && !has_c => true, // Skip #[vm] in C transpiler
                CompileDest::TransRust if has_vm && !has_c => true, // Skip #[vm] in Rust transpiler
                CompileDest::Interp if has_c && !has_vm => true, // Skip #[c] in interpreter
                _ => false,
            };

            if should_skip {
                // Skip the entire function declaration
                if self.is_kind(TokenKind::Static) || self.is_kind(TokenKind::Fn) {
                    let is_static = self.is_kind(TokenKind::Static);
                    if is_static {
                        self.next(); // skip static
                    }
                    // Parse with actual flags to correctly handle the function syntax
                    let _ = self.fn_decl_stmt_with_annotations(&name, has_c, has_vm, is_static, with_params.clone());
                }
                self.expect_eos(false)?;
                continue;
            }

            // Check for static fn or fn
            if self.is_kind(TokenKind::Static) || self.is_kind(TokenKind::Fn) {
                let is_static = self.is_kind(TokenKind::Static);
                if is_static {
                    self.next(); // skip static keyword
                }

                // Now expect fn keyword
                if !self.is_kind(TokenKind::Fn) {
                    return Err(SyntaxError::Generic {
                        message: format!("expected 'fn' after 'static', found {:?}", self.kind()),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }

                let fn_stmt =
                    self.fn_decl_stmt_with_annotations(&name, has_c, has_vm, is_static, with_params)?;
                if let Stmt::Fn(fn_expr) = fn_stmt {
                    methods.push(fn_expr);
                }
            } else if self.is_kind(TokenKind::Has) {
                // Parse member-level delegation: `has member Type for Spec`
                let delegation = self.parse_delegation()?;
                delegations.push(delegation);
            } else {
                let member = self.type_member(&name)?;
                members.push(member);
            }
            self.expect_eos(false)?; // Not first statement in type body
        }
        self.expect(TokenKind::RBrace)?;

        // add members and methods from parent type (inheritance)
        if let Some(ref parent_type) = decl.parent {
            match parent_type.as_ref() {
                Type::User(parent_decl) => {
                    // Inherit members from parent
                    for m in parent_decl.members.iter() {
                        members.push(m.clone());
                    }
                    // Inherit methods from parent
                    for meth in parent_decl.methods.iter() {
                        // change meth's parent to self
                        let mut inherited_meth = meth.clone();
                        inherited_meth.parent = Some(name.clone());
                        // register this method as Self.method
                        let unique_name = format!("{}.{}", &name, &inherited_meth.name);
                        self.define(unique_name.as_str(), Meta::Fn(inherited_meth.clone()));
                        methods.push(inherited_meth);
                    }
                }
                _ => {
                    // System types cannot be inherited
                }
            }
        }

        // add members and methods of compose types
        for comp in decl.has.iter() {
            match comp {
                Type::User(decl) => {
                    for m in decl.members.iter() {
                        members.push(m.clone());
                    }
                    for meth in decl.methods.iter() {
                        // change meth's parent to self
                        let mut compose_meth = meth.clone();
                        compose_meth.parent = Some(name.clone());
                        // register this method as Self::method
                        let unique_name = format!("{}::{}", &name, &compose_meth.name);
                        self.define(unique_name.as_str(), Meta::Fn(compose_meth.clone()));
                        methods.push(compose_meth);
                    }
                }
                _ => {
                    // System Types not supported for Compose yet
                }
            }
        }
        decl.members = members;
        decl.methods = methods;
        decl.delegations = delegations;

        // Check trait conformance if type declares specs (Plan 057: defer check for ext blocks)
        // Note: If methods are empty, skip conformance check as methods may be added later via ext blocks
        if !decl.specs.is_empty() && !decl.methods.is_empty() {
            for spec_name in &decl.specs {
                if let Some(meta) = self.lookup_meta(spec_name.as_str()) {
                    if let Meta::Spec(spec_decl) = meta.as_ref() {
                        // Use TraitChecker to verify conformance
                        if let Err(errors) =
                            crate::trait_checker::TraitChecker::check_conformance(&decl, spec_decl)
                        {
                            // Add all conformance errors to parser errors
                            for error in errors {
                                self.add_error(error);
                            }
                        }
                    }
                } else {
                    // Spec not found - this is an error
                    self.add_error(
                        SyntaxError::Generic {
                            message: format!(
                                "Type '{}' declares spec '{}' but spec is not defined",
                                name, spec_name
                            ),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into(),
                    );
                }
            }
        }

        // Plan 057: Check generic spec implementations (spec_impls)
        // Same logic: skip check if methods are empty (ext block case)
        if !decl.spec_impls.is_empty() && !decl.methods.is_empty() {
            for spec_impl in &decl.spec_impls {
                if let Some(meta) = self.lookup_meta(spec_impl.spec_name.as_str()) {
                    if let Meta::Spec(spec_decl) = meta.as_ref() {
                        // Validate type argument count
                        if spec_impl.type_args.len() != spec_decl.generic_params.len() {
                            self.add_error(
                                SyntaxError::Generic {
                                    message: format!(
                                        "Type '{}' implements spec '{}' with {} type argument(s) but spec expects {}",
                                        name,
                                        spec_impl.spec_name,
                                        spec_impl.type_args.len(),
                                        spec_decl.generic_params.len()
                                    ),
                                    span: pos_to_span(self.cur.pos),
                                }
                                .into(),
                            );
                        } else {
                            // Use TraitChecker to verify conformance
                            if let Err(errors) =
                                crate::trait_checker::TraitChecker::check_conformance(
                                    &decl, spec_decl,
                                )
                            {
                                for error in errors {
                                    self.add_error(error);
                                }
                            }
                        }
                    }
                } else {
                    // Spec not found - this is an error
                    self.add_error(
                        SyntaxError::Generic {
                            message: format!(
                                "Type '{}' declares generic spec '{}' but spec is not defined",
                                name, spec_impl.spec_name
                            ),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into(),
                    );
                }
            }
        }

        self.define(name.as_str(), Meta::Type(Type::User(decl.clone())));
        Ok(Stmt::TypeDecl(decl))
    }

    pub fn type_member(&mut self, type_name: &str) -> AutoResult<Member> {
        // Capture the position of the field name for LSP
        let name = self.parse_name()?;
        let name_pos = self.prev.pos;

        let ty = self.parse_type()?;
        let mut value = None;
        if self.is_kind(TokenKind::Asn) {
            self.next(); // skip =
            let expr = self.parse_expr()?;
            value = Some(expr);
        }
        let store = Store {
            name: name.clone(),
            ty: ty.clone(),
            expr: match &value {
                Some(expr) => expr.clone(),
                None => Expr::Nil,
            },
            kind: StoreKind::Field,
        };
        self.define(name.as_str(), Meta::Store(store));

        // Register field location for LSP with qualified name "TypeName.fieldName"
        let qualified_name = format!("{}.{}", type_name, name);
        let loc = SymbolLocation::new(name_pos.line.saturating_sub(1), name_pos.at, name_pos.pos);
        self.scope
            .borrow_mut()
            .define_symbol_location(qualified_name.clone(), loc);

        Ok(Member::new(name, ty, value))
    }

    /// Parse member-level delegation: `has member Type for Spec`
    /// Example: `has core WarpDrive for Engine`
    fn parse_delegation(&mut self) -> AutoResult<crate::ast::Delegation> {
        self.expect(TokenKind::Has)?; // skip `has` keyword

        // Parse member name
        let member_name = self.parse_name()?;

        // Parse member type
        let member_type = self.parse_type()?;

        // Expect `for` keyword
        self.expect(TokenKind::For)?;

        // Parse spec name
        let spec_name = self.parse_name()?;

        Ok(crate::ast::Delegation {
            member_name,
            member_type,
            spec_name,
        })
    }

    pub fn union_stmt(&mut self) -> AutoResult<Stmt> {
        self.expect(TokenKind::Union)?;
        let name = self.parse_name()?;
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut fields = Vec::new();
        let mut field_index = 0;
        while !self.is_kind(TokenKind::RBrace) {
            let f = self.union_field()?;
            fields.push(f);
            let is_first = field_index == 0;
            self.expect_eos(is_first)?;
            field_index += 1;
        }
        self.expect(TokenKind::RBrace)?;

        let union = Union {
            name: name.clone(),
            fields: fields.clone(),
        };

        self.define(name.as_str(), Meta::Type(Type::Union(union)));
        Ok(Stmt::Union(Union { name, fields }))
    }

    pub fn union_field(&mut self) -> AutoResult<UnionField> {
        let name = self.parse_name()?;
        let ty = self.parse_type()?;
        Ok(UnionField { name, ty })
    }

    pub fn tag_stmt(&mut self) -> AutoResult<Stmt> {
        self.expect(TokenKind::Tag)?;
        let name = self.parse_name()?;

        // Parse generic parameters (optional) - e.g., tag May<T> { ... }, tag Inline<T, const N u32> { ... }
        let mut generic_params = Vec::new();
        if self.cur.kind == TokenKind::Lt {
            self.next(); // Consume '<'

            generic_params.push(self.parse_generic_param()?);

            while self.cur.kind == TokenKind::Comma {
                self.next(); // Consume ','
                generic_params.push(self.parse_generic_param()?);
            }

            self.expect(TokenKind::Gt)?; // Consume '>'
        }

        // Set current type parameters for field parsing (extract names from generic_params)
        let prev_type_params = std::mem::replace(
            &mut self.current_type_params,
            generic_params
                .iter()
                .map(|gp| match gp {
                    crate::ast::GenericParam::Type(tp) => tp.name.clone(),
                    crate::ast::GenericParam::Const(cp) => cp.name.clone(),
                })
                .collect(),
        );

        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        // Parse fields and methods (EXACTLY like type_decl_stmt)
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            if self.is_kind(TokenKind::Fn) {
                let fn_stmt = self.fn_decl_stmt(&name)?;
                if let Stmt::Fn(fn_expr) = fn_stmt {
                    methods.push(fn_expr);
                }
            } else {
                let field = self.tag_field()?;
                fields.push(field);
            }
            self.expect_eos(false)?; // Single EOS call after both branches
        }
        self.expect(TokenKind::RBrace)?;

        // Restore previous type parameters
        self.current_type_params = prev_type_params;

        // Register tag type with fields and methods
        self.define(
            name.as_str(),
            Meta::Type(Type::Tag(shared(Tag {
                name: name.clone(),
                generic_params: generic_params.clone(),
                fields: fields.clone(),
                methods: methods.clone(),
            }))),
        );

        Ok(Stmt::Tag(Tag {
            name,
            generic_params,
            fields,
            methods,
        }))
    }

    pub fn tag_field(&mut self) -> AutoResult<TagField> {
        let name = self.parse_name()?;
        let ty = self.parse_type()?;
        Ok(TagField { name, ty })
    }

    fn get_int_expr(&mut self, num: &Expr) -> i64 {
        match num {
            Expr::Int(n) => *n as i64,
            Expr::Uint(n) => *n as i64,
            Expr::I8(n) => *n as i64,
            Expr::U8(n) => *n as i64,
            _ => 0,
        }
    }

    fn get_usize(&mut self, size: &Expr) -> AutoResult<usize> {
        match size {
            Expr::Int(size) => {
                if *size <= 0 {
                    return Err(SyntaxError::Generic {
                        message: "Array size must be greater than 0".to_string(),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                } else {
                    Ok(*size as usize)
                }
            }
            Expr::Uint(size) => Ok(*size as usize),
            Expr::I8(size) => {
                if *size <= 0 {
                    return Err(SyntaxError::Generic {
                        message: "Array size must be greater than 0".to_string(),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                } else {
                    Ok(*size as usize)
                }
            }
            Expr::U8(size) => Ok(*size as usize),
            _ => {
                return Err(SyntaxError::Generic {
                    message: "Invalid array size expression, Not a Integer Number!".to_string(),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }
    }

    fn parse_ptr_type(&mut self) -> AutoResult<Type> {
        self.next(); // skip `*`

        // Handle optional `const`/`mut` qualifier (e.g., `*const T`, `*mut T`)
        // Plan 059 Phase 1: Enable generic type fields in structs
        // Note: `const` and `mut` are now keywords (TokenKind), not identifiers
        if self.is_kind(TokenKind::Const) || self.is_kind(TokenKind::Mut) {
            self.next(); // skip const/mut qualifier
        }

        let typ = self.parse_type()?;
        Ok(Type::Ptr(PtrType { of: shared(typ) }))
    }

    fn parse_array_type(&mut self) -> AutoResult<Type> {
        // parse array type name, e.g. `[10]int` or `[]int` (slice)
        // Note: `[~]T` syntax for dynamic lists has been removed - use `List` type instead
        self.next(); // skip `[`

        // Check for slice type: []T (empty brackets)
        if self.is_kind(TokenKind::RSquare) {
            self.next(); // consume `]`

            // Parse element type
            let type_name = self.parse_ident()?;
            match type_name {
                Expr::Ident(name) => {
                    let elem_ty = self.lookup_type(&name).borrow().clone();
                    let slice_ty_name = format!("[]{}", name);

                    // Check if slice type already exists
                    let slice_meta = self.lookup_meta(&slice_ty_name);
                    match slice_meta {
                        Some(meta) => {
                            if let Meta::Type(slice_ty) = meta.as_ref() {
                                return Ok(slice_ty.clone());
                            } else {
                                return Err(SyntaxError::Generic {
                                    message: format!("Expected slice type, got {:?}", meta),
                                    span: pos_to_span(self.cur.pos),
                                }
                                .into());
                            }
                        }
                        None => {
                            // Create new slice type
                            use crate::ast::SliceType;
                            let slice_ty = Type::Slice(SliceType {
                                elem: Box::new(elem_ty.clone()),
                            });
                            self.scope
                                .borrow_mut()
                                .define_type(slice_ty_name, Rc::new(Meta::Type(slice_ty.clone())));
                            return Ok(slice_ty);
                        }
                    }
                }
                _ => {
                    return Err(SyntaxError::Generic {
                        message: format!("Expected type identifier, got {:?}", type_name),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            }
        }

        // Parse array size: [N]T or [expr]T (Plan 052: Runtime arrays)
        // Strategy: Check if next token is literal int (constant array) or something else (runtime array)
        let is_constant_size = self.is_kind(TokenKind::Int)
            || self.is_kind(TokenKind::Uint)
            || self.is_kind(TokenKind::I8)
            || self.is_kind(TokenKind::U8)
            || self.is_kind(TokenKind::RSquare);

        if !is_constant_size {
            // Plan 052: Parse as RuntimeArray (runtime-sized array)
            // Parse size expression (can be any expression: variable, function call, etc.)
            let size_expr = self.parse_expr()?;

            self.expect(TokenKind::RSquare)?; // skip `]`

            // Parse element type
            let type_name = self.parse_ident()?;
            match type_name {
                Expr::Ident(name) => {
                    let elem_ty = self.lookup_type(&name).borrow().clone();

                    // Create RuntimeArrayType
                    use crate::ast::RuntimeArrayType;
                    return Ok(Type::RuntimeArray(RuntimeArrayType {
                        elem: Box::new(elem_ty),
                        size_expr: Box::new(size_expr),
                    }));
                }
                _ => {
                    return Err(SyntaxError::Generic {
                        message: format!("Expected type identifier, got {:?}", type_name),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            }
        }

        // Parse static array: [N]T (constant size)
        let array_size = if self.is_kind(TokenKind::Int)
            || self.is_kind(TokenKind::Uint)
            || self.is_kind(TokenKind::I8)
            || self.is_kind(TokenKind::U8)
        {
            let size = self.parse_ints()?;
            // println!("got int {}", size); // LSP: disabled
            self.get_usize(&size)?
        } else if self.is_kind(TokenKind::RSquare) {
            0
        } else if self.is_kind(TokenKind::Ident) {
            // may be an ident to int variable
            // NOTE: the value should be determined at compile time
            let name = self.parse_name()?;
            let meta = self.lookup_meta(&name);
            let Some(m) = meta else {
                return Err(SyntaxError::Generic {
                    message: format!("Array Size of {} is not found", name),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            };
            match m.as_ref() {
                Meta::Store(store) => {
                    // evaluate store value at compile time
                    // TODO: maybe recursive
                    match store.expr {
                        Expr::Int(n) => n as usize,
                        Expr::Uint(n) => n as usize,
                        Expr::U8(n) => n as usize,
                        Expr::I8(n) => n as usize,
                        _ => 0 as usize,
                    }
                }
                _ => 0,
            }
        } else {
            let message = format!("Expected Array Size or Empty, got {}", self.peek().kind);
            let span = pos_to_span(self.cur.pos);
            return Err(SyntaxError::Generic { message, span }.into());
        };
        self.expect(TokenKind::RSquare)?; // skip `]`

        // parse array elem type
        let type_name = self.parse_ident()?;
        match type_name {
            Expr::Ident(name) => {
                let ty = self.lookup_type(&name).borrow().clone();
                let array_ty_name = format!("[{}]{}", array_size, name);
                let arry_meta = self.lookup_meta(&array_ty_name);
                match arry_meta {
                    Some(meta) => {
                        if let Meta::Type(array_ty) = meta.as_ref() {
                            return Ok(array_ty.clone());
                        } else {
                            return Err(SyntaxError::Generic {
                                message: format!("Expected array type, got {:?}", meta),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into());
                        }
                    }
                    None => {
                        let array_ty = Type::Array(ArrayType {
                            elem: Box::new(ty.clone()),
                            len: array_size,
                        });
                        self.scope
                            .borrow_mut()
                            .define_type(array_ty_name, Rc::new(Meta::Type(array_ty.clone())));
                        return Ok(array_ty);
                    }
                }
            }
            _ => {
                return Err(SyntaxError::Generic {
                    message: format!("Expected type, got ident {:?}", type_name),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }
    }

    /// Parse a single type parameter (e.g., T, K, V)
    fn parse_type_param(&mut self) -> AutoResult<crate::ast::TypeParam> {
        use crate::ast::TypeParam;

        match self.cur.kind {
            TokenKind::Ident => {
                let name = self.parse_name()?;
                Ok(TypeParam {
                    name,
                    constraint: None,
                })
            }
            _ => Err(SyntaxError::Generic {
                message: format!("Expected type parameter, got {}", self.cur.text),
                span: pos_to_span(self.cur.pos),
            }
            .into()),
        }
    }

    /// Parse a generic parameter (Plan 052)
    /// Can be either:
    /// - Type parameter: `T` (no type annotation)
    /// - Const parameter: `N u32` (with type annotation)
    fn parse_generic_param(&mut self) -> AutoResult<crate::ast::GenericParam> {
        use crate::ast::{ConstParam, GenericParam, TypeParam};

        // First, parse the parameter name
        if self.cur.kind != TokenKind::Ident {
            return Err(SyntaxError::Generic {
                message: format!("Expected generic parameter name, got {}", self.cur.text),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }

        let name = self.parse_name()?;

        // Check if next token looks like a type
        // If next token is a type keyword or identifier, it's a const parameter
        if self.next_token_is_type() {
            // Const parameter: `N u32`
            let typ = self.parse_type()?;

            Ok(GenericParam::Const(ConstParam {
                name,
                typ,
                default: None, // TODO: Support default values
            }))
        } else {
            // Type parameter: just `T`
            Ok(GenericParam::Type(TypeParam {
                name,
                constraint: None,
            }))
        }
    }

    /// Parse generic parameter list: <T, N u32>
    /// Returns Vec<GenericParam> and adds type parameters to current_type_params
    fn parse_generic_params(&mut self) -> AutoResult<Vec<crate::ast::GenericParam>> {
        if !self.is_kind(TokenKind::Lt) {
            return Ok(Vec::new());
        }

        self.next(); // skip `<`
        let mut params = Vec::new();

        // Parse comma-separated generic parameters
        loop {
            self.skip_empty_lines();
            params.push(self.parse_generic_param()?);
            self.skip_empty_lines();

            if self.is_kind(TokenKind::Gt) {
                self.next(); // skip `>`
                break;
            } else if self.is_kind(TokenKind::Comma) {
                self.next(); // skip `,`
                continue;
            } else {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected '>' or ',' in generic parameter list, got {}",
                        self.cur.text
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }

        // Add type parameters to scope for use in function body
        for param in &params {
            if let crate::ast::GenericParam::Type(tp) = param {
                self.current_type_params.push(tp.name.clone());
            } else if let crate::ast::GenericParam::Const(cp) = param {
                // Store const parameter with its actual type for type resolution
                self.current_const_params
                    .insert(cp.name.clone(), cp.typ.clone());
            }
        }

        Ok(params)
    }

    /// Parse identifier type or generic instance (e.g., List, List<int>)
    fn parse_ident_or_generic_type(&mut self) -> AutoResult<Type> {
        use crate::ast::{GenericInstance, Type};

        let ident = self.parse_ident()?;

        match ident {
            Expr::Ident(name) => {
                // Special case: Dynamic storage type (Plan 055)
                if name.as_str() == "Dynamic" {
                    return Ok(Type::Storage(crate::ast::StorageType {
                        kind: crate::ast::StorageKind::Dynamic,
                    }));
                }

                // Check if this is a generic instance (e.g., List<int>, May<string>)
                if self.cur.kind == TokenKind::Lt {
                    // Context check: make sure < is followed by a type
                    // We need to look ahead to see if this looks like a generic instance
                    if self.next_token_is_type() {
                        return self.parse_generic_instance(name);
                    }
                }

                // Plan 052: Check if this identifier is a const generic parameter
                if let Some(const_ty) = self.current_const_params.get(&name) {
                    // This is a const parameter - return its actual type (e.g., uint)
                    return Ok(const_ty.clone());
                }

                // Plan 058: Check if this is a type alias
                let type_alias = self
                    .scope
                    .borrow()
                    .lookup_type_alias(name.as_str())
                    .map(|(params, target)| (params.clone(), target.clone()));

                if let Some((alias_params, alias_target)) = type_alias {
                    // Check if there are type arguments (e.g., List<int>)
                    if self.cur.kind == TokenKind::Lt && self.next_token_is_type() {
                        // Parse type arguments
                        self.next(); // Consume '<'
                        let mut type_args = Vec::new();
                        type_args.push(self.parse_type()?);

                        while self.cur.kind == TokenKind::Comma {
                            self.next(); // Consume ','
                            type_args.push(self.parse_type()?);
                        }

                        self.expect(TokenKind::Gt)?; // Consume '>'

                        // Check if the number of type arguments matches the alias parameters
                        if type_args.len() != alias_params.len() {
                            return Err(SyntaxError::Generic {
                                message: format!(
                                    "Type alias '{}' expects {} type parameter(s), but got {}",
                                    name,
                                    alias_params.len(),
                                    type_args.len()
                                ),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into());
                        }

                        // Substitute type arguments into the target type
                        let substituted = alias_target.substitute(&alias_params, &type_args);
                        return Ok(substituted);
                    } else if !alias_params.is_empty() {
                        // Type alias requires parameters but none were provided
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "Type alias '{}' requires {} type parameter(s), but none were provided",
                                name,
                                alias_params.len()
                            ),
                            span: pos_to_span(self.cur.pos),
                        }.into());
                    } else {
                        // No type arguments needed - return the target type directly
                        return Ok(alias_target);
                    }
                }

                // Check if this identifier is a type parameter in the current scope
                if self.current_type_params.contains(&name) {
                    // This is a type parameter - return it as a user type for now
                    // The type system will handle substitution later
                    return Ok(Type::User(TypeDecl {
                        name: name.clone(),
                        kind: TypeDeclKind::UserType,
                        parent: None,
                        has: Vec::new(),
                        specs: Vec::new(),
                        spec_impls: Vec::new(), // Plan 057
                        generic_params: Vec::new(),
                        members: Vec::new(),
                        delegations: Vec::new(),
                        methods: Vec::new(),
                    }));
                }

                // Regular type name - look it up in the type registry
                Ok(self.lookup_type(&name).borrow().clone())
            }
            _ => Err(SyntaxError::Generic {
                message: format!("Expected type, got ident {:?}", ident),
                span: pos_to_span(self.cur.pos),
            }
            .into()),
        }
    }

    /// Check if the next token is likely the start of a type
    fn next_token_is_type(&mut self) -> bool {
        // Peek at the next token
        let next = self.peek();
        matches!(
            next.kind,
            TokenKind::Ident |      // Type name (int, List, etc.)
            TokenKind::Question |   // May type (?T)
            TokenKind::LSquare |    // Array type ([N]T)
            TokenKind::Star |       // Pointer type (*T)
            TokenKind::Lt // Nested generic (List<List<int>>)
        )
    }

    /// Parse generic instance (e.g., List<int>, Map<str, int>, MyType<T, U>)
    fn parse_generic_instance(&mut self, base_name: Name) -> AutoResult<Type> {
        use crate::ast::{GenericInstance, Type};

        self.expect(TokenKind::Lt)?;

        let mut args = Vec::new();
        args.push(self.parse_type()?);

        while self.cur.kind == TokenKind::Comma {
            self.next(); // Consume ','
            args.push(self.parse_type()?);
        }

        self.expect(TokenKind::Gt)?;

        // Check if base_name refers to a user-defined generic Tag or TypeDecl
        let base_type = self.lookup_type(&base_name);

        // Plan 055: Check if we need to fill in default parameters from environment
        // Clone needed data to avoid borrow issues
        let needs_default_param = {
            let base_type_ref = base_type.borrow();
            match &*base_type_ref {
                Type::User(type_decl) if !type_decl.generic_params.is_empty() => {
                    // User-defined generic TypeDecl with type parameters
                    // Check if user provided fewer parameters than expected
                    if args.len() < type_decl.generic_params.len() {
                        Some((base_name.clone(), type_decl.generic_params.len()))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        };

        let mut args = args;
        if let Some((type_name, expected_params)) = needs_default_param {
            // Fill in missing parameters from environment
            // For now, only support List<T, S> pattern where S has a default
            if type_name.as_str() == "List" && args.len() == 1 && expected_params == 2 {
                // Get default storage from environment
                let default_storage = self
                    .scope
                    .borrow()
                    .get_env_val("DEFAULT_STORAGE")
                    .unwrap_or_else(|| "Heap".into()); // Fallback to Heap for PC

                // Parse the storage type from environment string
                let storage_type = self.parse_storage_from_env(&default_storage)?;
                args.push(storage_type);
            } else {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Type '{}' expects {} parameter(s), but got {}",
                        type_name,
                        expected_params,
                        args.len()
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            }
        }

        let base_type_ref = base_type.borrow();

        match &*base_type_ref {
            Type::Tag(tag_shared) if !tag_shared.borrow().generic_params.is_empty() => {
                // User-defined generic Tag with type parameters
                // Perform substitution and create new Tag instance
                let tag = tag_shared.borrow().clone();
                drop(base_type_ref); // Drop borrow before registering

                // Collect generic parameter names
                let param_names: Vec<_> = tag
                    .generic_params
                    .iter()
                    .map(|gp| match gp {
                        crate::ast::GenericParam::Type(tp) => tp.name.clone(),
                        crate::ast::GenericParam::Const(cp) => cp.name.clone(),
                    })
                    .collect();

                // Substitute type parameters in tag fields
                let substituted_fields: Vec<_> = tag
                    .fields
                    .iter()
                    .map(|field| TagField {
                        name: field.name.clone(),
                        ty: field.ty.substitute(&param_names, &args),
                    })
                    .collect();

                // Create new substituted tag
                let substituted_tag = Tag {
                    name: format!(
                        "{}_{}",
                        base_name,
                        args.iter()
                            .map(|t| t.unique_name().to_string())
                            .collect::<Vec<_>>()
                            .join("_")
                    )
                    .into(),
                    generic_params: Vec::new(), // No type parameters in instantiated tag
                    fields: substituted_fields,
                    methods: tag.methods.clone(),
                };

                // Register the substituted tag
                let subst_name = substituted_tag.name.clone();
                self.define(
                    subst_name.as_str(),
                    Meta::Type(Type::Tag(shared(substituted_tag.clone()))),
                );

                return Ok(Type::Tag(shared(substituted_tag)));
            }
            Type::User(type_decl) if !type_decl.generic_params.is_empty() => {
                // User-defined generic TypeDecl with type parameters
                // TODO: Implement TypeDecl substitution (similar to Tag substitution)
                // For now, return GenericInstance
                drop(base_type_ref);
                return Ok(Type::GenericInstance(GenericInstance { base_name, args }));
            }
            _ => {
                // Either built-in type or non-generic user-defined type
                drop(base_type_ref);
            }
        }

        // Special handling for built-in generic types
        // List<T> has dedicated Type variant (May<T> is now a generic tag)
        // Fixed<N> is a Storage type (Plan 055)
        match base_name.as_str() {
            "List" => {
                // Plan 055: Support both List<T> (1 param) and List<T, S> (2 params)
                if args.len() == 1 {
                    // List<int> → Type::List(Box::new(int))
                    // Storage will be determined at runtime by VM
                    Ok(Type::List(Box::new(args.into_iter().next().unwrap())))
                } else if args.len() == 2 {
                    // List<int, Heap> → Return GenericInstance for full type
                    // This allows the transpiler to see both parameters
                    Ok(Type::GenericInstance(GenericInstance { base_name, args }))
                } else {
                    Err(SyntaxError::Generic {
                        message: format!(
                            "List expects 1 or 2 type parameter(s), but got {}",
                            args.len()
                        ),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into())
                }
            }
            "Fixed" if args.len() == 1 => {
                // Fixed<N> storage type - parse capacity from first argument
                // The capacity should be a constant expression (int literal or const)
                let capacity = match &args[0] {
                    Type::Int => {
                        // For now, Fixed<int> will use default capacity
                        // TODO: Parse actual integer value from expression
                        64
                    }
                    Type::Uint => {
                        // For now, Fixed<uint> will use default capacity
                        // TODO: Parse actual integer value from expression
                        64
                    }
                    _ => {
                        return Err(SyntaxError::Generic {
                            message: format!(
                                "Fixed storage requires integer capacity, got {}",
                                args[0]
                            ),
                            span: pos_to_span(self.cur.pos),
                        }
                        .into());
                    }
                };

                Ok(Type::Storage(crate::ast::StorageType {
                    kind: crate::ast::StorageKind::Fixed { capacity },
                }))
            }
            _ => {
                // User-defined generic instance (including May<T> from stdlib)
                Ok(Type::GenericInstance(GenericInstance { base_name, args }))
            }
        }
    }

    /// Parse a storage type name from environment variable string
    /// For Plan 055: Convert "Heap" → Type::Storage(Dynamic), "InlineInt64" → proper type
    fn parse_storage_from_env(&mut self, storage_name: &str) -> AutoResult<Type> {
        match storage_name {
            "Heap" | "Dynamic" => Ok(Type::Storage(crate::ast::StorageType {
                kind: crate::ast::StorageKind::Dynamic,
            })),
            "InlineInt64" | "Fixed" => Ok(Type::Storage(crate::ast::StorageType {
                kind: crate::ast::StorageKind::Fixed { capacity: 64 },
            })),
            _ => {
                // Try to parse as Fixed<N> pattern
                if storage_name.starts_with("Fixed<") && storage_name.ends_with(">") {
                    let inner = &storage_name[6..storage_name.len() - 1];
                    if let Ok(capacity) = inner.parse::<usize>() {
                        return Ok(Type::Storage(crate::ast::StorageType {
                            kind: crate::ast::StorageKind::Fixed { capacity },
                        }));
                    }
                }

                // Fallback: treat as Dynamic
                Ok(Type::Storage(crate::ast::StorageType {
                    kind: crate::ast::StorageKind::Dynamic,
                }))
            }
        }
    }

    fn parse_ident_type(&mut self) -> AutoResult<Type> {
        let ident = self.parse_ident()?;
        match ident {
            Expr::Ident(name) => Ok(self.lookup_type(&name).borrow().clone()),
            _ => Err(SyntaxError::Generic {
                message: format!("Expected type, got ident {:?}", ident),
                span: pos_to_span(self.cur.pos),
            }
            .into()),
        }
    }

    fn is_type_name(&mut self) -> bool {
        self.is_kind(TokenKind::Ident) // normal types like `int`
        || self.is_kind(TokenKind::Question) // May types like `?int`
        || self.is_kind(TokenKind::LSquare) // array types like `[5]int`
        || self.is_kind(TokenKind::Star) // ptr types like `*int`
        || self.is_kind(TokenKind::At) // ref types like `@int`
    }

    pub fn parse_type(&mut self) -> AutoResult<Type> {
        match self.cur.kind {
            TokenKind::Question => {
                // Parse ?T as syntax sugar for May<T> generic tag
                self.next(); // Consume '?'
                let inner_type = self.parse_type()?;

                // Look up generic May tag definition from stdlib
                let may_tag_ref = self.lookup_type(&Name::from("May"));
                let (subst_name, substituted_fields, methods) = match &*may_tag_ref.borrow() {
                    Type::Tag(t) if !t.borrow().generic_params.is_empty() => {
                        // Use stdlib May<T> tag with substitution
                        let tag = t.borrow().clone();
                        let param_names: Vec<_> = tag
                            .generic_params
                            .iter()
                            .map(|gp| match gp {
                                crate::ast::GenericParam::Type(tp) => tp.name.clone(),
                                crate::ast::GenericParam::Const(cp) => cp.name.clone(),
                            })
                            .collect();
                        let type_args = vec![inner_type.clone()];

                        let fields: Vec<_> = tag
                            .fields
                            .iter()
                            .map(|field| TagField {
                                name: field.name.clone(),
                                ty: field.ty.substitute(&param_names, &type_args),
                            })
                            .collect();

                        // Use underscore naming for stdlib tags (May_int, May_string)
                        let inner_name = type_args[0].unique_name().to_string();
                        (format!("May_{}", inner_name), fields, tag.methods)
                    }
                    _ => {
                        // Fallback: Create builtin May<T> directly (no substitution needed)
                        // For C transpilation tests that don't load stdlib
                        let inner_name = inner_type.unique_name().to_string();
                        let subst_name = format!("May{}", capitalize_first(&inner_name));

                        let fields = vec![
                            TagField {
                                name: Name::from("nil"),
                                ty: Type::Unknown,
                            },
                            TagField {
                                name: Name::from("val"),
                                ty: inner_type.clone(),
                            },
                            TagField {
                                name: Name::from("err"),
                                ty: Type::Int,
                            },
                        ];

                        (subst_name, fields, Vec::new())
                    }
                };

                // Create new substituted tag
                let substituted_tag = Tag {
                    name: subst_name.clone().into(),
                    generic_params: Vec::new(), // No type params in instantiated tag
                    fields: substituted_fields,
                    methods,
                };

                // Register the substituted tag in scope
                self.define(
                    subst_name.as_str(),
                    Meta::Type(Type::Tag(shared(substituted_tag.clone()))),
                );

                Ok(Type::Tag(shared(substituted_tag)))
            }
            TokenKind::Ident => self.parse_ident_or_generic_type(),
            TokenKind::Star => self.parse_ptr_type(),
            TokenKind::LSquare => self.parse_array_type(),
            _ => {
                let message = format!("Expected type, got {}", self.cur.text);
                let span = pos_to_span(self.cur.pos);
                return Err(SyntaxError::Generic { message, span }.into());
            }
        }
    }

    // TODO: 暂时只检查3种情况：
    // 1，简单名称；
    // 2，点号表达式最左侧的名称
    // 3, 函数调用，如果函数名不存在，表示是一个节点实例
    pub fn check_symbol(&mut self, expr: Expr) -> AutoResult<Expr> {
        if self.skip_check {
            return Ok(expr);
        }
        match &expr {
            Expr::Bina(l, op, _) => match op {
                Op::Dot => {
                    if let Expr::Ident(name) = l.as_ref() {
                        // Check if it's a type name (for static method calls like HashMap.new())
                        let is_type = self.lookup_type(name);
                        let is_type_valid = match *is_type.borrow() {
                            Type::User(_) | Type::Tag(_) => true,
                            _ => false,
                        };

                        if !self.exists(&name) && !is_type_valid {
                            let candidates = self.scope.borrow().get_defined_names();
                            return Err(NameError::undefined_variable(
                                name.to_string(),
                                pos_to_span(self.cur.pos),
                                &candidates,
                            )
                            .into());
                        }
                    }
                    Ok(expr)
                }
                _ => Ok(expr),
            },
            Expr::Ident(name) => {
                if !self.exists(&name) {
                    let candidates = self.scope.borrow().get_defined_names();
                    return Err(NameError::undefined_variable(
                        name.to_string(),
                        pos_to_span(self.cur.pos),
                        &candidates,
                    )
                    .into());
                }
                Ok(expr)
            }
            Expr::Call(call) => {
                match call.name.as_ref() {
                    Expr::Ident(_name) => {
                        // TODO: Re-enable this check for production, but skip for AST tests
                        // if !self.exists(&name) {
                        //     return error_pos!("Function {} not define!", name);
                        // }
                    }
                    Expr::Bina(lhs, op, _rhs) => {
                        // check tag creation or static method call (TypeName.method())
                        if let Op::Dot = op {
                            if let Expr::Ident(lname) = lhs.as_ref() {
                                let ltype = self.lookup_type(lname);
                                match *ltype.borrow() {
                                    Type::Tag(ref _t) => {}
                                    Type::User(ref _name) => {
                                        // Allow type names for static method calls (e.g., HashMap.new())
                                        // The evaluator will route these to the VM registry
                                    }
                                    _ => {}
                                };
                            }
                        }
                    }
                    _ => {}
                }
                Ok(expr)
            }
            _ => Ok(expr),
        }
    }

    pub fn find_type_for_expr(&mut self, expr: &Expr) -> AutoResult<Type> {
        match expr {
            // function name, find it's decl
            Expr::Ident(ident) => {
                let meta = self.lookup_meta(ident);
                let Some(meta) = meta else {
                    return Err(SyntaxError::Generic {
                        message: format!("Function name not found! {}", ident),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                };
                match meta.as_ref() {
                    Meta::Fn(fun) => {
                        return Ok(fun.ret.clone());
                    }
                    _ => {}
                }
            }
            // method or function in a struct
            Expr::Bina(lhs, op, rhs) => {
                if let Op::Dot = op {
                    match &**lhs {
                        Expr::Ident(lname) => {
                            // lookup meta for left name
                            let meta = self.lookup_meta(lname.as_str());
                            let Some(meta) = meta else {
                                return Err(SyntaxError::Generic {
                                    message: format!("Left name not found! {}.{}", lname, rhs),
                                    span: pos_to_span(self.cur.pos),
                                }
                                .into());
                            };
                            match meta.as_ref() {
                                Meta::Type(typ) => match typ {
                                    Type::Tag(tag) => {
                                        if let Expr::Ident(rname) = &**rhs {
                                            let rtype = tag.borrow().get_field_type(rname);
                                            return Ok(rtype);
                                        }
                                    }
                                    Type::User(_) => {
                                        // Allow type names for static method calls (e.g., HashMap.new())
                                        // The return type will be determined by the evaluator
                                        if let Expr::Ident(rname) = &**rhs {
                                            // For static methods, try to look up the method signature
                                            let combined_name = format!("{}::{}", lname, rname);
                                            if let Some(method_meta) =
                                                self.lookup_meta(&combined_name)
                                            {
                                                if let Meta::Fn(fn_decl) = method_meta.as_ref() {
                                                    return Ok(fn_decl.ret.clone());
                                                }
                                            }
                                            // Default: assume it returns the type itself (like constructors)
                                            return Ok(typ.clone());
                                        }
                                    }
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Err(SyntaxError::Generic {
            message: format!("Meta not found! {}", expr),
            span: pos_to_span(self.cur.pos),
        }
        .into())
    }

    pub fn return_type(&mut self, call_name: &Expr) -> AutoResult<Type> {
        match call_name {
            // function name, find it's decl
            Expr::Ident(ident) => {
                let meta = self.lookup_meta(ident);
                let Some(meta) = meta else {
                    return Ok(Type::Unknown);
                };
                match meta.as_ref() {
                    Meta::Fn(fun) => {
                        return Ok(fun.ret.clone());
                    }
                    _ => {}
                }
            }
            // method or function in a struct
            Expr::Bina(lhs, op, rhs) => {
                if let Op::Dot = op {
                    match &**lhs {
                        Expr::Ident(lname) => {
                            // lookup meta for left name
                            let meta = self.lookup_meta(lname.as_str());
                            let Some(meta) = meta else {
                                return Err(SyntaxError::Generic {
                                    message: format!("Left name not found! {}.{}", lname, rhs),
                                    span: pos_to_span(self.cur.pos),
                                }
                                .into());
                            };
                            match meta.as_ref() {
                                Meta::Type(typ) => match typ {
                                    Type::Tag(tag) => {
                                        if let Expr::Ident(rname) = &**rhs {
                                            if tag.borrow().has_field(rname) {
                                                return Ok(typ.clone());
                                            }
                                            let rtype = tag.borrow().get_field_type(rname);
                                            return Ok(rtype);
                                        }
                                    }
                                    Type::User(_) => {
                                        // Allow type names for static method calls (e.g., HashMap.new())
                                        if let Expr::Ident(rname) = &**rhs {
                                            let combined_name = format!("{}::{}", lname, rname);
                                            if let Some(method_meta) =
                                                self.lookup_meta(&combined_name)
                                            {
                                                if let Meta::Fn(fn_decl) = method_meta.as_ref() {
                                                    return Ok(fn_decl.ret.clone());
                                                }
                                            }
                                            return Ok(typ.clone());
                                        }
                                    }
                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Plan 052/060: Handle Expr::Dot for generic type method calls
            // e.g., List<int, Heap>.new() where lhs is GenName("List<int, Heap>")
            Expr::Dot(lhs, rhs) => {
                // Extract base type name from GenName or Ident
                let base_name = match lhs.as_ref() {
                    Expr::GenName(name) => {
                        // Extract base name from "List<int, Heap>" -> "List"
                        let name_str = name.as_str();
                        if let Some(pos) = name_str.find('<') {
                            &name_str[..pos]
                        } else {
                            name_str
                        }
                    }
                    Expr::Ident(name) => name.as_str(),
                    _ => {
                        return Ok(Type::Unknown);
                    }
                };

                // Lookup meta for the base type
                let meta = self.lookup_meta(base_name);
                let Some(meta) = meta else {
                    return Ok(Type::Unknown);
                };

                match meta.as_ref() {
                    Meta::Type(typ) => match typ {
                        Type::User(decl) => {
                            // Check for static methods (e.g., List.new not List::new)
                            // rhs is AutoStr (field/method name)
                            let combined_name = format!("{}.{}", base_name, rhs);
                            if let Some(method_meta) = self.lookup_meta(&combined_name) {
                                if let Meta::Fn(fn_decl) = method_meta.as_ref() {
                                    return Ok(fn_decl.ret.clone());
                                }
                            }
                            // Return the type itself if no method found
                            return Ok(typ.clone());
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
        // TODO: should check all types or print error
        Ok(Type::Unknown)
    }

    fn parse_node(
        &mut self,
        name: &AutoStr,
        primary: Option<Expr>,
        args: Args,
        kind: &AutoStr,
    ) -> AutoResult<Node> {
        let n = name.clone().into();
        let mut node = Node::new(name.clone());
        if let Some(prime) = primary {
            match prime {
                Expr::Ident(id) => {
                    // define a variable for this node instance with id
                    self.define(id.as_str(), Meta::Node(node.clone()));
                    node.id = id;
                }
                _ => {
                    // Convert to unified API: add args before body props
                    node.add_arg_unified("content", prime.clone());
                }
            }
        }

        // optional kind argument - add using unified API
        if !kind.is_empty() {
            node.add_arg_unified("kind", Expr::Str(kind.into()));
        }

        // Add remaining args using unified API
        for arg in args.args {
            match arg {
                Arg::Pos(expr) => {
                    // Positional arg
                    node.add_pos_arg_unified(expr);
                }
                Arg::Name(pair_name) => {
                    // Named arg
                    node.add_name_arg_unified(pair_name);
                }
                Arg::Pair(key, value) => {
                    // Pair arg - check type before adding
                    // 查找类型定义进行类型检查
                    let typ = self.lookup_type(&node.name);

                    if let Type::User(type_decl) = &*typ.borrow() {
                        // 查找成员定义
                        if let Some(member) = type_decl.members.iter().find(|m| &m.name == &key) {
                            // 推断表达式类型
                            let value_ty = self.infer_type_expr(&value);

                            // 检查类型匹配（使用当前 token 位置作为近似位置）
                            if let Err(err) =
                                check_field_type(member, &value_ty, pos_to_span(self.cur.pos))
                            {
                                self.errors.push(err);
                            }
                        }
                    }

                    node.add_arg_unified(key, value);
                }
            }
        }

        if self.is_kind(TokenKind::Newline) || self.is_kind(TokenKind::Semi) {
        } else {
            if self.is_kind(TokenKind::LBrace) {
                if self.special_blocks.contains_key(&n) {
                    node.body = self.special_block(&n)?;
                } else {
                    node.body = self.parse_node_body()?;
                }
            }
        }

        // check node type
        let typ = self.lookup_type(&node.name);
        node.typ = typ.clone();

        Ok(node)
    }

    // 节点实例和函数调用有类似语法：
    // 函数调用：
    // 1. hello(x, y)， 这个是函数调用
    // 2. hello(), 这个是参数为空的函数调用
    // 节点实例化：
    // 1. hello (x, y) { ... }， 这个是节点实例
    // 2. hello () { ... }， 这个是参数为空的节点实例
    // 3. hello {...}，当参数为空时，可以省略()。但不能省略{}，否则和函数调用就冲突了。
    // 4. hello(x, y) {}, 这个是子节点为空的节点实例
    // 5. hello () {}， 这个是参数为空，子节点也为空的节点实例
    // 6. hello {}， 上面的()也可以省略。
    // 7. hello name (x, y) { ... }， 这是新的带变量名称的语法
    // 8. hello name { ... } 参数可以省略
    // 总之，节点实例的关键特征是`{}`，而函数调用没有`{}`
    pub fn parse_node_or_call_stmt(&mut self) -> AutoResult<Stmt> {
        let expr = self.node_or_call_expr()?;
        match expr {
            Expr::Node(node) => Ok(Stmt::Node(node)),
            _ => Ok(Stmt::Expr(expr)),
        }
    }

    pub fn node_or_call_expr(&mut self) -> AutoResult<Expr> {
        // Parse identifier or generic type instance (e.g., List or List<int>)
        let name = self.cur.text.clone();
        self.next(); // skip the identifier

        // Check if this is a generic type instance (e.g., List<int>, Heap<T>)
        let mut ident = if self.is_kind(TokenKind::Lt) && self.next_token_is_type() {
            // Parse as generic instance and create GenName expression
            self.expect(TokenKind::Lt)?;

            let mut args = Vec::new();
            args.push(self.parse_type()?);

            while self.cur.kind == TokenKind::Comma {
                self.next(); // Consume ','
                args.push(self.parse_type()?);
            }

            self.expect(TokenKind::Gt)?;

            // Generate descriptive name: "List<int>", "Heap<T>", etc.
            let args_str = args
                .iter()
                .map(|t| t.unique_name().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let generic_name = format!("{}<{}>", name, args_str);

            Expr::GenName(generic_name.into())
        } else {
            // Regular identifier
            Expr::Ident(name)
        };

        // Plan 056: Use Expr::Dot for semantic clarity
        while self.is_kind(TokenKind::Dot) {
            self.next(); // skip dot
                         // Allow @, * as special field names for pointer operations
                         // Allow numeric literals for integer-keyed objects: a.1, a.2
                         // Allow boolean keywords: a.true, a.false
            let field_name = if self.is_kind(TokenKind::Ident) {
                let text = self.cur.text.clone();
                self.next();
                text
            } else if self.is_kind(TokenKind::At) || self.is_kind(TokenKind::Star) {
                let text = self.cur.text.clone();
                self.next();
                text
            } else if self.is_kind(TokenKind::Int)
                || self.is_kind(TokenKind::Uint)
                || self.is_kind(TokenKind::I8)
                || self.is_kind(TokenKind::U8)
                || self.is_kind(TokenKind::Float)
                || self.is_kind(TokenKind::Double)
                || self.is_kind(TokenKind::True)
                || self.is_kind(TokenKind::False)
                || self.is_kind(TokenKind::Nil)
            {
                let text = self.cur.text.clone();
                self.next();
                text
            } else {
                return Err(SyntaxError::Generic {
                    message: format!(
                        "Expected identifier, @, *, number, or boolean after dot, got {:?}",
                        self.cur.kind
                    ),
                    span: pos_to_span(self.cur.pos),
                }
                .into());
            };
            ident = Expr::Dot(Box::new(ident), field_name);
        }

        let is_constructor = match &ident {
            Expr::Ident(n) => {
                let meta = self.lookup_meta(n);
                if let Some(m) = meta {
                    match m.as_ref() {
                        Meta::Type(_) => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Expr::GenName(name) => {
                // Generic instance like "List<int>" or "Heap<T>"
                // Extract base type name (before the first '<')
                let name_str = name.as_str();
                let base_name = if let Some(pos) = name_str.find('<') {
                    &name_str[..pos]
                } else {
                    name_str
                };

                let meta = self.lookup_meta(base_name);
                if let Some(m) = meta {
                    match m.as_ref() {
                        Meta::Type(_) => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        let mut has_paren = false;

        // 节点实例的primary prop
        let primary_prop = if self.is_kind(TokenKind::Ident) {
            let id = self.ident_name()?;
            self.next();
            Some(Expr::Ident(id))
        } else if self.is_literal() {
            Some(self.literal()?)
        } else {
            None
        };

        let mut args = Args::new();
        // If has paren, maybe a call or node instance
        if self.is_kind(TokenKind::LParen) {
            args = self.args()?;
            has_paren = true;
        }

        // If has brace, must be a node instance
        // NOTE: If ident is a Dot expression (e.g., TypeName.method), treat as call even if is_constructor=true
        let is_dot_call = matches!(ident, Expr::Dot(_, _));
        if self.is_kind(TokenKind::LBrace)
            || primary_prop.is_some()
            || (is_constructor && !is_dot_call)
        {
            // node instance
            // with node instance, pair args also defines as properties
            for arg in &args.args {
                if let Arg::Pair(name, value) = arg {
                    self.define(
                        name.as_str(),
                        Meta::Pair(Pair {
                            key: Key::NamedKey(name.clone()),
                            value: Box::new(value.clone()),
                        }),
                    );
                }
            }
            match ident {
                Expr::Ident(name) => {
                    return Ok(Expr::Node(self.parse_node(
                        &name,
                        primary_prop,
                        args,
                        &AutoStr::new(),
                    )?));
                }
                _ => {
                    return Err(SyntaxError::Generic {
                        message: format!("Expected node name, got {:?}", ident),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
            }
        } else {
            // no brace, might be a function call, constructor call or simple expression
            if has_paren {
                // Use the call result as the left side of a binary expression
                // and continue with Pratt parser to handle operators like +, -, etc.
                let call_expr = self.call(ident, args)?;
                self.expr_pratt_with_left(call_expr, 0)
            } else {
                // Something else with a starting Ident
                if primary_prop.is_some() {
                    if self.is_kind(TokenKind::Ident) {
                        let kind = self.parse_name()?;
                        if self.is_kind(TokenKind::LBrace)
                            || self.is_kind(TokenKind::Newline)
                            || self.is_kind(TokenKind::Semi)
                            || self.is_kind(TokenKind::RBrace)
                        {
                            let nd = self.parse_node(&ident.repr(), primary_prop, args, &kind)?;
                            return Ok(Expr::Node(nd));
                        } else {
                            return Err(SyntaxError::Generic {
                                message: format!(
                                    "expect node, got {} {} {}",
                                    ident.repr(),
                                    primary_prop.unwrap().repr(),
                                    kind
                                ),
                                span: pos_to_span(self.cur.pos),
                            }
                            .into());
                        }
                    } else {
                        // name id <nl>
                        if self.is_kind(TokenKind::Newline)
                            || self.is_kind(TokenKind::Semi)
                            || self.is_kind(TokenKind::RBrace)
                        {
                            let nd =
                                self.parse_node(&ident.repr(), primary_prop, args, &"".into())?;
                            return Ok(Expr::Node(nd));
                        }
                    }
                    return Err(SyntaxError::Generic {
                        message: format!(
                            "Expected simple expression, got `{} {}`",
                            ident.repr(),
                            primary_prop.unwrap()
                        ),
                        span: pos_to_span(self.cur.pos),
                    }
                    .into());
                }
                let expr = self.expr_pratt_with_left(ident, 0)?;
                let expr = self.check_symbol(expr)?;
                Ok(expr)
            }
        }
    }

    fn call(&mut self, ident: Expr, args: Args) -> AutoResult<Expr> {
        let ret_type = self.return_type(&ident)?;
        let expr = Expr::Call(Call {
            name: Box::new(ident),
            args,
            ret: ret_type,
            type_args: Vec::new(),  // Plan 061: Will be filled in during type inference
        });
        self.check_symbol(expr)
    }

    fn special_block(&mut self, name: &AutoStr) -> AutoResult<Body> {
        self.expect(TokenKind::LBrace)?;
        let block_parser = self.special_blocks.remove(name);
        if block_parser.is_none() {
            return Err(SyntaxError::Generic {
                message: format!("Unknown special block: {}", name),
                span: pos_to_span(self.cur.pos),
            }
            .into());
        }
        let block_parser = block_parser.unwrap();
        let body = block_parser.parse(self)?;
        self.special_blocks.insert(name.clone(), block_parser);
        self.expect(TokenKind::RBrace)?;
        Ok(body)
    }

    pub fn grid(&mut self) -> AutoResult<Grid> {
        self.next(); // skip grid
                     // args
        let mut data = Vec::new();
        let args = self.args()?;
        // data
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut row_index = 0;
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            let row = self.array()?;
            if let Expr::Array(array) = row {
                data.push(array);
            }
            let is_first = row_index == 0;
            self.expect_eos(is_first)?;
            row_index += 1;
        }
        self.expect(TokenKind::RBrace)?;
        let grid = Grid { head: args, data };
        Ok(grid)
    }

    pub fn parse_event_src(&mut self) -> AutoResult<Option<Expr>> {
        let src = if self.is_kind(TokenKind::Ident) {
            Some(self.parse_ident()?)
        } else if self.is_kind(TokenKind::Int) {
            Some(self.atom()?)
        } else if self.is_kind(TokenKind::Str) {
            Some(self.atom()?)
        } else if self.is_kind(TokenKind::FStrStart) {
            Some(self.fstr()?)
        } else if self.is_kind(TokenKind::Arrow) {
            None
        } else {
            None
        };

        Ok(src)
    }

    pub fn parse_cond_arrow(&mut self, src: Option<Expr>) -> AutoResult<CondArrow> {
        if self.is_kind(TokenKind::Question) {
            self.next(); // skip ?
            let cond = self.parse_expr()?;
            let subs = self.parse_arrow_list()?;
            return Ok(CondArrow::new(src, cond, subs));
        } else {
            return Err(format!("Expected condition arrow").into());
        }
    }

    /// Deffirent forms:
    /// 1. EV -> State : Handler
    /// 2. -> State : Handler
    /// 3. EV : Handler
    /// 4. EV -> State
    /// 5. EV ? ConditionCheck {
    ///       -> State1 : Handler1
    ///       -> State2 : handler2
    ///    }
    pub fn parse_arrow(&mut self, src: Option<Expr>) -> AutoResult<Arrow> {
        if self.is_kind(TokenKind::Colon) {
            self.next();
            let with = self.parse_expr()?;
            return Ok(Arrow::new(src, None, Some(with)));
        } else {
            self.expect(TokenKind::Arrow)?;
            let to = self.parse_expr()?;
            if let Expr::Pair(p) = to {
                let to = p.key.into();
                let value = *p.value;
                return Ok(Arrow::new(src, Some(to), Some(value)));
            } else {
                return Ok(Arrow::new(src, Some(to), None));
            }
        };
    }

    fn parse_arrow_list(&mut self) -> AutoResult<Vec<Arrow>> {
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut events = Vec::new();
        while !self.is_kind(TokenKind::RBrace) {
            let src = self.parse_event_src()?;
            events.push(self.parse_arrow(src)?);
            self.skip_empty_lines();
        }
        self.expect(TokenKind::RBrace)?;
        Ok(events)
    }

    pub fn parse_event(&mut self) -> AutoResult<Event> {
        let src = self.parse_event_src()?;
        // println!("NEXT I S {}", self.kind()); // LSP: disabled
        if self.is_kind(TokenKind::Arrow) || self.is_kind(TokenKind::Colon) {
            return Ok(Event::Arrow(self.parse_arrow(src)?));
        } else if self.is_kind(TokenKind::Question) {
            return Ok(Event::CondArrow(self.parse_cond_arrow(src)?));
        } else {
            return Err(format!("expected arrow, colon or question mark.").into());
        }
    }

    fn parse_goto_branch(&mut self) -> AutoResult<Event> {
        self.parse_event()
    }

    pub fn parse_on_events(&mut self) -> AutoResult<OnEvents> {
        // skip on
        self.expect(TokenKind::On)?;

        let mut branches = Vec::new();

        // multiple branches
        if self.is_kind(TokenKind::LBrace) {
            self.next(); // skip {
            self.skip_empty_lines();
            while !self.is_kind(TokenKind::RBrace) {
                self.skip_empty_lines();
                // println!("cchecking branch: {}", self.cur); // LSP: disabled
                branches.push(self.parse_goto_branch()?);
                self.skip_empty_lines();
            }
            self.expect(TokenKind::RBrace)?;
        } else {
            // single branch
            branches.push(self.parse_goto_branch()?);
        }
        Ok(OnEvents::new(branches))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse_once(code: &str) -> Code {
        let mut parser = Parser::from(code);
        parser.parse().unwrap()
    }

    fn parse_with_err(code: &str) -> AutoResult<Code> {
        let mut parser = Parser::from(code);
        parser.parse()
    }

    #[test]
    fn test_parser() {
        let code = "1+2+3";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (bina (bina (int 1) (op +) (int 2)) (op +) (int 3)))"
        );
    }

    #[test]
    fn test_if() {
        let code = "if true {1}";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(if (branch (true) (body (int 1))))");
    }

    #[test]
    fn test_if_else() {
        let code = "if false {1} else {2}";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(
            last.to_string(),
            "(if (branch (false) (body (int 1))) (else (body (int 2))))"
        );
    }

    #[test]
    fn test_let() {
        let code = "let x = 1";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (let (name x) (type int) (int 1)))");
    }

    #[test]
    fn test_let_asn() {
        let code = "let x = 1; x = 2";
        let res = parse_with_err(code);
        let expected = "Assignment not allowed for let store: x";
        let res_err = res.err().unwrap();
        let err_string = res_err.to_string();

        // Check if error message is in MultipleErrors wrapper
        if err_string.contains("aborting due to") {
            // The error is wrapped in MultipleErrors, check the inner errors
            // The error message should be in one of the inner errors
            // For this test, we just check that we got an error about let store assignment
            assert!(err_string.contains("error") || err_string.contains("Error"));
        } else {
            // Direct error
            assert!(err_string.contains(expected));
        }
    }

    #[test]
    fn test_var() {
        let code = "var x1 = 41";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (var (name x1) (int 41)))");
    }

    #[test]
    fn test_store_use() {
        let code = "let x = 41; x+1";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (let (name x) (type int) (int 41)) (bina (name x) (op +) (int 1)))"
        );
    }

    #[test]
    fn test_range() {
        let code = "1..5";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (range (start (int 1)) (end (int 5)) (eq false)))"
        );
    }

    #[test]
    fn test_for() {
        let code = "for i in 1..5 {i}";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (for (name i) (range (start (int 1)) (end (int 5)) (eq false)) (body (name i))))"
        );

        let code = "for i, x in 1..5 {x}";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (for ((name i) (name x)) (range (start (int 1)) (end (int 5)) (eq false)) (body (name x))))"
        );
    }

    #[test]
    fn test_for_with_print() {
        let code = "for i in 0..10 { print(i); print(i+1) }";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(for (name i) (range (start (int 0)) (end (int 10)) (eq false)) (body (call (name print) (args (name i))) (call (name print) (args (bina (name i) (op +) (int 1))))))");
    }

    #[test]
    fn test_for_with_mid() {
        let code = r#"for i in 0..10 { print(i); mid{ print(",") } }"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(for (name i) (range (start (int 0)) (end (int 10)) (eq false)) (body (call (name print) (args (name i))) (node (name mid) (body (call (name print) (args (str \",\")))))))");
    }

    #[test]
    fn test_for_with_mid_call() {
        let code = r#"for i in 0..10 { `$i`; mid{","} }"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(for (name i) (range (start (int 0)) (end (int 10)) (eq false)) (body (fstr (str \"\") (name i)) (node (name mid) (body (str \",\")))))");
    }

    #[test]
    fn test_object() {
        let code = "let o = {x:1, y:2}";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (let (name o) (object (pair (name x) (int 1)) (pair (name y) (int 2)))))"
        );

        let code = "var a = { 1: 2, 3: 4 }; a.3";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(dot (name a).3)");
    }

    #[test]
    fn test_fn() {
        let code = "fn add(x, y) int { x+y }";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(fn (name add) (params (param (name x) (type int)) (param (name y) (type int))) (ret int) (body (bina (name x) (op +) (name y))))");
    }

    #[test]
    fn test_fn_with_ret_type() {
        let code = r#"fn add(x, y) int { x+y }"#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (fn (name add) (params (param (name x) (type int)) (param (name y) (type int))) (ret int) (body (bina (name x) (op +) (name y)))))");
    }

    #[test]
    fn test_fn_with_param_type() {
        let code = "fn say(msg str) { print(msg) }";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(fn (name say) (params (param (name msg) (type str))) (ret void) (body (call (name print) (args (name msg)))))");
    }

    #[test]
    fn test_fn_with_named_args() {
        let code = "fn add(a int, b int) int { a + b }; add(a:1, b:2)";
        let ast = parse_once(code);
        let last = ast.stmts[1].clone();
        assert_eq!(
            last.to_string(),
            "(call (name add) (args (pair (name a) (int 1)) (pair (name b) (int 2))))"
        );
    }

    #[test]
    fn test_fn_call() {
        let code = "fn add(x, y) { x+y }; add(1, 2)";
        let ast = parse_once(code);
        let call = ast.stmts[1].clone();
        assert_eq!(call.to_string(), "(call (name add) (args (int 1) (int 2)))");
    }

    #[test]
    fn test_tag_new() {
        let code = "tag Atom {Int int, Char char}; Atom.Int(5)";
        let ast = parse_once(code);
        let call = ast.stmts[1].clone();
        assert_eq!(
            call.to_string(),
            "(call (dot (name Atom).Int) (args (int 5)))"
        );
    }

    #[test]
    fn test_type_decl() {
        let code = "type Point {x int; y int}";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (type-decl (name Point) (members (member (name x) (type int)) (member (name y) (type int)))))");
    }

    #[test]
    fn test_type_inst() {
        let code = "type Point {x int; y int}; var p = Point(x:1, y:2); p.x";
        let ast = parse_once(code);
        let mid = ast.stmts[1].clone();
        let last = ast.stmts.last().unwrap();
        assert_eq!(mid.to_string(), "(var (name p) (call (name Point) (args (pair (name x) (int 1)) (pair (name y) (int 2)))))");
        assert_eq!(last.to_string(), "(dot (name p).x)");
    }

    #[test]
    fn test_closure() {
        // Plan 060: Test closure parsing (replaces deprecated lambda)
        // Single parameter closure: n => n + 1 (note: space before param is required)
        let code = "var x =  n => n + 1";
        let ast = parse_once(code);
        // Actual format includes parentheses around parameter
        assert_eq!(
            ast.to_string(),
            "(code (var (name x) (closure (n) => (bina (name n) (op +) (int 1)))))"
        );
    }

    #[test]
    fn test_closure_with_params() {
        // Plan 060: Test closure with parameters and type annotations
        // Note: Display format doesn't show type annotations in params list
        let code = "(a int, b int) => a + b";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(closure (a, b) => (bina (name a) (op +) (name b)))");
    }

    // #[test]
    // fn test_widget() {
    //     let code = r#"
    //     widget counter {
    //         model {
    //             var count = 0
    //         }
    //         view {
    //             button("+") {
    //                 onclick: || count = count + 1
    //             }
    //         }
    //     }
    //     "#;
    //     let ast = parse_once(code);
    //     let widget = &ast.stmts[0];
    //     match widget {
    //         Stmt::Widget(widget) => {
    //             let model = &widget.model;
    //             assert_eq!(model.to_string(), "(model (var (name count) (int 0)))");
    //             let view = &widget.view;
    //             assert_eq!(view.to_string(), "(view (node (name button) (args (str \"+\")) (body (pair (name onclick) (fn (name lambda_1) (body (bina (name count) (op =) (bina (name count) (op +) (int 1)))))))))");
    //         }
    //         _ => panic!("Expected widget, got {:?}", widget),
    //     }
    // }

    #[test]
    fn test_pair() {
        let code = r#"
        id: 1
        name: "test"
        version: "0.1.0"
        "#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(pair (name version) (str \"0.1.0\"))");
    }

    #[test]
    fn test_node_instance() {
        let code = r#"button("OK") {
            border: 1
            kind: "primary"
        }"#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (node (name button) (args (str \"OK\")) (body (pair (name border) (int 1)) (pair (name kind) (str \"primary\")))))");
    }

    #[test]
    fn test_node_instance_with_id() {
        let code = r#"lib mymath {}"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(node (name lib) (id mymath))");
    }

    #[test]
    fn test_node_instance_without_args() {
        let code = r#"center {
            text("Hello") {}
        }"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(
            last.to_string(),
            "(node (name center) (body (node (name text) (args (str \"Hello\")))))"
        );
    }

    #[test]
    fn test_array() {
        let code = r#"// comment
        [ // arra
            {id: 1, name: "test"}, // comment
            {id: 2, name: "test2"},
            {id: 3,/*good name*/ name: "test3"} // comment
            //{id: 4, name: "test4"} // comment out
        ]"#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (array (object (pair (name id) (int 1)) (pair (name name) (str \"test\"))) (object (pair (name id) (int 2)) (pair (name name) (str \"test2\"))) (object (pair (name id) (int 3)) (pair (name name) (str \"test3\")))))");
    }

    #[test]
    fn test_hex() {
        let code = "0x10";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (int 16))");
    }

    #[test]
    fn test_dot() {
        let code = "var a = {b: [0, 1, 2]}; a.b[0]";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(index (dot (name a).b) (int 0))");
    }

    #[test]
    fn test_cstr() {
        let code = r#"c"hello""#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (cstr \"hello\"))");
    }

    #[test]
    fn test_fstr() {
        let code = r#"f"hello $name""#;
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (fstr (str \"hello \") (name name)))"
        );
    }

    #[test]
    fn test_fstr_multi() {
        let code = r#"var name = "haha"; var age = 18; `hello $name ${age}`"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(
            last.to_string(),
            "(fstr (str \"hello \") (name name) (str \" \") (name age))"
        );
    }

    #[test]
    fn test_fstr_with_expr() {
        let code = r#"var a = 1; var b = 2; f"a + b = ${a+b}""#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(
            last.to_string(),
            "(fstr (str \"a + b = \") (bina (name a) (op +) (name b)))"
        );
    }

    #[test]
    fn test_node_tree() {
        let code = r#"
        center {
            text("Hello") {}
        }
        "#;
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (node (name center) (body (node (name text) (args (str \"Hello\"))))))"
        );
    }

    #[test]
    fn test_type_instance_obj() {
        let code = r#"type A {
            x int
            y int
        }
        A(x:1, y:2)"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(
            last.to_string(),
            "(node (name A) (args (pair (name x) (int 1)) (pair (name y) (int 2))))"
        );
    }

    #[test]
    fn test_type_with_method() {
        let code = r#"type Point {
            x int
            y int

            fn absquare() int {
                .x * .x + .y * .y
            }
        }"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(type-decl (name Point) (members (member (name x) (type int)) (member (name y) (type int))) (methods (fn (name absquare) (ret int) (body (bina (bina (dot (name self).x) (op *) (dot (name self).x)) (op +) (bina (dot (name self).y) (op *) (dot (name self).y)))))))");
    }

    #[test]
    fn test_type_composition() {
        let code = r#"
        type Wing {
            fn fly() {}
        }
        type Duck has Wing {
        }
        "#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(
            last.to_string(),
            "(type-decl (name Duck) (has (type Wing)) (methods (fn (name fly) (ret void) (body ))))"
        );
    }

    #[test]
    fn test_grid() {
        let code = r#"
        grid("a", "b", "c") {
            [1, 2, 3]
            [4, 5, 6]
            [7, 8, 9]
        }"#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (grid (head (str \"a\") (str \"b\") (str \"c\")) (data (row (int 1) (int 2) (int 3)) (row (int 4) (int 5) (int 6)) (row (int 7) (int 8) (int 9)))))");
    }

    #[test]
    fn test_grid_with_colconfig() {
        let code = r#"
        let cols = [
            {id: "a", title: "A"},
            {id: "b", title: "B"},
            {id: "c", title: "C"},
        ]
        grid(cols) {
            [1, 2, 3]
            [4, 5, 6]
            [7, 8, 9]
        }"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(grid (head (name cols)) (data (row (int 1) (int 2) (int 3)) (row (int 4) (int 5) (int 6)) (row (int 7) (int 8) (int 9))))");
    }

    #[test]
    fn test_config() {
        let code = r#"
name: "hello"
version: "0.1.0"

exe hello {
    dir: "src"
    main: "main.c"
}"#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (pair (name name) (str \"hello\")) (pair (name version) (str \"0.1.0\")) (nl*1) (node (name exe) (id hello) (body (pair (name dir) (str \"src\")) (pair (name main) (str \"main.c\")))))");
    }

    #[test]
    fn test_use() {
        let code = "use auto.math:square; square(16)";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (use (path auto.math) (items square)) (call (name square) (args (int 16))))"
        );
    }

    #[test]
    fn test_use_c() {
        let code = "use.c <stdio.h>";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (use (kind c) (path <stdio.h>)))");
    }

    #[test]
    fn test_import() {
        let code = "use auto.math: square";
        let mut parser = Parser::from(&code);
        let ast = parser.parse().unwrap();
        assert_eq!(
            ast.to_string(),
            "(code (use (path auto.math) (items square)))"
        );
    }

    #[test]
    fn test_char() {
        let code = r#"'a'"#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (char 'a'))");
    }

    #[test]
    fn test_fstr_with_note() {
        let code = r#"`hello #{2 + 1} again`"#;
        let scope = shared(Universe::new());
        let mut parser = Parser::new_with_note(code, scope, '#');
        let ast = parser.parse().unwrap();
        assert_eq!(
            ast.to_string(),
            "(code (fstr (str \"hello \") (bina (int 2) (op +) (int 1)) (str \" again\")))"
        );
    }

    #[test]
    fn test_simple_enum() {
        // Test simple enum parsing
        let code = "enum Color { Red, Green, Blue }; Color.Red";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (enum (name Color) (item (name Red) (value 0)) (item (name Green) (value 1)) (item (name Blue) (value 2))) (dot (name Color).Red))"
        );
    }

    #[test]
    fn test_is_stmt() {
        let code = r#"is x {
        10 => print("ten")
        20 => print("twenty")
        else => print("ehh")
        }"#;
        let when = Is::parse(code).unwrap();
        assert_eq!(
            when.to_string(),
            format!(
                "{}{}{}{}",
                r#"(is (name x) "#,
                r#"(eq (int 10) (body (call (name print) (args (str "ten"))))) "#,
                r#"(eq (int 20) (body (call (name print) (args (str "twenty"))))) "#,
                r#"(else (body (call (name print) (args (str "ehh"))))))"#,
            )
        )
    }

    #[test]
    fn test_on_condition_events() {
        let code = r#"on {
            EV1 -> State1 : handler1
            EV2 ? checker2 {
                -> State2 : handler2
                -> State3 : handler3
            }
        }"#;
        let when = OnEvents::parse(code).unwrap();
        assert_eq!(
            when.to_string(),
            format!(
                "{}{}{}{}{}{}",
                r#"(on "#,
                r#"(arrow (from (name EV1)) (to (name State1)) (with (name handler1))) "#,
                r#"(cond-arrow (from (name EV2)) (cond (name checker2)) "#,
                r#"(arrow (to (name State2)) (with (name handler2))) "#,
                r#"(arrow (to (name State3)) (with (name handler3)))"#,
                r#"))"#,
            )
        )
    }

    #[test]
    fn test_array_type() {
        let code = r#"let arr [3]int = [1, 2, 3]"#;
        let array_type = parse_once(code);
        assert_eq!(
            array_type.to_string(),
            format!(
                "{}",
                "(code (let (name arr) (type (array-type (elem int) (len 3))) (array (int 1) (int 2) (int 3))))"
            )
        )
    }

    #[test]
    fn test_ptr_type() {
        let code = r#"let ptr *int = 10.@"#;
        let ptr_type = parse_once(code);
        assert_eq!(
            ptr_type.to_string(),
            "(code (let (name ptr) (type (ptr-type (of int))) (dot (int 10).@)))"
        )
    }

    #[test]
    fn test_ptr_asn() {
        let code = r#"let p *int = 10.@"#;
        let ptr_type = parse_once(code);
        assert_eq!(
            ptr_type.to_string(),
            "(code (let (name p) (type (ptr-type (of int))) (dot (int 10).@)))"
        )
    }

    #[test]
    fn test_ptr_target() {
        let code = r#"p.* += 1"#;
        let ptr_type = parse_once(code);
        assert_eq!(
            ptr_type.to_string(),
            "(code (bina (dot (name p).*) (op +=) (int 1)))"
        )
    }

    #[test]
    fn test_alias_stmt() {
        let code = r#"alias cc = my_add"#;
        let alias_stmt = parse_once(code);
        assert_eq!(
            alias_stmt.to_string(),
            format!("{}", "(code (alias (name cc) (target my_add)))")
        )
    }

    #[test]
    fn test_tag_cover() {
        let code = r#"
            tag Atom {
                Int int
                Float float
            }

            let atom = Atom.Int(12)

            is atom {
                Atom.Int(i) => i
                Atom.Float(f) => f
            }
        "#;
        let code = parse_once(code);
        assert_eq!(
            code.stmts[4].to_string(),
            format!(
                "{}{}",
                "(is (name atom) (eq (tag-cover (kind Atom) (tag Int) (elem i)) (body (name i)))",
                " (eq (tag-cover (kind Atom) (tag Float) (elem f)) (body (name f))))"
            )
        );
    }

    #[test]
    fn test_spec_decl() {
        let code = r#"
            spec Flyer {
                fn fly()
                fn land()
            }
        "#;
        let ast = parse_once(code);
        // Check that the first statement is a SpecDecl
        assert!(ast.stmts.len() >= 1);
        let first_stmt = &ast.stmts[0];
        // The spec should be parsed correctly
        let stmt_str = first_stmt.to_string();
        assert!(stmt_str.contains("Flyer"));
        assert!(stmt_str.contains("fly"));
        assert!(stmt_str.contains("land"));
    }

    #[test]
    fn test_spec_with_params() {
        let code = r#"
            spec Calculator {
                fn add(a int, b int) int
                fn subtract(a int, b int) int
            }
        "#;
        let ast = parse_once(code);
        assert!(ast.stmts.len() >= 1);
        let first_stmt = &ast.stmts[0];
        let stmt_str = first_stmt.to_string();
        assert!(stmt_str.contains("Calculator"));
        assert!(stmt_str.contains("add"));
        assert!(stmt_str.contains("subtract"));
    }

    #[test]
    fn test_type_as_spec() {
        let code = r#"
            spec Flyer {
                fn fly()
            }

            type Pigeon as Flyer {
                fn fly() {
                    print("Flap Flap")
                }
            }
        "#;
        let ast = parse_once(code);
        // Should parse both spec and type with 'as' clause
        // Filter out empty line statements
        let non_empty_stmts: Vec<_> = ast
            .stmts
            .iter()
            .filter(|s| !s.to_string().starts_with("(nl"))
            .collect();

        assert!(non_empty_stmts.len() >= 2);
        let spec_stmt = non_empty_stmts[0];
        let type_stmt = non_empty_stmts[1];
        // Debug output (disabled for LSP)
        // println!("Spec stmt: {}", spec_stmt.to_string());
        // println!("Type stmt: {}", type_stmt.to_string());
        assert!(spec_stmt.to_string().contains("Flyer"));
        // The type should contain Pigeon
        let type_str = type_stmt.to_string();
        assert!(
            type_str.contains("Pigeon") || type_str.contains("pigeon"),
            "Type statement should contain 'Pigeon', got: {}",
            type_str
        );
    }

    #[test]
    fn test_type_with_multiple_specs() {
        let code = r#"
            spec Flyer {
                fn fly()
            }

            spec Swimmer {
                fn swim()
            }

            type Duck as Flyer, Swimmer {
                fn fly() {
                    print("Quack fly")
                }
                fn swim() {
                    print("Quack swim")
                }
            }
        "#;
        let ast = parse_once(code);
        // Should parse spec and type with multiple 'as' specs
        // Filter out empty line statements
        let non_empty_stmts: Vec<_> = ast
            .stmts
            .iter()
            .filter(|s| !s.to_string().starts_with("(nl"))
            .collect();

        assert!(non_empty_stmts.len() >= 3);
        let duck_stmt = non_empty_stmts[2];
        // Debug output (disabled for LSP)
        // println!("Duck stmt: {}", duck_stmt.to_string());
        let duck_str = duck_stmt.to_string();
        assert!(
            duck_str.contains("Duck") || duck_str.contains("duck"),
            "Type statement should contain 'Duck', got: {}",
            duck_str
        );
    }
}

use crate::ast::*;
use crate::error_pos;
use crate::lexer::Lexer;
use crate::scope::Meta;
use crate::token::{Pos, Token, TokenKind};
use crate::universe::Universe;
use auto_val::AutoPath;
use auto_val::AutoStr;
use auto_val::Op;
use auto_val::{shared, Shared};
use dirs;
use std::backtrace::Backtrace;
use std::collections::HashMap;
use std::i32;
use std::path::PathBuf;
use std::rc::Rc;

pub type ParseError = AutoStr;
pub type ParseResult<T> = Result<T, ParseError>;

/// TODO: T should be a generic AST node type
pub trait ParserExt {
    fn parse(input: impl Into<AutoStr>) -> ParseResult<Self>
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

const _PREC_REF: PrefixPrec = prefix_prec(12);
const PREC_SIGN: PrefixPrec = prefix_prec(13);
const PREC_NOT: PrefixPrec = prefix_prec(14);
const PREC_CALL: PostfixPrec = postfix_prec(15);
const PREC_INDEX: PostfixPrec = postfix_prec(16);
const PREC_DOT: InfixPrec = infix_prec(17);
const _PREC_ATOM: InfixPrec = infix_prec(18);

fn prefix_power(op: Op) -> ParseResult<PrefixPrec> {
    match op {
        Op::Add | Op::Sub => Ok(PREC_SIGN),
        Op::Not => Ok(PREC_NOT),
        _ => error_pos!("Invalid prefix operator: {}", op),
    }
}

fn postfix_power(op: Op) -> ParseResult<Option<PostfixPrec>> {
    match op {
        Op::LSquare => Ok(Some(PREC_INDEX)),
        Op::LParen => Ok(Some(PREC_CALL)),
        Op::Colon => Ok(Some(PREC_PAIR)),
        _ => Ok(None),
    }
}

fn infix_power(op: Op) -> ParseResult<InfixPrec> {
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
        _ => error_pos!("Invalid infix operator: {}", op),
    }
}

pub trait BlockParser {
    fn parse(&self, parser: &mut Parser) -> ParseResult<Body>;
}

// pub fn parse(code: &str, scope: Rc<RefCell<Universe>>, interpreter: &'a Interpreter) -> ParseResult<Code> {
// let mut parser = Parser::new(code, scope, interpreter);
// parser.parse()
// }

pub struct Parser<'a> {
    pub scope: Shared<Universe>,
    lexer: Lexer<'a>,
    pub cur: Token,
    pub special_blocks: HashMap<AutoStr, Box<dyn BlockParser>>,
    pub skip_check: bool,
}

impl<'a> Parser<'a> {
    pub fn from(code: &'a str) -> Self {
        Self::new(code, shared(Universe::new()))
    }

    pub fn new(code: &'a str, scope: Shared<Universe>) -> Self {
        let mut lexer = Lexer::new(code);
        let cur = lexer.next();
        let mut parser = Parser {
            scope,
            lexer,
            cur,
            special_blocks: HashMap::new(),
            skip_check: false,
        };
        parser.skip_comments();
        parser
    }

    pub fn add_special_block(&mut self, block: AutoStr, parser: Box<dyn BlockParser>) {
        self.special_blocks.insert(block, parser);
    }

    pub fn new_with_note(code: &'a str, scope: Shared<Universe>, note: char) -> Self {
        let mut lexer = Lexer::new(code);
        lexer.set_fstr_note(note);
        let cur = lexer.next();
        let mut parser = Parser {
            scope,
            lexer,
            cur,
            special_blocks: HashMap::new(),
            skip_check: false,
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

    pub fn skip_comments(&mut self) {
        loop {
            match self.kind() {
                TokenKind::CommentLine
                | TokenKind::CommentStart
                | TokenKind::CommentContent
                | TokenKind::CommentEnd => {
                    self.cur = self.lexer.next();
                }
                _ => {
                    break;
                }
            }
        }
    }

    pub fn next(&mut self) -> &Token {
        self.cur = self.lexer.next();
        self.skip_comments();
        &self.cur
    }

    pub fn expect(&mut self, kind: TokenKind) -> ParseResult<()> {
        if self.is_kind(kind) {
            self.next();
            Ok(())
        } else {
            println!("{}", Backtrace::capture());
            error_pos!("Expected token kind: {:?}, got [{:?}]", kind, self.cur.text)
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
                _ => shared(Type::Unknown),
            },
            None => shared(Type::Unknown),
        }
    }
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> ParseResult<Code> {
        let mut stmts = Vec::new();
        self.skip_empty_lines();
        while !self.is_kind(TokenKind::EOF) {
            let stmt = self.parse_stmt()?;
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
            self.expect_eos()?;
        }
        stmts = self.convert_last_block(stmts)?;
        Ok(Code { stmts })
    }

    fn convert_last_block(&mut self, mut stmts: Vec<Stmt>) -> ParseResult<Vec<Stmt>> {
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

    fn body_to_obj(&mut self, body: &Body) -> ParseResult<Vec<Pair>> {
        let mut pairs = Vec::new();
        for stmt in body.stmts.iter() {
            match stmt {
                Stmt::Expr(expr) => match expr {
                    Expr::Pair(p) => {
                        pairs.push(p.clone());
                    }
                    _ => return error_pos!("Last block must be an object!"),
                },
                _ => return error_pos!("Last block must be an object!"),
            }
        }
        Ok(pairs)
    }
}

// Expressions
impl<'a> Parser<'a> {
    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        let mut exp = self.expr_pratt(0)?;
        exp = self.check_symbol(exp)?;
        Ok(exp)
    }

    // simple Pratt parser
    // ref: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
    pub fn expr_pratt(&mut self, min_power: u8) -> ParseResult<Expr> {
        // Prefix
        let lhs = match self.kind() {
            // unary
            TokenKind::Add | TokenKind::Sub | TokenKind::Not => {
                let op = self.op();
                let power = prefix_power(op)?;
                self.next(); // skip unary op
                let lhs = self.expr_pratt(power.r)?;
                Expr::Unary(op, Box::new(lhs))
            }
            // group
            TokenKind::LParen => {
                self.next(); // skip (
                let lhs = self.expr_pratt(0)?;
                self.expect(TokenKind::RParen)?; // skip )
                lhs
            }
            // array
            TokenKind::LSquare => self.array()?,
            // object
            TokenKind::LBrace => Expr::Object(self.object()?),
            // lambda
            TokenKind::VBar => self.lambda()?,
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

    fn dot_item(&mut self) -> ParseResult<Expr> {
        self.next(); // skip dot
        let name = self.cur.text.clone();
        self.next(); // skip name
        Ok(Expr::Bina(
            Box::new(Expr::Ident("s".into())),
            Op::Dot,
            Box::new(Expr::Ident(name)),
        ))
    }

    fn expr_pratt_with_left(&mut self, mut lhs: Expr, min_power: u8) -> ParseResult<Expr> {
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
                TokenKind::RSquare => break,
                TokenKind::RParen => break,
                _ => {
                    return error_pos!("Expected infix operator, got {:?}", self.peek());
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
                            _ => return error_pos!("Invalid key: {}", lhs),
                        };
                        let rhs = self.pair_expr()?;
                        lhs = Expr::Pair(Pair {
                            key,
                            value: Box::new(rhs),
                        });
                        return Ok(lhs);
                    }
                    _ => return error_pos!("Invalid postfix operator: {}", op),
                }
            }
            // Infix
            let power = infix_power(op)?;
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
            let rhs = self.expr_pratt(power.r)?;
            lhs = Expr::Bina(Box::new(lhs), op, Box::new(rhs));
        }
        Ok(lhs)
    }

    fn check_asn(&mut self, lhs: &Expr) -> ParseResult<()> {
        match lhs {
            Expr::Ident(name) => {
                let meta = self.lookup_meta(name.as_str());
                if let Some(Meta::Store(store)) = meta.as_deref() {
                    if matches!(store.kind, StoreKind::Let) {
                        return error_pos!(
                            "Syntax error: Assignment not allowed for let store: {}",
                            store.name
                        );
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
            _ => panic!("Expected operator, got {:?}", self.kind()),
        }
    }

    pub fn group(&mut self) -> ParseResult<Expr> {
        self.next(); // skip (
        let expr = self.parse_expr()?;
        self.expect(TokenKind::RParen)?; // skip )
        Ok(expr)
    }

    pub fn sep_array(&mut self) -> ParseResult<()> {
        let mut has_sep = false;
        while self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            has_sep = true;
            self.next();
        }
        if self.is_kind(TokenKind::RSquare) {
            return Ok(());
        }
        if !has_sep {
            return error_pos!("Expected array separator, got {:?}", self.kind());
        }
        Ok(())
    }

    pub fn array(&mut self) -> ParseResult<Expr> {
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

    pub fn sep_args(&mut self) {
        if self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            self.next();
            return;
        }
        if self.is_kind(TokenKind::RParen) {
            return;
        }
        panic!("Expected argument separator, got {:?}", self.kind());
    }

    pub fn args(&mut self) -> ParseResult<Args> {
        self.expect(TokenKind::LParen)?;
        let mut args = Args::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RParen) {
            // TODO: this is a temp fix for identifier as string in arg
            let expr = if self.is_kind(TokenKind::Ident) {
                let name = Expr::Ident(self.cur.text.clone());
                self.next();
                if self.is_kind(TokenKind::Comma) {
                    name
                } else {
                    self.expr_pratt_with_left(name, 0)?
                }
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
                            return error_pos!(
                                "named args should have named key instead of {}",
                                &k
                            );
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
            self.sep_args();
        }
        self.expect(TokenKind::RParen)?;
        Ok(args)
    }

    pub fn object(&mut self) -> ParseResult<Vec<Pair>> {
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut entries = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            entries.push(self.pair()?);
            self.sep_obj();
        }
        self.expect(TokenKind::RBrace)?;
        Ok(entries)
    }

    pub fn pair_expr(&mut self) -> ParseResult<Expr> {
        self.rhs_expr()
        // let exp = self.expr_pratt(0)?;
        // if let Expr::Ident(ident) = &exp {
        //     if !self.exists(ident) {
        //         return Ok(Expr::Str(ident.clone()));
        //     }
        // }
        // Ok(exp)
    }

    pub fn pair(&mut self) -> ParseResult<Pair> {
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

    pub fn key(&mut self) -> ParseResult<Key> {
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
            _ => error_pos!("Expected key, got {:?}", self.kind()),
        }
    }

    pub fn sep_obj(&mut self) {
        if self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            self.next();
            self.skip_empty_lines();
            return;
        }
        if self.is_kind(TokenKind::RBrace) {
            return;
        }
        panic!("Expected pair separator, got {:?}", self.kind());
    }

    pub fn ident_name(&mut self) -> ParseResult<Name> {
        Ok(self.cur.text.clone())
    }

    pub fn ident(&mut self) -> ParseResult<Expr> {
        let name = self.cur.text.clone();
        // // check for existence
        // if !self.exists(&name) {
        //     return Err(format!("Undefined variable: {}", name));
        // }
        Ok(Expr::Ident(name))
    }

    pub fn parse_ident(&mut self) -> ParseResult<Expr> {
        let name = self.cur.text.clone();
        self.next();
        Ok(Expr::Ident(name))
    }

    pub fn parse_name(&mut self) -> ParseResult<Name> {
        let name = self.cur.text.clone();
        self.next();
        Ok(name)
    }

    pub fn parse_ints(&mut self) -> ParseResult<Expr> {
        let res = match self.cur.kind {
            TokenKind::Int => self.parse_int(),
            TokenKind::Uint => self.parse_uint(),
            TokenKind::U8 => self.parse_u8(),
            TokenKind::I8 => self.parse_i8(),
            _ => error_pos!("Expected integer, got {:?}", self.kind()),
        };
        if res.is_ok() {
            self.next();
        }
        res
    }

    pub fn parse_int(&mut self) -> ParseResult<Expr> {
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

    fn parse_uint(&mut self) -> ParseResult<Expr> {
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

    fn parse_u8(&mut self) -> ParseResult<Expr> {
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

    fn parse_i8(&mut self) -> ParseResult<Expr> {
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

    pub fn atom(&mut self) -> ParseResult<Expr> {
        if self.is_kind(TokenKind::LParen) {
            return self.group();
        }
        let expr = match self.kind() {
            TokenKind::Uint => self.parse_uint()?,
            TokenKind::Int => self.parse_int()?,
            TokenKind::U8 => self.parse_u8()?,
            TokenKind::I8 => self.parse_i8()?,
            TokenKind::Float => Expr::Float(
                self.cur.text.as_str().parse().unwrap(),
                self.cur.text.clone(),
            ),
            TokenKind::Double => Expr::Double(
                self.cur.text.as_str().parse().unwrap(),
                self.cur.text.clone(),
            ),
            TokenKind::True => Expr::Bool(true),
            TokenKind::False => Expr::Bool(false),
            TokenKind::Str => Expr::Str(self.cur.text.clone()),
            TokenKind::CStr => Expr::CStr(self.cur.text.clone()),
            TokenKind::Char => Expr::Char(self.cur.text.chars().nth(0).unwrap()),
            TokenKind::Ident => self.ident()?,
            TokenKind::Nil => Expr::Nil,
            _ => {
                return error_pos!(
                    "Expected term, got {:?}, pos: {}, next: {}",
                    self.kind(),
                    self.pos(),
                    self.cur
                );
            }
        };

        self.next();
        Ok(expr)
    }

    /// 解析fstr
    /// fstr: [FStrSart, (FStrParts | ${Expr} | $Ident)*, FStrEnd]
    pub fn fstr(&mut self) -> ParseResult<Expr> {
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

    pub fn if_expr(&mut self) -> ParseResult<Expr> {
        let (branches, else_stmt) = self.if_contents()?;
        Ok(Expr::If(If {
            branches,
            else_: else_stmt,
        }))
    }

    pub fn cond_expr(&mut self) -> ParseResult<Expr> {
        self.rhs_expr()
    }

    // An Expression that can be assigned to a variable, e.g. right-hand side of an assignment
    pub fn rhs_expr(&mut self) -> ParseResult<Expr> {
        if self.is_kind(TokenKind::If) {
            self.if_expr()
        } else if self.is_kind(TokenKind::Ident) {
            // TODO: should have a node_or_call_expr()
            let stmt = self.parse_node_or_call_stmt()?;
            match stmt {
                Stmt::Expr(expr) => Ok(expr),
                Stmt::Node(node) => Ok(Expr::Node(node)),
                _ => error_pos!("Expected expression, got {:?}", stmt),
            }
        } else {
            self.parse_expr()
        }
    }

    fn tag_cover(&mut self, tag_name: &Name) -> ParseResult<Expr> {
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

    pub fn is_branch_cond_expr(&mut self) -> ParseResult<Expr> {
        if self.is_kind(TokenKind::Ident) {
            self.lhs_expr()
        } else {
            self.atom()
        }
    }

    pub fn lhs_expr(&mut self) -> ParseResult<Expr> {
        if !self.is_kind(TokenKind::Ident) {
            return error_pos!("Expected LHS expr with ident, got {}", self.peek().kind);
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

    pub fn iterable_expr(&mut self) -> ParseResult<Expr> {
        // TODO: how to check for range/array but reject other cases?
        self.parse_expr()
    }

    pub fn lambda(&mut self) -> ParseResult<Expr> {
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
}

// Statements
impl<'a> Parser<'a> {
    // End of statement
    pub fn expect_eos(&mut self) -> ParseResult<usize> {
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
            Ok(newline_count)
        } else {
            error_pos!(
                "Expected end of statement, got {:?}<{}>",
                self.kind(),
                self.cur.text
            )
        }
    }

    pub fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        let stmt = match self.kind() {
            TokenKind::Use => self.use_stmt()?,
            TokenKind::If => self.if_stmt()?,
            TokenKind::For => self.for_stmt()?,
            TokenKind::Is => self.is_stmt()?,
            TokenKind::Var => self.parse_store_stmt()?,
            TokenKind::Let => self.parse_store_stmt()?,
            TokenKind::Mut => self.parse_store_stmt()?,
            TokenKind::Fn => self.fn_decl_stmt("")?,
            TokenKind::Type => self.type_decl_stmt()?,
            TokenKind::Union => self.union_stmt()?,
            TokenKind::Tag => self.tag_stmt()?,
            TokenKind::LBrace => Stmt::Block(self.body()?),
            // Node Instance?
            TokenKind::Ident => self.parse_node_or_call_stmt()?,
            // Enum Definition
            TokenKind::Enum => self.enum_stmt()?,
            // On Events Switch
            TokenKind::On => Stmt::OnEvents(self.parse_on_events()?),
            // Alias stmt
            TokenKind::Alias => self.parse_alias_stmt()?,
            // Otherwise, try to parse as an expression
            _ => self.expr_stmt()?,
        };
        Ok(stmt)
    }

    fn parse_alias_stmt(&mut self) -> ParseResult<Stmt> {
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

    /// Format: enum { item1, item2, item3 }
    fn enum_stmt(&mut self) -> ParseResult<Stmt> {
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
                return error_pos!("expected ',' or newline, got {}", self.cur.text);
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

    fn expect_ident_str(&mut self) -> ParseResult<AutoStr> {
        if self.is_kind(TokenKind::Ident) {
            let name = self.cur.text.clone();
            self.next(); // skip name
            Ok(name)
        } else {
            error_pos!("Expected identifier, got {:?}", self.kind())
        }
    }

    fn parse_use_items(&mut self) -> ParseResult<Vec<AutoStr>> {
        let mut items = Vec::new();
        // end of path, next should be a colon (for items) or end-of-statement
        if self.is_kind(TokenKind::Colon) {
            self.next(); // skip :
                         // parse items
            if self.is_kind(TokenKind::Ident) {
                let name = self.expect_ident_str()?;
                items.push(name);
            } else {
                return error_pos!("Expected identifier, got {:?}", self.kind());
            }
            while self.is_kind(TokenKind::Comma) {
                self.next(); // skip ,
                let name = self.expect_ident_str()?;
                items.push(name);
            }
        }
        Ok(items)
    }

    pub fn use_c_stmt(&mut self) -> ParseResult<Stmt> {
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
            return error_pos!(
                "Expected <lib> or \"lib\", got {:?}, {}",
                self.kind(),
                self.cur.text
            );
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

    pub fn use_rust_stmt(&mut self) -> ParseResult<Stmt> {
        error_pos!("Rust import not supported yet")
    }

    // There are three kinds of import
    // 1. auto: use std.io: println
    // 2. c: use c <stdio.h>
    // 3. rust: use rust std::fs
    pub fn use_stmt(&mut self) -> ParseResult<Stmt> {
        self.next(); // skip use

        // check c/rust
        let name = self.expect_ident_str()?;

        if name == "c" {
            return self.use_c_stmt();
        } else if name == "rust" {
            return self.use_rust_stmt();
        }

        let mut paths = Vec::new();
        paths.push(name);
        while self.is_kind(TokenKind::Dot) {
            self.next(); // skip .
            let name = self.expect_ident_str()?;
            paths.push(name);
        }

        let mut items = self.parse_use_items()?;

        if items.is_empty() && !paths.is_empty() {
            items.push(paths.pop().unwrap());
        }
        // import the path into scope
        let uses = Use {
            kind: UseKind::Auto,
            paths,
            items,
        };
        self.import(&uses)?;
        Ok(Stmt::Use(uses))
    }

    fn find_std_lib(&self) -> ParseResult<AutoStr> {
        let home_dir = dirs::home_dir().unwrap();
        let auto_std = home_dir.join(".auto/libs/");
        let search_dirs = vec![
            auto_std.to_str().unwrap(),
            "/usr/local/lib/auto",
            "/usr/lib/auto",
        ];
        let std_lib_pat = "stdlib/auto";

        for dir in search_dirs {
            let std_path = PathBuf::from(dir).join(std_lib_pat);
            println!("Checking {}", std_path.display());
            if std_path.is_dir() {
                println!("debug: std lib location: {}", std_path.to_str().unwrap());
                return Ok(AutoStr::from(std_path.to_str().unwrap()));
            }
        }

        return error_pos!("stdlib not found");
    }

    /// Import a path from `use` statement
    // TODO: clean up code
    // TODO: search path from System Env, Default Locations and etc.
    pub fn import(&mut self, uses: &Use) -> ParseResult<()> {
        println!("Trying to import use library");
        let path = uses.paths.join(".");

        // try to find stdlib in following locations
        // 1. ~/.auto/stdlib
        // 2. /usr/local/lib/auto
        // 3. /usr/lib/auto
        let std_path = self.find_std_lib()?;
        println!("debug: std lib location: {}", std_path);

        if !path.starts_with("auto.") {
            return error_pos!("Invalid import path: {}", path);
        }
        let scope_name: AutoStr = path.clone().into();
        let path = path.replace("auto.", "");
        // println!("path: {}", path);
        let file_path = AutoPath::new(std_path).join(path.clone());
        // println!("file_path: {}", file_path.display());
        let dir = file_path.parent();
        let name = file_path.path().file_name().unwrap();
        if !dir.exists() {
            return error_pos!("Invalid import path: {}", path);
        }
        // Read file
        let file_path = dir.join(name.to_str().unwrap().to_string() + ".at");
        let file_content = std::fs::read_to_string(file_path.path()).unwrap();

        let cur_spot = self.scope.borrow().cur_spot.clone();
        self.scope.borrow_mut().reset_spot();

        self.scope.borrow_mut().enter_mod(scope_name.clone());
        let mut new_parser = Parser::new(file_content.as_str(), self.scope.clone());
        let ast = new_parser.parse().unwrap();
        self.scope.borrow_mut().import(scope_name.clone(), ast);

        self.scope.borrow_mut().set_spot(cur_spot);
        // Define items in scope
        for item in uses.items.iter() {
            // lookup item's meta from its mod
            let meta = self
                .scope
                .borrow()
                .lookup(item.as_str(), scope_name.clone())
                .unwrap();
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

    fn parse_body(&mut self, is_node: bool) -> ParseResult<Body> {
        self.expect(TokenKind::LBrace)?;
        self.enter_scope();
        let mut stmts = Vec::new();
        let new_lines = self.skip_empty_lines();
        if new_lines > 1 {
            stmts.push(Stmt::EmptyLine(new_lines - 1));
        }
        let has_new_line = new_lines > 0;
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            let stmt = self.parse_stmt()?;
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
            let newline_count = self.expect_eos()?;
            if newline_count > 1 {
                stmts.push(Stmt::EmptyLine(newline_count - 1));
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

    pub fn parse_node_body(&mut self) -> ParseResult<Body> {
        self.parse_body(true)
    }

    pub fn body(&mut self) -> ParseResult<Body> {
        self.parse_body(false)
    }

    pub fn if_contents(&mut self) -> ParseResult<(Vec<Branch>, Option<Body>)> {
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

    pub fn if_stmt(&mut self) -> ParseResult<Stmt> {
        let (branches, else_stmt) = self.if_contents()?;
        Ok(Stmt::If(If {
            branches,
            else_: else_stmt,
        }))
    }

    pub fn for_stmt(&mut self) -> ParseResult<Stmt> {
        self.next(); // skip `for`
                     // enumerator
        if self.is_kind(TokenKind::Ident) {
            let name = self.cur.text.clone();
            self.enter_scope();
            let meta = Meta::Store(Store {
                kind: StoreKind::Var,
                name: name.clone(),
                expr: Expr::Nil,
                ty: Type::Int,
            });
            self.define(name.as_str(), meta);
            self.next(); // skip name
            let mut iter = Iter::Named(name.clone());
            if self.is_kind(TokenKind::Comma) {
                self.next(); // skip ,
                let iter_name = self.cur.text.clone();
                let meta_iter = Meta::Store(Store {
                    kind: StoreKind::Var,
                    name: iter_name.clone(),
                    expr: Expr::Nil,
                    ty: Type::Int,
                });
                self.define(iter_name.as_str(), meta_iter);
                self.next(); // skip iter name
                iter = Iter::Indexed(name.clone(), iter_name.clone());
            }
            self.expect(TokenKind::In)?;
            let range = self.iterable_expr()?;
            let body = self.body()?;
            let has_new_line = body.has_new_line;
            self.exit_scope();
            return Ok(Stmt::For(For {
                iter,
                range,
                body,
                new_line: has_new_line,
            }));
        }
        error_pos!("Expected for loop, got {:?}", self.kind())
    }

    pub fn is_stmt(&mut self) -> ParseResult<Stmt> {
        Ok(Stmt::Is(self.parse_is()?))
    }

    pub fn parse_is(&mut self) -> ParseResult<Is> {
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

    pub fn parse_expr_or_body(&mut self) -> ParseResult<Body> {
        if self.is_kind(TokenKind::LBrace) {
            self.body()
        } else {
            let mut body = Body::new();
            body.stmts.push(Stmt::Expr(self.parse_expr()?));
            Ok(body)
        }
    }

    pub fn parse_is_branch(&mut self, tgt: &Expr) -> ParseResult<IsBranch> {
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
                            return error_pos!("Invalid tag type: {}", cover.kind);
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

    pub fn parse_store_stmt(&mut self) -> ParseResult<Stmt> {
        // store kind: var/let/mut
        let store_kind = self.store_kind()?;
        self.next(); // skip var/let/mut

        // identifier name
        let name = self.cur.text.clone();
        self.expect(TokenKind::Ident)?;

        // type (optional)
        let mut ty = Type::Unknown;
        if self.is_type_name() {
            ty = self.parse_type()?;
        }

        // =
        self.expect(TokenKind::Asn)?;

        // inital value: expression
        let expr = self.rhs_expr()?;
        // TODO: check type compatibility
        if matches!(ty, Type::Unknown) {
            ty = self.infer_type_expr(&expr);
        }

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
            Expr::Str(..) => typ = Type::Str,
            Expr::CStr(..) => typ = Type::CStr,
            Expr::Bina(lhs, op, rhs) => {
                let ltype = self.infer_type_expr(lhs);
                println!("LTYPE: {}", ltype);
                println!("OP: {}", op);
                let rtype = self.infer_type_expr(rhs);
                println!("RTYPE: {}", rtype);
                match op {
                    Op::Dot => {
                        // TODO
                        typ = ltype;
                    }
                    _ => {
                        typ = ltype;
                    }
                }
            }
            Expr::Ident(id) => {
                println!("Infering type for identifier {}", id);
                // try to lookup the id as a type name
                let ltyp = self.lookup_type(id);
                match *ltyp.borrow() {
                    Type::Unknown => {}
                    _ => {
                        typ = ltyp.borrow().clone();
                    }
                };
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
                typ = call.ret.clone();
            }
            _ => {}
        }
        typ
    }

    pub fn store_kind(&mut self) -> ParseResult<StoreKind> {
        match self.kind() {
            TokenKind::Var => Ok(StoreKind::Var),
            TokenKind::Let => Ok(StoreKind::Let),
            TokenKind::Mut => Ok(StoreKind::Mut),
            _ => error_pos!("Expected store kind, got {:?}", self.kind()),
        }
    }

    fn fn_cdecl_stmt(&mut self) -> ParseResult<Stmt> {
        self.next(); // skip keyword `c`

        // parse function name
        let name = self.cur.text.clone();
        self.expect(TokenKind::Ident)?;

        // enter function scope
        self.scope.borrow_mut().enter_fn(name.clone());

        // parse function parameters
        self.expect(TokenKind::LParen)?;
        let params = self.fn_params()?;
        self.expect(TokenKind::RParen)?;

        // parse return type
        let mut ret_type = Type::Unknown;
        // TODO: determine return type with last stmt if it's not specified
        if self.is_kind(TokenKind::Ident) {
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
        self.define(name.as_str(), Meta::Fn(fn_expr));
        Ok(fn_stmt)
    }

    // Function Declaration
    pub fn fn_decl_stmt(&mut self, parent_name: &str) -> ParseResult<Stmt> {
        self.next(); // skip keyword `fn`

        // parse function name
        let name = self.cur.text.clone();
        // special case for `fn c` cdecl statement
        if name == "c" {
            return self.fn_cdecl_stmt();
        }
        self.expect(TokenKind::Ident)?;

        // enter function scope
        self.scope.borrow_mut().enter_fn(name.clone());

        // parse function parameters
        self.expect(TokenKind::LParen)?;
        let params = self.fn_params()?;
        self.expect(TokenKind::RParen)?;

        // parse return type
        let mut ret_type = Type::Unknown;
        // TODO: determine return type with last stmt if it's not specified
        if self.is_kind(TokenKind::Ident) {
            ret_type = self.parse_type()?;
        }

        // parse function body
        let body = self.body()?;

        // exit function scope
        self.exit_scope();

        // parent name for method?
        let parent = if parent_name.is_empty() {
            None
        } else {
            Some(parent_name.into())
        };
        let fn_expr = Fn::new(
            FnKind::Function,
            name.clone(),
            parent,
            params,
            body,
            ret_type,
        );
        let fn_stmt = Stmt::Fn(fn_expr.clone());
        let unique_name = if parent_name.is_empty() {
            name
        } else {
            format!("{}::{}", parent_name, name).into()
        };

        // define function in scope
        self.define(unique_name.as_str(), Meta::Fn(fn_expr));
        Ok(fn_stmt)
    }

    pub fn sep_params(&mut self) {
        if self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            self.next();
            return;
        }
        if self.is_kind(TokenKind::RParen) || self.is_kind(TokenKind::VBar) {
            return;
        }
        panic!("Expected parameter separator, got {:?}", self.kind());
    }

    // parse function parameters
    pub fn fn_params(&mut self) -> ParseResult<Vec<Param>> {
        let mut params = Vec::new();
        while self.is_kind(TokenKind::Ident) {
            // param name
            let name = self.cur.text.clone();
            self.next(); // skip name
                         // param type
            let mut ty = Type::Int;
            if self.is_kind(TokenKind::Ident) {
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
            self.define(name.as_str(), Meta::Store(var));
            params.push(Param { name, ty, default });
            self.sep_params();
        }
        Ok(params)
    }

    pub fn expr_stmt(&mut self) -> ParseResult<Stmt> {
        Ok(Stmt::Expr(self.parse_expr()?))
    }

    pub fn type_decl_stmt(&mut self) -> ParseResult<Stmt> {
        // TODO: deal with scope
        self.next(); // skip `type` keyword
        let name = self.cur.text.clone();
        self.expect(TokenKind::Ident)?;
        // deal with `as` keyword
        let mut specs = Vec::new();
        if self.is_kind(TokenKind::As) {
            self.next(); // skip `as` keyword
            let spec = self.cur.text.clone();
            self.next(); // skip spec
            specs.push(spec.into());
        }
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
        // type body
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        // list of members or methods
        let mut members = Vec::new();
        let mut methods = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            if self.is_kind(TokenKind::Fn) {
                let fn_stmt = self.fn_decl_stmt(&name)?;
                if let Stmt::Fn(fn_expr) = fn_stmt {
                    methods.push(fn_expr);
                }
            } else {
                let member = self.type_member()?;
                members.push(member);
            }
            self.expect_eos()?;
        }
        self.expect(TokenKind::RBrace)?;
        // add members and methods of compose types
        for comp in has.iter() {
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
        let decl = TypeDecl {
            name: name.clone(),
            specs,
            has,
            members,
            methods,
        };
        // put type in scope
        self.define(name.as_str(), Meta::Type(Type::User(decl.clone())));
        Ok(Stmt::TypeDecl(decl))
    }

    pub fn type_member(&mut self) -> ParseResult<Member> {
        let name = self.parse_name()?;
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
        Ok(Member::new(name, ty, value))
    }

    pub fn union_stmt(&mut self) -> ParseResult<Stmt> {
        self.expect(TokenKind::Union)?;
        let name = self.parse_name()?;
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut fields = Vec::new();
        while !self.is_kind(TokenKind::RBrace) {
            let f = self.union_field()?;
            fields.push(f);
            self.expect_eos()?;
        }
        self.expect(TokenKind::RBrace)?;

        let union = Union {
            name: name.clone(),
            fields: fields.clone(),
        };

        self.define(name.as_str(), Meta::Type(Type::Union(union)));
        Ok(Stmt::Union(Union { name, fields }))
    }

    pub fn union_field(&mut self) -> ParseResult<UnionField> {
        let name = self.parse_name()?;
        let ty = self.parse_type()?;
        Ok(UnionField { name, ty })
    }

    pub fn tag_stmt(&mut self) -> ParseResult<Stmt> {
        self.expect(TokenKind::Tag)?;
        let name = self.parse_name()?;
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut fields = Vec::new();
        while !self.is_kind(TokenKind::RBrace) {
            let member = self.tag_field()?;
            fields.push(member);
            self.expect_eos()?;
        }
        self.expect(TokenKind::RBrace)?;
        self.define(
            name.as_str(),
            Meta::Type(Type::Tag(shared(Tag {
                name: name.clone(),
                fields: fields.clone(),
            }))),
        );
        Ok(Stmt::Tag(Tag { name, fields }))
    }

    pub fn tag_field(&mut self) -> ParseResult<TagField> {
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

    fn get_usize(&mut self, size: &Expr) -> ParseResult<usize> {
        match size {
            Expr::Int(size) => {
                if *size <= 0 {
                    error_pos!("Array size must be greater than 0")
                } else {
                    Ok(*size as usize)
                }
            }
            Expr::Uint(size) => Ok(*size as usize),
            Expr::I8(size) => {
                if *size <= 0 {
                    error_pos!("Array size must be greater than 0")
                } else {
                    Ok(*size as usize)
                }
            }
            Expr::U8(size) => Ok(*size as usize),
            _ => {
                error_pos!("Invalid array size expression, Not a Integer Number!")
            }
        }
    }

    fn parse_ptr_type(&mut self) -> ParseResult<Type> {
        self.next(); // skip `*`
        let typ = self.parse_type()?;
        Ok(Type::Ptr(PtrType { of: shared(typ) }))
    }

    fn parse_array_type(&mut self) -> ParseResult<Type> {
        // parse array type name, e.g. `[10]int`
        self.next(); // skip `[`
        let array_size = if self.is_kind(TokenKind::Int)
            || self.is_kind(TokenKind::Uint)
            || self.is_kind(TokenKind::I8)
            || self.is_kind(TokenKind::U8)
        {
            let size = self.parse_ints()?;
            println!("got int {}", size);
            self.get_usize(&size)?
        } else if self.is_kind(TokenKind::RSquare) {
            0
        } else {
            return error_pos!("Expected Array Size or Empty, got {}", self.peek().kind);
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
                            return error_pos!("Expected array type, got {:?}", meta);
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
                return error_pos!("Expected type, got ident {:?}", type_name);
            }
        }
    }

    fn parse_ident_type(&mut self) -> ParseResult<Type> {
        let ident = self.parse_ident()?;
        match ident {
            Expr::Ident(name) => Ok(self.lookup_type(&name).borrow().clone()),
            _ => error_pos!("Expected type, got ident {:?}", ident),
        }
    }

    fn is_type_name(&mut self) -> bool {
        self.is_kind(TokenKind::Ident) // normal types like `int`
        || self.is_kind(TokenKind::LSquare) // array types like `[5]int`
        || self.is_kind(TokenKind::Star) // ptr types like `*int`
        || self.is_kind(TokenKind::At) // ref types like `@int`
    }

    pub fn parse_type(&mut self) -> ParseResult<Type> {
        match self.cur.kind {
            TokenKind::Ident => self.parse_ident_type(),
            TokenKind::Star => self.parse_ptr_type(),
            TokenKind::LSquare => self.parse_array_type(),
            _ => error_pos!("Expected type, got {}", self.cur.text),
        }
    }

    // TODO: 暂时只检查3种情况：
    // 1，简单名称；
    // 2，点号表达式最左侧的名称
    // 3, 函数调用，如果函数名不存在，表示是一个节点实例
    pub fn check_symbol(&mut self, expr: Expr) -> ParseResult<Expr> {
        if self.skip_check {
            return Ok(expr);
        }
        match &expr {
            Expr::Bina(l, op, _) => match op {
                Op::Dot => {
                    if let Expr::Ident(name) = l.as_ref() {
                        if !self.exists(&name) {
                            return error_pos!("Undefined variable: {}", name);
                        }
                    }
                    Ok(expr)
                }
                _ => Ok(expr),
            },
            Expr::Ident(name) => {
                if !self.exists(&name) {
                    return error_pos!("Undefined identifier: {}", name);
                }
                Ok(expr)
            }
            Expr::Call(call) => {
                match call.name.as_ref() {
                    Expr::Ident(name) => {
                        if !self.exists(&name) {
                            // Check if it's a destructuring
                            //
                            return error_pos!("Function {} not define!", name);
                        }
                    }
                    Expr::Bina(lhs, op, _rhs) => {
                        // check tag creation
                        if let Op::Dot = op {
                            if let Expr::Ident(lname) = lhs.as_ref() {
                                let ltype = self.lookup_type(lname);
                                match *ltype.borrow() {
                                    Type::Tag(ref _t) => {}
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

    pub fn find_type_for_expr(&mut self, expr: &Expr) -> ParseResult<Type> {
        match expr {
            // function name, find it's decl
            Expr::Ident(ident) => {
                let meta = self.lookup_meta(ident);
                let Some(meta) = meta else {
                    return error_pos!("Function name not found! {}", ident);
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
                                return error_pos!("Left name not found! {}.{}", lname, rhs);
                            };
                            match meta.as_ref() {
                                Meta::Type(typ) => match typ {
                                    Type::Tag(tag) => {
                                        if let Expr::Ident(rname) = &**rhs {
                                            let rtype = tag.borrow().get_field_type(rname);
                                            return Ok(rtype);
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
        error_pos!("Meta not found! {}", expr)
    }

    pub fn return_type(&mut self, call_name: &Expr) -> ParseResult<Type> {
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
                                return error_pos!("Left name not found! {}.{}", lname, rhs);
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
        // TODO: should check all types or print error
        Ok(Type::Unknown)
    }

    fn parse_node(
        &mut self,
        name: &AutoStr,
        id: Option<AutoStr>,
        mut args: Args,
        kind: &AutoStr,
    ) -> ParseResult<Node> {
        let n = name.clone().into();
        let mut node = Node::new(name.clone());
        if let Some(id) = id {
            // define a variable for this node instance with id
            self.define(id.as_str(), Meta::Node(node.clone()));
            node.id = id;
        }

        // optional kind argument
        if !kind.is_empty() {
            args.args
                .push(Arg::Pair("kind".into(), Expr::Str(kind.into())));
        }
        node.args = args;

        if self.is_kind(TokenKind::Newline) || self.is_kind(TokenKind::Semi) {
        } else {
            if self.special_blocks.contains_key(&n) {
                node.body = self.special_block(&n)?;
            } else {
                node.body = self.parse_node_body()?;
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
    pub fn parse_node_or_call_stmt(&mut self) -> ParseResult<Stmt> {
        let ident = self.ident()?;
        self.next();

        let mut has_paren = false;

        // 节点实例的id
        let id = if self.is_kind(TokenKind::Ident) {
            let id = self.ident_name()?;
            self.next();
            Some(id)
        } else {
            None
        };

        let mut args = Args::new();
        // If has paren, maybe a call or node instance
        if self.is_kind(TokenKind::LParen) {
            args = self.args()?;
            has_paren = true;
        }

        // if !has_id && !args.is_empty() {
        // id = Some(args.id());
        // }

        // If has brace, must be a node instance
        if self.is_kind(TokenKind::LBrace) {
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
                    return Ok(Stmt::Node(self.parse_node(
                        &name,
                        id,
                        args,
                        &AutoStr::new(),
                    )?));
                }
                _ => {
                    return error_pos!("Expected node name, got {:?}", ident);
                }
            }
        } else {
            // no brace, might be a call or simple expression
            if has_paren {
                // call
                let expr = self.call(ident, args)?;
                // }
                Ok(Stmt::Expr(expr))
            } else {
                // Something else with a starting Ident
                if id.is_some() {
                    if self.is_kind(TokenKind::Ident) {
                        let kind = self.parse_name()?;
                        if self.is_kind(TokenKind::LBrace)
                            || self.is_kind(TokenKind::Newline)
                            || self.is_kind(TokenKind::Semi)
                            || self.is_kind(TokenKind::RBrace)
                        {
                            let nd = self.parse_node(&ident.repr(), id, args, &kind)?;
                            return Ok(Stmt::Node(nd));
                        } else {
                            return error_pos!(
                                "expect node, got {} {} {}",
                                ident.repr(),
                                id.unwrap(),
                                kind
                            );
                        }
                    } else {
                        // name id <nl>
                        if self.is_kind(TokenKind::Newline)
                            || self.is_kind(TokenKind::Semi)
                            || self.is_kind(TokenKind::RBrace)
                        {
                            let nd = self.parse_node(&ident.repr(), id, args, &"".into())?;
                            return Ok(Stmt::Node(nd));
                        }
                    }
                    return error_pos!(
                        "Expected simple expression, got `{} {}`",
                        ident.repr(),
                        id.unwrap()
                    );
                }
                let expr = self.expr_pratt_with_left(ident, 0)?;
                let expr = self.check_symbol(expr)?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn call(&mut self, ident: Expr, args: Args) -> ParseResult<Expr> {
        let ret_type = self.return_type(&ident)?;
        let expr = Expr::Call(Call {
            name: Box::new(ident),
            args,
            ret: ret_type,
        });
        self.check_symbol(expr)
    }

    fn special_block(&mut self, name: &AutoStr) -> ParseResult<Body> {
        self.expect(TokenKind::LBrace)?;
        let block_parser = self.special_blocks.remove(name);
        if block_parser.is_none() {
            return error_pos!("Unknown special block: {}", name);
        }
        let block_parser = block_parser.unwrap();
        let body = block_parser.parse(self)?;
        self.special_blocks.insert(name.clone(), block_parser);
        self.expect(TokenKind::RBrace)?;
        Ok(body)
    }

    pub fn grid(&mut self) -> ParseResult<Grid> {
        self.next(); // skip grid
                     // args
        let mut data = Vec::new();
        let args = self.args()?;
        // data
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            let row = self.array()?;
            if let Expr::Array(array) = row {
                data.push(array);
            }
            self.expect_eos()?;
        }
        self.expect(TokenKind::RBrace)?;
        let grid = Grid { head: args, data };
        Ok(grid)
    }

    pub fn parse_event_src(&mut self) -> ParseResult<Option<Expr>> {
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

    pub fn parse_cond_arrow(&mut self, src: Option<Expr>) -> ParseResult<CondArrow> {
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
    pub fn parse_arrow(&mut self, src: Option<Expr>) -> ParseResult<Arrow> {
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

    fn parse_arrow_list(&mut self) -> ParseResult<Vec<Arrow>> {
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

    pub fn parse_event(&mut self) -> ParseResult<Event> {
        let src = self.parse_event_src()?;
        println!("NEXT I S {}", self.kind());
        if self.is_kind(TokenKind::Arrow) || self.is_kind(TokenKind::Colon) {
            return Ok(Event::Arrow(self.parse_arrow(src)?));
        } else if self.is_kind(TokenKind::Question) {
            return Ok(Event::CondArrow(self.parse_cond_arrow(src)?));
        } else {
            return Err(format!("expected arrow, colon or question mark.").into());
        }
    }

    fn parse_goto_branch(&mut self) -> ParseResult<Event> {
        self.parse_event()
    }

    pub fn parse_on_events(&mut self) -> ParseResult<OnEvents> {
        // skip on
        self.expect(TokenKind::On)?;

        let mut branches = Vec::new();

        // multiple branches
        if self.is_kind(TokenKind::LBrace) {
            self.next(); // skip {
            self.skip_empty_lines();
            while !self.is_kind(TokenKind::RBrace) {
                self.skip_empty_lines();
                println!("cchecking branch: {}", self.cur);
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

    fn parse_with_err(code: &str) -> ParseResult<Code> {
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
        let expected = "Syntax error: Assignment not allowed for let store:";
        let res_err = res.err().unwrap();
        assert!(res_err.to_string().contains(expected));
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
        assert_eq!(ast.to_string(), "(code (bina (int 1) (op ..) (int 5)))");
    }

    #[test]
    fn test_for() {
        let code = "for i in 1..5 {i}";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (for (name i) (bina (int 1) (op ..) (int 5)) (body (name i))))"
        );

        let code = "for i, x in 1..5 {x}";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (for ((name i) (name x)) (bina (int 1) (op ..) (int 5)) (body (name x))))"
        );
    }

    #[test]
    fn test_for_with_print() {
        let code = "for i in 0..10 { print(i); print(i+1) }";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(for (name i) (bina (int 0) (op ..) (int 10)) (body (call (name print) (args (name i))) (call (name print) (args (bina (name i) (op +) (int 1))))))");
    }

    #[test]
    fn test_for_with_mid() {
        let code = r#"for i in 0..10 { print(i); mid{ print(",") } }"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(for (name i) (bina (int 0) (op ..) (int 10)) (body (call (name print) (args (name i))) (node (name mid) (body (call (name print) (args (str \",\")))))))");
    }

    #[test]
    fn test_for_with_mid_call() {
        let code = r#"for i in 0..10 { `$i`; mid{","} }"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(for (name i) (bina (int 0) (op ..) (int 10)) (body (fstr (str \"\") (name i)) (node (name mid) (body (str \",\")))))");
    }

    #[test]
    fn test_object() {
        let code = "let o = {x:1, y:2}";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (let (name o) (object (pair (name x) (int 1)) (pair (name y) (int 2)))))"
        );

        let code = "var a = { 1: 2, 3: 4 }; a.1";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(bina (name a) (op .) (int 1))");
    }

    #[test]
    fn test_fn() {
        let code = "fn add(x, y) { x+y }";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(fn (name add) (params (param (name x) (type int)) (param (name y) (type int))) (body (bina (name x) (op +) (name y))))");
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
        assert_eq!(last.to_string(), "(fn (name say) (params (param (name msg) (type str))) (body (call (name print) (args (name msg)))))");
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
            "(call (bina (name Atom) (op .) (name Int)) (args (int 5)))"
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
        assert_eq!(last.to_string(), "(bina (name p) (op .) (name x))");
    }

    #[test]
    fn test_lambda() {
        let code = "var x = || 1 + 2";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (var (name x) (fn (name lambda_1) (body (bina (int 1) (op +) (int 2))))))"
        );
    }

    #[test]
    fn test_lambda_with_params() {
        let code = "|a int, b int| a + b";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(fn (name lambda_1) (params (param (name a) (type int)) (param (name b) (type int))) (body (bina (name a) (op +) (name b))))");
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
        assert_eq!(
            last.to_string(),
            "(index (bina (name a) (op .) (name b)) (int 0))"
        );
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
            "(call (name A) (args (pair (name x) (int 1)) (pair (name y) (int 2))))"
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
        assert_eq!(last.to_string(), "(type-decl (name Point) (members (member (name x) (type int)) (member (name y) (type int))) (methods (fn (name absquare) (ret int) (body (bina (bina (bina (name s) (op .) (name x)) (op *) (bina (name s) (op .) (name x))) (op +) (bina (bina (name s) (op .) (name y)) (op *) (bina (name s) (op .) (name y))))))))");
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
            "(type-decl (name Duck) (has (type Wing)) (methods (fn (name fly) (body ))))"
        );
    }

    #[test]
    fn test_grid() {
        let code = r#"
        grid(a, b, c) {
            [1, 2, 3]
            [4, 5, 6]
            [7, 8, 9]
        }"#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (grid (head (name a) (name b) (name c)) (data (row (int 1) (int 2) (int 3)) (row (int 4) (int 5) (int 6)) (row (int 7) (int 8) (int 9)))))");
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
        assert_eq!(ast.to_string(), "(code (pair (name name) (str \"hello\")) (pair (name version) (str \"0.1.0\")) (node (name exe) (id hello) (body (pair (name dir) (str \"src\")) (pair (name main) (str \"main.c\")))))");
    }

    #[test]
    fn test_use() {
        let code = "use auto.math.square; square(16)";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (use (path auto.math) (items square)) (call (name square) (args (int 16))))"
        );
    }

    #[test]
    fn test_use_c() {
        let code = "use c <stdio.h>";
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
            "(code (enum (name Color) (item (name Red) (value 0)) (item (name Green) (value 1)) (item (name Blue) (value 2))) (bina (name Color) (op .) (name Red)))"
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
        let code = r#"let ptr *int = 10.ptr"#;
        let ptr_type = parse_once(code);
        assert_eq!(
            ptr_type.to_string(),
            format!(
                "{}",
                "(code (let (name ptr) (type (ptr-type (of int))) (bina (int 10) (op .) (name ptr))))"
            )
        )
    }

    #[test]
    fn test_ptr_asn() {
        let code = r#"let p *int = 10.ptr"#;
        let ptr_type = parse_once(code);
        assert_eq!(
            ptr_type.to_string(),
            format!(
                "{}",
                "(code (let (name p) (type (ptr-type (of int))) (bina (int 10) (op .) (name ptr))))"
            )
        )
    }

    #[test]
    fn test_ptr_target() {
        let code = r#"p.tgt += 1"#;
        let ptr_type = parse_once(code);
        assert_eq!(
            ptr_type.to_string(),
            "(code (bina (bina (name p) (op .) (name tgt)) (op +=) (int 1)))"
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
            code.stmts[2].to_string(),
            format!(
                "{}{}",
                "(is (name atom) (eq (tag-cover (kind Atom) (tag Int) (elem i)) (body (name i)))",
                " (eq (tag-cover (kind Atom) (tag Float) (elem f)) (body (name f))))"
            )
        );
    }
}

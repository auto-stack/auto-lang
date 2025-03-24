use crate::ast::*;
use crate::error_pos;
use crate::lexer::Lexer;
use crate::scope::Meta;
use crate::token::{Pos, Token, TokenKind};
use crate::universe::Universe;
use auto_val::AutoStr;
use auto_val::Op;
use std::cell::RefCell;
use std::i32;
use std::path::Path;
use std::rc::Rc;

type ParseError = String;

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
const PREC_PAIR: PostfixPrec = postfix_prec(2);
const _PREC_OR: InfixPrec = infix_prec(3);
const _PREC_AND: InfixPrec = infix_prec(4);
const PREC_EQ: InfixPrec = infix_prec(5);
const PREC_CMP: InfixPrec = infix_prec(6);
const PREC_RANGE: InfixPrec = infix_prec(7);
const PREC_ADD: InfixPrec = infix_prec(8);
const PREC_MUL: InfixPrec = infix_prec(9);
const _PREC_REF: PrefixPrec = prefix_prec(10);
const PREC_SIGN: PrefixPrec = prefix_prec(11);
const PREC_NOT: PrefixPrec = prefix_prec(12);
const PREC_CALL: PostfixPrec = postfix_prec(13);
const PREC_INDEX: PostfixPrec = postfix_prec(14);
const PREC_DOT: InfixPrec = infix_prec(15);
const _PREC_ATOM: InfixPrec = infix_prec(16);

fn prefix_power(op: Op) -> Result<PrefixPrec, ParseError> {
    match op {
        Op::Add | Op::Sub => Ok(PREC_SIGN),
        Op::Not => Ok(PREC_NOT),
        _ => error_pos!("Invalid prefix operator: {}", op),
    }
}

fn postfix_power(op: Op) -> Result<Option<PostfixPrec>, ParseError> {
    match op {
        Op::LSquare => Ok(Some(PREC_INDEX)),
        Op::LParen => Ok(Some(PREC_CALL)),
        Op::Colon => Ok(Some(PREC_PAIR)),
        _ => Ok(None),
    }
}

fn infix_power(op: Op) -> Result<InfixPrec, ParseError> {
    match op {
        Op::Add | Op::Sub => Ok(PREC_ADD),
        Op::Mul | Op::Div => Ok(PREC_MUL),
        Op::Asn => Ok(PREC_ASN),
        Op::Eq | Op::Neq => Ok(PREC_EQ),
        Op::Lt | Op::Gt | Op::Le | Op::Ge => Ok(PREC_CMP),
        Op::Range | Op::RangeEq => Ok(PREC_RANGE),
        Op::Dot => Ok(PREC_DOT),
        _ => error_pos!("Invalid infix operator: {}", op),
    }
}

// pub fn parse(code: &str, scope: Rc<RefCell<Universe>>, interpreter: &'a Interpreter) -> Result<Code, ParseError> {
// let mut parser = Parser::new(code, scope, interpreter);
// parser.parse()
// }

pub struct Parser<'a> {
    pub scope: Rc<RefCell<Universe>>,
    lexer: Lexer<'a>,
    cur: Token,
}

impl<'a> Parser<'a> {
    pub fn new(code: &'a str, scope: Rc<RefCell<Universe>>) -> Self {
        let mut lexer = Lexer::new(code);
        let cur = lexer.next();
        let mut parser = Parser { scope, lexer, cur };
        parser.skip_comments();
        parser
    }

    pub fn new_with_note(code: &'a str, scope: Rc<RefCell<Universe>>, note: char) -> Self {
        let mut lexer = Lexer::new(code);
        lexer.set_fstr_note(note);
        let cur = lexer.next();
        let mut parser = Parser { scope, lexer, cur };
        parser.skip_comments();
        parser
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

    pub fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if self.is_kind(kind) {
            self.next();
            Ok(())
        } else {
            error_pos!("Expected token kind: {:?}, got {:?}", kind, self.cur.text)
        }
    }

    fn define(&mut self, name: &str, meta: Meta) {
        self.scope.borrow_mut().define(name, Rc::new(meta));
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
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> Result<Code, ParseError> {
        let mut stmts = Vec::new();
        self.skip_empty_lines();
        while !self.is_kind(TokenKind::EOF) {
            let stmt = self.stmt()?;
            // First level pairs are viewed as variable declarations
            // TODO: this should only happen in a Config scenario
            if let Stmt::Expr(Expr::Pair(Pair { key, value })) = &stmt {
                if let Some(name) = key.name() {
                    self.define(
                        name,
                        Meta::Store(Store {
                            name: Name::new(name),
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

    fn convert_last_block(&mut self, mut stmts: Vec<Stmt>) -> Result<Vec<Stmt>, ParseError> {
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

    fn body_to_obj(&mut self, body: &Body) -> Result<Vec<Pair>, ParseError> {
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
    pub fn expr(&mut self) -> Result<Expr, ParseError> {
        let mut exp = self.expr_pratt(0)?;
        exp = self.check_symbol(exp)?;
        Ok(exp)
    }

    // simple Pratt parser
    // ref: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
    pub fn expr_pratt(&mut self, min_power: u8) -> Result<Expr, ParseError> {
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
            // ref
            TokenKind::Ref => {
                self.next(); // skip ref
                let name = self.cur.text.clone();
                self.next(); // skip name
                Expr::Ref(Name::new(name))
            }
            // fstr
            TokenKind::FStrStart => self.fstr()?,
            // grid
            TokenKind::Grid => Expr::Grid(self.grid()?),
            // normal
            _ => self.atom()?,
        };
        self.expr_pratt_with_left(lhs, min_power)
    }

    fn expr_pratt_with_left(&mut self, mut lhs: Expr, min_power: u8) -> Result<Expr, ParseError> {
        loop {
            let op = match self.kind() {
                TokenKind::EOF
                | TokenKind::Newline
                | TokenKind::Semi
                | TokenKind::LBrace
                | TokenKind::RBrace
                | TokenKind::Comma => break,
                TokenKind::Add
                | TokenKind::Sub
                | TokenKind::Mul
                | TokenKind::Div
                | TokenKind::Not => self.op(),
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
                        lhs = Expr::Call(Call {
                            name: Box::new(lhs),
                            args,
                        });
                        continue;
                    }
                    // Pair
                    Op::Colon => {
                        self.next(); // skip :
                        let key = match &lhs {
                            Expr::Ident(name) => Key::NamedKey(name.clone()),
                            Expr::Int(i) => Key::IntKey(*i),
                            Expr::Bool(b) => Key::BoolKey(*b),
                            _ => return error_pos!("Invalid key: {}", lhs),
                        };
                        let rhs = self.expr()?;
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

    fn check_asn(&mut self, lhs: &Expr) -> Result<(), ParseError> {
        match lhs {
            Expr::Ident(name) => {
                let meta = self.lookup_meta(name.text.as_str());
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
            TokenKind::Mul => Op::Mul,
            TokenKind::Div => Op::Div,
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

    pub fn group(&mut self) -> Result<Expr, ParseError> {
        self.next(); // skip (
        let expr = self.expr()?;
        self.expect(TokenKind::RParen)?; // skip )
        Ok(expr)
    }

    pub fn sep_array(&mut self) -> Result<(), ParseError> {
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

    pub fn array(&mut self) -> Result<Expr, ParseError> {
        self.expect(TokenKind::LSquare)?;
        self.skip_empty_lines();
        let mut elems = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RSquare) {
            elems.push(self.expr()?);
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

    pub fn args(&mut self) -> Result<Args, ParseError> {
        self.expect(TokenKind::LParen)?;
        let mut args = Args::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RParen) {
            // TODO: this is a temp fix for identifier as string in arg
            let expr = if self.is_kind(TokenKind::Ident) {
                let name = Expr::Ident(Name::new(self.cur.text.clone()));
                self.next();
                if self.is_kind(TokenKind::Comma) {
                    name
                } else {
                    self.expr_pratt_with_left(name, 0)?
                }
            } else {
                self.expr()?
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
                    let name = name.text.clone();
                    match self.lookup_meta(&name) {
                        Some(_) => {
                            args.args.push(Arg::Pos(expr.clone()));
                        }
                        None => {
                            args.args.push(Arg::Name(Name::new(name)));
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

    pub fn object(&mut self) -> Result<Vec<Pair>, ParseError> {
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

    pub fn pair(&mut self) -> Result<Pair, ParseError> {
        let key = self.key()?;
        self.expect(TokenKind::Colon)?;
        let value = self.expr()?;
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

    pub fn key(&mut self) -> Result<Key, ParseError> {
        match self.kind() {
            TokenKind::Ident => {
                let name = self.cur.text.clone();
                self.next();
                Ok(Key::NamedKey(Name::new(name)))
            }
            TokenKind::Int => {
                let value = self.cur.text.parse().unwrap();
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
                Ok(Key::NamedKey(Name::new(value)))
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

    pub fn ident(&mut self) -> Result<Expr, ParseError> {
        let name = self.cur.text.clone();
        // // check for existence
        // if !self.exists(&name) {
        //     return Err(format!("Undefined variable: {}", name));
        // }
        Ok(Expr::Ident(Name::new(name)))
    }

    pub fn atom(&mut self) -> Result<Expr, ParseError> {
        if self.is_kind(TokenKind::LParen) {
            return self.group();
        }
        let expr = match self.kind() {
            TokenKind::Uint => {
                if self.cur.text.starts_with("0x") {
                    // trim 0x
                    let trim = &self.cur.text[2..];
                    let val = u32::from_str_radix(trim, 16).unwrap();
                    Expr::Uint(val)
                } else {
                    let val = self.cur.text.parse::<u32>().unwrap();
                    Expr::Uint(val)
                }
            }
            TokenKind::Int => {
                if self.cur.text.starts_with("0x") {
                    // trim 0x
                    let trim = &self.cur.text[2..];
                    let val = i32::from_str_radix(trim, 16).unwrap();
                    Expr::Int(val)
                } else {
                    let val = self.cur.text.parse::<i32>().unwrap();
                    Expr::Int(val)
                }
            }
            TokenKind::Float => Expr::Float(self.cur.text.parse().unwrap()),
            TokenKind::True => Expr::Bool(true),
            TokenKind::False => Expr::Bool(false),
            TokenKind::Str => Expr::Str(self.cur.text.clone()),
            TokenKind::Char => Expr::Char(self.cur.text.chars().nth(0).unwrap()),
            TokenKind::Ident => self.ident()?,
            TokenKind::Model => Expr::Ident(Name::new("model".to_string())),
            TokenKind::View => Expr::Ident(Name::new("view".to_string())),
            TokenKind::Nil => Expr::Nil,
            _ => {
                return error_pos!("Expected term, got {:?}, pos: {}", self.kind(), self.pos());
            }
        };

        self.next();
        Ok(expr)
    }

    /// 解析fstr
    /// fstr: [FStrSart, (FStrParts | ${Expr} | $Ident)*, FStrEnd]
    pub fn fstr(&mut self) -> Result<Expr, ParseError> {
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

    pub fn if_expr(&mut self) -> Result<Expr, ParseError> {
        let (branches, else_stmt) = self.if_contents()?;
        Ok(Expr::If(branches, else_stmt))
    }

    // An Expression that can be assigned to a variable, e.g. right-hand side of an assignment
    pub fn rhs_expr(&mut self) -> Result<Expr, ParseError> {
        if self.is_kind(TokenKind::If) {
            self.if_expr()
        } else {
            self.expr()
        }
    }

    pub fn iterable_expr(&mut self) -> Result<Expr, ParseError> {
        // TODO: how to check for range/array but reject other cases?
        self.expr()
    }

    pub fn lambda(&mut self) -> Result<Expr, ParseError> {
        self.next(); // skip |
        let params = self.fn_params()?;
        self.expect(TokenKind::VBar)?; // skip |
        let id = self.scope.borrow_mut().gen_lambda_id();
        let lambda = if self.is_kind(TokenKind::LBrace) {
            let body = self.body()?;
            Fn::new(Name::new(id.clone()), None, params, body, Type::Unknown)
        } else {
            // single expression
            let expr = self.expr()?;
            Fn::new(
                Name::new(id.clone()),
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
    pub fn expect_eos(&mut self) -> Result<(), ParseError> {
        let mut has_sep = false;
        while self.is_kind(TokenKind::Semi)
            || self.is_kind(TokenKind::Newline)
            || self.is_kind(TokenKind::Comma)
        {
            has_sep = true;
            self.next();
        }
        if self.is_kind(TokenKind::EOF) || self.is_kind(TokenKind::RBrace) {
            return Ok(());
        }
        if has_sep {
            Ok(())
        } else {
            error_pos!("Expected end of statement, got {:?}", self.kind())
        }
    }

    pub fn stmt(&mut self) -> Result<Stmt, ParseError> {
        let stmt = match self.kind() {
            TokenKind::Use => self.use_stmt()?,
            TokenKind::If => self.if_stmt()?,
            TokenKind::For => self.for_stmt()?,
            TokenKind::Var => self.store_stmt()?,
            TokenKind::Let => self.store_stmt()?,
            TokenKind::Mut => self.store_stmt()?,
            TokenKind::Fn => self.fn_decl_stmt("")?,
            TokenKind::Type => self.type_decl_stmt()?,
            TokenKind::LBrace => Stmt::Block(self.body()?),
            // AutoUI Stmts
            TokenKind::Widget => self.widget_stmt()?,
            // Node Instance?
            TokenKind::Ident => self.node_or_call_stmt()?,
            // Otherwise, try to parse as an expression
            _ => self.expr_stmt()?,
        };
        Ok(stmt)
    }

    fn expect_ident_str(&mut self) -> Result<String, ParseError> {
        if self.is_kind(TokenKind::Ident) {
            let name = self.cur.text.clone();
            self.next(); // skip name
            Ok(name)
        } else {
            error_pos!("Expected identifier, got {:?}", self.kind())
        }
    }

    pub fn use_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // skip use
        let mut paths = Vec::new();
        let mut items = Vec::new();
        let name = self.expect_ident_str()?;
        paths.push(name);
        while self.is_kind(TokenKind::Dot) {
            self.next(); // skip .
            let name = self.expect_ident_str()?;
            paths.push(name);
        }
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
        if items.is_empty() && !paths.is_empty() {
            items.push(paths.pop().unwrap());
        }
        // import the path into scope
        let uses = Use { paths, items };
        self.import(&uses)?;
        Ok(Stmt::Use(uses))
    }

    // Import a path from `use` statement
    // TODO: clean up code
    // TODO: search path from System Env, Default Locations and etc.
    pub fn import(&mut self, uses: &Use) -> Result<(), ParseError> {
        let path = uses.paths.join(".");
        // locate file from path
        let base_path = std::env::current_dir().unwrap();
        let base_path = base_path.as_path();
        let std_path = base_path.parent().unwrap();
        let std_path = std_path.parent().unwrap();
        let std_path = std_path.join("std");
        // println!("std_path: {}", std_path.display());
        if !path.starts_with("std.") {
            return error_pos!("Invalid import path: {}", path);
        }
        let path = path.replace("std.", "");
        // println!("path: {}", path);
        let file_path = std_path.join(Path::new(path.as_str()));
        // println!("file_path: {}", file_path.display());
        let dir = file_path.parent().unwrap();
        let name = file_path.file_name().unwrap();
        if !dir.exists() {
            return error_pos!("Invalid import path: {}", path);
        }
        // Read file
        let file_path = dir.join(name.to_str().unwrap().to_string() + ".at");
        let file_content = std::fs::read_to_string(file_path).unwrap();

        let mut new_parser = Parser::new(file_content.as_str(), self.scope.clone());
        let ast = new_parser.parse().unwrap();
        let path: AutoStr = path.into();
        self.scope.borrow_mut().import(path.clone(), ast);
        // Define items in scope
        for item in uses.items.iter() {
            // lookup item's meta from its mod
            let meta = self
                .scope
                .borrow()
                .lookup(item.as_str(), path.clone())
                .unwrap();
            // println!("meta: {:?}", meta);
            // define iten with its name in current scope
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

    fn parse_body(&mut self, is_node: bool) -> Result<Body, ParseError> {
        self.expect(TokenKind::LBrace)?;
        self.enter_scope();
        let mut stmts = Vec::new();
        let new_lines = self.skip_empty_lines();
        let has_new_line = new_lines > 0;
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            let stmt = self.stmt()?;
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
            self.expect_eos()?;
        }
        stmts = self.convert_last_block(stmts)?;
        self.exit_scope();
        self.expect(TokenKind::RBrace)?;
        Ok(Body {
            stmts,
            has_new_line,
        })
    }

    pub fn node_body(&mut self) -> Result<Body, ParseError> {
        self.parse_body(true)
    }

    pub fn body(&mut self) -> Result<Body, ParseError> {
        self.parse_body(false)
    }

    pub fn if_contents(&mut self) -> Result<(Vec<Branch>, Option<Body>), ParseError> {
        let mut branches = Vec::new();
        self.next(); // skip if
        let cond = self.expr()?;
        let body = self.body()?;
        branches.push(Branch { cond, body });

        let mut else_stmt = None;
        while self.is_kind(TokenKind::Else) {
            self.next(); // skip else
                         // more branches
            if self.is_kind(TokenKind::If) {
                self.next(); // skip if
                let cond = self.expr()?;
                let body = self.body()?;
                branches.push(Branch { cond, body });
            } else {
                // last else
                else_stmt = Some(self.body()?);
            }
        }
        Ok((branches, else_stmt))
    }

    pub fn if_stmt(&mut self) -> Result<Stmt, ParseError> {
        let (branches, else_stmt) = self.if_contents()?;
        Ok(Stmt::If(branches, else_stmt))
    }

    pub fn for_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // skip `for`
                     // enumerator
        if self.is_kind(TokenKind::Ident) {
            let name = self.cur.text.clone();
            self.enter_scope();
            let meta = Meta::Var(Var {
                name: Name::new(name.clone()),
                expr: Expr::Nil,
            });
            self.define(name.as_str(), meta);
            self.next(); // skip name
            let mut iter = Iter::Named(Name::new(name.clone()));
            if self.is_kind(TokenKind::Comma) {
                self.next(); // skip ,
                let iter_name = self.cur.text.clone();
                let meta_iter = Meta::Var(Var {
                    name: Name::new(iter_name.clone()),
                    expr: Expr::Nil,
                });
                self.define(iter_name.as_str(), meta_iter);
                self.next(); // skip iter name
                iter = Iter::Indexed(Name::new(name.clone()), Name::new(iter_name.clone()));
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

    pub fn store_stmt(&mut self) -> Result<Stmt, ParseError> {
        // store kind: var/let/mut
        let store_kind = self.store_kind()?;
        self.next(); // skip var/let/mut
                     // identifier name
        let name = self.cur.text.clone();
        self.expect(TokenKind::Ident)?;

        // type (optional)
        let mut ty = Type::Unknown;
        if self.is_kind(TokenKind::Ident) {
            ty = self.type_name()?;
        }

        // =
        self.expect(TokenKind::Asn)?;

        // inital value: expression
        let expr = self.rhs_expr()?;
        // TODO: check type compatibility
        if matches!(ty, Type::Unknown) {
            ty = self.find_expr_type(&expr)?;
        }

        let store = Store {
            kind: store_kind,
            name: Name::new(name.clone()),
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

    pub fn find_expr_type(&mut self, expr: &Expr) -> Result<Type, ParseError> {
        let mut ty = Type::Unknown;
        match &expr {
            Expr::Int(_) => {
                ty = Type::Int;
            }
            Expr::Float(_) => {
                ty = Type::Float;
            }
            Expr::Bool(_) => {
                ty = Type::Bool;
            }
            Expr::Str(_) => {
                ty = Type::Str;
            }
            Expr::Array(arr) => {
                // check first element
                if arr.len() > 0 {
                    let first = &arr[0];
                    let elem_ty = self.find_expr_type(first)?;
                    ty = Type::Array(ArrayType {
                        elem: Box::new(elem_ty),
                        len: arr.len(),
                    });
                } else {
                    ty = Type::Array(ArrayType {
                        elem: Box::new(Type::Unknown),
                        len: 0,
                    });
                }
            }
            _ => {}
        }
        Ok(ty)
    }

    pub fn store_kind(&mut self) -> Result<StoreKind, ParseError> {
        match self.kind() {
            TokenKind::Var => Ok(StoreKind::Var),
            TokenKind::Let => Ok(StoreKind::Let),
            TokenKind::Mut => Ok(StoreKind::Mut),
            _ => error_pos!("Expected store kind, got {:?}", self.kind()),
        }
    }

    // Function Declaration
    pub fn fn_decl_stmt(&mut self, parent_name: &str) -> Result<Stmt, ParseError> {
        self.next(); // skip keyword `fn`
                     // parse function name
        let name = self.cur.text.clone();
        if self.is_kind(TokenKind::View) {
            self.next(); // skip view
        } else {
            self.expect(TokenKind::Ident)?;
        }

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
            ret_type = self.type_name()?;
        }

        // parse function body
        let body = self.body()?;

        // exit function scope
        self.exit_scope();

        // parent name for method?
        let parent = if parent_name.is_empty() {
            None
        } else {
            Some(Name::new(parent_name.to_string()))
        };
        let fn_expr = Fn::new(Name::new(name.clone()), parent, params, body, ret_type);
        let fn_stmt = Stmt::Fn(fn_expr.clone());
        let unique_name = if parent_name.is_empty() {
            name
        } else {
            format!("{}::{}", parent_name, name)
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
    pub fn fn_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        while self.is_kind(TokenKind::Ident) {
            // param name
            let name = self.cur.text.clone();
            self.next(); // skip name
                         // param type
            let mut ty = Type::Int;
            if self.is_kind(TokenKind::Ident) {
                ty = self.type_name()?;
            }
            // default val
            let mut default = None;
            if self.is_kind(TokenKind::Asn) {
                self.next(); // skip =
                let expr = self.expr()?;
                default = Some(expr);
            }
            // define param in current scope (currently in fn scope)
            let var = Var {
                name: Name::new(name.clone()),
                expr: default.clone().unwrap_or(Expr::Nil),
            };
            // TODO: should we consider Meta::Param instead of Meta::Var?
            self.define(name.as_str(), Meta::Var(var.clone()));
            params.push(Param {
                name: Name::new(name),
                ty,
                default,
            });
            self.sep_params();
        }
        Ok(params)
    }

    pub fn type_name(&mut self) -> Result<Type, ParseError> {
        let ty = self.type_expr()?;
        Ok(ty)
    }

    pub fn expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        Ok(Stmt::Expr(self.expr()?))
    }

    pub fn type_decl_stmt(&mut self) -> Result<Stmt, ParseError> {
        // TODO: deal with scope
        self.next(); // skip `type` keyword
        let name = Name::new(self.cur.text.clone());
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
                let typ = self.type_name()?;
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
                let fn_stmt = self.fn_decl_stmt(&name.text)?;
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
                        let unique_name = format!("{}::{}", &name.text, &compose_meth.name.text);
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
        self.define(name.text.as_str(), Meta::Type(Type::User(decl.clone())));
        Ok(Stmt::TypeDecl(decl))
    }

    pub fn type_member(&mut self) -> Result<Member, ParseError> {
        let name = Name::new(self.cur.text.clone());
        self.expect(TokenKind::Ident)?;
        let ty = self.type_expr()?;
        let mut value = None;
        if self.is_kind(TokenKind::Asn) {
            self.next(); // skip =
            let expr = self.expr()?;
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
        self.define(name.text.as_str(), Meta::Store(store));
        Ok(Member::new(name, ty, value))
    }

    pub fn type_expr(&mut self) -> Result<Type, ParseError> {
        let type_name = self.ident()?;
        self.next();
        match type_name {
            Expr::Ident(name) => {
                let meta = self
                    .lookup_meta(&name.text)
                    .ok_or(format!("Undefined type: {}", name.text))?;
                if let Meta::Type(ty) = meta.as_ref() {
                    Ok(ty.clone())
                } else {
                    error_pos!("Expected type, got {:?}", meta)
                }
            }
            _ => error_pos!("Expected type, got {:?}", type_name),
        }
    }

    // TODO: 暂时只检查3种情况：
    // 1，简单名称；
    // 2，点号表达式最左侧的名称
    // 3, 函数调用，如果函数名不存在，表示是一个节点实例
    pub fn check_symbol(&mut self, expr: Expr) -> Result<Expr, ParseError> {
        match &expr {
            Expr::Bina(l, op, _) => match op {
                Op::Dot => {
                    if let Expr::Ident(name) = l.as_ref() {
                        if !self.exists(&name.text) {
                            return error_pos!("Undefined variable: {}", name.text);
                        }
                    }
                    Ok(expr)
                }
                _ => Ok(expr),
            },
            Expr::Ident(name) => {
                if !self.exists(&name.text) {
                    return error_pos!("Undefined identifier: {}", name.text);
                }
                Ok(expr)
            }
            Expr::Call(call) => {
                if let Expr::Ident(name) = call.name.as_ref() {
                    if !self.exists(&name.text) {
                        // 表示是一个节点实例
                        let node = Node::from(call.clone());
                        return Ok(Expr::Node(node));
                    }
                }
                Ok(expr)
            }
            _ => Ok(expr),
        }
    }

    pub fn widget_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // skip widget
        let name = self.cur.text.clone();
        self.expect(TokenKind::Ident)?;
        let (model, view) = self.widget_body()?;
        let mut widget = Widget::new(Name::new(name.clone()));
        widget.model = model;
        widget.view = view;
        self.define(name.as_str(), Meta::Widget(widget.clone()));
        Ok(Stmt::Widget(widget))
    }

    pub fn widget_body(&mut self) -> Result<(Model, View), ParseError> {
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut has_content = false;
        let mut model = Model::default();
        let mut view = View::default();
        if self.is_kind(TokenKind::Model) {
            model = self.model_decl()?;
            has_content = true;
        }
        self.skip_empty_lines();
        if self.is_kind(TokenKind::View) {
            view = self.view_decl()?;
            has_content = true;
        }
        if !has_content {
            return error_pos!("Widget has no model nor view");
        }
        self.skip_empty_lines();
        self.expect(TokenKind::RBrace)?;
        Ok((model, view))
    }

    pub fn model_decl(&mut self) -> Result<Model, ParseError> {
        self.next(); // skip model
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();
        let mut model = Model::default();
        // parse multiple var declarations
        while self.is_kind(TokenKind::Var) {
            let store = self.store_stmt()?;
            match store {
                Stmt::Store(store) => {
                    model.vars.push(store);
                }
                _ => {
                    return error_pos!("Expected store declaration, got {:?}", store);
                }
            }
            self.expect_eos()?;
        }
        self.expect(TokenKind::RBrace)?;
        Ok(model)
    }

    pub fn view_decl(&mut self) -> Result<View, ParseError> {
        self.next(); // skip view
        let mut view = View::default();
        self.expect(TokenKind::LBrace)?; // skip {
        self.skip_empty_lines();
        // parse multiple node instances
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            let node = self.node_or_call_stmt()?;
            match node {
                Stmt::Node(node) => {
                    view.nodes.push((node.name.clone(), node));
                }
                Stmt::Expr(Expr::Call(call)) => {
                    // Call to node
                    let node: Node = call.into();
                    view.nodes.push((node.name.clone(), node));
                }
                _ => {
                    return error_pos!("Expected node in view body, got {:?}", node);
                }
            }
            self.expect_eos()?;
        }
        self.expect(TokenKind::RBrace)?;
        Ok(view)
    }

    // 节点实例和函数调用有类似语法：
    // 1. hello(x, y)， 这个是函数调用
    // 2. hello(), 这个是参数为空的函数调用
    // 3. hello(x, y) { ... }， 这个是节点实例
    // 4. hello() { ... }， 这个是参数为空的节点实例
    // 5. hello {...}，当参数为空时，可以省略()。但不能省略{}，否则和函数调用就冲突了。
    // 6. hello(x, y) {}, 这个是子节点为空的节点实例
    // 7. hello(){}， 这个是参数为空，子节点也为空的节点实例
    // 8. hello {}， 上面的()也可以省略。
    pub fn node_or_call_stmt(&mut self) -> Result<Stmt, ParseError> {
        let ident = self.ident()?;
        self.next();

        let mut args = Args::new();
        let mut has_paren = false;
        // If has paren, maybe a call or node instance
        if self.is_kind(TokenKind::LParen) {
            args = self.args()?;
            has_paren = true;
        }

        // If has brace, must be a node instance
        if self.is_kind(TokenKind::LBrace) {
            // node instance
            // with node instance, pair args also defines as properties
            for arg in &args.args {
                if let Arg::Pair(name, value) = arg {
                    self.define(
                        name.text.as_str(),
                        Meta::Pair(Pair {
                            key: Key::NamedKey(name.clone()),
                            value: Box::new(value.clone()),
                        }),
                    );
                }
            }
            let body = self.node_body()?;
            match ident {
                Expr::Ident(name) => {
                    let mut node = Node::new(name.clone());
                    node.args = args;
                    node.body = body;
                    return Ok(Stmt::Node(node));
                }
                _ => {
                    return error_pos!("Expected node name, got {:?}", ident);
                }
            }
        } else {
            // no brace, might be a call or simple expression
            if has_paren {
                // call
                let mut expr = Expr::Call(Call {
                    name: Box::new(ident),
                    args,
                });
                expr = self.check_symbol(expr)?;
                if let Expr::Node(node) = expr {
                    return Ok(Stmt::Node(node));
                }
                Ok(Stmt::Expr(expr))
            } else {
                // Something else with a starting Ident
                let expr = self.expr_pratt_with_left(ident, 0)?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    pub fn grid(&mut self) -> Result<Grid, ParseError> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::pretty;
    fn parse_once(code: &str) -> Code {
        let scope = Rc::new(RefCell::new(Universe::new()));
        let mut parser = Parser::new(code, scope);
        parser.parse().unwrap()
    }

    fn parse_with_err(code: &str) -> Result<Code, ParseError> {
        let scope = Rc::new(RefCell::new(Universe::new()));
        let mut parser = Parser::new(code, scope);
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
        let code = r#"for i in 0..10 { `$i`; mid(",") }"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(for (name i) (bina (int 0) (op ..) (int 10)) (body (fstr (str \"\") (name i)) (node (name mid) (args (str \",\")))))");
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
    fn test_node_instance_without_args() {
        let code = r#"center {
            text("Hello")
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
            text("Hello")
        }
        "#;
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (node (name center) (body (node (name text) (args (str \"Hello\"))))))"
        );
    }

    // #[test]
    // fn test_app() {
    //     let code = r#"
    //     widget hello {
    //         model {
    //             var name = ""
    //         }

    //         view {
    //             text(f"Hello $name")
    //         }
    //     }

    //     app {
    //         center {
    //             hello(name:"You")
    //         }
    //     }"#;
    //     let ast = parse_once(code);
    //     let last = ast.stmts.last().unwrap();
    //     println!("{}", pretty(&last.to_string()));
    //     assert_eq!(last.to_string(), "(node (name app) (body (node (name center) (body (call (name hello) (args (pair (name name) (str \"You\"))))))))");
    // }

    #[test]
    fn test_ref() {
        let code = "var a = 1; var b = ref a; b";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (var (name a) (int 1)) (var (name b) (ref a)) (name b))"
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
                x * x + y * y
            }
        }"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(type-decl (name Point) (members (member (name x) (type int)) (member (name y) (type int))) (methods (fn (name absquare) (ret int) (body (bina (bina (name x) (op *) (name x)) (op +) (bina (name y) (op *) (name y)))))))");
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

exe(hello) {
    dir: "src"
    main: "main.c"
}"#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (pair (name name) (str \"hello\")) (pair (name version) (str \"0.1.0\")) (node (name exe) (args (name hello)) (body (pair (name dir) (str \"src\")) (pair (name main) (str \"main.c\")))))");
    }

    #[test]
    fn test_use() {
        let code = "use std.math.square; square(16)";
        let ast = parse_once(code);
        assert_eq!(
            ast.to_string(),
            "(code (use (path std.math) (items square)) (call (name square) (args (int 16))))"
        );
    }

    #[test]
    fn test_import() {
        let code = "use std.math.square";
        let scope = Universe::new();
        let mut parser = Parser::new(&code, Rc::new(RefCell::new(scope)));
        let ast = parser.parse().unwrap();
        assert_eq!(
            ast.to_string(),
            "(code (use (path std.math) (items square)))"
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
        let scope = Rc::new(RefCell::new(Universe::new()));
        let mut parser = Parser::new_with_note(code, scope, '#');
        let ast = parser.parse().unwrap();
        assert_eq!(
            ast.to_string(),
            "(code (fstr (str \"hello \") (bina (int 2) (op +) (int 1)) (str \" again\")))"
        );
    }
}

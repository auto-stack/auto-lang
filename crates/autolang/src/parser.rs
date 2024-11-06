use crate::token::{Token, TokenKind, Pos};
use crate::ast::*;
use crate::lexer::Lexer;
use crate::scope::Universe;
use crate::scope::Meta;
use autoval::value::Op;
use std::i32;
use crate::error_pos;
use std::rc::Rc;

type ParseError = String;


pub struct PostfixPrec {
    l: u8,
    r: (),
}

pub struct InfixPrec {
    l: u8,
    r: u8,
}

pub struct PrefixPrec {
    l: (),
    r: u8,
}

const fn prefix_prec(n: u8) -> PrefixPrec {
    PrefixPrec { l: (), r: 2*n }
}

const fn postfix_prec(n: u8) -> PostfixPrec {
    PostfixPrec { l: 2*n, r: () }
}

const fn infix_prec(n: u8) -> InfixPrec {
    if n == 0 {
        InfixPrec { l: 0, r: 0 }
    } else {
        InfixPrec { l: 2*n-1, r: 2*n }
    }
}

const PREC_ASN: InfixPrec = infix_prec(1);
const PREC_PAIR: PostfixPrec = postfix_prec(2);
const PREC_OR: InfixPrec = infix_prec(3);
const PREC_AND: InfixPrec = infix_prec(4);
const PREC_EQ: InfixPrec = infix_prec(5);
const PREC_CMP: InfixPrec = infix_prec(6);
const PREC_Range: InfixPrec = infix_prec(7);
const PREC_ADD: InfixPrec = infix_prec(8);
const PREC_MUL: InfixPrec = infix_prec(9);
const PREC_SIGN: PrefixPrec = prefix_prec(10);
const PREC_NOT: PrefixPrec = prefix_prec(11);
const PREC_CALL: PostfixPrec = postfix_prec(12);
const PREC_INDEX: PostfixPrec = postfix_prec(13);
const PREC_DOT: InfixPrec = infix_prec(14);
const PREC_ATOM: InfixPrec = infix_prec(15);

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
        Op::Range | Op::RangeEq => Ok(PREC_Range),
        Op::Dot => Ok(PREC_DOT),
        _ => error_pos!("Invalid infix operator: {}", op),
    }
}

pub fn parse(code: &str, scope: &mut Universe) -> Result<Code, ParseError> {
    let mut parser = Parser::new(code, scope);
    parser.parse()
}


pub struct Parser<'a> {
    scope: &'a mut Universe,
    lexer: Lexer<'a>,
    cur: Token,
}

impl<'a> Parser<'a> {
    pub fn new(code: &'a str, scope: &'a mut Universe) -> Self {
        let mut lexer = Lexer::new(code);
        let cur = lexer.next();
        Parser { lexer, cur, scope }
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

    pub fn next(&mut self) -> &Token {
        self.cur = self.lexer.next();
        &self.cur
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError>{
        if self.is_kind(kind) {
            self.next();
            Ok(())
        } else {
            error_pos!("Expected token kind: {:?}, got {:?}", kind, self.cur.text)
        }
    }


}

impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> Result<Code, ParseError> {
        let mut stmts = Vec::new();
        self.skip_empty_lines();
        while !self.is_kind(TokenKind::EOF) {
            stmts.push(self.stmt()?);
            self.expect_eos()?; 
        }
        Ok(Code { stmts })
    }

}

// Expressions
impl<'a> Parser<'a> {
    pub fn expr(&mut self) -> Result<Expr, ParseError> {
        let exp = self.expr_pratt(0)?;
        self.check_symbol(&exp)?;
        Ok(exp)
    }

    // simple Pratt parser
    // ref: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
    pub fn expr_pratt(&mut self, min_power: u8) -> Result<Expr, ParseError> {
        // Prefix
        let mut lhs = match self.kind() {
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
            // normal
            _ => self.atom()?,
        };
        self.expr_pratt_with_left(lhs, min_power)
    }

    fn expr_pratt_with_left(&mut self, mut lhs: Expr, min_power: u8) -> Result<Expr, ParseError> {
        loop {
            let op = match self.kind() {
                TokenKind::EOF | TokenKind::Newline | TokenKind::Semi | TokenKind::LBrace | TokenKind::RBrace | TokenKind::Comma => break,
                TokenKind::Add | TokenKind::Sub | TokenKind::Mul | TokenKind::Div | TokenKind::Not => self.op(),
                TokenKind::Dot => self.op(),
                TokenKind::Colon => self.op(),
                TokenKind::Range | TokenKind::RangeEq => self.op(),
                TokenKind::LSquare => self.op(),
                TokenKind::LParen => self.op(),
                TokenKind::Asn => self.op(),
                TokenKind::Eq | TokenKind::Neq | TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge => self.op(),
                TokenKind::RSquare => break,
                TokenKind::RParen => break,
                _ => {
                    return error_pos!("Expected infix operator, got {:?}", self.peek());
                }
            };
            // Postfix

            if let Ok(Some(power)) = postfix_power(op) {
                if power.l < min_power { break; }

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
                        lhs = Expr::Call(Call{name: Box::new(lhs), args});
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
                        lhs = Expr::Pair(Pair { key, value: Box::new(rhs) });
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
            let rhs = self.expr_pratt(power.r)?;
            lhs = Expr::Bina(Box::new(lhs), op, Box::new(rhs));
        }
        Ok(lhs)
    }

    pub fn op(&mut self) -> Op {
        match self.kind() {
            TokenKind::Add => { Op::Add },
            TokenKind::Sub => { Op::Sub },
            TokenKind::Mul => { Op::Mul },
            TokenKind::Div => { Op::Div },
            TokenKind::LSquare => { Op::LSquare },
            TokenKind::LParen => { Op::LParen },
            TokenKind::LBrace => { Op::LBrace },
            TokenKind::Not => { Op::Not },
            TokenKind::Asn => { Op::Asn },
            TokenKind::Eq => { Op::Eq },
            TokenKind::Neq => { Op::Neq },
            TokenKind::Lt => { Op::Lt },
            TokenKind::Gt => { Op::Gt },
            TokenKind::Le => { Op::Le },
            TokenKind::Ge => { Op::Ge },
            TokenKind::Range => { Op::Range },
            TokenKind::RangeEq => { Op::RangeEq },
            TokenKind::Dot => { Op::Dot },
            TokenKind::Colon => { Op::Colon },
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
        let mut is_named_started = false;
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RParen) {
            let expr = self.expr()?;
            // Check for named argument
            match expr {
                Expr::Bina(name, Op::Asn, val) => { // Named argument
                    is_named_started = true;
                    match &*name {
                        Expr::Ident(name) => args.map.push((name.clone(), *val)),
                        _ => return error_pos!("Expected identifier, got {:?}", name),
                    }
                }
                _ => {
                    if is_named_started {
                        return error_pos!("all positional args should come before named args: {}", expr);
                    }
                    args.array.push(expr);
                }
            }
            self.sep_args();
        }
        self.expect(TokenKind::RParen)?;
        Ok(args)
    }

    pub fn object(&mut self) -> Result<Vec<Pair>, ParseError> {
        self.expect(TokenKind::LBrace)?;
        let mut entries = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            entries.push(self.pair()?);
            self.sep_pair();
        }
        self.expect(TokenKind::RBrace)?;
        Ok(entries)
    }

    pub fn pair(&mut self) -> Result<Pair, ParseError> {
        let key = self.key()?;
        self.expect(TokenKind::Colon)?;
        let value = self.expr()?;
        Ok(Pair { key, value: Box::new(value) })
    }

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

    pub fn sep_pair(&mut self) {
        if self.is_kind(TokenKind::Comma) {
            self.next();
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
        // if !self.scope.exists(&name) {
        //     return Err(format!("Undefined variable: {}", name));
        // }
        Ok(Expr::Ident(Name::new(name)))
    }

    pub fn atom(&mut self) -> Result<Expr, ParseError> {
        if self.is_kind(TokenKind::LParen) {
            return self.group();
        }
        let expr = match self.kind() {
            TokenKind::Int => {
                if self.cur.text.starts_with("0x") {
                    // trim 0x
                    let trim = &self.cur.text[2..];
                    Expr::Int(i32::from_str_radix(trim, 16).unwrap())
                } else {
                    Expr::Int(self.cur.text.parse().unwrap())
                }
            }
            TokenKind::Float => Expr::Float(self.cur.text.parse().unwrap()),
            TokenKind::True => Expr::Bool(true),
            TokenKind::False => Expr::Bool(false),
            TokenKind::Str => Expr::Str(self.cur.text.clone()),
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

    pub fn fstr(&mut self) -> Result<Expr, ParseError> {
        let mut parts = Vec::new();
        let start_text = self.cur.text.clone();
        parts.push(Expr::Str(start_text));
        self.expect(TokenKind::FStrStart)?;
        if self.is_kind(TokenKind::Dollar) {
            self.next(); // skip $
            if self.is_kind(TokenKind::LBrace) {
                self.expect(TokenKind::LBrace)?;
                while !self.is_kind(TokenKind::RBrace) {
                    let expr = self.expr()?;
                    parts.push(expr);
                    self.expect_eos()?;
                }
                self.expect(TokenKind::RBrace)?;
            } else {
                let ident = self.ident()?;
                parts.push(ident);
                self.expect(TokenKind::Ident)?;
            }
        }
        let end_text = self.cur.text.clone();
        parts.push(Expr::Str(end_text));
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
        if self.is_kind(TokenKind::LBrace) {
            let body = self.body()?;
            return Ok(Expr::Lambda(Lambda::new(params, body)));
        } else { // single expression
            let expr = self.expr()?;
            return Ok(Expr::Lambda(Lambda::new(params, Body { stmts: vec![Stmt::Expr(expr)] })));
        }
    }
}

// Statements
impl<'a> Parser<'a> {
    // End of statement
    pub fn expect_eos(&mut self) -> Result<(), ParseError> {
        let mut has_sep = false;
        while self.is_kind(TokenKind::Semi) || self.is_kind(TokenKind::Newline) {
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
            TokenKind::If => self.if_stmt()?,
            TokenKind::For => self.for_stmt()?,
            TokenKind::Var => self.var_stmt()?,
            TokenKind::Fn => self.fn_stmt()?,
            TokenKind::Type => self.type_stmt()?,
            // AutoUI Stmts
            TokenKind::Widget => self.widget_stmt()?,
            // Node Instance?
            TokenKind::Ident => {
                self.node_stmt()?
            }
            _ => self.expr_stmt()?,
        };
        Ok(stmt)
    }

    pub fn skip_empty_lines(&mut self) {
        while self.is_kind(TokenKind::Newline) {
            self.next();
        }
    }

    pub fn body(&mut self) -> Result<Body, ParseError> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        self.skip_empty_lines();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            stmts.push(self.stmt()?);
            self.expect_eos()?;
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Body { stmts })
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
            self.scope.enter_scope();
            let meta = Meta::Var(Var { name: Name::new(name.clone()), expr: Expr::Nil });
            self.scope.define(name.as_str(), Rc::new(meta));
            self.next(); // skip name
            self.expect(TokenKind::In)?;
            let range = self.iterable_expr()?;
            let body = self.body()?;
            self.scope.exit_scope();
            return Ok(Stmt::For(Name::new(name), range, body));
        }
        error_pos!("Expected for loop, got {:?}", self.kind())
    }

    pub fn var_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // skip var
        let name = self.cur.text.clone();
        self.expect(TokenKind::Ident)?;
        self.expect(TokenKind::Asn)?;
        let expr = self.rhs_expr()?;
        match expr.clone() {
            Expr::Lambda(lambda) => {
                let fn_decl = lambda.into();
                self.scope.define(name.as_str(), Rc::new(Meta::Fn(fn_decl)));
            }
            _ => {
                self.scope.define(name.as_str(), Rc::new(Meta::Var(Var { name: Name::new(name.clone()), expr: expr.clone() })));
            }
        }
        let var = Var { name: Name::new(name), expr };
        Ok(Stmt::Var(var))
    }

    pub fn fn_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // skip fn
        let name = self.cur.text.clone();
        self.expect(TokenKind::Ident)?;
        self.expect(TokenKind::LParen)?;
        self.scope.enter_scope();
        let params = self.fn_params()?;
        self.expect(TokenKind::RParen)?;
        let body = self.body()?;
        self.scope.exit_scope();
        let fn_expr = Fn::new(Name::new(name.clone()), params, body, Some(Type::Int));
        let fn_stmt = Stmt::Fn(fn_expr.clone());
        self.scope.define(name.as_str(), Rc::new(Meta::Fn(fn_expr)));
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
            let var = Var { name: Name::new(name.clone()), expr: default.clone().unwrap_or(Expr::Nil) };
            self.scope.define(name.as_str(), Rc::new(Meta::Var(var.clone())));
            params.push(Param { name: Name::new(name), ty, default });
            self.sep_params();
        }
        Ok(params)
    }

    pub fn type_name(&mut self) -> Result<Type, ParseError> {
        let ty = self.type_expr()?;
        Ok(ty)
    }

    pub fn expr_stmt(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expr()?;
        Ok(Stmt::Expr(expr))
    }

    pub fn type_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.next(); // skip type
        let name = Name::new(self.cur.text.clone());
        self.expect(TokenKind::Ident)?;
        self.expect(TokenKind::LBrace)?;
        // list of members or methods
        let mut members = Vec::new();
        let mut methods = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            if self.is_kind(TokenKind::Fn) {
                let fn_stmt = self.fn_stmt()?;
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
        let decl = TypeDecl { name: name.clone(), members, methods };
        // put type in scope
        self.scope.define(name.text.as_str(), Rc::new(Meta::Type(Type::User(decl.clone()))));
        Ok(Stmt::TypeDecl(decl))
    }

    pub fn type_member(&mut self) -> Result<Member, ParseError> {
        let name = Name::new(self.cur.text.clone());
        self.expect(TokenKind::Ident)?;
        let ty = self.type_expr()?;
        Ok(Member { name, ty })
    }

    pub fn type_expr(&mut self) -> Result<Type, ParseError> {
        let type_name = self.ident()?;
        self.next();
        match type_name {
            Expr::Ident(name) => {  
                let meta = self.scope.get_symbol(&name.text).ok_or(format!("Undefined type: {}", name.text))?;
                if let Meta::Type(ty) = meta.as_ref() {
                    Ok(ty.clone())
                } else {
                    error_pos!("Expected type, got {:?}", meta)
                }
            }
            _ => error_pos!("Expected type, got {:?}", type_name),
        }
    }

    // TODO: 暂时只检查两种情况：1，简单名称；2，点号表达式最左侧的名称
    pub fn check_symbol(&mut self, expr: &Expr) -> Result<(), ParseError> {
        match expr {
            Expr::Bina(l, op, _) => {
                match op {
                    Op::Dot => {
                        if let Expr::Ident(name) = l.as_ref() {
                            if !self.scope.exists(&name.text) {
                                return error_pos!("Undefined variable: {}", name.text);
                            }
                        }
                        Ok(())
                    }
                    _ => Ok(()),
                }
            }   
            Expr::Ident(name) => {
                if !self.scope.exists(&name.text) {
                    return error_pos!("Undefined variable: {}", name.text);
                }
                Ok(())
            }
            _ => Ok(()),
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
        self.scope.define(name.as_str(), Rc::new(Meta::Widget(widget.clone())));
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
            let var = self.var_stmt()?;
            match var {
                Stmt::Var(var) => {
                    model.vars.push(var);
                }
                _ => {
                    return error_pos!("Expected var declaration, got {:?}", var);
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
            let node = self.node_instance()?;
            view.nodes.push((node.name.clone(), node));
            self.expect_eos()?;
        }
        self.expect(TokenKind::RBrace)?;
        Ok(view)
    }

    pub fn node_stmt(&mut self) -> Result<Stmt, ParseError> {
        let ident = self.ident()?;
        self.next();

        let mut args = Args::new();
        let mut is_call = false;
        if self.is_kind(TokenKind::LParen) {
            args = self.args()?;
            is_call = true;
        }
        if self.is_kind(TokenKind::LBrace) { // node instance
            let body = self.body()?;
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
        } else { // call
            if is_call {
                return Ok(Stmt::Expr(Expr::Call(Call { name: Box::new(ident), args })));
            } else {
                return Ok(Stmt::Expr(self.expr_pratt_with_left(ident, 0)?));
            }
        }
    }

    fn node_arg_body(&mut self, name: &Name) -> Result<Node, ParseError> {
        let mut node = Node::new(name.clone());
        if self.is_kind(TokenKind::LParen) {
            // args
            let args = self.args()?;
            node.args = args;
        }

        // body
        if self.is_kind(TokenKind::LBrace) {
            self.expect(TokenKind::LBrace)?;
            self.skip_empty_lines();
            while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
                let pair = self.pair()?;
                node.props.insert(pair.key, pair.value.as_ref().clone());
                self.expect_eos()?;
            }
            self.expect(TokenKind::RBrace)?;
        }
        Ok(node)
    }

    pub fn node_body(&mut self, name: &Name) -> Result<Node, ParseError> {
        let mut node = Node::new(name.clone());
        node.body = self.body()?;
        Ok(node)
    }

    pub fn node_instance(&mut self) -> Result<Node, ParseError> {
        if self.is_kind(TokenKind::Ident) {
            let name = self.ident()?;
            if let Expr::Ident(name) = name {
                // name
                self.next();
                return self.node_arg_body(&name);
            }
        }
        error_pos!("Expected node name, got {:?}", self.kind())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_once(code: &str) -> Code {
        let mut scope = Universe::new();
        parse(code, &mut scope).unwrap()
    }

    #[test]
    fn test_parser() {
        let code = "1+2+3";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (stmt (bina (bina (int 1) (op +) (int 2)) (op +) (int 3))))");
    }

    #[test]
    fn test_if() {
        let code = "if true {1}";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (if (branch (true) (body (stmt (int 1))))");
    }

    #[test]
    fn test_if_else() {
        let code = "if false {1} else {2}";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (if (branch (false) (body (stmt (int 1))) (else (body (stmt (int 2))))");
    }

    #[test]
    fn test_var() {
        let code = "var x = 41";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (var (name x) (int 41)))");
    }

    #[test]
    fn test_var_use() {
        let code = "var x = 41; x+1";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (var (name x) (int 41)) (stmt (bina (name x) (op +) (int 1))))");
    }

    #[test]
    fn test_range() {
        let code = "1..5";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (stmt (bina (int 1) (op ..) (int 5))))");
    }

    #[test]
    fn test_for() {
        let code = "for i in 1..5 {i}";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (for (name i) (bina (int 1) (op ..) (int 5)) (body (stmt (name i))))");
    }

    #[test]
    fn test_for_with_print() {
        let code = "for i in 0..10 { print(i); print(i+1) }";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (for (name i) (bina (int 0) (op ..) (int 10)) (body (stmt (call (name print) (args (name i))) (stmt (call (name print) (args (bina (name i) (op +) (int 1))))))");
    }

    #[test]
    fn test_object() {
        let code = "{x:1, y:2}";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (stmt (object (pair (name x) (int 1)) (pair (name y) (int 2)))))");


        let code = "var a = { 1: 2, 3: 4 }; a.1";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(stmt (bina (name a) (op .) (int 1)))");
    }


    #[test]
    fn test_fn() {
        let code = "fn add(x, y) { x+y }";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (fn (name add) (params (param (name x) (type int)) (param (name y) (type int))) (body (stmt (bina (name x) (op +) (name y))))");
    }

    #[test]
    fn test_fn_call() {
        let code = "fn add(x, y) { x+y }; add(1, 2)";
        let ast = parse_once(code);
        let call = ast.stmts[1].clone();
        assert_eq!(call.to_string(), "(stmt (call (name add) (args (int 1) (int 2)))");
    }

    #[test]
    fn test_type_decl() {
        let code = "type Point {x int; y int}";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (type-decl (name Point) (members (member (name x) (type int)) (member (name y) (type int)))))");
    }

    #[test]
    fn test_type_inst() {
        let code = "type Point {x int; y int}; var p = Point(x=1, y=2); p.x";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(stmt (bina (name p) (op .) (name x)))");
    }


    #[test]
    fn test_lambda() {
        let code = "var x = || 1 + 2";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (var (name x) (lambda (body (stmt (bina (int 1) (op +) (int 2)))))");
    }

    #[test]
    fn test_lambda_with_params() {
        let code = "|a int, b int| a + b";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (stmt (lambda (params (param (name a) (type int)) (param (name b) (type int))) (body (stmt (bina (name a) (op +) (name b)))))");
    }

    #[test]
    fn test_widget() {
        let code = r#"
        widget counter {
            model {
                var count = 0
            }
            view {
                button("+") {
                    onclick: || count = count + 1
                }
            }
        }
        "#;
        let ast = parse_once(code);
        let widget = &ast.stmts[0];
        match widget { 
            Stmt::Widget(widget) => {
                let model = &widget.model;
                assert_eq!(model.to_string(), "(model (var (name count) (int 0)))");
                let view = &widget.view;
                assert_eq!(view.to_string(), "(view (node (name button) (args (str \"+\")) (props (pair (name onclick) (lambda (body (stmt (bina (name count) (op =) (bina (name count) (op +) (int 1))))))))");
            }
            _ => panic!("Expected widget, got {:?}", widget),
        }
    }


    #[test]
    fn test_pair() {
        let code = r#"
        id: 1
        name: "test"
        version: "0.1.0"
        "#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(stmt (pair (name version) (str \"0.1.0\")))");
    }

    #[test]
    fn test_node_instance() {
        let code = r#"button("OK") {
            border: 1
            kind: "primary"
        }"#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (node (name button) (args (str \"OK\")) (body (stmt (pair (name border) (int 1))) (stmt (pair (name kind) (str \"primary\")))))");
    }

    #[test]
    fn test_node_instance_without_args() {
        let code = r#"center {
            text("Hello")
        }"#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (node (name center) (body (stmt (call (name text) (args (str \"Hello\")))))");
    }


    #[test]
    fn test_array() {
        let code = r#"[
            {id: 1, name: "test"},
            {id: 2, name: "test2"},
            {id: 3, name: "test3"}
        ]"#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (stmt (array (object (pair (name id) (int 1)) (pair (name name) (str \"test\"))) (object (pair (name id) (int 2)) (pair (name name) (str \"test2\"))) (object (pair (name id) (int 3)) (pair (name name) (str \"test3\"))))))");
    }


    #[test]
    fn test_hex() {
        let code = "0x10";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (stmt (int 16)))");
    }

    #[test]
    fn test_dot() {
        let code = "var a = {b: [0, 1, 2]}; a.b[0]";
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(stmt (index (bina (name a) (op .) (name b)) (int 0)))");
    }

    #[test]
    fn test_fstr() {
        let code = r#"f"hello $name"#;
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (stmt (fstr (str \"hello \") (name name) (str \"\"))))");
    }

    #[test]
    fn test_fstr_with_expr() {
        let code = r#"var a = 1; var b = 2; f"a + b = ${a+b}"#;
        let ast = parse_once(code);
        let last = ast.stmts.last().unwrap();
        assert_eq!(last.to_string(), "(stmt (fstr (str \"a + b = \") (bina (name a) (op +) (name b)) (str \"\")))");
    }
}


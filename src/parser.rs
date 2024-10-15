use crate::token::{Token, TokenKind};
use crate::ast::{Code, Stmt, Expr, Op, Branch, Body, Name, Var, Fn, Param};
use crate::lexer::Lexer;
use crate::scope::Universe;
use crate::scope::Meta;

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
const PREC_OR: InfixPrec = infix_prec(2);
const PREC_AND: InfixPrec = infix_prec(3);
const PREC_EQ: InfixPrec = infix_prec(4);
const PREC_CMP: InfixPrec = infix_prec(5);
const PREC_Range: InfixPrec = infix_prec(6);
const PREC_ADD: InfixPrec = infix_prec(7);
const PREC_MUL: InfixPrec = infix_prec(8);
const PREC_SIGN: PrefixPrec = prefix_prec(9);
const PREC_NOT: PrefixPrec = prefix_prec(10);
const PREC_CALL: PostfixPrec = postfix_prec(11);
const PREC_INDEX: PostfixPrec = postfix_prec(12);
const PREC_ATOM: InfixPrec = infix_prec(13);

fn prefix_power(op: Op) -> Result<PrefixPrec, String> {
    match op {
        Op::Add | Op::Sub => Ok(PREC_SIGN),
        Op::Not => Ok(PREC_NOT),
        _ => return Err(format!("Invalid prefix operator: {}", op)),
    }
}

fn postfix_power(op: Op) -> Result<Option<PostfixPrec>, String> {
    match op {
        Op::LSquare => Ok(Some(PREC_INDEX)),
        Op::LParen => Ok(Some(PREC_CALL)),
        _ => Ok(None),
    }
}

fn infix_power(op: Op) -> Result<InfixPrec, String> {
    match op {
        Op::Add | Op::Sub => Ok(PREC_ADD),
        Op::Mul | Op::Div => Ok(PREC_MUL),
        Op::Asn => Ok(PREC_ASN),
        Op::Eq | Op::Neq => Ok(PREC_EQ),
        Op::Lt | Op::Gt | Op::Le | Op::Ge => Ok(PREC_CMP),
        Op::Range | Op::RangeEq => Ok(PREC_Range),
        _ => return Err(format!("Invalid infix operator: {}", op)),
    }
}

pub fn parse(code: &str, scope: &mut Universe) -> Result<Code, String> {
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

    pub fn is_kind(&mut self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    pub fn next(&mut self) -> &Token {
        self.cur = self.lexer.next();
        &self.cur
    }

    pub fn expect(&mut self, kind: TokenKind) -> Result<(), String>{
        if self.is_kind(kind) {
            self.next();
            Ok(())
        } else {
            Err(format!("Expected token kind: {:?}, got {:?}", kind, self.kind()))
        }
    }


}

impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> Result<Code, String> {
        let mut stmts = Vec::new();
        while !self.is_kind(TokenKind::EOF) {
            stmts.push(self.stmt()?)
        }
        Ok(Code { stmts })
    }

}

// Expressions
impl<'a> Parser<'a> {
    pub fn expr(&mut self) -> Result<Expr, String> {
        self.expr_pratt(0)
    }

    // simple Pratt parser
    // ref: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
    pub fn expr_pratt(&mut self, min_power: u8) -> Result<Expr, String> {
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
            // normal
            _ => self.atom()?,
        };
        loop {
            let op = match self.kind() {
                TokenKind::EOF | TokenKind::Semi | TokenKind::LBrace | TokenKind::RBrace | TokenKind::Comma => break,
                TokenKind::Add | TokenKind::Sub | TokenKind::Mul | TokenKind::Div | TokenKind::Not => self.op(),
                TokenKind::Range | TokenKind::RangeEq => self.op(),
                TokenKind::LSquare => self.op(),
                TokenKind::LParen => self.op(),
                TokenKind::Asn => self.op(),
                TokenKind::Eq | TokenKind::Neq | TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge => self.op(),
                TokenKind::RSquare => break,
                TokenKind::RParen => break,
                _ => return Err(format!("Expected operator, got {:?}", self.peek())),
            };
            // Postfix

            if let Ok(Some(power)) = postfix_power(op) {
                if power.l < min_power { break; }
                self.next(); // skip postfix op

                match op {
                    Op::LSquare => {
                        let rhs = self.expr_pratt(0)?;
                        self.expect(TokenKind::RSquare)?;
                        lhs = Expr::Bina(Box::new(lhs), op, Box::new(rhs));
                        continue;
                    }
                    Op::LParen => {
                        let args = self.args()?;
                        self.expect(TokenKind::RParen)?;
                        lhs = Expr::Call(Box::new(lhs), args);
                        continue;
                    }
                    _ => return Err(format!("Invalid postfix operator: {}", op)),
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
            _ => panic!("Expected operator, got {:?}", self.kind()),
        }
    }

    pub fn group(&mut self) -> Result<Expr, String> {
        self.next(); // skip (
        let expr = self.expr()?;
        self.expect(TokenKind::RParen)?; // skip )
        Ok(expr)
    }

    pub fn sep_array(&mut self) {
        if self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            self.next();
            return;
        }
        if self.is_kind(TokenKind::RSquare) {
            return;
        }
        panic!("Expected array separator, got {:?}", self.kind());
    }

    pub fn array(&mut self) -> Result<Expr, String> {
        self.next(); // skip [
        let mut elems = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RSquare) {
            elems.push(self.expr()?);
            self.sep_array();
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

    pub fn args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RParen) {
            args.push(self.expr()?);
            self.sep_args();
        }
        Ok(args)
    }
    
    pub fn ident(&mut self) -> Result<Expr, String> {
        let name = self.cur.text.clone();
        // check for existence
        if !self.scope.exists(&name) {
            return Err(format!("Undefined variable: {}", name));
        }
        Ok(Expr::Ident(name))
    }

    pub fn atom(&mut self) -> Result<Expr, String> {
        if self.is_kind(TokenKind::LParen) {
            return self.group();
        }
        let expr = match self.kind() {
            TokenKind::Int => Expr::Integer(self.cur.text.parse().unwrap()),
            TokenKind::Float => Expr::Float(self.cur.text.parse().unwrap()),
            TokenKind::True => Expr::Bool(true),
            TokenKind::False => Expr::Bool(false),
            TokenKind::Str => Expr::Str(self.cur.text.clone()),
            TokenKind::Ident => self.ident()?,
            TokenKind::Nil => Expr::Nil,
            _ => return Err(format!("Expected term, got {:?}", self.kind())),
        };

        self.next();
        Ok(expr)
    }
    
    pub fn if_expr(&mut self) -> Result<Expr, String> {
        let (branches, else_stmt) = self.if_contents()?;
        Ok(Expr::If(branches, else_stmt))
    }

    // An Expression that can be assigned to a variable, e.g. right-hand side of an assignment
    pub fn rhs_expr(&mut self) -> Result<Expr, String> {
        if self.is_kind(TokenKind::If) {
            self.if_expr()
        } else {
            self.expr()
        }
    }

    pub fn iterable_expr(&mut self) -> Result<Expr, String> {
        // TODO: how to check for range/array but reject other cases?
        self.expr()
    }
}

// Statements
impl<'a> Parser<'a> {
    // End of statement
    pub fn expect_eos(&mut self) -> Result<(), String> {
        while self.is_kind(TokenKind::Semi) || self.is_kind(TokenKind::Newline) {
            self.next();
        }
        Ok(())
    }

    pub fn stmt(&mut self) -> Result<Stmt, String> {
        let stmt = match self.kind() {
            TokenKind::If => self.if_stmt()?,
            TokenKind::For => self.for_stmt()?,
            TokenKind::Var => self.var_stmt()?,
            TokenKind::Fn => self.fn_stmt()?,
            _ => self.expr_stmt()?,
        };
        self.expect_eos()?;
        Ok(stmt)
    }

    pub fn body(&mut self) -> Result<Body, String> {
        self.expect(TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            stmts.push(self.stmt()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Body { stmts })
    }

    pub fn if_contents(&mut self) -> Result<(Vec<Branch>, Option<Body>), String> {
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

    pub fn if_stmt(&mut self) -> Result<Stmt, String> {
        let (branches, else_stmt) = self.if_contents()?;
        Ok(Stmt::If(branches, else_stmt))
    }

    pub fn for_stmt(&mut self) -> Result<Stmt, String> {
        self.next(); // skip `for`
        // enumerator
        if self.is_kind(TokenKind::Ident) {
            let name = self.cur.text.clone();
            self.scope.enter_scope();
            let meta = Meta::Var(Var { name: Name::new(name.clone()), expr: Expr::Nil });
            self.scope.define(name.clone(), meta);
            self.next(); // skip name
            self.expect(TokenKind::In)?;
            let range = self.iterable_expr()?;
            let body = self.body()?;
            self.scope.exit_scope();
            return Ok(Stmt::For(Name::new(name), range, body));
        }
        Err(format!("Expected for loop, got {:?}", self.kind()))
    }

    pub fn var_stmt(&mut self) -> Result<Stmt, String> {
        self.next(); // skip var
        let name = self.cur.text.clone();
        self.expect(TokenKind::Ident)?;
        self.expect(TokenKind::Asn)?;
        let expr = self.rhs_expr()?;
        let var = Var { name: Name::new(name.clone()), expr };
        self.scope.define(name.clone(), Meta::Var(var.clone()));
        Ok(Stmt::Var(var))
    }

    pub fn fn_stmt(&mut self) -> Result<Stmt, String> {
        self.next(); // skip fn
        let name = self.cur.text.clone();
        self.expect(TokenKind::Ident)?;
        self.expect(TokenKind::LParen)?;
        self.scope.enter_scope();
        let params = self.fn_params()?;
        self.expect(TokenKind::RParen)?;
        let body = self.body()?;
        self.scope.exit_scope();
        let fn_expr = Fn { name: Name::new(name.clone()), params, body };
        let fn_stmt = Stmt::Fn(fn_expr.clone());
        self.scope.define(name.clone(), Meta::Fn(fn_expr));
        Ok(fn_stmt)
    }

    pub fn sep_params(&mut self) {
        if self.is_kind(TokenKind::Comma) || self.is_kind(TokenKind::Newline) {
            self.next();
            return;
        }
        if self.is_kind(TokenKind::RParen) {
            return;
        }
        panic!("Expected parameter separator, got {:?}", self.kind());
    }

    // parse function parameters
    pub fn fn_params(&mut self) -> Result<Vec<Param>, String> {
        let mut params = Vec::new();
        while self.is_kind(TokenKind::Ident) {
            let name = self.cur.text.clone();
            self.next(); // skip name
            let mut default = None;
            if self.is_kind(TokenKind::Asn) {
                self.next(); // skip =
                let expr = self.expr()?;
                default = Some(expr);
            }
            let var = Var { name: Name::new(name.clone()), expr: default.clone().unwrap_or(Expr::Nil) };
            self.scope.define(name.clone(), Meta::Var(var.clone()));
            params.push(Param { name: Name::new(name), default });
            self.sep_params();
        }
        Ok(params)
    }

    pub fn expr_stmt(&mut self) -> Result<Stmt, String> {
        let expr = self.expr()?;
        if self.is_kind(TokenKind::Newline) || self.is_kind(TokenKind::Semi) {
            self.next();
        }
        Ok(Stmt::Expr(expr))
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
    fn test_fn() {
        let code = "fn add(x, y) { x+y }";
        let ast = parse_once(code);
        assert_eq!(ast.to_string(), "(code (fn (name add) (params (param x) (param y)) (body (stmt (bina (name x) (op +) (name y))))");
    }

    #[test]
    fn test_fn_call() {
        let code = "fn add(x, y) { x+y }; add(1, 2)";
        let ast = parse_once(code);
        let call = ast.stmts[1].clone();
        assert_eq!(call.to_string(), "(stmt (call (name add) (args (int 1) (int 2)))");
    }
}




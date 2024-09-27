use crate::token::{Token, TokenKind};
use crate::ast::{Code, Stmt, Expr, Op, Branch, Body, Name, Var};
use crate::lexer::Lexer;

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

const PREC_NONE: InfixPrec = InfixPrec { l: 0, r: 0 };
const PREC_ASN: InfixPrec = InfixPrec { l: 1, r: 2 };
const PREC_OR: InfixPrec = InfixPrec { l: 3, r: 4 };
const PREC_AND: InfixPrec = InfixPrec { l: 5, r: 6 };
const PREC_EQ: InfixPrec = InfixPrec { l: 7, r: 8 };
const PREC_CMP: InfixPrec = InfixPrec { l: 9, r: 10 };
const PREC_ADD: InfixPrec = InfixPrec { l: 11, r: 12 };
const PREC_MUL: InfixPrec = InfixPrec { l: 13, r: 14 };
const PREC_SIGN: PrefixPrec = PrefixPrec { l: (), r: 15 };
const PREC_NOT: PrefixPrec = PrefixPrec { l: (), r: 16 };
const PREC_INDEX: PostfixPrec = PostfixPrec { l: 17, r: () };
const PREC_CALL: InfixPrec  = InfixPrec { l: 19, r: 20 };
const PREC_ATOM: InfixPrec = InfixPrec { l: 21, r: 22 };


pub struct Parser<'a> {
    lexer: Lexer<'a>,
    cur: Token,
}

fn prefix_power(op: Op) -> PrefixPrec {
    match op {
        Op::Add | Op::Sub => PREC_SIGN,
        Op::Not => PREC_NOT,
        _ => panic!("Invalid prefix operator"),
    }
}

fn postfix_power(op: Op) -> Option<PostfixPrec> {
    match op {
        Op::LSquare => Some(PREC_INDEX),
        _ => None
    }
}

fn infix_power(op: Op) -> InfixPrec {
    match op {
        Op::Add | Op::Sub => PREC_ADD,
        Op::Mul | Op::Div => PREC_MUL,
        Op::Asn => PREC_ASN,
        Op::Eq | Op::Neq => PREC_EQ,
        Op::Lt | Op::Gt | Op::Le | Op::Ge => PREC_CMP,
        _ => panic!("Invalid infix operator"),
    }
}

impl<'a> Parser<'a> {
    pub fn new(code: &'a str) -> Self {
        let mut lexer = Lexer::new(code);
        let cur = lexer.next();
        Parser { lexer, cur }
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

    pub fn expect(&mut self, kind: TokenKind) {
        if self.is_kind(kind) {
            self.next();
        } else {
            panic!("Expected token kind: {:?}, got {:?}", kind, self.kind());
        }
    }

    pub fn expect_any(&mut self, kinds: &[TokenKind]) {
        if kinds.contains(&self.kind()) {
            self.next();
        } else {
            panic!("Expected token kind: {:?}, got {:?}", kinds, self.kind());
        }
    }
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> Code {
        let mut stmts = Vec::new();
        while !self.is_kind(TokenKind::EOF) {
            stmts.push(self.parse_stmt());
        }
        Code { stmts }
    }

    pub fn expr(&mut self) -> Expr {
        self.expr_pratt(0)
    }

    // simple Pratt parser
    // ref: https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html
    pub fn expr_pratt(&mut self, min_power: u8) -> Expr {
        // Prefix
        let mut lhs = match self.kind() {
            // unary
            TokenKind::Add | TokenKind::Sub | TokenKind::Not => {
                let op = self.op();
                let power = prefix_power(op);
                self.next(); // skip unary op
                let lhs = self.expr_pratt(power.r);
                Expr::Unary(op, Box::new(lhs))
            }
            // group
            TokenKind::LParen => {
                self.next(); // skip (
                let lhs = self.expr_pratt(0);
                self.expect(TokenKind::RParen); // skip )
                lhs
            }
            // array
            TokenKind::LSquare => self.array(),
            // normal
            _ => self.atom(),
        };
        loop {
            let op = match self.kind() {
                TokenKind::EOF | TokenKind::Semi | TokenKind::LBrace | TokenKind::RBrace | TokenKind::Comma => break,
                TokenKind::Add | TokenKind::Sub | TokenKind::Mul | TokenKind::Div | TokenKind::Not => self.op(),
                TokenKind::LSquare => self.op(),
                TokenKind::Asn => self.op(),
                TokenKind::Eq | TokenKind::Neq | TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge => self.op(),
                TokenKind::RSquare => break,
                TokenKind::RParen => break,
                _ => panic!("Expected operator, got {:?}", self.kind()),
            };
            // Postfix
            if let Some(power) = postfix_power(op) {
                if power.l < min_power { break; }
                self.next(); // skip postfix op

                match op {
                    Op::LSquare => {
                        let rhs = self.expr_pratt(0);
                        self.expect(TokenKind::RSquare);
                        lhs = Expr::Bina(Box::new(lhs), op, Box::new(rhs));
                        continue;
                    }
                    _ => panic!("Invalid postfix operator"),
                }
            }
            // Infix
            let power = infix_power(op);
            if power.l < min_power {
                break;
            }
            self.next(); // skip binary op
            let rhs = self.expr_pratt(power.r);
            lhs = Expr::Bina(Box::new(lhs), op, Box::new(rhs));
        }

        lhs
    }

    pub fn op(&mut self) -> Op {
        match self.kind() {
            TokenKind::Add => { Op::Add },
            TokenKind::Sub => { Op::Sub },
            TokenKind::Mul => { Op::Mul },
            TokenKind::Div => { Op::Div },
            TokenKind::LSquare => { Op::LSquare },
            TokenKind::Not => { Op::Not },
            TokenKind::Asn => { Op::Asn },
            TokenKind::Eq => { Op::Eq },
            TokenKind::Neq => { Op::Neq },
            TokenKind::Lt => { Op::Lt },
            TokenKind::Gt => { Op::Gt },
            TokenKind::Le => { Op::Le },
            TokenKind::Ge => { Op::Ge },
            _ => panic!("Expected operator"),
        }
    }

    pub fn group(&mut self) -> Expr {
        self.next(); // skip (
        let expr = self.expr();
        self.next(); // skip )
        expr
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

    pub fn array(&mut self) -> Expr {
        self.next(); // skip [
        let mut elems = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RSquare) {
            elems.push(self.expr());
            self.sep_array();
        }
        self.expect(TokenKind::RSquare); // skip ]
        Expr::Array(elems)
    }

    pub fn atom(&mut self) -> Expr {
        if self.is_kind(TokenKind::LParen) {
            return self.group();
        }
        let expr = match self.kind() {
            TokenKind::Integer => Expr::Integer(self.cur.text.parse().unwrap()),
            TokenKind::Float => Expr::Float(self.cur.text.parse().unwrap()),
            TokenKind::True => Expr::Bool(true),
            TokenKind::False => Expr::Bool(false),
            TokenKind::Str => Expr::Str(self.cur.text.clone()),
            TokenKind::Ident => Expr::Ident(self.cur.text.clone()),
            TokenKind::Nil => Expr::Nil,
            _ => panic!("Expected term, got {:?}", self.kind()),
        };

        self.next();
        expr
    }

    // End of statement
    pub fn expect_eos(&mut self) {
        while self.is_kind(TokenKind::Semi) || self.is_kind(TokenKind::Newline) {
            self.next();
        }
    }

    pub fn parse_stmt(&mut self) -> Stmt {
        let stmt = match self.kind() {
            TokenKind::If => self.parse_if(),
            TokenKind::For => self.parse_for(),
            TokenKind::Var => self.parse_var(),
            _ => self.parse_expr_stmt(),
        };
        self.expect_eos();
        stmt
    }

    pub fn body(&mut self) -> Body {
        self.expect(TokenKind::LBrace);
        let mut stmts = Vec::new();
        while !self.is_kind(TokenKind::EOF) && !self.is_kind(TokenKind::RBrace) {
            stmts.push(self.parse_stmt());
        }
        self.expect(TokenKind::RBrace);
        Body { stmts }
    }

    pub fn parse_if_contents(&mut self) -> (Vec<Branch>, Option<Body>) {
        let mut branches = Vec::new();
        self.next(); // skip if
        let cond = self.expr();
        let body = self.body();
        branches.push(Branch { cond, body });

        let mut else_stmt = None;
        while self.is_kind(TokenKind::Else) {
            self.next(); // skip else
            // more branches
            if self.is_kind(TokenKind::If) {
                self.next(); // skip if
                let cond = self.expr();
                let body = self.body();
                branches.push(Branch { cond, body });
            } else {
                // last else
                else_stmt = Some(self.body());
            }
        }
        (branches, else_stmt)
    }

    pub fn parse_if_expr(&mut self) -> Expr {
        let (branches, else_stmt) = self.parse_if_contents();
        Expr::If(branches, else_stmt)
    }

    pub fn parse_if(&mut self) -> Stmt {
        let (branches, else_stmt) = self.parse_if_contents();
        Stmt::If(branches, else_stmt)
    }

    pub fn parse_for(&mut self) -> Stmt {
        self.next(); // skip for
        let cond = self.expr();
        let body = self.body();
        Stmt::For(cond, body)
    }

    // An Expression that can be assigned to a variable
    pub fn asn_expr(&mut self) -> Expr {
        if self.is_kind(TokenKind::If) {
            self.parse_if_expr()
        } else {
            self.expr()
        }
    }

    pub fn parse_var(&mut self) -> Stmt {
        self.next(); // skip var
        let name = self.cur.text.clone();
        self.expect(TokenKind::Ident);
        self.expect(TokenKind::Asn);
        let expr = self.asn_expr();
        Stmt::Var(Var { name: Name::new(name), expr })
    }

    pub fn parse_expr_stmt(&mut self) -> Stmt {
        let expr = self.expr();
        if self.is_kind(TokenKind::Newline) || self.is_kind(TokenKind::Semi) {
            self.next();
        }
        Stmt::Expr(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let code = "1+2+3";
        let mut parser = Parser::new(code);
        let ast = parser.parse();
        assert_eq!(ast.to_string(), "(code (stmt (bina (bina (int 1) (op +) (int 2)) (op +) (int 3))))");
    }

    #[test]
    fn test_if() {
        let code = "if true {1}";
        let mut parser = Parser::new(code);
        let ast = parser.parse();
        assert_eq!(ast.to_string(), "(code (if (branch (true) (body (stmt (int 1))))");
    }

    #[test]
    fn test_if_else() {
        let code = "if false {1} else {2}";
        let mut parser = Parser::new(code);
        let ast = parser.parse();
        assert_eq!(ast.to_string(), "(code (if (branch (false) (body (stmt (int 1))) (else (body (stmt (int 2))))");
    }

    #[test]
    fn test_for() {
        let code = "for true {1}";
        let mut parser = Parser::new(code);
        let ast = parser.parse();
        assert_eq!(ast.to_string(), "(code (for (true) (body (stmt (int 1))))");
    }

    #[test]
    fn test_var() {
        let code = "var x = 41";
        let mut parser = Parser::new(code);
        let ast = parser.parse();
        assert_eq!(ast.to_string(), "(code (var (name x) (int 41)))");
    }
}




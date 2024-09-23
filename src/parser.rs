use crate::token::{Token, TokenKind};
use crate::ast::{Code, Stmt, Expr, Op};
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
            panic!("Expected token kind: {:?}", kind);
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
            // normal
            _ => self.atom(),
        };
        loop {
            let op = match self.kind() {
                TokenKind::EOF => break,
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
            _ => panic!("Expected term"),
        };

        self.next();
        expr
    }

    pub fn parse_stmt(&mut self) -> Stmt {
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
}




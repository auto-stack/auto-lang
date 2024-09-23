use crate::token::{Token, TokenKind};
use crate::ast::{Code, Stmt, Expr, Op};
use crate::lexer::Lexer;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    cur: Token,
}

fn binding_power(op: Op) -> (u8, u8) {
    match op {
        Op::Add | Op::Sub => (1, 2),
        Op::Mul | Op::Div => (3, 4),
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
        let mut lhs = self.term();
        println!("Pratt:{:?}", lhs);

        loop {
            println!("op: {:?}", self.kind());
            let op = match self.kind() {
                TokenKind::EOF => break,
                TokenKind::Add | TokenKind::Sub | TokenKind::Mul | TokenKind::Div => self.op(),
                _ => panic!("Expected operator"),
            };
            let (lpower, rpower) = binding_power(op);
            println!("lpower: {:?}, rpower: {:?}, min_power: {:?}", lpower, rpower, min_power);
            if lpower < min_power {
                break;
            }

            self.next();

            let rhs = self.expr_pratt(rpower);
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
            _ => panic!("Expected operator"),
        }
    }

    pub fn group(&mut self) -> Expr {
        self.next(); // skip (
        let expr = self.expr();
        self.next(); // skip )
        expr
    }

    pub fn term(&mut self) -> Expr {
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




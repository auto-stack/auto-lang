use crate::token::{Token, TokenKind};
use crate::ast::{Code, Stmt, Expr, Op};
use crate::lexer::Lexer;
use std::iter::Peekable;

pub struct Parser<'a> {
    lexer: Box<Peekable<Lexer<'a>>>,
}

impl<'a> Parser<'a> {
    pub fn new(code: &'a str) -> Self {
        Parser {
            lexer: Box::new(Lexer::new(code).peekable())
        }
    }

    pub fn peek(&mut self) -> Option<&Token> {
        self.lexer.peek()
    }

    pub fn kind(&mut self) -> Option<TokenKind> {
        self.peek().map(|token| token.kind)
    }

    pub fn is_kind(&mut self, kind: TokenKind) -> bool {
        self.peek().map_or(false, |token| token.kind == kind)
    }

    pub fn next(&mut self) -> Option<Token> {
        self.lexer.next()
    }
}

impl<'a> Parser<'a> {
    pub fn parse(&mut self) -> Code {
        let mut stmts = Vec::new();
        while let Some(stmt) = self.parse_stmt() {
            stmts.push(stmt);
        }
        Code { stmts }
    }

    pub fn expr(&mut self) -> Option<Expr> {
        self.bina()
    }

    pub fn op(&mut self) -> Option<Op> {
        match self.kind() {
            Some(TokenKind::Add) => { self.next(); Some(Op::Add) },
            Some(TokenKind::Sub) => { self.next(); Some(Op::Sub) },
            Some(TokenKind::Mul) => { self.next(); Some(Op::Mul) },
            Some(TokenKind::Div) => { self.next(); Some(Op::Div) },
            _ => None,
        }
    }

    pub fn bina(&mut self) -> Option<Expr> {
        if let Some(left) = self.term() {
            if let Some(op) = self.op() {
                if let Some(right) = self.term() {
                    return Some(Expr::Bina(Box::new(left), op, Box::new(right)));
                }
            }
            return Some(left);
        }
        None
    }

    pub fn group(&mut self) -> Option<Expr> {
        self.next(); // skip (
        let expr = self.expr();
        self.next(); // skip )
        expr
    }

    pub fn term(&mut self) -> Option<Expr> {
        if self.is_kind(TokenKind::LParen) {
            return self.group();
        }
        match self.next() {
            Some(token) => {
                match token.kind {
                    TokenKind::Integer => Some(Expr::Integer(token.text.parse().unwrap())),
                    TokenKind::Float => Some(Expr::Float(token.text.parse().unwrap())),
                    TokenKind::True => Some(Expr::Bool(true)),
                    TokenKind::False => Some(Expr::Bool(false)),
                    TokenKind::Str => Some(Expr::Str(token.text)),
                    TokenKind::Ident => Some(Expr::Ident(token.text)),
                    TokenKind::Nil => Some(Expr::Nil),
                    _ => None,
                }
            },
            None => None,
        }
    }

    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        if let Some(expr) = self.expr() {
            if let Some(token) = self.next() {
                if token.kind == TokenKind::Newline || token.kind == TokenKind::Semi || token.kind == TokenKind::EOF {
                    return Some(Stmt::Expr(expr));
                }
            } else {
                return Some(Stmt::Expr(expr));
            }
        }
        None
    }
}

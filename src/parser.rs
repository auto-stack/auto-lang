use crate::token::TokenKind;
use crate::ast::{Code, Stmt, Expr};
use crate::lexer::Lexer;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(code: &'a str) -> Self {
        Parser {
            lexer: Lexer::new(code),
        }
    }

    pub fn parse(&mut self) -> Code {
        let mut stmts = Vec::new();
        while let Some(token) = self.lexer.next() {
            match token.kind {
                TokenKind::Integer => {
                    let stmt = Stmt::Expr(Expr::Integer(token.text.parse().unwrap()));
                    stmts.push(stmt);
                }
                TokenKind::Float => {
                    let stmt = Stmt::Expr(Expr::Float(token.text.parse().unwrap()));
                    stmts.push(stmt);
                }
                TokenKind::True => {
                    let stmt = Stmt::Expr(Expr::Bool(true));
                    stmts.push(stmt);
                }
                TokenKind::False => {
                    let stmt = Stmt::Expr(Expr::Bool(false));
                    stmts.push(stmt);
                }
                TokenKind::Str => {
                    let stmt = Stmt::Expr(Expr::Str(token.text));
                    stmts.push(stmt);
                }
                TokenKind::Ident => {
                    let stmt = Stmt::Expr(Expr::Ident(token.text));
                    stmts.push(stmt);
                }
                _ => panic!("Unexpected token: {:?}", token.kind),
            }
        }
        Code { stmts }
    }
}

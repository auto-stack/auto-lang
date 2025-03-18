use super::{Transpiler, ToStrError};
use auto_val::AutoStr;
use std::io::Write;
use auto_val::Op;
use crate::ast::*;
use crate::AutoResult;
pub struct CTranspiler {
    indent: usize,
    includes: Vec<u8>,
    pub header: Vec<u8>,
    name: AutoStr,
}

impl CTranspiler {
    pub fn new(name: AutoStr) -> Self {
        Self { indent: 0, includes: Vec::new(), header: Vec::new(), name }
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent -= 1;
    }

    fn print_indent(&self, out: &mut impl Write) -> AutoResult<()> {
        for _ in 0..self.indent {
            out.write(b"    ").to()?;
        }
        Ok(())
    }
}

impl CTranspiler {

    pub fn code(&mut self, code: &Code, out: &mut impl Write) -> AutoResult<()> {
        for stmt in code.stmts.iter() {
            self.stmt(stmt, out)?;
            out.write(b"\n").to()?;
        }
        Ok(())
    }

    fn eos(&mut self, out: &mut impl Write) -> AutoResult<()> {
        out.write(b";").to()
    }

    fn stmt(&mut self, stmt: &Stmt, out: &mut impl Write) -> AutoResult<()> {
        match stmt {
            Stmt::Expr(expr) => {self.expr(expr, out)?; self.eos(out)},
            Stmt::Store(store) => {self.store(store, out)?; self.eos(out)},
            Stmt::Fn(fn_decl) => self.fn_decl(fn_decl, out),
            Stmt::For(for_stmt) => self.for_stmt(for_stmt, out),
            Stmt::If(branches, otherwise) => self.if_stmt(branches, otherwise, out),
            _ => Err(format!("C Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    fn expr(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match expr {
            Expr::Int(i) => out.write_all(i.to_string().as_bytes()).to(),
            Expr::Bina(lhs, op, rhs) => {
                match op {
                    Op::Range => self.range(lhs, rhs, out)?,
                    _ => {
                        self.expr(lhs, out)?;
                        out.write(format!(" {} ", op.op()).as_bytes()).to()?;
                        self.expr(rhs, out)?
                    }
                }
                Ok(())
            }
            Expr::Unary(op, expr) => {
                out.write(format!("{}", op.op()).as_bytes()).to()?;
                self.expr(expr, out)?;
                Ok(())
            }
            Expr::Ident(name) => out.write_all(name.text.as_bytes()).to(),
            Expr::Str(s) => out.write_all(format!("\"{}\"", s).as_bytes()).to(),
            Expr::Call(call) => self.call(call, out),
            Expr::Array(array) => self.array(array, out), 
            _ => Err(format!("C Transpiler: unsupported expression: {}", expr).into()),
        }
    }

    fn fn_decl(&mut self, fn_decl: &Fn, out: &mut impl Write) -> AutoResult<()> {
        // header
        let mut header = Vec::new();
        self.fn_sig(&fn_decl, &mut header)?;
        self.header.extend(header);
        self.header.write(b";\n").to()?;

        // source
        self.fn_sig(&fn_decl, out)?;
        out.write(b" ").to()?;
        self.body(&fn_decl.body, out, true)?;
        Ok(())
    }

    fn fn_sig(&mut self, fn_decl: &Fn, out: &mut impl Write) -> AutoResult<()> {
        // return type
        if !matches!(fn_decl.ret, Type::Unknown) {
            out.write(format!("{} ", fn_decl.ret).as_bytes()).to()?;
        } else {
            out.write(b"void ").to()?;
        }
        // name
        let name = fn_decl.name.clone();
        out.write(name.text.as_bytes()).to()?;
        // params
        out.write(b"(").to()?;
        let params = fn_decl
            .params
            .iter()
            .map(|p| format!("int {}", p.name.text))
            .collect::<Vec<_>>()
            .join(", ");
        out.write(params.as_bytes()).to()?;
        out.write(b")").to()?;

        Ok(())
    }

    fn body(&mut self, body: &Body, out: &mut impl Write, has_return: bool) -> AutoResult<()> {
        out.write(b"{\n").to()?;
        self.indent();
        for (i, stmt) in body.stmts.iter().enumerate() {
            self.print_indent(out)?;
            if i < body.stmts.len() - 1 {
                self.stmt(stmt, out)?;
                out.write(b"\n").to()?;
            } else {
                if has_return {
                    out.write(b"return ").to()?;
                }
                self.stmt(stmt, out)?;
                out.write(b"\n").to()?;
            }
        }
        self.dedent();
        out.write(b"}").to()?;
        Ok(())
    }

    fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
        if matches!(store.kind, StoreKind::Var) {
            return Err(format!("C Transpiler: unsupported store kind: {:?}", store.kind).into());
        }
        match &store.ty {
            Type::Array(array_type) => {
                let elem_type = &array_type.elem;
                let len = array_type.len;
                out.write(format!("{} {}[{}] = ", elem_type, store.name.text, len).as_bytes()).to()?;
            }
            _ => {
                out.write(format!("{} {} = ", store.ty, store.name.text).as_bytes()).to()?;
            }
        }
        self.expr(&store.expr, out)?;
        Ok(())
    }

    fn for_stmt(&mut self, for_stmt: &For, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"for (").to()?;
        self.expr(&for_stmt.range, out)?;
        out.write(b") ").to()?;
        self.body(&for_stmt.body, out, false)?;
        Ok(())
    }

    fn range(&mut self, start: &Expr, end: &Expr, out: &mut impl Write) -> AutoResult<()> {
        // TODO: check index name for deep loops
        out.write(b"int i = ").to()?;
        self.expr(start, out)?;
        out.write(b"; i < ").to()?;
        self.expr(end, out)?;
        out.write(b"; i++").to()?;
        Ok(())
    }

    fn if_stmt(&mut self, branches: &Vec<Branch>, otherwise: &Option<Body>, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"if ").to()?;
        for (i, branch) in branches.iter().enumerate() {
            out.write(b"(").to()?;
            self.expr(&branch.cond, out)?;
            out.write(b") ").to()?;
            self.body(&branch.body, out, false)?;
            if i < branches.len() - 1 {
                out.write(b" else ").to()?;
            }
        }
        if let Some(body) = otherwise {
            out.write(b" else ").to()?;
            self.body(body, out, false)?;
        }
        Ok(())
    }

    fn process_print(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // TODO: check type of the args and format accordingly
        // get number and type of args
        let mut arg_types = Vec::new();
        for arg in call.args.args.iter() {
            match arg {
                Arg::Pos(expr) => {
                    match expr {
                        Expr::Int(_) => arg_types.push("%d"),
                        Expr::Str(_) => arg_types.push("%s"),
                        Expr::Float(_) => arg_types.push("%f"),
                        // TODO: check the actual type of the identifier
                        Expr::Ident(_) => arg_types.push("%d"),
                        _ => {
                            // other types are now viewed as ints
                            arg_types.push("%d");
                        }
                    }
                }
                _ => {
                    // TODO: implement identifier args and named args
                }
            }
        }
        let fmt = format!("printf(\"{}\", ", arg_types.join(" "));
        out.write(fmt.as_bytes()).to()
    }
    

    fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        if let Expr::Ident(name) = &call.name.as_ref() {
            if name.text == "print" {
                self.process_print(call, out)?;
            } else {
                self.expr(&call.name, out)?;
                out.write(b"(").to()?;
            }
        } else {
            self.expr(&call.name, out)?;
            out.write(b"(").to()?;
        }
        for (i, arg) in call.args.args.iter().enumerate() {
            self.arg(arg, out)?;
            if i < call.args.args.len() - 1 {
                out.write(b", ").to()?;
            }
        }
        // TODO: support named args in C
        // Find where a named arg is positioned, and insert default arg values in between
        // // // for (name, expr) in &call.args.map {
        // //     self.expr(expr, out)?;
        // }
        out.write(b")").to()?;
        Ok(())
    }

    fn arg(&mut self, arg: &Arg, out: &mut impl Write) -> AutoResult<()> {
        match arg {
            Arg::Name(name) => self.str(name.text.as_str(), out),
            Arg::Pair(_, expr) => self.expr(expr, out),
            Arg::Pos(expr) => self.expr(expr, out),
        }
    }

    fn str(&mut self, s: &str, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"\"").to()?;
        out.write(s.as_bytes()).to()?;
        out.write(b"\"").to()?;
        Ok(())
    }

    fn array(&mut self, array: &Vec<Expr>, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"{").to()?;
        for (i, expr) in array.iter().enumerate() {
            self.expr(expr, out)?;
            if i < array.len() - 1 {
                out.write(b", ").to()?;
            }
        }
        out.write(b"}").to()?;
        Ok(())
    }

    fn is_returnable(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr) => {
                match expr {
                    Expr::Call(call) => {
                        if let Expr::Ident(name) = &call.name.as_ref() {
                            if name.text == "print" {
                                return false;
                            }
                        }
                        true
                    }
                    _ => true,
                }
            }
            _ => false,
        }
    }
}

impl Transpiler for CTranspiler {
    fn transpile(&mut self, ast: Code, out: &mut impl Write) -> AutoResult<()> {
        // Split stmts into decls and main
        // TODO: handle potential includes when needed
        let mut decls: Vec<Stmt> = Vec::new();
        let mut main: Vec<Stmt> = Vec::new();

        for stmt in ast.stmts.into_iter() {
            match stmt {
                Stmt::Fn(_) => decls.push(stmt),
                Stmt::Store(_) => decls.push(stmt),
                Stmt::For(_) => main.push(stmt),
                Stmt::If(_, _) => main.push(stmt),
                Stmt::Expr(ref expr) => {
                    match expr {
                        Expr::Call(call) => {
                            if let Expr::Ident(name) = &call.name.as_ref() {
                                if name.text == "print" {
                                    self.includes.write(b"#include <stdio.h>\n").to()?;
                                }
                            }
                        }
                        _ => { }
                    }
                    main.push(stmt);
                }
                _ => {}
            }
        }

        // write header guards
        let upper = self.name.to_uppercase();
        let name_bytes = upper.as_bytes();
        self.header.write(b"#ifndef ").to()?;
        self.header.write(name_bytes).to()?;
        self.header.write(b"_H\n#define ").to()?;
        self.header.write(name_bytes).to()?;
        self.header.write(b"_H\n\n").to()?;

        // TODO: Includes on demand
        if !self.includes.is_empty() {
            out.write(&self.includes).to()?;
        }

        // Decls
        for decl in decls.iter() {
            self.stmt(decl, out)?;
            out.write(b"\n").to()?;
        }
        if !decls.is_empty() {
            out.write(b"\n").to()?;
        }

        // Main
        // TODO: check wether auto code already has a main function
        if !main.is_empty() {
            out.write(b"int main(void) {\n").to()?;
            self.indent();
            for (i, stmt) in main.iter().enumerate() {
                self.print_indent(out)?;
                if i < main.len() - 1 {
                    self.stmt(stmt, out)?;
                    out.write(b"\n").to()?;
                } else {
                    if self.is_returnable(stmt) {
                        out.write(b"return ").to()?;
                        self.stmt(stmt, out)?;
                        out.write(b"\n").to()?;
                    } else {
                        self.stmt(stmt, out)?;
                        out.write(b"\n").to()?;
                        self.print_indent(out)?;
                        out.write(b"return 0;\n").to()?;
                    }
                }
            }
            self.dedent();
            out.write(b"}\n").to()?;
        }

        // header guard end
        self.header.write(b"\n#endif\n\n").to()?;
        Ok(())
    }
}

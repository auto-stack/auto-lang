use super::{Sink, ToStrError, Trans};
use crate::ast::Type;
use crate::ast::*;
use crate::parser::Parser;
use crate::scope::Meta;
use crate::universe::Universe;
use crate::AutoResult;
use auto_val::AutoStr;
use auto_val::Op;
use auto_val::{shared, Shared};
use std::cell::RefCell;
use std::collections::HashSet;
use std::io::Write;
use std::rc::Rc;

pub struct CTrans {
    indent: usize,
    libs: HashSet<AutoStr>,
    pub header: Vec<u8>,
    name: AutoStr,
    scope: Shared<Universe>,
}

impl CTrans {
    pub fn new(name: AutoStr) -> Self {
        Self {
            indent: 0,
            libs: HashSet::new(),
            header: Vec::new(),
            name,
            scope: shared(Universe::default()),
        }
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

impl CTrans {
    pub fn code(&mut self, code: Code, sink: &mut Sink) -> AutoResult<()> {
        for (i, stmt) in code.stmts.iter().enumerate() {
            if i > 0 {
                sink.body.write(b"\n")?;
            }
            self.stmt(stmt, sink)?;
        }
        if let Some(stmt) = code.stmts.last() {
            if !stmt.is_new_block() {
                sink.body.write(b"\n")?;
            }
        }
        Ok(())
    }

    fn eos(&mut self, out: &mut impl Write) -> AutoResult<()> {
        out.write(b";").to()
    }

    fn stmt(&mut self, stmt: &Stmt, sink: &mut Sink) -> AutoResult<()> {
        let out = &mut sink.body;
        match stmt {
            Stmt::TypeDecl(type_decl) => self.type_decl(type_decl, sink),
            Stmt::Expr(expr) => {
                self.expr(expr, out)?;
                self.eos(out)
            }
            Stmt::Store(store) => {
                self.store(store, out)?;
                self.eos(out)
            }
            Stmt::Fn(fn_decl) => self.fn_decl(fn_decl, sink),
            Stmt::For(for_stmt) => self.for_stmt(for_stmt, sink),
            Stmt::If(if_) => self.if_stmt(if_, sink),
            Stmt::Use(use_stmt) => self.use_stmt(use_stmt, out),
            Stmt::EnumDecl(enum_decl) => self.enum_decl(enum_decl, out),
            Stmt::Alias(alias) => self.alias(alias, out),
            _ => Err(format!("C Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    fn alias(&mut self, alias: &Alias, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"#define ")?;
        out.write(alias.alias.as_bytes())?;
        out.write(b" ")?;
        out.write(alias.target.as_bytes())?;
        out.write(b"\n")?;
        Ok(())
    }

    fn enum_decl(&mut self, enum_decl: &EnumDecl, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"enum ")?;
        out.write(enum_decl.name.as_bytes())?;
        out.write(b" {\n")?;
        for item in enum_decl.items.iter() {
            println!("Enum Item: {}", item.name);
            out.write(b"    ")?;
            out.write(format!("{}_", enum_decl.name.to_uppercase()).as_bytes())?;
            out.write(format!("{}", item.name).as_bytes())?;
            out.write(b" = ")?;
            out.write(format!("{}", item.value).as_bytes())?;
            out.write(b",\n")?;
        }
        out.write(b"};")?;
        Ok(())
    }

    fn type_decl(&mut self, type_decl: &TypeDecl, sink: &mut Sink) -> AutoResult<()> {
        let out = &mut sink.body;
        // write type body
        out.write(b"struct ")?;
        out.write(type_decl.name.as_bytes())?;
        out.write(b" {\n")?; // TODO: no newline for short decls
        for field in type_decl.members.iter() {
            out.write(b"    ")?;
            out.write(field.ty.unique_name().as_bytes())?;
            out.write(b" ")?;
            out.write(field.name.as_bytes())?;
            out.write(b";\n")?;
        }
        out.write(b"};\n")?;

        // write methods
        if !type_decl.methods.is_empty() {
            out.write(b"\n")?;
        }
        for method in type_decl.methods.iter() {
            let out = &mut sink.body;
            out.write(method.ret.unique_name().as_bytes())?;
            out.write(b" ")?;
            out.write(method.name.as_bytes())?;
            out.write(b"(")?;
            // self
            out.write(b"struct ")?;
            out.write(type_decl.name.as_bytes())?;
            out.write(b" *s")?;
            if !method.params.is_empty() {
                out.write(b", ")?;
            }
            out.write(
                method
                    .params
                    .iter()
                    .map(|p| p.ty.unique_name())
                    .collect::<Vec<_>>()
                    .join(", ")
                    .as_bytes(),
            )?;
            out.write(b") ")?;
            // method body
            self.body(&method.body, sink, !matches!(method.ret, Type::Void))?;
            sink.body.write(b"\n")?;
        }
        Ok(())
    }

    fn use_stmt(&mut self, use_stmt: &Use, _out: &mut impl Write) -> AutoResult<()> {
        for path in use_stmt.paths.iter() {
            if !self.libs.contains(path) {
                self.libs.insert(path.clone());
            }
        }
        Ok(())
    }

    fn float(&mut self, _f: &f64, txt: &str, out: &mut impl Write) -> AutoResult<()> {
        out.write_all(txt.as_bytes()).to()
    }

    fn dot(&mut self, lhs: &Expr, rhs: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match lhs {
            Expr::Ident(ident) => {
                if ident == "s" {
                    out.write(b"s->")?;
                    self.expr(rhs, out)?;
                    return Ok(());
                }
                let ty = self.lookup_type(ident);
                match ty {
                    Type::Enum(_) => match rhs {
                        Expr::Ident(rid) => {
                            out.write(
                                format!("{}_{}", ident.to_uppercase(), rid.to_uppercase())
                                    .as_bytes(),
                            )?;
                            return Ok(());
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            _ => {}
        }
        // if rhs is ptr or tgt
        match rhs {
            Expr::Ident(id) => match id.as_str() {
                "ptr" => {
                    out.write(b"&").to()?;
                    self.expr(lhs, out)?;
                }
                "tgt" => {
                    out.write(b"*").to()?;
                    self.expr(lhs, out)?;
                }
                _ => {
                    self.expr(lhs, out)?;
                    out.write(format!(".").as_bytes()).to()?;
                    self.expr(rhs, out)?;
                }
            },
            _ => {
                println!("got {:?}", rhs);
                self.expr(lhs, out)?;
                out.write(format!(".").as_bytes()).to()?;
                self.expr(rhs, out)?;
            }
        }
        Ok(())
    }

    fn expr(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match expr {
            Expr::Int(i) => out.write_all(i.to_string().as_bytes()).to(),
            Expr::Bina(lhs, op, rhs) => {
                match op {
                    Op::Range => self.range(lhs, rhs, out)?,
                    _ => match op {
                        Op::Dot => {
                            self.dot(lhs, rhs, out)?;
                        }
                        _ => {
                            self.expr(lhs, out)?;
                            _ = out.write(format!(" {} ", op.op()).as_bytes()).to()?;
                            self.expr(rhs, out)?
                        }
                    },
                }
                Ok(())
            }
            Expr::Unary(op, expr) => {
                out.write(format!("{}", op.op()).as_bytes()).to()?;
                self.expr(expr, out)?;
                Ok(())
            }
            Expr::Ident(name) => out.write_all(name.as_bytes()).to(),
            Expr::Str(s) => out.write_all(format!("\"{}\"", s).as_bytes()).to(),
            Expr::CStr(s) => out.write_all(format!("\"{}\"", s).as_bytes()).to(),
            Expr::Call(call) => self.call(call, out),
            Expr::Array(array) => self.array(array, out),
            Expr::Float(f, t) => self.float(f, t, out),
            Expr::Double(d, t) => self.float(d, t, out),
            Expr::Index(arr, idx) => self.index(arr, idx, out),
            Expr::Node(nd) => self.node(nd, out),
            Expr::Pair(pair) => self.pair(pair, out),
            _ => Err(format!("C Transpiler: unsupported expression: {}", expr).into()),
        }
    }

    fn key(&mut self, key: &Key, out: &mut impl Write) -> AutoResult<()> {
        out.write(format!(".{}", key.to_astr()).as_bytes())?;
        Ok(())
    }

    fn pair(&mut self, pair: &Pair, out: &mut impl Write) -> AutoResult<()> {
        self.key(&pair.key, out)?;
        out.write(b" = ")?;
        self.expr(&pair.value, out)?;
        Ok(())
    }

    fn node(&mut self, node: &Node, out: &mut impl Write) -> AutoResult<()> {
        // out.write(node.name.as_bytes())?;
        out.write(b"{")?;
        for (i, stmt) in node.body.stmts.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            match stmt {
                Stmt::Expr(expr) => self.expr(expr, out)?,
                _ => {
                    return Err(format!(
                        "C Transpiler: unsupported statement in node body: {}",
                        stmt
                    )
                    .into())
                }
            }
        }
        out.write(b"}")?;
        Ok(())
    }

    fn index(&mut self, arr: &Box<Expr>, idx: &Box<Expr>, out: &mut impl Write) -> AutoResult<()> {
        self.expr(arr, out)?;
        out.write(b"[")?;
        self.expr(idx, out)?;
        out.write(b"]")?;
        Ok(())
    }

    fn fn_decl(&mut self, fn_decl: &Fn, sink: &mut Sink) -> AutoResult<()> {
        let out = &mut sink.body;
        // header
        let mut header = Vec::new();
        self.fn_sig(&fn_decl, &mut header)?;
        self.header.extend(header);
        self.header.write(b";\n").to()?;

        // source
        self.fn_sig(&fn_decl, out)?;
        out.write(b" ").to()?;

        self.scope.borrow_mut().enter_fn(fn_decl.name.clone());
        self.body(&fn_decl.body, sink, true)?;
        self.scope.borrow_mut().exit_fn();

        sink.body.write(b"\n")?;
        Ok(())
    }

    fn fn_sig(&mut self, fn_decl: &Fn, out: &mut impl Write) -> AutoResult<()> {
        // special: main
        // TODO: main with args
        if fn_decl.name == "main" {
            out.write(b"int main(void)").to()?;
            return Ok(());
        }
        // return type
        if !matches!(fn_decl.ret, Type::Unknown) {
            out.write(format!("{} ", fn_decl.ret).as_bytes()).to()?;
        } else {
            out.write(b"void ").to()?;
        }
        // name
        let name = fn_decl.name.clone();
        out.write(name.as_bytes()).to()?;
        // params
        out.write(b"(").to()?;
        let params = fn_decl
            .params
            .iter()
            .map(|p| format!("int {}", p.name))
            .collect::<Vec<_>>()
            .join(", ");
        out.write(params.as_bytes()).to()?;
        out.write(b")").to()?;

        Ok(())
    }

    fn body(&mut self, body: &Body, sink: &mut Sink, has_return: bool) -> AutoResult<()> {
        self.scope.borrow_mut().enter_scope();
        sink.body.write(b"{\n").to()?;
        self.indent();
        for (i, stmt) in body.stmts.iter().enumerate() {
            self.print_indent(&mut sink.body)?;
            if i < body.stmts.len() - 1 {
                self.stmt(stmt, sink)?;
                sink.body.write(b"\n").to()?;
            } else {
                // last stmt
                if has_return {
                    if self.is_returnable(stmt) {
                        sink.body.write(b"return ").to()?;
                    }
                }
                self.stmt(stmt, sink)?;
                sink.body.write(b"\n").to()?;
                if has_return && !self.is_returnable(stmt) {
                    self.print_indent(&mut sink.body)?;
                    sink.body.write(b"return 0;\n").to()?;
                }
            }
        }
        self.dedent();
        sink.body.write(b"}").to()?;
        self.scope.borrow_mut().exit_scope();
        Ok(())
    }

    fn c_type_name(&self, ty: &Type) -> String {
        match ty {
            Type::Int => "int".to_string(),
            Type::Float => "float".to_string(),
            Type::Double => "double".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Str => "char*".to_string(),
            Type::CStr => "char*".to_string(),
            Type::Array(array_type) => {
                let elem_type = &array_type.elem;
                let len = array_type.len;
                format!("{}[{}]", self.c_type_name(elem_type), len)
            }
            Type::User(usr_type) => format!("struct {}", usr_type.name),
            Type::Ptr(ptr) => {
                format!("{}*", self.c_type_name(&ptr.of.borrow()))
            }
            Type::Enum(en) => {
                format!("enum {}", en.borrow().name)
            }
            Type::Unknown => "unknown".to_string(),
            _ => {
                println!("Unsupported type for C transpiler: {}", ty);
                panic!("Unsupported type for C transpiler: {}", ty);
            }
        }
    }

    fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
        if matches!(store.kind, StoreKind::Var) {
            return Err(format!("C Transpiler: unsupported store kind: {:?}", store.kind).into());
        }
        match &store.ty {
            Type::Array(array_type) => {
                let elem_type = &array_type.elem;
                let len = array_type.len;
                out.write(format!("{} {}[{}] = ", elem_type, store.name, len).as_bytes())
                    .to()?;
            }
            _ => {
                let type_name = self.c_type_name(&store.ty);
                out.write(format!("{} {} = ", type_name, store.name).as_bytes())
                    .to()?;
            }
        }
        self.expr(&store.expr, out)?;
        Ok(())
    }

    fn for_stmt(&mut self, for_stmt: &For, sink: &mut Sink) -> AutoResult<()> {
        sink.body.write(b"for (").to()?;
        self.expr(&for_stmt.range, &mut sink.body)?;
        sink.body.write(b") ").to()?;
        self.body(&for_stmt.body, sink, false)?;
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

    fn if_stmt(&mut self, if_: &If, sink: &mut Sink) -> AutoResult<()> {
        sink.body.write(b"if ").to()?;
        for (i, branch) in if_.branches.iter().enumerate() {
            sink.body.write(b"(").to()?;
            self.expr(&branch.cond, &mut sink.body)?;
            sink.body.write(b") ").to()?;
            self.body(&branch.body, sink, false)?;
            if i < if_.branches.len() - 1 {
                sink.body.write(b" else ").to()?;
            }
        }
        if let Some(body) = &if_.else_ {
            sink.body.write(b" else ").to()?;
            self.body(body, sink, false)?;
        }
        Ok(())
    }

    fn lookup_meta(&self, ident: &AutoStr) -> Option<Rc<Meta>> {
        self.scope.borrow().lookup_meta(ident)
    }

    fn lookup_type(&self, ident: &AutoStr) -> Type {
        self.scope.borrow().lookup_type(ident)
    }

    fn process_print(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // TODO: check type of the args and format accordingly
        self.libs.insert("<stdio.h>".into());
        // get number and type of args
        let mut arg_types = Vec::new();
        for arg in call.args.args.iter() {
            match arg {
                Arg::Pos(expr) => {
                    match expr {
                        Expr::Int(_) => arg_types.push("%d"),
                        Expr::Str(_) => arg_types.push("%s"),
                        Expr::CStr(_) => arg_types.push("%s"),
                        Expr::Float(_, _) => arg_types.push("%f"),
                        // TODO: check the actual type of the identifier
                        Expr::Ident(ident) => {
                            let meta = self.lookup_meta(ident);
                            if let Some(meta) = meta {
                                match meta.as_ref() {
                                    Meta::Store(st) => match st.ty {
                                        Type::Str | Type::CStr => {
                                            arg_types.push("%s");
                                        }
                                        _ => {
                                            println!("Got store: {:?}", st);
                                            arg_types.push("%d");
                                        }
                                    },
                                    _ => {
                                        arg_types.push("%d");
                                    }
                                }
                            } else {
                                arg_types.push("%d");
                            }
                        }
                        _ => {
                            println!("Other expr types: {:?}", expr);
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
        let fmt = format!("printf(\"{}\\n\", ", arg_types.join(" "));
        out.write(fmt.as_bytes()).to()
    }

    fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // method call
        if let Expr::Bina(lhs, op, rhs) = call.name.as_ref() {
            if matches!(op, Op::Dot) {
                // get type decl of lhs
                match lhs.as_ref() {
                    Expr::Ident(name) => {
                        let meta = self.lookup_meta(name);
                        if let Some(meta) = meta {
                            match meta.as_ref() {
                                Meta::Store(store) => {
                                    if let Type::User(decl) = &store.ty {
                                        // check rhs is a method call
                                        if let Expr::Ident(method_name) = rhs.as_ref() {
                                            // write the method call as method_name(&s, args...)
                                            out.write(method_name.as_bytes())?;
                                            out.write(b"(")?;
                                            for m in decl.methods.iter() {
                                                if m.name == *method_name {
                                                    out.write(b"&")?;
                                                    out.write(name.as_bytes())?;
                                                    if !call.args.is_empty() {
                                                        out.write(b", ")?;
                                                        for (i, arg) in
                                                            call.args.args.iter().enumerate()
                                                        {
                                                            if i > 0 {
                                                                out.write(b", ")?;
                                                            }
                                                            self.expr(&arg.get_expr(), out)?;
                                                        }
                                                    }
                                                    out.write(b")").to()?;
                                                }
                                            }
                                            return Ok(());
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // normal call
        if let Expr::Ident(name) = &call.name.as_ref() {
            if name == "print" {
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
            Arg::Name(name) => self.str(name.as_str(), out),
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
            Stmt::Expr(expr) => match expr {
                Expr::Call(call) => {
                    if let Expr::Ident(name) = &call.name.as_ref() {
                        if name == "print" {
                            return false;
                        }
                    }
                    true
                }
                _ => true,
            },
            _ => false,
        }
    }
}

impl Trans for CTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // Split stmts into decls and main
        // TODO: handle potential includes when needed
        let mut decls: Vec<Stmt> = Vec::new();
        let mut main: Vec<Stmt> = Vec::new();

        // preprocess
        for stmt in ast.stmts.into_iter() {
            if stmt.is_decl() {
                decls.push(stmt);
            } else {
                match stmt {
                    Stmt::For(_) => main.push(stmt),
                    Stmt::If(_) => main.push(stmt),
                    Stmt::Expr(ref expr) => {
                        match expr {
                            Expr::Call(call) => {
                                if let Expr::Ident(name) = &call.name.as_ref() {
                                    if name == "print" {
                                        self.libs.insert("<stdio.h>".into());
                                    }
                                }
                            }
                            _ => {}
                        }
                        main.push(stmt);
                    }
                    Stmt::Use(use_stmt) => self.use_stmt(&use_stmt, &mut sink.body)?,
                    _ => {}
                }
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

        // // TODO: Includes on demand
        // if !self.libs.is_empty() {
        //     for path in self.libs.iter() {
        //         sink.body.write(b"#include ").to()?;
        //         sink.body.write(path.as_bytes()).to()?;
        //         sink.body.write(b"\n").to()?;
        //     }
        //     sink.body.write(b"\n").to()?;
        // }

        // Decls
        for (i, decl) in decls.iter().enumerate() {
            if i > 0 {
                sink.body.write(b"\n").to()?;
            }
            self.stmt(decl, sink)?;
        }

        // Main
        // TODO: check wether auto code already has a main function
        if !main.is_empty() {
            if !decls.is_empty() {
                sink.body.write(b"\n").to()?;
            }

            sink.body.write(b"int main(void) {\n").to()?;
            self.indent();
            for (i, stmt) in main.iter().enumerate() {
                self.print_indent(&mut sink.body)?;
                if i < main.len() - 1 {
                    self.stmt(stmt, sink)?;
                    sink.body.write(b"\n").to()?;
                } else {
                    if self.is_returnable(stmt) {
                        sink.body.write(b"return ").to()?;
                        self.stmt(stmt, sink)?;
                        sink.body.write(b"\n").to()?;
                    } else {
                        self.stmt(stmt, sink)?;
                        sink.body.write(b"\n").to()?;
                        self.print_indent(&mut sink.body)?;
                        sink.body.write(b"return 0;\n").to()?;
                    }
                }
            }
            self.dedent();
            sink.body.write(b"}\n").to()?;
        }

        // header guard end
        self.header.write(b"\n#endif\n\n").to()?;

        sink.header = self.header.clone();

        // includes
        for path in self.libs.iter() {
            sink.includes.write(b"#include ").to()?;
            sink.includes.write(path.as_bytes()).to()?;
            sink.includes.write(b"\n").to()?;
        }

        Ok(())
    }
}

pub fn transpile_part(code: &str) -> AutoResult<AutoStr> {
    let mut transpiler = CTrans::new("part".into());
    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code, scope);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut out = Sink::new();
    transpiler.code(ast, &mut out)?;
    Ok(String::from_utf8(out.body).unwrap().into())
}

pub struct CCode {
    pub includes: Vec<u8>,
    pub source: Vec<u8>,
    pub header: Vec<u8>,
}

// Transpile the code into a whole C program
pub fn transpile_c(name: impl Into<AutoStr>, code: &str) -> AutoResult<Sink> {
    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code, scope);
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut out = Sink::new();
    let mut transpiler = CTrans::new(name.into());
    transpiler.scope = parser.scope.clone();
    transpiler.trans(ast, &mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c() {
        let code = "41";
        let out = transpile_part(code).unwrap();
        assert_eq!(out, "41;\n");
    }

    #[test]
    fn test_c_fn() {
        let code = "fn add(x, y) int { x+y }";
        let out = transpile_part(code).unwrap();
        let expected = r#"int add(int x, int y) {
    return x + y;
}
"#;
        assert_eq!(out, expected);
    }

    #[test]
    fn test_c_let() {
        let code = "let x = 41";
        let out = transpile_part(code).unwrap();
        let expected = "int x = 41;\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn test_c_for() {
        let code = "for i in 1..5 { print(i) }";
        let out = transpile_part(code).unwrap();
        let expected = r#"for (int i = 1; i < 5; i++) {
    printf("%d\n", i);
}
"#;
        assert_eq!(out, expected);
    }

    #[test]
    fn test_c_if() {
        let code = "let x = 41; if x > 0 { print(x) }";
        let out = transpile_part(code).unwrap();
        let expected = r#"int x = 41;
if (x > 0) {
    printf("%d\n", x);
}
"#;
        assert_eq!(out, expected);
    }

    #[test]
    fn test_c_if_else() {
        let code = "let x = 41; if x > 0 { print(x) } else { print(-x) }";
        let out = transpile_part(code).unwrap();
        let expected = r#"int x = 41;
if (x > 0) {
    printf("%d\n", x);
} else {
    printf("%d\n", -x);
}
"#;
        assert_eq!(out, expected);
    }

    #[test]
    fn test_c_array() {
        let code = "let x = [1, 2, 3]";
        let out = transpile_part(code).unwrap();
        let expected = "int x[3] = {1, 2, 3};\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn test_c_mut_assign() {
        let code = "mut x = 41; x = 42";
        let out = transpile_part(code).unwrap();
        let expected = "int x = 41;\nx = 42;\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn test_c_return_42() {
        let code = r#"42"#;
        let mut sink = transpile_c("test", code).unwrap();
        let expected = r#"int main(void) {
    return 42;
}
"#;
        assert_eq!(String::from_utf8(sink.done().clone()).unwrap(), expected);
    }

    #[test]
    fn test_math() {
        let code = r#"fn add(x int, y int) int { x+y }
add(1, 2)"#;
        let mut sink = transpile_c("test", code).unwrap();
        let expected = r#"int add(int x, int y) {
    return x + y;
}

int main(void) {
    return add(1, 2);
}
"#;
        let expected_header = r#"#ifndef TEST_H
#define TEST_H

int add(int x, int y);

#endif

"#;
        assert_eq!(String::from_utf8(sink.done().clone()).unwrap(), expected);
        assert_eq!(String::from_utf8(sink.header).unwrap(), expected_header);
    }

    fn test_a2c(case: &str) -> AutoResult<()> {
        use std::fs::read_to_string;
        use std::fs::File;
        use std::io::Write;
        use std::path::PathBuf;

        // split number from name: 000_hello -> hello
        let parts: Vec<&str> = case.split("_").collect();
        let name = parts[1];

        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let src_path = format!("test/a2c/{}/{}.at", case, name);
        let src_path = d.join(src_path);
        let src = read_to_string(src_path.as_path())?;

        let exp_path = format!("test/a2c/{}/{}.expected.c", case, name);
        let exp_path = d.join(exp_path);
        let expected = read_to_string(exp_path.as_path())?;

        let mut ccode = transpile_c(name, &src)?;
        let str = String::from_utf8(ccode.done().clone()).unwrap();

        if str != expected {
            // out put generated code to a gen file
            let gen_path = format!("test/a2c/{}/{}.wrong.c", case, name);
            let gen_path = d.join(gen_path);
            let mut file = File::create(gen_path.as_path())?;
            file.write_all(str.as_bytes())?;
        }

        assert_eq!(str, expected);
        Ok(())
    }

    #[test]
    fn test_000_hello() {
        test_a2c("000_hello").unwrap();
    }

    #[test]
    fn test_001_sqrt() {
        test_a2c("001_sqrt").unwrap();
    }

    #[test]
    fn test_002_array() {
        test_a2c("002_array").unwrap();
    }

    #[test]
    fn test_003_func() {
        test_a2c("003_func").unwrap();
    }

    #[test]
    fn test_004_cstr() {
        test_a2c("004_cstr").unwrap();
    }

    #[test]
    fn test_005_pointer() {
        test_a2c("005_pointer").unwrap();
    }

    #[test]
    fn test_006_struct() {
        test_a2c("006_struct").unwrap();
    }

    #[test]
    fn test_007_enum() {
        test_a2c("007_enum").unwrap();
    }

    #[test]
    fn test_008_method() {
        test_a2c("008_method").unwrap();
    }

    #[test]
    fn test_009_alias() {
        test_a2c("009_alias").unwrap();
    }

    #[test]
    fn test_010_if() {
        test_a2c("010_if").unwrap();
    }
}

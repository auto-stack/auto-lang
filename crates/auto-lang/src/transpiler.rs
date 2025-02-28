use super::ast::*;
use std::io;
use std::io::Write;
use auto_val::Op;
use auto_val::AutoStr;
use crate::parser::Parser;
use crate::scope;
use std::rc::Rc;
use std::cell::RefCell;

pub trait Transpiler {
    fn transpile(&mut self, ast: Code, out: &mut impl Write) -> Result<(), String>;
}

pub struct CTranspiler {
    indent: usize,
    includes: Vec<u8>,
    header: Vec<u8>,
    name: AutoStr,
}

impl CTranspiler {
    fn new(name: AutoStr) -> Self {
        Self { indent: 0, includes: Vec::new(), header: Vec::new(), name }
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent -= 1;
    }

    fn print_indent(&self, out: &mut impl Write) -> Result<(), String> {
        for _ in 0..self.indent {
            out.write(b"    ").to()?;
        }
        Ok(())
    }
}

impl CTranspiler {

    pub fn code(&mut self, code: &Code, out: &mut impl Write) -> Result<(), String> {
        for stmt in code.stmts.iter() {
            self.stmt(stmt, out)?;
            out.write(b"\n").to()?;
        }
        Ok(())
    }

    fn eos(&mut self, out: &mut impl Write) -> Result<(), String> {
        out.write(b";").to()
    }

    fn stmt(&mut self, stmt: &Stmt, out: &mut impl Write) -> Result<(), String> {
        match stmt {
            Stmt::Expr(expr) => {self.expr(expr, out)?; self.eos(out)},
            Stmt::Store(store) => {self.store(store, out)?; self.eos(out)},
            Stmt::Fn(fn_decl) => self.fn_decl(fn_decl, out),
            Stmt::For(for_stmt) => self.for_stmt(for_stmt, out),
            Stmt::If(branches, otherwise) => self.if_stmt(branches, otherwise, out),
            _ => Err(format!("C Transpiler: unsupported statement: {:?}", stmt)),
        }
    }

    fn expr(&mut self, expr: &Expr, out: &mut impl Write) -> Result<(), String> {
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
            _ => Err(format!("C Transpiler: unsupported expression: {}", expr)),
        }
    }

    fn fn_decl(&mut self, fn_decl: &Fn, out: &mut impl Write) -> Result<(), String> {
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

    fn fn_sig(&mut self, fn_decl: &Fn, out: &mut impl Write) -> Result<(), String> {
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

    fn body(&mut self, body: &Body, out: &mut impl Write, has_return: bool) -> Result<(), String> {
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

    fn store(&mut self, store: &Store, out: &mut impl Write) -> Result<(), String> {
        if matches!(store.kind, StoreKind::Var) {
            return Err(format!("C Transpiler: unsupported store kind: {:?}", store.kind));
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

    fn for_stmt(&mut self, for_stmt: &For, out: &mut impl Write) -> Result<(), String> {
        out.write(b"for (").to()?;
        self.expr(&for_stmt.range, out)?;
        out.write(b") ").to()?;
        self.body(&for_stmt.body, out, false)?;
        Ok(())
    }

    fn range(&mut self, start: &Expr, end: &Expr, out: &mut impl Write) -> Result<(), String> {
        // TODO: check index name for deep loops
        out.write(b"int i = ").to()?;
        self.expr(start, out)?;
        out.write(b"; i < ").to()?;
        self.expr(end, out)?;
        out.write(b"; i++").to()?;
        Ok(())
    }

    fn if_stmt(&mut self, branches: &Vec<Branch>, otherwise: &Option<Body>, out: &mut impl Write) -> Result<(), String> {
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

    fn process_print(&mut self, call: &Call, out: &mut impl Write) -> Result<(), String> {
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
    

    fn call(&mut self, call: &Call, out: &mut impl Write) -> Result<(), String> {
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

    fn arg(&mut self, arg: &Arg, out: &mut impl Write) -> Result<(), String> {
        match arg {
            Arg::Name(name) => self.str(name.text.as_str(), out),
            Arg::Pair(_, expr) => self.expr(expr, out),
            Arg::Pos(expr) => self.expr(expr, out),
        }
    }

    fn str(&mut self, s: &str, out: &mut impl Write) -> Result<(), String> {
        out.write(b"\"").to()?;
        out.write(s.as_bytes()).to()?;
        out.write(b"\"").to()?;
        Ok(())
    }

    fn array(&mut self, array: &Vec<Expr>, out: &mut impl Write) -> Result<(), String> {
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
    fn transpile(&mut self, ast: Code, out: &mut impl Write) -> Result<(), String> {
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

pub trait ToStrError {
    fn to(self) -> Result<(), String>;
}

impl ToStrError for Result<(), io::Error> {
    fn to(self) -> Result<(), String> {
        self.map_err(|e| e.to_string())
    }
}

impl ToStrError for Result<usize, io::Error> {
    fn to(self) -> Result<(), String> {
        match self {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}

pub fn transpile_part(code: &str) -> Result<String, String> {
    let mut transpiler = CTranspiler::new("part".into());
    let scope = Rc::new(RefCell::new(scope::Universe::new()));
    let mut parser = Parser::new(code, scope);
    let ast = parser.parse()?;
    let mut out = Vec::new();
    transpiler.code(&ast, &mut out)?;
    Ok(String::from_utf8(out).unwrap())
}

pub struct CCode {
    pub source: Vec<u8>,
    pub header: Vec<u8>,
}

// Transpile the code into a whole C program
pub fn transpile_c(name: impl Into<AutoStr>, code: &str) -> Result<CCode, String> {
    let scope = Rc::new(RefCell::new(scope::Universe::new()));
    let mut parser = Parser::new(code, scope);
    let ast = parser.parse()?;
    let mut out = Vec::new();
    let mut transpiler = CTranspiler::new(name.into());
    transpiler.transpile(ast, &mut out)?;
    let header = transpiler.header;
    Ok(CCode {
        source: out,
        header
    })
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
    printf("%d", i);
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
    printf("%d", x);
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
    printf("%d", x);
} else {
    printf("%d", -x);
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
        let ccode = transpile_c("test", code).unwrap();
        let expected = r#"int main(void) {
    return 42;
}
"#;
        assert_eq!(ccode.source, expected.as_bytes());
    }

    #[test]
    fn test_math() {
        let code = r#"fn add(x int, y int) int { x+y }
add(1, 2)"#;
        let ccode = transpile_c("test", code).unwrap();
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
        assert_eq!(String::from_utf8(ccode.source).unwrap(), expected);
        assert_eq!(String::from_utf8(ccode.header).unwrap(), expected_header);
    }
}

use super::ast::*;
use std::io;
use std::io::Write;
use autoval::{Op, Value};
use crate::parser;
use crate::scope;

pub trait Transpiler {
    fn transpile(&mut self, ast: Code, out: &mut impl Write) -> Result<(), String>;
}

pub struct CTranspiler {
    indent: usize,
}

impl CTranspiler {
    fn new() -> Self {
        Self { indent: 0 }
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
    fn stmt(&mut self, stmt: &Stmt, out: &mut impl Write) -> Result<(), String> {
        match stmt {
            Stmt::Expr(expr) => self.expr(expr, out),
            Stmt::Fn(fn_decl) => self.fn_decl(fn_decl, out),
            Stmt::Store(store_decl) => self.store_decl(store_decl, out),
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
            Expr::Call(call) => self.call(call, out),
            Expr::Array(array) => self.array(array, out), 
            _ => Err(format!("C Transpiler: unsupported expression: {}", expr)),
        }
    }

    fn fn_decl(&mut self, fn_decl: &Fn, out: &mut impl Write) -> Result<(), String> {
        // return type
        if let Some(ret) = &fn_decl.ret {
            out.write(format!("{} ", ret).as_bytes()).to()?;
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
        out.write(b") ").to()?;
        // body
        self.body(&fn_decl.body, out, true)?;
        Ok(())
    }

    fn body(&mut self, body: &Body, out: &mut impl Write, has_return: bool) -> Result<(), String> {
        out.write(b"{\n").to()?;
        self.indent();
        for (i, stmt) in body.stmts.iter().enumerate() {
            self.print_indent(out)?;
            if i < body.stmts.len() - 1 {
                self.stmt(stmt, out)?;
                out.write(b";\n").to()?;
            } else {
                if has_return {
                    out.write(b"return ").to()?;
                }
                self.stmt(stmt, out)?;
                out.write(b";\n").to()?;
            }
        }
        self.dedent();
        out.write(b"}").to()?;
        Ok(())
    }

    fn store_decl(&mut self, store_decl: &Store, out: &mut impl Write) -> Result<(), String> {
        if matches!(store_decl.kind, StoreKind::Var) {
            return Err(format!("C Transpiler: unsupported store kind: {:?}", store_decl.kind));
        }
        match &store_decl.ty {
            Type::Array(array_type) => {
                let elem_type = &array_type.elem;
                let len = array_type.len;
                out.write(format!("{} {}[{}] = ", elem_type, store_decl.name.text, len).as_bytes()).to()?;
            }
            _ => {
                out.write(format!("{} {} = ", store_decl.ty, store_decl.name.text).as_bytes()).to()?;
            }
        }
        self.expr(&store_decl.expr, out)?;
        out.write(b";").to()?;
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

    fn call(&mut self, call: &Call, out: &mut impl Write) -> Result<(), String> {
        self.expr(&call.name, out)?;
        out.write(b"(").to()?;
        for (i, arg) in call.args.array.iter().enumerate() {
            self.expr(arg, out)?;
            if i < call.args.array.len() - 1 {
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
}

impl Transpiler for CTranspiler {
    fn transpile(&mut self, ast: Code, out: &mut impl Write) -> Result<(), String> {
        // includes

        // main function?

        // statements
        for stmt in ast.stmts.iter() {
            self.stmt(stmt, out)?;
            out.write(b"\n").to()?;
        }

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

pub fn transpile_c(code: &str) -> Result<String, String> {
    let mut transpiler = CTranspiler::new();
    let mut scope = scope::Universe::new();
    let ast = parser::parse(code, &mut scope)?;
    let mut out = Vec::new();
    transpiler.transpile(ast, &mut out)?;
    Ok(String::from_utf8(out).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c() {
        let code = "41";
        let out = transpile_c(code).unwrap();
        assert_eq!(out, "41\n");
    }

    #[test]
    fn test_c_fn() {
        let code = "fn add(x, y) { x+y }";
        let out = transpile_c(code).unwrap();
        let expected = r#"int add(int x, int y) {
    return x + y;
}
"#;
        assert_eq!(out, expected);
    }


    #[test]
    fn test_c_let() {
        let code = "let x = 41";
        let out = transpile_c(code).unwrap();
        let expected = "int x = 41;\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn test_c_for() {
        let code = "for i in 1..5 { print(i) }";
        let out = transpile_c(code).unwrap();
        let expected = r#"for (int i = 1; i < 5; i++) {
    print(i);
}
"#;
        assert_eq!(out, expected);
    }

    #[test]
    fn test_c_if() {
        let code = "let x = 41; if x > 0 { print(x) }";
        let out = transpile_c(code).unwrap();
        let expected = r#"int x = 41;
if (x > 0) {
    print(x);
}
"#;
        assert_eq!(out, expected);
    }

    #[test]
    fn test_c_if_else() {
        let code = "let x = 41; if x > 0 { print(x) } else { print(-x) }";
        let out = transpile_c(code).unwrap();
        let expected = r#"int x = 41;
if (x > 0) {
    print(x);
} else {
    print(-x);
}
"#;
        assert_eq!(out, expected);
    }

    #[test]
    fn test_c_array() {
        let code = "let x = [1, 2, 3]";
        let out = transpile_c(code).unwrap();
        let expected = "int x[3] = {1, 2, 3};\n";
        assert_eq!(out, expected);
    }
}

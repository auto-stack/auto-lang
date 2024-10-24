use super::ast::*;
use std::io;
use std::io::Write;

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
            Stmt::Var(var) => self.var(var, out),
            Stmt::For(name, range, body) => self.for_stmt(name, range, body, out),
            Stmt::If(branches, otherwise) => self.if_stmt(branches, otherwise, out),
            _ => Err(format!("unsupported statement: {:?}", stmt)),
        }
    }

    fn expr(&mut self, expr: &Expr, out: &mut impl Write) -> Result<(), String> {
        match expr {
            Expr::Integer(i) => out.write_all(i.to_string().as_bytes()).to(),
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
            Expr::Ident(name) => out.write_all(name.text.as_bytes()).to(),
            Expr::Call(name, args) => self.call(name, args, out),
            _ => Err(format!("unsupported expression: {:?}", expr)),
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
        out.write(b")").to()?;
        // body
        self.body(&fn_decl.body, out, true)?;
        Ok(())
    }

    fn body(&mut self, body: &Body, out: &mut impl Write, has_return: bool) -> Result<(), String> {
        out.write(b" {\n").to()?;
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
        out.write(b"}\n").to()?;
        Ok(())
    }

    fn var(&mut self, var: &Var, out: &mut impl Write) -> Result<(), String> {
        out.write(format!("int {} = ", var.name.text).as_bytes()).to()?;
        self.expr(&var.expr, out)?;
        out.write(b";").to()?;
        Ok(())
    }

    fn for_stmt(&mut self, name: &Name, range: &Expr, body: &Body, out: &mut impl Write) -> Result<(), String> {
        out.write(b"for (").to()?;
        self.expr(range, out)?;
        out.write(b")").to()?;
        self.body(body, out, false)?;
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
            out.write(b")").to()?;
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

    fn call(&mut self, name: &Expr, args: &Vec<Expr>, out: &mut impl Write) -> Result<(), String> {
        self.expr(name, out)?;
        out.write(b"(").to()?;
        for arg in args {
            self.expr(arg, out)?;
        }
        out.write(b")").to()?;
        Ok(())
    }
}

impl Transpiler for CTranspiler {
    fn transpile(&mut self, ast: Code, out: &mut impl Write) -> Result<(), String> {
        // includes

        // main function?

        // statements
        for (i, stmt) in ast.stmts.iter().enumerate()    {
            self.stmt(stmt, out)?;
            if i < ast.stmts.len() - 1 {
                out.write(b"\n").to()?;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::scope;

    #[test]
    fn test_c() {
        let mut transpiler = CTranspiler::new();
        let code = "41";
        let mut scope = scope::Universe::new();
        let ast = parser::parse(code, &mut scope).unwrap();
        let mut out = Vec::new();
        transpiler.transpile(ast, &mut out).unwrap();
        println!("{}", String::from_utf8(out).unwrap());
    }

    #[test]
    fn test_c_fn() {
        let mut transpiler = CTranspiler::new();
        let code = "fn add(x, y) { x+y }";
        let mut scope = scope::Universe::new();
        let ast = parser::parse(code, &mut scope).unwrap();
        let mut out = Vec::new();
        transpiler.transpile(ast, &mut out).unwrap();
        let expected = r#"int add(int x, int y) {
    return x + y;
}
"#;
        assert_eq!(String::from_utf8(out).unwrap(), expected);
    }


    #[test]
    fn test_c_var() {
        let mut transpiler = CTranspiler::new();
        let code = "var x = 41";
        let mut scope = scope::Universe::new();
        let ast = parser::parse(code, &mut scope).unwrap();
        let mut out = Vec::new();
        transpiler.transpile(ast, &mut out).unwrap();
        let expected = "int x = 41;";
        assert_eq!(String::from_utf8(out).unwrap(), expected);
    }

    #[test]
    fn test_c_for() {
        let mut transpiler = CTranspiler::new();
        let code = "for i in 1..5 { print(i) }";
        let mut scope = scope::Universe::new();
        let ast = parser::parse(code, &mut scope).unwrap();
        let mut out = Vec::new();
        transpiler.transpile(ast, &mut out).unwrap();
        let expected = r#"for (int i = 1; i < 5; i++) {
    print(i);
}
"#;
        assert_eq!(String::from_utf8(out).unwrap(), expected);
    }

    #[test]
    fn test_c_if() {
        let mut transpiler = CTranspiler::new();
        let code = "var x = 41; if x > 0 { print(x) }";
        let mut scope = scope::Universe::new();
        let ast = parser::parse(code, &mut scope).unwrap();
        let mut out = Vec::new();
        transpiler.transpile(ast, &mut out).unwrap();
        let expected = r#"int x = 41;
if (x > 0) {
    print(x);
}
"#;
        assert_eq!(String::from_utf8(out).unwrap(), expected);
    }
}

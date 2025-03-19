use super::ast::*;
use std::io;
use std::io::Write;
use auto_val::AutoStr;
use crate::parser::Parser;
use crate::universe::Universe;
use crate::AutoResult;
use std::rc::Rc;
use std::cell::RefCell;

pub mod c;
pub mod rust;
pub trait Transpiler {
    fn transpile(&mut self, ast: Code, out: &mut impl Write) -> AutoResult<()>;
}

pub trait ToStrError {
    fn to(self) -> AutoResult<()>;
}

impl ToStrError for Result<(), io::Error> {
    fn to(self) -> AutoResult<()> {
        self.map_err(|e| e.to_string().into())
    }
}

impl ToStrError for Result<usize, io::Error> {
    fn to(self) -> AutoResult<()> {
        match self {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string().into()),
        }
    }
}

pub fn transpile_part(code: &str) -> AutoResult<String> {
    let mut transpiler = c::CTranspiler::new("part".into());
    let scope = Rc::new(RefCell::new(Universe::new()));
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
pub fn transpile_c(name: impl Into<AutoStr>, code: &str) -> AutoResult<CCode> {
    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code, scope);
    let ast = parser.parse()?;
    let mut out = Vec::new();
    let mut transpiler = c::CTranspiler::new(name.into());
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

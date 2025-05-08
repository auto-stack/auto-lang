use super::*;
use crate::parser::{ParseResult, Parser, ParserExt};

impl ParserExt for Code {
    fn parse(code: impl Into<AutoStr>) -> ParseResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        let ast = parser.parse()?;
        Ok(ast)
    }
}

impl ParserExt for Name {
    fn parse(name: impl Into<AutoStr>) -> ParseResult<Self> {
        let n = Name::from(name.into());
        Ok(n)
    }
}

impl ParserExt for Expr {
    fn parse(code: impl Into<AutoStr>) -> ParseResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        let ast = parser.expr()?;
        Ok(ast)
    }
}

impl ParserExt for When {
    fn parse(code: impl Into<AutoStr>) -> ParseResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        parser.parse_when()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::parsers::Code;

    #[test]
    fn test_parse_code() {
        let ast = Code::parse("let x = 1").unwrap();
        assert_eq!(ast.to_string(), "(code (let (name x) (type int) (int 1)))");
    }

    #[test]
    fn test_parse_name() {
        let name = Name::parse("x").unwrap();
        assert_eq!(name.to_string(), "x");
    }

    #[test]
    fn test_parse_expr() {
        let expr = Expr::parse("1 + 2").unwrap();
        assert_eq!(expr.to_string(), "(bina (int 1) (op +) (int 2))");
    }
}

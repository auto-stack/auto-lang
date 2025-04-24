use super::*;
use crate::parser::{ParseResult, Parser, ParserExt};
use crate::Universe;
use auto_val::shared;

impl ParserExt for Code {
    fn parse(code: impl Into<AutoStr>) -> ParseResult<Self> {
        let universe = shared(Universe::new());
        let code = code.into();
        let mut parser = Parser::new(code.as_str(), universe);
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
    fn parse(expr: impl Into<AutoStr>) -> ParseResult<Self> {
        let expr = expr.into();
        let mut parser = Parser::new(expr.as_str(), shared(Universe::new()));
        let ast = parser.expr()?;
        Ok(ast)
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

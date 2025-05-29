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
        let ast = parser.parse_expr()?;
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

impl ParserExt for Goto {
    fn parse(code: impl Into<AutoStr>) -> ParseResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        parser.parse_goto()
    }
}

impl ParserExt for GotoSwitch {
    fn parse(code: impl Into<AutoStr>) -> ParseResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        parser.parse_goto_switch()
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

    #[test]
    fn test_parse_goto() {
        let goto = Goto::parse("5 -> 7 : 8").unwrap();
        assert_eq!(
            goto.to_string(),
            "(goto (from (int 5)) (to (int 7)) (with (int 8)))"
        );
    }

    #[test]
    fn test_parse_goto_switch() {
        let code = r#"
            on {
                5 -> 7 : 10
                9 -> 10 : 11
            }
        "#;
        let goto_switch = GotoSwitch::parse(code.trim()).unwrap();
        assert_eq!(
            goto_switch.to_string(),
            "(goto-switch (goto (from (int 5)) (to (int 7)) (with (int 10))) (goto (from (int 9)) (to (int 10)) (with (int 11))))"
        );
    }
}

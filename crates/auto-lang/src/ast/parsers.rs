use super::*;
use crate::parser::{Parser, ParserExt};
use auto_val::AutoResult;

impl ParserExt for Code {
    fn parse(code: impl Into<AutoStr>) -> AutoResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        let ast = parser.parse()?;
        Ok(ast)
    }
}

impl ParserExt for Name {
    fn parse(name: impl Into<AutoStr>) -> AutoResult<Self> {
        let n = Name::from(name.into());
        Ok(n)
    }
}

impl ParserExt for Expr {
    fn parse(code: impl Into<AutoStr>) -> AutoResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        let ast = parser.parse_expr()?;
        Ok(ast)
    }
}

impl ParserExt for Is {
    fn parse(code: impl Into<AutoStr>) -> AutoResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        parser.parse_is()
    }
}

impl ParserExt for Arrow {
    fn parse(code: impl Into<AutoStr>) -> AutoResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str());
        let ev = parser.parse_event_src()?;
        let arrow = parser.parse_arrow(ev)?;
        Ok(arrow)
    }
}

impl ParserExt for OnEvents {
    fn parse(code: impl Into<AutoStr>) -> AutoResult<Self> {
        let code = code.into();
        let mut parser = Parser::from(code.as_str()).skip_check();
        parser.parse_on_events()
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
    fn test_parse_arrow() {
        let arrow = Arrow::parse("5 -> 7 : 8").unwrap();
        assert_eq!(
            arrow.to_string(),
            "(arrow (from (int 5)) (to (int 7)) (with (int 8)))"
        );
    }

    #[test]
    fn test_parse_on_events() {
        let code = r#"
            on {
                5 -> 7 : 10
                9 -> 10 : 11
            }
        "#;
        let on = OnEvents::parse(code.trim()).unwrap();
        assert_eq!(
            on.to_string(),
            "(on (arrow (from (int 5)) (to (int 7)) (with (int 10))) (arrow (from (int 9)) (to (int 10)) (with (int 11))))"
        );
    }
}

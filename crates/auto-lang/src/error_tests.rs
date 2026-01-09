#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{SyntaxError, SyntaxErrorWithSource, pos_to_span};
    use miette::{Diagnostic, NamedSource};
    use auto_lang::token::Pos;

    #[test]
    fn test_source_code_in_error() {
        let source_code = "let x: int = \"invalid\"";
        let syntax_err = SyntaxError::Generic {
            message: "type error".to_string(),
            span: pos_to_span(Pos {
                line: 1,
                at: 10,
                pos: 10,
                len: 10,
            }),
        };

        let err = SyntaxErrorWithSource {
            source: NamedSource::new("test.at".to_string(), source_code.to_string()),
            error: syntax_err,
        };

        // Check if source_code() works
        assert!(err.source_code().is_some());

        // Verify the source has content
        let source = err.source_code().unwrap();
        assert_eq!(source.len(), source_code.len());
    }
}

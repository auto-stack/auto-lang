#[cfg(test)]
mod widget_macro_tests {
    use crate::macro_::preprocess;

    #[test]
    fn test_widget_macro_in_full_code() {
        let code = r#"
widget Hello {
    msg str

    fn view() View {
        text(msg) {}
    }
}
"#;

        let processed = preprocess(code);
        println!("Original:\n{}", code);
        println!("Processed:\n{}", processed);

        // Verify macro was expanded
        assert!(processed.contains("type Hello is Widget"));
        assert!(!processed.contains("widget Hello"));

        // Verify content is preserved
        assert!(processed.contains("msg str"));
        assert!(processed.contains("fn view() View"));
        assert!(processed.contains("text(msg)"));
    }

    #[test]
    fn test_multiple_widgets_with_imports() {
        let code = r#"
use auto.ui: View

widget Header {
    title str
}

widget Footer {
    copyright str
}
"#;

        let processed = preprocess(code);
        assert!(processed.contains("type Header is Widget"));
        assert!(processed.contains("type Footer is Widget"));
        assert!(processed.contains("use auto.ui: View"));
    }
}

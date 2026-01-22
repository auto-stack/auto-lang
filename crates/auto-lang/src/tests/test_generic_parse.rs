use crate::parse_preserve_error;
use crate::error::{AutoError, attach_source};
use miette::Diagnostic;

#[test]
fn test_show_parse_error() {
    let code = r#"
tag May<T> {
    nil Nil
    val T
}

fn main() {
    let x May<int>
    x = May.val(42)
    x
}
"#;

    match parse_preserve_error(code) {
        Ok(_) => {}
        Err(e) => {
            // Attach source to get detailed error messages
            let error_with_source = attach_source(e, "test_generic.at".to_string(), code.to_string());

            eprintln!("\n=== Parse Error ===\n");

            match &error_with_source {
                AutoError::MultipleErrors { count, errors, .. } => {
                    eprintln!("Multiple errors ({}):\n", count);
                    for (i, err) in errors.iter().enumerate() {
                        eprintln!("--- Error {} ---", i + 1);
                        eprintln!("{}\n", err);
                    }
                }
                _ => {
                    eprintln!("{}\n", error_with_source);
                }
            }

            panic!("Parse failed");
        }
    }
}

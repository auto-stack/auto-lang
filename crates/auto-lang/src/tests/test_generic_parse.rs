use crate::parse_preserve_error;
use crate::error::{AutoError, attach_source};

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
        Ok(_) => {
            // Generics are not yet fully implemented, so this test is expected to fail
            // When generics are implemented, this branch will be taken
            println!("Parse succeeded - generics are implemented!");
        }
        Err(e) => {
            // Attach source to get detailed error messages
            let error_with_source = attach_source(e, "test_generic.at".to_string(), code.to_string());

            println!("\n=== Parse Error ===\n");

            match &error_with_source {
                AutoError::MultipleErrors { count, errors, .. } => {
                    println!("Multiple errors ({}):\n", count);
                    for (i, err) in errors.iter().enumerate() {
                        println!("--- Error {} ---", i + 1);
                        println!("{}\n", err);
                    }
                }
                _ => {
                    println!("{}\n", error_with_source);
                }
            }

            // Don't panic - this test is meant to demonstrate error reporting
            // Once generics are fully implemented, this test will start succeeding
            println!("Note: Generics are not yet fully implemented in the parser");
        }
    }
}

use crate::parse_preserve_error;
use crate::error::{AutoError, attach_source};

#[test]
fn test_let_with_just_type() {
    // Test: let x May (no generics)
    let code1 = r#"
tag May { nil Nil, val int }

fn main() {
    let x May
}
"#;
    match parse_preserve_error(code1) {
        Ok(_) => println!("✓ let x May (non-generic) parsed"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code1.to_string());
            eprintln!("✗ let x May failed: {}\n", err_with_src);
        }
    }

    // Test: let x May = May.val(42) (no generics, with init)
    let code2 = r#"
tag May { nil Nil, val int }

fn main() {
    let x = May.val(42)
}
"#;
    match parse_preserve_error(code2) {
        Ok(_) => println!("✓ let x = May.val(42) (non-generic) parsed"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code2.to_string());
            eprintln!("✗ let x = May.val(42) failed: {}\n", err_with_src);
        }
    }

    // Test: let x = May.val(42) where May is generic
    let code3 = r#"
tag May<T> { nil Nil, val T }

fn main() {
    let x = May.val(42)
}
"#;
    match parse_preserve_error(code3) {
        Ok(_) => println!("✓ let x = May.val(42) (generic tag) parsed"),
        Err(e) => {
            let err_with_src = attach_source(e, "test.at".to_string(), code3.to_string());
            eprintln!("✗ let x = May.val(42) (generic) failed:\n");
            match &err_with_src {
                AutoError::MultipleErrors { count, errors, .. } => {
                    eprintln!("Multiple errors ({}):\n", count);
                    for (i, err) in errors.iter().enumerate() {
                        eprintln!("--- Error {} ---", i + 1);
                        eprintln!("{}\n", err);
                    }
                }
                _ => eprintln!("{}\n", err_with_src),
            }
        }
    }
}

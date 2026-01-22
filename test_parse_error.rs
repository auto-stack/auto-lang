use auto_lang::parse_preserve_error;
use auto_lang::error::AutoError;

fn main() {
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
        Ok(_) => println!("Parse successful!"),
        Err(e) => {
            eprintln!("Parse error:\n");

            match &e {
                AutoError::MultipleErrors { count, errors, .. } => {
                    eprintln!("Multiple errors ({}):", count);
                    for (i, err) in errors.iter().enumerate() {
                        eprintln!("\n--- Error {} ---", i + 1);
                        eprintln!("{}", err);
                    }
                }
                _ => {
                    eprintln!("{}", e);
                }
            }
        }
    }
}

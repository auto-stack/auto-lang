mod token;
mod lexer;
mod ast;
mod parser;
mod eval;
mod value;
pub mod repl;

pub fn run(code: &str) -> Result<String, String> {
    let mut lexer = lexer::Lexer::new(code);
    lexer.print();
    let ast = parser::Parser::new(code).parse();
    println!("{}", ast);
    let value = eval::Evaler::new().eval(&ast);
    Ok(value.to_string())
}

// add tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        let code = "1+2*3";
        let result = run(code).expect("Failed to run code");
        assert_eq!(result, "7");
    }

    #[test]
    fn test_unary() {
        let code = "-2*3";
        let result = run(code).expect("Failed to run code");
        assert_eq!(result, "-6");
    }

    #[test]
    fn test_group() {
        let code = "(1+2)*3";
        let result = run(code).expect("Failed to run code");
        assert_eq!(result, "9");
    }
}



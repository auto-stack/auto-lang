mod token;
mod lexer;
mod ast;
mod parser;
mod eval;
mod value;
pub mod repl;

pub fn run(code: &str) -> Result<String, String> {
    let ast = parser::Parser::new(code).parse();
    let value = eval::Evaler::new().eval(&ast);
    Ok(value.to_string())
}
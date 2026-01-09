use auto_lang::parser::Parser;
use auto_lang::Universe;
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let code = r#"
fn new_point(x int, y int) Point {
    Point(x, y)
}
"#;
    let scope = Rc::new(RefCell::new(Universe::new()));
    let mut parser = Parser::new(code, scope.clone());
    let ast = parser.parse().unwrap();
    
    if let Some(stmt) = ast.stmts.first() {
        if let auto_lang::ast::Stmt::Fn(fn_decl) = stmt {
            println!("Return type: {:?}", fn_decl.ret);
            println!("Return type Display: {}", fn_decl.ret);
            println!("Return type to_atom: {}", fn_decl.ret.to_atom());
        }
    }
}

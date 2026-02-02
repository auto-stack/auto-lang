use auto_lang::parser::Parser as AutoParser;
use auto_lang::vm::codegen::Codegen;

fn main() {
    let source = std::fs::read_to_string("tmp/test_debug.at").unwrap();
    let mut parser = AutoParser::from(&source);
    let code = parser.parse().unwrap();

    let mut codegen = Codegen::new();
    for stmt in code.stmts {
        codegen.compile_stmt(&stmt).unwrap();
    }

    // Dump bytecode
    println!("Bytecode ({} bytes):", codegen.code.len());
    for (i, byte) in codegen.code.iter().enumerate() {
        print!("{:02x} ", byte);
        if (i + 1) % 16 == 0 {
            println!();
        }
    }
    println!();

    // Show exports
    println!("\nExports:");
    for (name, addr) in &codegen.exports {
        println!("  {} @ {}", name, addr);
    }

    // Show symbol table
    println!("\nScopes:");
    for (i, scope) in codegen.scope_stack.iter().enumerate() {
        println!("  Scope {}:", i);
        for (name, idx) in scope {
            println!("    {} -> {}", name, idx);
        }
    }
}

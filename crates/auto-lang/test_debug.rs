extern crate auto_lang;
use auto_lang::{CompileSession, compile::compile_module_to_bytecode};

fn main() {
    let mut session = CompileSession::new();
    let source = std::fs::read_to_string("stdlib/auto/test_mod.at").unwrap();
    println!("=== Source ===\n{}", source);
    
    // Parse and check exports
    let module = session.compile_module_to_bytecode(&source, "test_mod.at").unwrap();
    println!("=== Module exports ===");
    for (name, addr) in &module.exports {
        println!("  {} -> 0x{:04x}", name, addr);
    }
}

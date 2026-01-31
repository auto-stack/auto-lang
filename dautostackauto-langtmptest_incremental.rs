// Test incremental compilation
use auto_lang::{run_with_session, compile::CompileSession};

fn main() {
    let mut session = CompileSession::new();
    
    // First run - define a function
    println!("=== First run - define function ===");
    let code1 = "fn add(a int, b int) int { a + b }";
    match run_with_session(&mut session, code1) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    // Second run - use the function
    println!("\n=== Second run - use function ===");
    let code2 = "add(10, 20)";
    match run_with_session(&mut session, code2) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    // Third run - use the function again
    println!("\n=== Third run - use function again ===");
    let code3 = "add(5, 15)";
    match run_with_session(&mut session, code3) {
        Ok(result) => println!("Result: {}", result),
        Err(e) => println!("Error: {}", e),
    }
    
    println!("\n✅ All runs completed!");
}

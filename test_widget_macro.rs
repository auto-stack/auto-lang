// Simple test script to verify widget macro expansion
use std::path::Path;

fn main() {
    // Read the test file
    let code = r#"
widget Hello {
    msg str

    fn view() View {
        text(msg) {}
    }
}
"#;

    println!("=== Original Code ===");
    println!("{}", code);

    // Test macro expansion
    let processed = auto_lang::macro_::preprocess(code);

    println!("\n=== Processed Code ===");
    println!("{}", processed);

    // Verify
    if processed.contains("type Hello is Widget") {
        println!("\n✅ SUCCESS: widget macro expanded correctly!");
    } else {
        println!("\n❌ FAILED: widget macro not expanded");
        std::process::exit(1);
    }

    if !processed.contains("widget Hello") {
        println!("✅ SUCCESS: original 'widget' keyword removed!");
    } else {
        println!("❌ FAILED: 'widget' keyword still present");
        std::process::exit(1);
    }

    println!("\n✅ All checks passed!");
}

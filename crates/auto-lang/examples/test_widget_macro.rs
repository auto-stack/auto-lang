// Complete example showing how the widget macro is used in practice

fn main() {
    println!("=== Widget Macro Usage Examples ===\n");

    // Example 1: Direct macro expansion (low-level)
    println!("--- Example 1: Direct Macro Expansion ---");
    let code = r#"
widget Hello {
    msg str

    fn view() View {
        text(msg) {}
    }
}
"#;

    println!("Original code:");
    println!("{}", code);

    // This shows what happens internally
    let processed = auto_lang::macro_::preprocess(code);
    println!("After macro expansion:");
    println!("{}", processed);

    // Example 2: Using with eval_config (automatic preprocessing)
    println!("\n--- Example 2: Using with eval_config() ---");
    let ui_code = r#"
widget Counter {
    count int

    fn view() View {
        text(count) {}
    }
}

type MyApp is App {
    title: "Counter"
    count: 0
}
"#;

    println!("UI code:");
    println!("{}", ui_code);

    // In real usage, you'd use eval_config which automatically preprocesses
    // let interpreter = auto_lang::eval_config(ui_code, &auto_val::Obj::new())?;
    // The macro expansion happens transparently inside eval_config()

    // For this demo, just show the expansion
    let expanded = auto_lang::macro_::preprocess(ui_code);
    println!("After preprocessing:");
    println!("{}", expanded);

    // Example 3: Real-world usage pattern
    println!("\n--- Example 3: Real-World Usage Pattern ---");
    println!("In your AutoUI application, you would:");
    println!();
    println!("1. Write UI code using widget macro:");
    println!("   widget Button {{ label str, onclick Msg }}");
    println!();
    println!("2. Load it using AutoConfig (macro auto-expands):");
    println!("   use auto_lang::config::AutoConfigReader;");
    println!("   let mut reader = AutoConfigReader::new();");
    println!("   let config = reader.parse(&code)?;");
    println!();
    println!("3. The config.root contains the parsed widget structure");
    println!("   ready to be converted to View<M> by auto-ui");
    println!();
    println!("The key point: Macro preprocessing is AUTOMATIC and TRANSPARENT.");
    println!("You just write 'widget Name {{ ... }}' and the system handles the rest.");
    println!();
    println!("✅ Widget macro is ready to use!");
}

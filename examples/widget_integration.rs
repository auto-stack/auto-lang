/// Example showing how widget macro integrates with AutoConfig system
use auto_lang::config::AutoConfigReader;

fn main() {
    // This is how you'd use it in a real application
    let ui_code = r#"
// Define a simple widget
widget HelloWidget {
    message str

    fn view() View {
        text(message) {}
    }
}

// Create an app instance
app MyApplication {
    HelloWidget("Hello from Auto!")
}
"#;

    println!("=== UI Code (with widget macro) ===");
    println!("{}", ui_code);
    println!("\n" + &"=".repeat(60));

    // Create a config reader (automatically preprocesses macros)
    let mut reader = AutoConfigReader::new();

    match reader.parse(ui_code) {
        Ok(config) => {
            println!("✅ Widget macro expanded and parsed!");
            println!("\nParsed config:");
            println!("  - Root node: {}", config.root.title());
            println!("  - Kids: {} items", config.root.kids_iter().count());

            // Show the structure
            println!("\nStructure:");
            for (name, kid) in config.root.kids_iter() {
                println!("  ├── {}", name);
                if let auto_val::Kid::Node(n) = kid {
                    for (prop_name, prop_value) in n.props_iter() {
                        println!("  │   └── {}: {}", prop_name, prop_value);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
}

/// Complete example showing how to use the widget macro in practice
use auto_lang::eval_config;
use auto_val::Obj;

fn main() {
    // Example 1: Simple widget definition
    let code = r#"
widget Counter {
    count int

    fn view() View {
        text(count) {}
    }
}

type CounterApp is App {
    title: "Counter App"

    fn run() {
        center {
            Counter(0)
        }
    }
}
"#;

    println!("=== Original Code ===");
    println!("{}", code);
    println!("\n" + &"=".repeat(60));

    // The macro preprocessing happens automatically inside eval_config
    match eval_config(code, &Obj::new()) {
        Ok(interpreter) => {
            println!("✅ Widget macro expanded successfully!");
            println!("\nResult: {}", interpreter.result.repr());
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }

    // Example 2: Widget with imports
    let code2 = r#"
use auto.ui: View, text, button, col

widget LoginForm {
    username str
    password str

    fn view() View {
        col {
            text("Username:")
            text(username)
            text("Password:")
            text(password)
        }
    }
}
"#;

    println!("\n" + &"=".repeat(60));
    println!("=== Example 2: Widget with Imports ===");
    println!("{}", code2);
    println!("\n" + &"=".repeat(60));

    match eval_config(code2, &Obj::new()) {
        Ok(interpreter) => {
            println!("✅ Widget with imports processed!");
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
}

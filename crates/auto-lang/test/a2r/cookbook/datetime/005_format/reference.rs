use chrono::Local;

fn main() {
    let now = Local::now();
    let formatted = now.format("%Y-%m-%d %H:%M:%S");
    println!("Formatted: {}", formatted);
}

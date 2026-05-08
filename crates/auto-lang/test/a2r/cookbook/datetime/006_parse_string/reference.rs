use chrono::NaiveDateTime;

fn main() {
    let dt = NaiveDateTime::parse_from_str("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S").unwrap();
    println!("Parsed: {}", dt);
}

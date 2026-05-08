use chrono::{Utc, TimeZone};

fn main() {
    let dt = Utc.timestamp_opt(1700000000, 0).single().unwrap();
    println!("From timestamp: {}", dt);
    let now = Utc::now();
    let ts = now.timestamp();
    println!("Current timestamp: {}", ts);
}

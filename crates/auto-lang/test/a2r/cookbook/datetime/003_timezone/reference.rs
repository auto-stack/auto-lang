use chrono::{Utc, Local, TimeZone};

fn main() {
    let utc_time = Utc::now();
    let local_time = Local::now();
    println!("UTC: {}", utc_time);
    println!("Local: {}", local_time);
}

use chrono::{Local, Datelike, Timelike};

fn main() {
    let now = Local::now();
    let year = now.year();
    let month = now.month();
    let day = now.day();
    let hour = now.hour();
    println!("Date: {}-{}-{}", year, month, day);
    println!("Time: {}:00", hour);
}

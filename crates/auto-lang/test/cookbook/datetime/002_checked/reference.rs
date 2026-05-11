use chrono::{DateTime, Duration, Utc};

fn main() {
    let now: DateTime<Utc> = Utc::now();
    println!("Current time: {}", now);
    let future = now.checked_add_signed(Duration::days(30));
    println!("30 days from now: {:?}", future);
}

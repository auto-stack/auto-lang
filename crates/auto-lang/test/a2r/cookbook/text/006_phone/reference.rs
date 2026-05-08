use regex::Regex;

fn main() {
    let re = Regex::new(r"\d{3}-\d{3}-\d{4}").unwrap();
    let text = "Call 555-123-4567 or 555-987-6543";
    for cap in re.find_iter(text) {
        println!("Phone: {}", cap.as_str());
    }
}

fn main() {
    let s = "hello".to_string();
    let lower = s.to_lowercase();
    let upper = s.to_uppercase();
    let trimmed = s.trim();
    let parts: Vec<&str> = s.split(",").collect();
    let has = s.contains("ell");
}

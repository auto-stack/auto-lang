struct User {
    #[serde(rename = "role_id")]
    role: i32,
    name: String,
}

fn main() {
    let u = User { role: 1, name: "Alice".to_string() };
}

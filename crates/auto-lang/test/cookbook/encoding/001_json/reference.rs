use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: f64,
    y: f64,
}

fn main() {
    let point = Point { x: 1.0, y: 2.0 };
    let serialized = serde_json::to_string(&point).unwrap();
    println!("Serialized: {}", serialized);
    let deserialized: Point = serde_json::from_str(&serialized).unwrap();
    println!("Deserialized: x={}, y={}", deserialized.x, deserialized.y);
}

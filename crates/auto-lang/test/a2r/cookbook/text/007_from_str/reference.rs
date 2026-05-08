use std::str::FromStr;
use std::num::ParseIntError;

#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl FromStr for Point {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(",").collect();
        let x = parts[0].parse::<i32>()?;
        let y = parts[1].parse::<i32>()?;
        Ok(Point { x, y })
    }
}

fn main() {
    let p: Point = "3,4".parse().unwrap();
    println!("Point: ({}, {})", p.x, p.y);
}

struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn modulus(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }
}

fn main() {
    let p: Point = Point { x: 3, y: 4 };
    let m: i32 = p.modulus();
    println!("Modulus: {}", m);
}

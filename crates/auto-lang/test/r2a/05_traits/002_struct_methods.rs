struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }
    fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
    fn translate(&mut self, dx: f64, dy: f64) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }
}

fn main() {
    let mut p = Point::new(0.0, 0.0);
    let q = Point::new(3.0, 4.0);
    let d = p.distance(&q);
    println!("{}", d);
    p.translate(1.0, 1.0);
}

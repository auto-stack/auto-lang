struct Point {
    x: i32,
    y: i32,
}

struct Circle {
    radius: f64,
    border: u32,
    center: Point,
}

fn main() {
    let mut p: Point = Point { x: 1, y: 2 };
    p.x = 3;
    print!("P: ", p.x, ", ", p.y);

    let circle: Circle = Circle { radius: 5.0, border: 1, center: Point { x: 50, y: 50 } };
    print!("C: ", circle.center.x, ", ", circle.center.y, ", ", circle.radius);
}

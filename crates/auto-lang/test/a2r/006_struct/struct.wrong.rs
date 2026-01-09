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
    let mut p: Point = Point { field0: 1, field1: 2 };
    p.x = 3;
    println!("{}", format!("P: {}, {}", p.x, p.y));

    let circle: Circle = Circle { field0: 5, field1: 1, field2: Point { field0: 50, field1: 50 } };
    println!("{}", format!("C: {}, {}, {}", circle.center.x, circle.center.y, circle.radius));
}

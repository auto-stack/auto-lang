struct Point {
    x: i32,
    y: i32,
}

enum Shape {
    Circle(i32),
    Rect(Point),
    None,
}

fn main() {
    let p = Point { x: 3, y: 4 };

    match p {
        Point { x, y } => println!("{}", x),
    }

    let s = Shape::Rect(p);
    match s {
        Shape::Circle(r) => println!("{}", r),
        Shape::Rect(pt) => {
            match pt {
                Point { x, y } => println!("{}", x),
            }
        },
        Shape::None => println!("none"),
    }
}

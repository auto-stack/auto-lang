enum Color {
    RED = 1,
    GREEN = 2,
    BLUE = 3,
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Color::RED => write!(f, "RED"),
            Color::GREEN => write!(f, "GREEN"),
            Color::BLUE => write!(f, "BLUE"),
        }
    }
}


fn main() {
    let color: Color = Color::BLUE;
    println!("The color is: {}", color);
}

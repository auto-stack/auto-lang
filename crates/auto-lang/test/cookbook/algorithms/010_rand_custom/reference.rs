use rand::Rng;
use rand::distributions::{Distribution, Standard};

#[derive(Debug)]
enum Color {
    Red,
    Green,
    Blue,
}

impl Distribution<Color> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Color {
        match rng.gen_range(0..3) {
            0 => Color::Red,
            1 => Color::Green,
            _ => Color::Blue,
        }
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let color: Color = rng.gen();
    println!("Random color: {:?}", color);
}

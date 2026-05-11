use rand::Rng;
use rand::distributions::{Distribution, Standard};

enum Pet {
    Dog,
    Cat,
    Bird,
}

impl Distribution<Pet> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Pet {
        match rng.gen_range(0..3) {
            0 => Pet::Dog,
            1 => Pet::Cat,
            _ => Pet::Bird,
        }
    }
}

fn main() {
    let mut rng = rand::thread_rng();
    let pet: Pet = rng.gen();
    println!("Got a pet!");
}

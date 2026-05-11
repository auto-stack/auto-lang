use rand::Rng;
use rand_distr::{Normal, Distribution};

fn main() {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(2.0, 3.0).unwrap();
    let v: f64 = normal.sample(&mut rng);
    println!("Random from Normal(2, 3): {}", v);
}

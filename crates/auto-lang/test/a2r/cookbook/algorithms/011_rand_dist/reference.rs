use rand::Rng;
use rand_distr::{Distribution, Normal};

fn main() {
    let mut rng = rand::thread_rng();
    let normal = Normal::new(0.0, 1.0).unwrap();
    let mut sum = 0.0;
    for _ in 0..1000 {
        let val: f64 = normal.sample(&mut rng);
        sum += val;
    }
    let avg = sum / 1000.0;
    println!("Average of 1000 normal samples: {}", avg);
}

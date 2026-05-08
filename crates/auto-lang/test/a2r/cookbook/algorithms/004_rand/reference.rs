use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let n: i32 = rng.gen_range(0..100);
    println!("Random number: {}", n);
}

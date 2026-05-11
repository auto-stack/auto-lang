use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let x: i32 = rng.gen_range(0..5);
    let y: i32 = rng.gen_range(10..20);
    println!("x = {}, y = {}", x, y);
}

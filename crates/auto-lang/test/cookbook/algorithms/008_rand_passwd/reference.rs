use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()";
    let password: String = (0..16)
        .map(|_| charset.chars().nth(rng.gen_range(0..charset.len())).unwrap())
        .collect();
    println!("Password: {}", password);
}

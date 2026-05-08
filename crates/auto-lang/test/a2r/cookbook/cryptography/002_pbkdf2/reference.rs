use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};

fn main() {
    let rng = SystemRandom::new();
    let mut salt = [0u8; 16];
    rng.fill(&mut salt).unwrap();

    let password = "hunter2";
    let mut result = [0u8; 32];
    pbkdf2::derive(pbkdf2::PBKDF2_HMAC_SHA256, 100_000, &salt, password.as_bytes(), &mut result);

    println!("Salt: {} bytes", salt.len());
    println!("Hash: {} bytes", result.len());
}

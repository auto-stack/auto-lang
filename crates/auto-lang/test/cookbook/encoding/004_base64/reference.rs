use base64::{Engine as _, engine::general_purpose};

fn main() {
    let encoded = general_purpose::STANDARD.encode(b"hello world");
    println!("Encoded: {}", encoded);
    let decoded = general_purpose::STANDARD.decode(&encoded).unwrap();
    println!("Decoded: {} bytes", decoded.len());
}

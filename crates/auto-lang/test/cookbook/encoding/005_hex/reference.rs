use data_encoding::HEXLOWER;

fn main() {
    let encoded = HEXLOWER.encode(b"hello world");
    println!("Hex: {}", encoded);
    let decoded = HEXLOWER.decode(encoded.as_bytes()).unwrap();
    println!("Decoded: {} bytes", decoded.len());
}

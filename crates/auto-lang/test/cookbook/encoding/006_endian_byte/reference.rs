fn main() {
    let value: u16 = 0x1234;
    let bytes = value.to_be_bytes();
    println!("BE bytes: {}, {}", bytes[0], bytes[1]);
    let le_bytes = value.to_le_bytes();
    println!("LE bytes: {}, {}", le_bytes[0], le_bytes[1]);
    let from_be = u16::from_be_bytes(bytes);
    println!("From BE: {}", from_be);
}

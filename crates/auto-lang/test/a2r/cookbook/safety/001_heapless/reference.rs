use heapless::Vec;

fn main() {
    let mut buf: Vec<u8, 64> = Vec::new();
    let data = b"hello";
    for &byte in data {
        buf.push(byte).unwrap();
    }
    println!("Buffer: {} bytes", buf.len());
}

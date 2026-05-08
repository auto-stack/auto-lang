use ring::hmac;

fn main() {
    let key = hmac::Key::new(hmac::HMAC_SHA256, b"secret key");
    let message = b"hello world";
    let tag = hmac::sign(&key, message);
    println!("HMAC tag length: {}", tag.as_ref().len());
    let valid = hmac::verify(&key, message, tag.as_ref()).is_ok();
    println!("Verification: {}", valid);
}

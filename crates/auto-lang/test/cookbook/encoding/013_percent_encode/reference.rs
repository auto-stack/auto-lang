use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

fn main() {
    let input = "hello world! foo@bar.com";
    let encoded = percent_encode(input.as_bytes(), NON_ALPHANUMERIC).to_string();
    println!("Encoded: {}", encoded);
}

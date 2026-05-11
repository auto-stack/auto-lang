use urlencoding::encode;

fn main() {
    let input = "hello world! foo=bar&baz=qux";
    let encoded = encode(input);
    println!("Encoded: {}", encoded);
}

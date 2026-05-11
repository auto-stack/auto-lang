use url::Url;

fn main() {
    let parsed = Url::parse("https://example.com/path?query=1").unwrap();
    println!("Scheme: {}", parsed.scheme());
    println!("Host: {:?}", parsed.host_str());
    println!("Path: {}", parsed.path());
}

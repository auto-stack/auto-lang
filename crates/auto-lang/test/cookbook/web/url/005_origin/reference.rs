use url::Url;

fn main() {
    let url = Url::parse("https://user:pass@example.com:8080/path?query=1#frag").unwrap();
    println!("Scheme: {}", url.scheme());
    println!("Host: {:?}", url.host_str());
    println!("Port: {:?}", url.port());
    println!("Origin: {}", url.origin());
}

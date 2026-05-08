use url::Url;

fn main() {
    let url = Url::parse("https://example.com/path?query=1").unwrap();
    let new_url = url.join("sub/page").unwrap();
    println!("Original: {}", url);
    println!("Joined: {}", new_url);
}

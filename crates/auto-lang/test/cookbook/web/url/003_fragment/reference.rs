use url::Url;

fn main() {
    let url = Url::parse("https://example.com/page#section").unwrap();
    let fragment = url.fragment();
    println!("Fragment: {:?}", fragment);
}

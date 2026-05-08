use url::Url;

fn main() {
    let url = Url::parse("https://example.com/rust?name=hello&age=20").unwrap();
    for pair in url.query_pairs() {
        println!("Param: {} = {}", pair.0, pair.1);
    }
}

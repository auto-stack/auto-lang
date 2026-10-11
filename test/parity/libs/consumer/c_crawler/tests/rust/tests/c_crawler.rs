/// Rust oracle for c_crawler consumer parity.
const BASE: &str = "http://127.0.0.1:18080";

fn http_get(url: &str) -> String {
    match ureq::get(url).call() {
        Ok(r) => r.into_string().unwrap_or_default(),
        Err(ureq::Error::Status(_, r)) => r.into_string().unwrap_or_default(),
        Err(_) => String::new(),
    }
}

fn crawl_status(url: &str) -> i32 {
    let _ = http_get(url);
    match ureq::get(url).call() {
        Ok(r) => r.status() as i32,
        Err(ureq::Error::Status(c, _)) => c as i32,
        Err(_) => 0,
    }
}

fn tap_ok(n: i32, name: &str) { println!("ok {} - {}", n, name); }

#[test] fn test_crawl_page1() { assert_eq!(http_get(&format!("{}/page1", BASE)), r#"{"page":1,"links":["/page2"]}"#); tap_ok(1, "test_crawl_page1"); }
#[test] fn test_crawl_page2() { assert_eq!(http_get(&format!("{}/page2", BASE)), r#"{"page":2,"content":"done"}"#); tap_ok(2, "test_crawl_page2"); }
#[test] fn test_crawl_200() { assert_eq!(crawl_status(&format!("{}/", BASE)), 200); tap_ok(3, "test_crawl_200"); }
#[test] fn test_crawl_404() { assert_eq!(crawl_status(&format!("{}/missing", BASE)), 404); tap_ok(4, "test_crawl_404"); }
#[test] fn test_crawl_file() { assert_eq!(http_get(&format!("{}/file.txt", BASE)), "hello wget"); tap_ok(5, "test_crawl_file"); }

/// Rust oracle for c_http_get consumer parity.
const BASE: &str = "http://127.0.0.1:18080";

fn fetch_body(url: &str) -> String {
    match ureq::get(url).call() {
        Ok(r) => r.into_string().unwrap_or_default(),
        Err(ureq::Error::Status(_, r)) => r.into_string().unwrap_or_default(),
        Err(_) => String::new(),
    }
}

fn fetch_status(url: &str) -> i32 {
    match ureq::get(url).call() {
        Ok(r) => r.status() as i32,
        Err(ureq::Error::Status(c, _)) => c as i32,
        Err(_) => 0,
    }
}

fn tap_ok(n: i32, name: &str) { println!("ok {} - {}", n, name); }

#[test] fn test_get_root() { assert_eq!(fetch_body(&format!("{}/", BASE)), r#"{"ok":true}"#); tap_ok(1, "test_get_root"); }
#[test] fn test_get_200() { assert_eq!(fetch_status(&format!("{}/", BASE)), 200); tap_ok(2, "test_get_200"); }
#[test] fn test_get_file() { assert_eq!(fetch_body(&format!("{}/file.txt", BASE)), "hello wget"); tap_ok(3, "test_get_file"); }
#[test] fn test_get_404() { assert_eq!(fetch_status(&format!("{}/missing", BASE)), 404); tap_ok(4, "test_get_404"); }
#[test] fn test_get_page1() { assert_eq!(fetch_body(&format!("{}/page1", BASE)), r#"{"page":1,"links":["/page2"]}"#); tap_ok(5, "test_get_page1"); }

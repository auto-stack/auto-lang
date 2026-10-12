/// Rust oracle for c_wget consumer parity.
const BASE: &str = "http://127.0.0.1:18080";
const TMP: &str = "c_wget_tmp";

fn fetch(url: &str) -> String {
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

fn download(url: &str, filepath: &str) -> String {
    let body = fetch(url);
    let _ = std::fs::create_dir_all(TMP);
    std::fs::write(filepath, &body).unwrap_or(());
    std::fs::read_to_string(filepath).unwrap_or_default()
}

fn tap_ok(n: i32, name: &str) { println!("ok {} - {}", n, name); }

#[test] fn test_fetch_file() { assert_eq!(fetch(&format!("{}/file.txt", BASE)), "hello wget"); tap_ok(1, "test_fetch_file"); }
#[test] fn test_fetch_200() { assert_eq!(fetch_status(&format!("{}/", BASE)), 200); tap_ok(2, "test_fetch_200"); }
#[test] fn test_download_readback() { assert_eq!(download(&format!("{}/file.txt", BASE), &format!("{}/dl1.txt", TMP)), "hello wget"); tap_ok(3, "test_download_readback"); }
#[test] fn test_download_json() { assert_eq!(download(&format!("{}/", BASE), &format!("{}/dl2.txt", TMP)), r#"{"ok":true}"#); tap_ok(4, "test_download_json"); }
#[test] fn test_fetch_404() { assert_eq!(fetch_status(&format!("{}/missing", BASE)), 404); tap_ok(5, "test_fetch_404"); }

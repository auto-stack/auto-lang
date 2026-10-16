
use std::io::{self, BufRead};
fn field(s: &str, key: &str) -> String {
    let needle = format!("\"{}\":\"", key);
    s.split(&needle).nth(1).unwrap().split('"').next().unwrap().to_string()
}
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        if !line.contains("\"request\":") { continue; }
        let n = field(&line, "nonce"); let h = field(&line, "request_hash");
        let ok = std::fs::read_to_string(std::env::var("MOCK_OK_FILE").unwrap()).unwrap();
        let schema=std::env::var("MOCK_SCHEMA").unwrap();
        println!("{{\"schema\":\"{}\",\"nonce\":\"{}\",\"request_hash\":\"{}\",\"ok\":{}}}", schema, n, h, ok);
        std::process::exit(std::env::var("MOCK_EXIT").unwrap().parse().unwrap());
    }
}

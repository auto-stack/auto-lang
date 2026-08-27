// HTTP mock server for parity tests.
//
// Listens on 127.0.0.1:18080. Supports:
//   POST /echo         → 200 {"echo":"ok"}
//   GET  /             → 200 {"ok":true}
//   GET  /page1        → 200 {"page":1,"links":["/page2"]}
//   GET  /page2        → 200 {"page":2,"content":"done"}
//   GET  /file.txt     → 200 "hello wget"
//   GET  /missing      → 404 {"err":"not found"}
//   * (other)          → 405 {"err":"method"}

use std::io::{Read, Write};
use std::net::TcpListener;

const HOST: &str = "127.0.0.1";
const PORT: u16 = 18080;

fn main() {
    let listener = TcpListener::bind((HOST, PORT)).unwrap_or_else(|e| {
        eprintln!("mock-server: failed to bind {HOST}:{PORT}: {e}");
        std::process::exit(1);
    });
    println!("mock-server: listening on http://{HOST}:{PORT}");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap_or(0);
        let req = String::from_utf8_lossy(&buf[..n]);
        let first_line = req.lines().next().unwrap_or("");
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        let method = parts.first().unwrap_or(&"");
        let path = parts.get(1).unwrap_or(&"/");

        let (status, content_type, body) = match (*method, *path) {
            ("POST", _) => (200, "application/json", r#"{"echo":"ok"}"#),
            ("GET", "/") => (200, "application/json", r#"{"ok":true}"#),
            ("GET", "/page1") => (200, "application/json", r#"{"page":1,"links":["/page2"]}"#),
            ("GET", "/page2") => (200, "application/json", r#"{"page":2,"content":"done"}"#),
            ("GET", "/file.txt") => (200, "text/plain", "hello wget"),
            ("GET", "/status204") => (204, "text/plain", ""),
            ("GET", p) if p.starts_with("/missing") || p == "/404" => {
                (404, "application/json", r#"{"err":"not found"}"#)
            }
            ("GET", _) => (200, "application/json", r#"{"ok":true}"#),
            _ => (405, "application/json", r#"{"err":"method"}"#),
        };

        let status_text = match status {
            200 => "OK", 204 => "No Content", 404 => "Not Found",
            405 => "Method Not Allowed", _ => "OK",
        };
        let resp = format!(
            "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(resp.as_bytes());
        let _ = stream.flush();
    }
}

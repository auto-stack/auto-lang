// Minimal HTTP mock server for http_client_sync parity tests.
//
// Listens on 127.0.0.1:18080, accepts any number of requests, and responds
// to every POST with a fixed JSON body `{"echo":"ok"}` and status 200. Other
// methods get a 405. Runs until killed (Ctrl+C / process termination).
//
// Run:  cargo run -p mock-server   (from this crate dir)
//       or:  cargo run --manifest-path parity/libs/http_client_sync/mock-server/Cargo.toml

use std::io::{Read, Write};
use std::net::TcpListener;

const HOST: &str = "127.0.0.1";
const PORT: u16 = 18080;
const JSON_BODY: &str = r#"{"echo":"ok"}"#;

fn main() {
    let listener = TcpListener::bind((HOST, PORT)).unwrap_or_else(|e| {
        eprintln!("mock-server: failed to bind {HOST}:{PORT}: {e}");
        std::process::exit(1);
    });
    println!("mock-server: listening on http://{HOST}:{PORT} (Ctrl+C to stop)");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("mock-server: accept error: {e}");
                continue;
            }
        };
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap_or(0);
        let req = String::from_utf8_lossy(&buf[..n]);
        let first_line = req.lines().next().unwrap_or("");
        let method = first_line.split_whitespace().next().unwrap_or("");

        let (status_line, body) = if method == "POST" {
            ("HTTP/1.1 200 OK", JSON_BODY)
        } else {
            ("HTTP/1.1 405 Method Not Allowed", r#"{"err":"method"}"#)
        };
        let resp = format!(
            "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(resp.as_bytes());
        let _ = stream.flush();
    }
}

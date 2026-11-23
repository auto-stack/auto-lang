//! PLAN-1123 test peers: REAL loopback endpoints for the secure-transport
//! cases — a rustls TLS server presenting the certificate under test, a
//! plaintext HTTP origin, and an HTTP proxy that tunnels CONNECT, forwards
//! absolute-form requests, rejects, closes or stalls. Every peer records the
//! request bytes it really received. Loopback only; every key here is a
//! TEST-ONLY identity for a 127.0.0.1 peer (see README.md).
#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const CA_TEST_PEM: &[u8] = include_bytes!("ca-test.cert.pem");
pub const CA_TEST_DER: &[u8] = include_bytes!("ca-test.cert.der");
pub const LEAF_OK_SERVER: &[u8] = include_bytes!("leaf-ok.server.pem");
pub const LEAF_NAME_SERVER: &[u8] = include_bytes!("leaf-name.server.pem");
pub const LEAF_CA_SERVER: &[u8] = include_bytes!("leaf-ca.server.pem");
/// The ordinary project's Auto source: HTTP request/response over the
/// secure transport (the single author of the glue both lanes execute).
pub const HTTPS_APP: &str = include_str!("https_app.at");

pub const BODY_OK: &[u8] = b"hello over tls";
pub const BODY_NEXT: &[u8] = b"landed";

/// A loopback endpoint that records every request it answers.
pub struct Peer {
    pub addr: SocketAddr,
    stop: Arc<AtomicBool>,
    requests: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

impl Peer {
    /// The request bytes received so far, in arrival order.
    pub fn received(&self) -> Vec<Vec<u8>> {
        self.requests
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

/// The plain HTTP answer to one request (the origin's routing table).
pub fn respond(request: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(request);
    let line = text.lines().next().unwrap_or("").to_string();
    let head_end = request
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|at| at + 4);
    let mut parts = line.split(' ');
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");
    let status_body: (u16, &[u8], &str) = match (method, path) {
        ("GET", "/ok") => (200, BODY_OK, "text/plain"),
        ("GET", "/start") => (302, b"", "text/plain"),
        ("GET", "/next") => (200, BODY_NEXT, "text/plain"),
        ("POST", "/echo") => (
            201,
            request
                .get(head_end.unwrap_or(request.len())..)
                .unwrap_or(b""),
            "application/octet-stream",
        ),
        _ => (404, b"missing", "text/plain"),
    };
    let reason = match status_body.0 {
        200 => "OK",
        201 => "Created",
        302 => "Found",
        _ => "Not Found",
    };
    let mut response = format!(
        "HTTP/1.1 {} {reason}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        status_body.0,
        status_body.2,
        status_body.1.len()
    );
    if status_body.0 == 302 {
        response.push_str("Location: /next\r\n");
    }
    response.push_str("\r\n");
    let mut bytes = response.into_bytes();
    bytes.extend_from_slice(status_body.1);
    bytes
}

/// One complete HTTP request: the head and the Content-Length body.
fn read_request(source: &mut impl Read) -> std::io::Result<Vec<u8>> {
    let mut request = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        let head_end = request
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .map(|at| at + 4);
        if let Some(head_end) = head_end {
            let head = String::from_utf8_lossy(&request[..head_end]).to_ascii_lowercase();
            let length = head
                .lines()
                .find_map(|line| line.strip_prefix("content-length:"))
                .and_then(|value| value.trim().parse::<usize>().ok())
                .unwrap_or(0);
            if request.len() >= head_end + length {
                return Ok(request);
            }
        }
        let read = match source.read(&mut buffer) {
            Ok(0) => return Ok(request),
            Ok(read) => read,
            Err(error) => return Err(error),
        };
        request.extend_from_slice(&buffer[..read]);
        if request.len() > 64 * 1024 {
            return Ok(request);
        }
    }
}

/// Run `handle` for every accepted connection until the peer is dropped.
fn serve(
    listener: TcpListener,
    stop: Arc<AtomicBool>,
    handle: impl Fn(TcpStream) + Send + Sync + 'static,
) {
    listener.set_nonblocking(true).unwrap();
    let handle = Arc::new(handle);
    std::thread::spawn(move || {
        while !stop.load(Ordering::SeqCst) {
            let (socket, _) = match listener.accept() {
                Ok(accepted) => accepted,
                Err(_) => {
                    std::thread::sleep(Duration::from_millis(2));
                    continue;
                }
            };
            // Windows hands the accepted socket the listener's non-blocking mode.
            let _ = socket.set_nonblocking(false);
            let _ = socket.set_read_timeout(Some(Duration::from_secs(5)));
            let handle = handle.clone();
            std::thread::spawn(move || handle(socket));
        }
    });
}

fn bind() -> (TcpListener, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    (listener, addr)
}

/// A real TLS server presenting `server_pem` (key + certificate). With
/// `clean_close` it sends close_notify before closing; otherwise it closes
/// without it (the truncation case).
pub fn tls_server(server_pem: &[u8], clean_close: bool) -> Peer {
    use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};
    let certs: Vec<CertificateDer<'static>> = CertificateDer::pem_slice_iter(server_pem)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let key = PrivateKeyDer::from_pem_slice(server_pem).unwrap();
    let config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
    .unwrap()
    .with_no_client_auth()
    .with_single_cert(certs, key)
    .unwrap();
    let (listener, addr) = bind();
    let stop = Arc::new(AtomicBool::new(false));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let (record, config) = (requests.clone(), Arc::new(config));
    serve(listener, stop.clone(), move |socket| {
        let Ok(conn) = rustls::ServerConnection::new(config.clone()) else {
            return;
        };
        let mut tls = rustls::StreamOwned::new(conn, socket);
        // The handshake rides the first read: a client that rejects the
        // certificate closes here and NOTHING is recorded.
        let request = match read_request(&mut tls) {
            Ok(request) if !request.is_empty() => request,
            _ => return,
        };
        record
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(request.clone());
        let response = respond(&request);
        let _ = tls.write_all(&response);
        let _ = tls.flush();
        if clean_close {
            let _ = tls.conn.send_close_notify();
            let _ = tls.write(&[]);
        }
        let _ = tls.get_mut().shutdown(Shutdown::Both);
    });
    Peer {
        addr,
        stop,
        requests,
    }
}

/// A plaintext origin with the same routing table (the target behind a
/// forwarding proxy).
pub fn plain_origin() -> Peer {
    let (listener, addr) = bind();
    let stop = Arc::new(AtomicBool::new(false));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let record = requests.clone();
    serve(listener, stop.clone(), move |mut socket| {
        let request = match read_request(&mut socket) {
            Ok(request) if !request.is_empty() => request,
            _ => return,
        };
        record
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(request.clone());
        let _ = socket.write_all(&respond(&request));
        let _ = socket.shutdown(Shutdown::Both);
    });
    Peer {
        addr,
        stop,
        requests,
    }
}

/// A loopback endpoint that accepts every connection and never answers
/// (the handshake-timeout case).
pub fn silent() -> Peer {
    let (listener, addr) = bind();
    let stop = Arc::new(AtomicBool::new(false));
    let requests = Arc::new(Mutex::new(Vec::new()));
    serve(listener, stop.clone(), move |mut socket| {
        // keep reading (and never answer) until the client gives up
        let _ = socket.set_read_timeout(Some(Duration::from_secs(10)));
        let mut sink = [0u8; 1024];
        while matches!(socket.read(&mut sink), Ok(read) if read > 0) {}
    });
    Peer {
        addr,
        stop,
        requests,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProxyBehavior {
    /// Answer CONNECT with 200 and pipe the tunnel; forward absolute-form
    /// requests to their origin.
    Forward,
    /// Answer CONNECT with this status.
    Reject(u16),
    /// Close without a CONNECT verdict.
    Close,
    /// Never answer a CONNECT.
    Stall,
}

/// A real loopback HTTP proxy recording every request head it receives.
pub struct Proxy {
    pub addr: SocketAddr,
    stop: Arc<AtomicBool>,
    heads: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl Drop for Proxy {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

impl Proxy {
    /// The request heads received so far, verbatim.
    pub fn received(&self) -> Vec<Vec<u8>> {
        self.heads.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

fn relay(from: &mut TcpStream, to: &mut TcpStream) {
    let _ = from.set_read_timeout(Some(Duration::from_secs(5)));
    let mut buffer = [0u8; 16 * 1024];
    loop {
        let read = match from.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(read) => read,
        };
        if to.write_all(&buffer[..read]).is_err() {
            break;
        }
    }
    let _ = to.shutdown(Shutdown::Write);
}

/// Copy both directions until both sides finish.
fn pipe(mut client: TcpStream, mut origin: TcpStream) {
    let (Ok(mut client_back), Ok(mut origin_back)) = (client.try_clone(), origin.try_clone())
    else {
        return;
    };
    let upstream = std::thread::spawn(move || relay(&mut client, &mut origin_back));
    relay(&mut origin, &mut client_back);
    let _ = upstream.join();
}

fn read_head(socket: &mut TcpStream) -> Option<Vec<u8>> {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !(head.ends_with(b"\r\n\r\n") || head.ends_with(b"\n\n")) {
        match socket.read(&mut byte) {
            Ok(0) | Err(_) => return None,
            Ok(_) => head.push(byte[0]),
        }
        if head.len() > 8 * 1024 {
            return None;
        }
    }
    Some(head)
}

pub fn proxy(behavior: ProxyBehavior) -> Proxy {
    let (listener, addr) = bind();
    let stop = Arc::new(AtomicBool::new(false));
    let heads = Arc::new(Mutex::new(Vec::new()));
    let record = heads.clone();
    let stall_until = stop.clone();
    serve(listener, stop.clone(), move |mut client| {
        let Some(head) = read_head(&mut client) else {
            return;
        };
        let first = String::from_utf8_lossy(&head)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        record
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(head.clone());
        if let Some(authority) = first.strip_prefix("CONNECT ") {
            let authority = authority.split(' ').next().unwrap_or("").to_string();
            match behavior {
                ProxyBehavior::Close => {}
                ProxyBehavior::Stall => {
                    while !stall_until.load(Ordering::SeqCst) {
                        std::thread::sleep(Duration::from_millis(20));
                    }
                }
                ProxyBehavior::Reject(code) => {
                    let _ = client.write_all(
                        format!("HTTP/1.1 {code} Forbidden\r\nContent-Length: 0\r\n\r\n")
                            .as_bytes(),
                    );
                }
                ProxyBehavior::Forward => {
                    if let Ok(origin) = TcpStream::connect(&authority) {
                        if client
                            .write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                            .is_ok()
                        {
                            pipe(client, origin);
                            return;
                        }
                    }
                }
            }
        } else if behavior == ProxyBehavior::Forward {
            // absolute-form: `GET http://host:port/path HTTP/1.1`
            let Some(url) = first.split(' ').nth(1) else {
                return;
            };
            let Some(rest) = url.strip_prefix("http://") else {
                return;
            };
            let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
            let Ok(mut origin) = TcpStream::connect(authority) else {
                return;
            };
            // the proxy received the bytes verbatim (recorded above); the
            // origin sees the origin-form request line
            let text = String::from_utf8_lossy(&head).to_string();
            let mut lines = text.split("\r\n");
            let request_line = lines.next().unwrap_or("");
            let mut words = request_line.split(' ');
            let method = words.next().unwrap_or("");
            let version = words.next_back().unwrap_or("HTTP/1.1");
            let forwarded = format!(
                "{method} /{path} {version}\r\n{}",
                lines.collect::<Vec<_>>().join("\r\n")
            );
            if origin.write_all(forwarded.as_bytes()).is_ok() {
                pipe(client, origin);
                return;
            }
        }
        let _ = client.shutdown(Shutdown::Both);
    });
    Proxy { addr, stop, heads }
}

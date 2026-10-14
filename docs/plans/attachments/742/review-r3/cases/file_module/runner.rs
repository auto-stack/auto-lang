#![allow(unused)]
use std::{future::Future, sync::Arc, task::{Context,Poll,Wake,Waker}};
#[derive(Debug,Clone)] pub struct Probe { pub text: String }
pub struct State<T>(T);
pub struct Path<T>(T);
pub type Db = ();
pub struct JsonResponse<T>(pub T);
#[derive(Debug)] pub struct StatusCode;
impl StatusCode { const OK: Self=Self; const NOT_FOUND:Self=Self; }
mod repository { pub fn outer(v: i64) -> i64 { v } pub fn measure(v: &str) -> i64 { v.len() as i64 } pub fn ping() -> i64 { 7 } pub fn take(v: &str) -> i64 { v.len() as i64 } pub fn path() -> String { "readme.txt".to_string() } }
mod axum { pub mod http { pub struct Method; pub struct HeaderMap; } pub mod response { #[derive(Debug)] pub struct Response(pub String); } }
mod auto_lang { pub mod a2r_std { pub mod http { pub struct FileResponse(pub String); pub fn file_response(_: &str, path: &str, _: &str) -> FileResponse { FileResponse(path.to_string()) } } } }
async fn __file_reply(d: auto_lang::a2r_std::http::FileResponse, _: &axum::http::Method, _: &axum::http::HeaderMap) -> axum::response::Response { axum::response::Response(d.0) }
struct Noop;
impl Wake for Noop { fn wake(self: Arc<Self>) {} }
fn ready<T>(f: impl Future<Output=T>) -> T {
    let w = Waker::from(Arc::new(Noop));
    let mut c = Context::from_waker(&w);
    let mut f = std::pin::pin!(f);
    match f.as_mut().poll(&mut c) { Poll::Ready(v)=>v, Poll::Pending=>panic!("unexpectedly pending") }
}
pub async fn echo(State(db): State<Db>, method: axum::http::Method, headers: axum::http::HeaderMap) -> axum::response::Response {
    let mut path: String = crate::repository::path();
    return __file_reply(auto_lang::a2r_std::http::file_response("D:/files", path.as_str(), "{}"), &method, &headers).await;
}

fn main() { println!("{:?}",ready(echo(State(()),axum::http::Method,axum::http::HeaderMap)).0); }
#![allow(unused)]
use std::{future::Future, sync::Arc, task::{Context,Poll,Wake,Waker}};
#[derive(Debug)] struct Probe { text: String }
struct State<T>(T);
type Db = ();
struct JsonResponse<T>(T);
struct Noop;
impl Wake for Noop { fn wake(self: Arc<Self>) {} }
fn main() {
    let w = Waker::from(Arc::new(Noop));
    let mut c = Context::from_waker(&w);
    let mut f = std::pin::pin!(echo(State(())));
    match f.as_mut().poll(&mut c) {
        Poll::Ready(v) => println!("{:?}", v.0.text),
        Poll::Pending => panic!("synchronous handler unexpectedly pending"),
    }
}
pub async fn echo(State(db): State<Db>) -> JsonResponse<Probe> {
    let mut msg: String = "crate::repository::filename".to_string();
    if false {
        return JsonResponse(Probe { text: msg.to_string() });
    }
    return JsonResponse(Probe { text: msg.to_string() });
}
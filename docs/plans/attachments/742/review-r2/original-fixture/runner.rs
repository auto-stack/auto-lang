#![allow(unused)]
use std::sync::{Arc, Mutex};
#[derive(Clone)] pub struct NoteDocument { schema_version: i64, note_id: String, title: String }
#[derive(Clone)] pub struct NoteView { schema_version: i64, note_id: String, title: String }
pub struct UpdateNoteDocumentInput { title: String }
pub type Db = Arc<Mutex<Vec<NoteDocument>>>;
pub struct State<T>(T);
pub struct Path<T>(T);
pub struct Json<T>(T);
pub struct JsonResponse<T>(T);
pub struct StatusCode;
impl StatusCode { const NOT_FOUND: Self = Self; }
mod repository { pub fn repo_list_active() -> Vec<super::NoteDocument> { vec![] } }

pub async fn list_notes(State(db): State<Db>) -> JsonResponse<Vec<NoteDocument>> {
    let mut docs: Vec<NoteDocument> = crate::repository::repo_list_active();
    let mut out: Vec<NoteDocument> = vec![];
    for d in &docs {
        out.push(d.clone().clone());
    }
    return JsonResponse(out);
}

pub async fn get_note(Path(id): Path<i64>, State(db): State<Db>) -> Result<JsonResponse<NoteView>, StatusCode> {
    let mut docs: Vec<NoteDocument> = crate::repository::repo_list_active();
    if (docs.len() as i64) > 0 {
        let mut d: NoteDocument = docs[0].clone();
        let mut v: NoteView = NoteView { schema_version: d.schema_version, note_id: d.note_id, title: d.title };
        return Ok(JsonResponse(v));
    }
    return Err(StatusCode::NOT_FOUND);
}

pub async fn update_note(Path(id): Path<i64>, State(db): State<Db>, Json(input): Json<UpdateNoteDocumentInput>) -> Result<JsonResponse<Vec<NoteDocument>>, StatusCode> {
    let title = input.title.clone();
    let mut docs: Vec<NoteDocument> = crate::repository::repo_list_active();
    let mut ok: bool = (docs.len() as i64) > 0;
    if ok {
        ok = docs[0].clone().title == title;
    }
    return Ok(JsonResponse(docs));
}

use std::{future::Future, task::{Wake,Waker,Context,Poll}};
struct Noop; impl Wake for Noop { fn wake(self: Arc<Self>) {} }
fn ready<T>(f:impl Future<Output=T>) -> T { let w=Waker::from(Arc::new(Noop));let mut c=Context::from_waker(&w);let mut f=std::pin::pin!(f);match f.as_mut().poll(&mut c){Poll::Ready(v)=>v,Poll::Pending=>panic!("unexpected pending")} }
fn state() -> State<Db> { State(Arc::new(Mutex::new(Vec::new()))) }
fn main(){println!("list={}",ready(list_notes(state())).0.len());match ready(get_note(Path(1),state())){Ok(_)=>println!("get=ok"),Err(_)=>println!("get=404")};match ready(update_note(Path(1),state(),Json(UpdateNoteDocumentInput{title:"t".to_string()}))){Ok(v)=>println!("update=ok:{}",v.0.len()),Err(_)=>println!("update=err")};}
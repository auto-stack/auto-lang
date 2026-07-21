// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum AppMsg {
    LoadNotes,
    SelectNote(i32),
    NewNote,
    DeleteNote,
    SaveNote,
    SearchChanged,
    Init,
    EditorPanel(EditorPanelMsg),
    __InitLoaded(Vec<serde_json::Value>),
}

#[derive(Debug)]
pub struct App {
    pub notes: Vec<serde_json::Value>,
    pub active_index: i32,
    pub search: String,
    pub editing: bool,
    pub edit_title: String,
    pub edit_body: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            notes: vec![],
            active_index: 0,
            search: "".to_string(),
            editing: false,
            edit_title: "".to_string(),
            edit_body: "".to_string(),
        }
    }
}
impl Default for App {
    fn default() -> Self { Self::new() }
}

impl Component for App {
    type Msg = AppMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            AppMsg::DeleteNote => {
                delete_note(self.active_index);
                self.notes = list_notes();
                if self.active_index >= self.notes.len() as i32 { self.active_index = 0 };
                self.editing = false
            }
            AppMsg::SearchChanged => {
                let _text = auto_lang::ui::iced::last_input_text();
                self.search = _text;
                self.notes = list_notes();
                self.active_index = 0
            }
            AppMsg::NewNote => {
                create_note("New Note".to_string(), "".to_string());
                self.notes = list_notes();
                self.active_index = self.notes.len() as i32 - 1;
                self.editing = true;
                self.edit_title = "New Note".to_string();
                self.edit_body = "".to_string();
                self.search = "".to_string()
            }
            AppMsg::SaveNote => {
                update_note(self.active_index, self.edit_title.clone(), self.edit_body.clone());
                self.notes = list_notes();
                self.editing = false
            }
            AppMsg::SelectNote(i) => {
                self.active_index = i;
                self.editing = false
            }
            AppMsg::Init => {
                // async init — data arrives via __InitLoaded
            }
            AppMsg::__InitLoaded(__data) => {
                self.notes = __data
            }
            AppMsg::EditorPanel(inner) => {
                let mut __child = EditorPanel::new(self.notes[self.active_index as usize].clone());
                __child.notes = self.notes.clone();
                __child.active_index = self.active_index.clone();
                __child.search = self.search.clone();
                __child.editing = self.editing.clone();
                __child.edit_title = self.edit_title.clone();
                __child.edit_body = self.edit_body.clone();
                __child.on(inner);
                self.notes = __child.notes;
                self.active_index = __child.active_index;
                self.search = __child.search;
                self.editing = __child.editing;
                self.edit_title = __child.edit_title;
                self.edit_body = __child.edit_body;
            }
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("w-full h-screen bg-white flex-col").child(View::row().style("w-full items-center p-4 border-b border-gray-200").child(View::text_styled("Notes".to_string(), "text-3xl font-bold text-xl font-bold text-gray-800")).child(View::button("+ New").style("ml-auto px-4 py-2 bg-blue-500 text-white rounded-lg text-sm hover:bg-blue-600").on_click(|_| AppMsg::NewNote).build()).build()).child(View::row().style("flex-1").child(View::col().style("w-64 border border-gray-200 p-3 flex-shrink-0 gap-1 overflow-y-auto").child(View::input("Search...").value(format!("{}", self.search)).style("w-full px-3 py-2 text-sm border rounded-lg mb-2").on_change(AppMsg::SearchChanged).build()).children(self.notes.iter().enumerate().filter(|(_, note)| { let __q = self.search.to_lowercase(); if __q.is_empty() { return true; } let __t = note["title"].as_str().unwrap_or_default().to_lowercase(); __t.contains(&__q) }).map(|(i, note)| { let i = i as i32; View::button(note["title"].as_str().unwrap_or_default().to_string()).style(if i == self.active_index { "w-full text-left p-3 rounded-lg text-sm font-semibold text-blue-600 hover:bg-blue-50 bg-blue-50".to_string() } else { "w-full text-left p-3 rounded-lg text-sm font-semibold text-blue-600 hover:bg-blue-50".to_string() }.as_str()).on_click(|_| AppMsg::SelectNote(i)).build() })).build()).child(View::col().style("flex-1").child(if self.notes.len ( ) > 0 { { let mut __editorpanel = EditorPanel::new(self.notes[self.active_index as usize].clone()); __editorpanel.editing = self.editing.clone(); __editorpanel.active_index = self.active_index.clone(); __editorpanel.edit_title = self.edit_title.clone(); __editorpanel.edit_body = self.edit_body.clone(); __editorpanel.view().map_msg(|m| AppMsg::EditorPanel(m)) } } else { View::Empty }).build()).build()).build()
    }
}


// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum EditorPanelMsg {
    Edit,
    Save,
    Cancel,
    EditBody,
    EditTitle,
    Delete,
}

#[derive(Debug)]
pub struct EditorPanel {
    pub note: serde_json::Value,
    pub editing: bool,
    pub edit_title: String,
    pub edit_body: String,
    pub notes: Vec<serde_json::Value>,
    pub active_index: i32,
    pub search: String,
}

impl EditorPanel {
    pub fn new(note: serde_json::Value) -> Self {
        Self {
            note: note,
            editing: false,
            edit_title: "".to_string(),
            edit_body: "".to_string(),
            notes: vec![],
            active_index: 0,
            search: "".to_string(),
        }
    }
}

impl Component for EditorPanel {
    type Msg = EditorPanelMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            EditorPanelMsg::Save => {
                self.note["title"] = serde_json::json!(self.edit_title);
                self.note["body"] = serde_json::json!(self.edit_body);
                update_note(self.note["id"].as_i64().unwrap_or(0) as i32, self.edit_title.clone(), self.edit_body.clone());
                self.editing = false
            }
            EditorPanelMsg::EditBody => {
                let _text = auto_lang::ui::iced::last_input_text();
                self.edit_body = _text;
            }
            EditorPanelMsg::Edit => {
                self.edit_title = self.note["title"].as_str().unwrap_or_default().to_string().to_string();
                self.edit_body = self.note["body"].as_str().unwrap_or_default().to_string().to_string();
                self.editing = true
            }
            EditorPanelMsg::EditTitle => {
                let _text = auto_lang::ui::iced::last_input_text();
                self.edit_title = _text;
            }
            EditorPanelMsg::Cancel => {
                self.editing = false
            }
            EditorPanelMsg::Delete => {
                delete_note(self.active_index);
                self.notes = list_notes();
                if self.active_index >= self.notes.len() as i32 { self.active_index = 0 };
                self.editing = false;
                self.search = "".to_string()
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("flex-1 flex-col").child(View::col().style("px-6 pt-6 pb-2 flex-1").child(if self.editing == false { View::text_styled(self.note["title"].as_str().unwrap_or_default().to_string(), "text-lg font-semibold text-gray-800") } else { View::Empty }).child(if self.editing == true { View::input("Note title...").value(format!("{}", self.edit_title)).style("text-lg font-semibold text-gray-800 border-b border-gray-200 outline-none w-full focus:border-blue-500 p-1").on_change(EditorPanelMsg::EditTitle).build() } else { View::Empty }).child(View::text_styled(self.note["time"].as_str().unwrap_or_default().to_string(), "text-xs text-gray-400 mt-1")).child(if self.editing == false { View::text_styled(self.note["body"].as_str().unwrap_or_default().to_string(), "text-gray-700 flex-1 leading-relaxed") } else { View::Empty }).child(if self.editing == true { View::textarea("Start writing...").value(format!("{}", self.edit_body)).style("flex-1 p-3 border rounded-lg text-sm text-gray-700 resize-none focus:outline-none focus:ring-2 focus:ring-blue-500").on_change(EditorPanelMsg::EditBody).build() } else { View::Empty }).build()).child(View::row().style("p-4 border-t border-gray-100").child(if self.editing == false { View::button("Edit").style("px-4 py-2 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600 font-medium").on_click(|_| EditorPanelMsg::Edit).build() } else { View::Empty }).child(if self.editing == true { View::row().style("gap-2").child(View::button("Save").style("px-4 py-2 text-sm bg-blue-500 text-white rounded-lg hover:bg-blue-600 font-medium").on_click(|_| EditorPanelMsg::Save).build()).child(View::button("Cancel").style("px-4 py-2 text-sm bg-gray-500 text-white rounded-lg hover:bg-gray-600").on_click(|_| EditorPanelMsg::Cancel).build()).build() } else { View::Empty }).child(View::button("Delete").style("ml-auto px-4 py-2 text-sm bg-red-500 text-white rounded-lg hover:bg-red-600").on_click(|_| EditorPanelMsg::Delete).build()).build()).build()
    }
}


// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum NoteItemMsg {
    Select,
    Delete,
}

#[derive(Debug)]
pub struct NoteItem {
    pub note: serde_json::Value,
    pub is_active: bool,
    pub notes: Vec<serde_json::Value>,
    pub active_index: i32,
    pub search: String,
    pub editing: bool,
    pub edit_title: String,
    pub edit_body: String,
}

impl NoteItem {
    pub fn new(note: serde_json::Value, is_active: bool) -> Self {
        Self {
            note: note,
            is_active: is_active,
            notes: vec![],
            active_index: 0,
            search: "".to_string(),
            editing: false,
            edit_title: "".to_string(),
            edit_body: "".to_string(),
        }
    }
}

impl Component for NoteItem {
    type Msg = NoteItemMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            NoteItemMsg::Select => {
                
            }
            NoteItemMsg::Delete => {
                
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::button("").style("w-full text-left").child(View::col().style("w-full text-left p-3 rounded-lg text-sm hover:bg-blue-50 transition-colors gap-0").child(View::text_styled(self.note["title"].as_str().unwrap_or_default().to_string(), "text-sm font-semibold truncate")).child(View::text_styled(self.note["body"].as_str().unwrap_or_default().to_string(), "text-xs text-gray-500 truncate mt-1")).child(View::text_styled(self.note["time"].as_str().unwrap_or_default().to_string(), "text-xs text-gray-400 mt-1")).build()).on_click(|_| NoteItemMsg::Select).build()
    }
}


// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum SidebarMsg {
    SelectNote,
    SearchChanged,
}

#[derive(Debug)]
pub struct Sidebar {
    pub notes: Vec<serde_json::Value>,
    pub active_id: i32,
    pub search: String,
    pub query: String,
    pub active_index: i32,
    pub editing: bool,
    pub edit_title: String,
    pub edit_body: String,
}

impl Sidebar {
    pub fn new(notes: Vec<serde_json::Value>, active_id: i32, search: String) -> Self {
        Self {
            notes: notes,
            active_id: active_id,
            search: search,
            query: "".to_string(),
            active_index: 0,
            editing: false,
            edit_title: "".to_string(),
            edit_body: "".to_string(),
        }
    }
}

impl Component for Sidebar {
    type Msg = SidebarMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            SidebarMsg::SelectNote => {
                
            }
            SidebarMsg::SearchChanged => {
                let _text = auto_lang::ui::iced::last_input_text();
                self.query = _text;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("w-64 border border-gray-200 p-3 flex-shrink-0 gap-1 overflow-y-auto").child(View::input("Search...").value(format!("{}", self.query)).style("w-full px-3 py-2 text-sm border rounded-lg mb-2").on_change(SidebarMsg::SearchChanged).build()).build()
    }
}




// Plan 349: TLS configuration helper.
// Set AUTO_TLS_SKIP_VERIFY=1 to skip certificate verification (dev/test).
// Set AUTO_TLS_CA_CERT=/path/to/ca.pem for custom CA (requires native-tls).
fn _tls_skip_verify() -> bool {
    std::env::var("AUTO_TLS_SKIP_VERIFY").as_deref() == Ok("1")
}

// Plan 349: File upload (multipart) + download utilities (a2r)

fn upload_file(url: &str, file_path: &str) -> serde_json::Value {
    std::thread::spawn(move || {
        let form = reqwest::blocking::multipart::Form::new()
            .file("file", file_path)
            .map_err(|e| e.to_string())?;
        let resp = reqwest::blocking::Client::new()
            .post(url)
            .multipart(form)
            .send()
            .map_err(|e| e.to_string())?;
        let text = resp.text().map_err(|e| e.to_string())?;
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null)
    }).join().unwrap_or(serde_json::Value::Null)
}

fn upload_file_with_fields(url: &str, file_path: &str, fields: &serde_json::Value) -> serde_json::Value {
    let url = url.to_string();
    let file_path = file_path.to_string();
    let fields = fields.clone();
    std::thread::spawn(move || {
        let mut form = reqwest::blocking::multipart::Form::new();
        if let Some(obj) = fields.as_object() {
            for (k, v) in obj {
                if let Some(s) = v.as_str() {
                    form = form.text(k.clone(), s.to_string());
                }
            }
        }
        if let Ok(part) = reqwest::blocking::multipart::Part::file(&file_path) {
            form = form.part("file", part);
        }
        let resp = reqwest::blocking::Client::new()
            .post(&url)
            .multipart(form)
            .send()
            .map_err(|e| e.to_string())?;
        let text = resp.text().map_err(|e| e.to_string())?;
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null)
    }).join().unwrap_or(serde_json::Value::Null)
}

fn download_file(url: &str, file_path: &str) -> bool {
    let url = url.to_string();
    let file_path = file_path.to_string();
    std::thread::spawn(move || {
        let resp = match reqwest::blocking::get(&url) { Ok(r) => r, Err(_) => return false };
        use std::io::Write;
        let mut file = match std::fs::File::create(&file_path) { Ok(f) => f, Err(_) => return false };
        match resp.bytes() {
            Ok(b) => file.write_all(&b).is_ok(),
            Err(_) => false,
        }
    }).join().unwrap_or(false)
}

fn download_file_resume(url: &str, file_path: &str, offset: u64) -> bool {
    let url = url.to_string();
    let file_path = file_path.to_string();
    std::thread::spawn(move || {
        let range = format!("bytes={}-", offset);
        let resp = match reqwest::blocking::Client::new()
            .get(&url).header("Range", &range).send() { Ok(r) => r, Err(_) => return false };
        use std::io::Write;
        let mut file = match std::fs::OpenOptions::new().append(true).open(&file_path) {
            Ok(f) => f, Err(_) => return false
        };
        match resp.bytes() {
            Ok(b) => file.write_all(&b).is_ok(),
            Err(_) => false,
        }
    }).join().unwrap_or(false)
}

// Plan 350: WebSocket client (a2r)
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref WS_CONNS: Mutex<HashMap<i32, WsConn>> = Mutex::new(HashMap::new());
    static ref WS_NEXT_ID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(1);
}

struct WsConn {
    sender: Option<std::sync::mpsc::Sender<String>>,
}

fn ws_connect(url: &str) -> i32 {
    let id = WS_NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    let url = url.to_string();

    std::thread::spawn(move || {
        use tungstenite::Message;
        let (mut socket, _) = match tungstenite::connect(&url) {
            Ok(pair) => pair,
            Err(_) => return,
        };
        loop {
            // Check for outgoing messages (non-blocking).
            if let Ok(msg) = rx.try_recv() {
                if socket.send(Message::Text(msg.into())).is_err() { break; }
            }
            match socket.read() {
                Ok(Message::Text(_)) | Ok(Message::Binary(_)) => {}
                Ok(Message::Close(_)) | Err(_) => break,
                _ => {}
            }
        }
    });

    WS_CONNS.lock().unwrap().insert(id, WsConn { sender: Some(tx) });
    id
}

fn ws_send(handle: i32, message: &str) -> bool {
    WS_CONNS.lock().unwrap()
        .get(&handle)
        .and_then(|conn| conn.sender.as_ref())
        .and_then(|tx| tx.send(message.to_string()).ok())
        .is_some()
}

fn ws_close(handle: i32) {
    if let Some(conn) = WS_CONNS.lock().unwrap().get_mut(&handle) {
        conn.sender = None;
    }
    WS_CONNS.lock().unwrap().remove(&handle);
}

fn list_notes() -> Vec<serde_json::Value> {
    ureq::get("http://127.0.0.1:8080/api/notes")
        .call().ok()
        .and_then(|r| r.into_json::<Vec<serde_json::Value>>().ok())
        .unwrap_or_default()
}

fn get_note(id: i32) -> Option<serde_json::Value> {
    ureq::get(&format!("http://127.0.0.1:8080/api/notes/{}", id))
        .call().ok()
        .and_then(|r| r.into_json::<serde_json::Value>().ok())
}

fn create_note(title: String, body: String) -> serde_json::Value {
    let url = "http://127.0.0.1:8080/api/notes";
    let body = serde_json::json!({"title": title, "body": body});
    let local_result = serde_json::json!({"title": title, "body": body});
    std::thread::spawn(move || { let _ = ureq::post(&url).send_json(body); });
    local_result
}

fn update_note(id: i32, title: String, body: String) {
    let url = format!("http://127.0.0.1:8080/api/notes/{}", id).to_string();
    let body = serde_json::json!({"title": title, "body": body});
    std::thread::spawn(move || { let _ = ureq::put(&url).send_json(body); });
}

fn delete_note(id: i32) {
    let url = format!("http://127.0.0.1:8080/api/notes/{}", id).to_string();
    std::thread::spawn(move || { let _ = ureq::delete(&url).call(); });
}

fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        println!("Running with Iced backend");
        let __init = std::cell::RefCell::new(Some(
            iced::Task::perform(
                async { tokio::task::spawn_blocking(|| list_notes()).await.unwrap_or_default() },
                |r| AppMsg::__InitLoaded(r)
            )
        ));
        return auto_lang::ui::iced::run_app_with_task_devtools(move || {
            let task = __init.borrow_mut().take().unwrap_or_else(iced::Task::none);
            (App::default(), task)
        });
    }
    #[cfg(feature = "ui-gpui")]
    {
        println!("Running with GPUI backend");
        return auto_lang::ui::gpui::run_app::<App>("notes");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}

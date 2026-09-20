// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// 014 内存哨兵:记账分配器(存活字节 + 大块分配点回溯;超限冻结时
// 自动落盘分配报告到 %TEMP%/auto-term-mem-report.txt)。
#[global_allocator]
static GUARD_ALLOC: auto_lang::ui::mem_guard::GuardAlloc = auto_lang::ui::mem_guard::GuardAlloc;

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum AppMsg {
    Init,
    NewNote,
    ToggleDarkMode,
    SetMode(String),
    SetAccent(String),
    ToggleSettings,
    NavTree(NavTreeMsg),
    EditorPanel(EditorPanelMsg),
}

#[derive(Clone, Debug)]
pub struct App {
    pub dark_mode: bool,
    pub accent_color: String,
    pub nav_tree: NavTree,
    pub editor_panel: EditorPanel,
    pub store: NotesStore,
}

impl App {
    pub fn new() -> Self {
        let mut __self = Self {
            dark_mode: true,
            accent_color: "indigo".to_string(),
            nav_tree: NavTree::default(),
            editor_panel: EditorPanel::default(),
            store: NotesStore::new(),
        };
        __self.on(AppMsg::Init);
        __self.nav_tree = NavTree::new();
        __self.editor_panel = EditorPanel::new();
        __self
    }
}
impl Default for App {
    fn default() -> Self { Self::new() }
}

impl Component for App {
    type Msg = AppMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            AppMsg::NewNote => {
                self.store.on(NotesStoreMsg::NewNote);
            }
            AppMsg::SetAccent(name) => {
                self.store.on(NotesStoreMsg::SetAccent(name));
                self.accent_color = self.store.accent_color.to_string();
            }
            AppMsg::SetMode(m) => {
                self.store.on(NotesStoreMsg::SetMode(m));
                self.dark_mode = self.store.dark_mode;
            }
            AppMsg::ToggleDarkMode => {
                self.store.on(NotesStoreMsg::ToggleDarkMode);
                self.dark_mode = self.store.dark_mode;
            }
            AppMsg::ToggleSettings => {
                self.store.on(NotesStoreMsg::ToggleSettings);
            }
            AppMsg::Init => {
                self.store.on(NotesStoreMsg::Init);
            }
            AppMsg::NavTree(inner) => {
                self.nav_tree.on(inner);
                self.store = self.nav_tree.store.clone();
            }
            AppMsg::EditorPanel(inner) => {
                self.editor_panel.on(inner);
                self.store = self.editor_panel.store.clone();
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("w-full h-screen flex-col bg-background text-foreground overflow-hidden").child(View::row().style("h-14 shrink-0 items-center justify-between gap-3 px-4 border-b border-border").child(View::row().style("w-auto items-center gap-2").child(View::image_styled("lucide:notebook".to_string(), "text-primary w-[18px] h-[18px]")).child(View::text_styled("Notes".to_string(), "text-sm font-semibold tracking-tight")).build()).child(View::row().style("w-auto items-center gap-1").child(View::button("").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-8 w-8 border-0 flex items-center justify-center rounded-md bg-transparent text-muted-foreground hover:text-foreground hover:bg-accent transition-colors").on_click(|_| AppMsg::ToggleDarkMode).build()).child(View::button(format!("{}", "New note".to_string())).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-9 w-auto border-0 flex items-center gap-1.5 px-3 rounded-md bg-primary text-primary-foreground text-sm font-medium shadow-sm hover:bg-primary/90 transition-colors").on_click(|_| AppMsg::NewNote).build()).build()).build()).child(View::row().style("flex-1 min-h-0").child({ let mut __c = self.nav_tree.clone(); __c.store = self.store.clone(); __c.view().map_msg(|m| AppMsg::NavTree(m)) }).child(View::col().style("flex-1 min-h-0").child(if self.store.notes.len ( ) > 0 { { let mut __c = self.editor_panel.clone(); __c.store = self.store.clone(); __c.view().map_msg(|m| AppMsg::EditorPanel(m)) } } else { View::col().style("flex-1 items-center justify-center gap-3").child(View::image_styled("lucide:notebook".to_string(), "text-muted-foreground/40 w-[40px] h-[40px]")).child(View::text_styled("No notes yet".to_string(), "text-sm text-muted-foreground")).child(View::button(format!("{}", "Create your first note".to_string())).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-9 w-auto border-0 flex items-center gap-1.5 px-3 rounded-md bg-primary text-primary-foreground text-sm font-medium shadow-sm hover:bg-primary/90 transition-colors").on_click(|_| AppMsg::NewNote).build()).build() }).build()).build()).build()
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("dark_mode".to_string(), auto_lang::ui::auto_val::Value::Bool(self.dark_mode));
        m.insert("accent_color".to_string(), auto_lang::ui::auto_val::Value::str(&self.accent_color));
        for (k, v) in self.store.state_snapshot() { m.insert(format!("{}.{}", "store", k), v); }
        for (k, v) in self.nav_tree.state_snapshot() { m.insert(format!("{}.{}", "nav_tree", k), v); }
        for (k, v) in self.editor_panel.state_snapshot() { m.insert(format!("{}.{}", "editor_panel", k), v); }
        m
    }
}


// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum NotesStoreMsg {
    Init,
    SeedVocab,
    ApplyFilter,
    LoadDraft,
    SelectFirstVisible,
    SelectNote(i32),
    NewNote,
    SaveDraft,
    EditTitle(String),
    EditBody(String),
    DeleteArmed,
    DeleteConfirmed,
    DeleteCancelled,
    TogglePin(i32),
    TogglePinActive,
    SetSearch(String),
    SelectFolder(String),
    SelectTag(String),
    ClearTag,
    TagAdded(String),
    TagRemoved(String),
    ToggleDarkMode,
    SetMode(String),
    SetAccent(String),
    ToggleSettings,
}

#[derive(Clone, Debug)]
pub struct NotesStore {
    pub notes: Vec<Note>,
    pub active_id: i32,
    pub search: String,
    pub active_folder: String,
    pub active_tag: String,
    pub visible_pinned: Vec<i32>,
    pub visible_notes: Vec<i32>,
    pub all_tags: Vec<String>,
    pub all_folders: Vec<String>,
    pub draft_id: i32,
    pub draft_title: String,
    pub draft_body: String,
    pub draft_time: String,
    pub draft_folder: String,
    pub draft_pinned: bool,
    pub draft_tags: Vec<String>,
    pub dirty: bool,
    pub confirm_delete: bool,
    pub dark_mode: bool,
    pub accent_color: String,
    pub show_settings: bool,
}

impl NotesStore {
    pub fn new() -> Self {
        let mut __self = Self {
            notes: vec![],
            active_id: 0,
            search: "".to_string(),
            active_folder: "all".to_string(),
            active_tag: "".to_string(),
            visible_pinned: vec![],
            visible_notes: vec![],
            all_tags: vec![],
            all_folders: vec![],
            draft_id: 0,
            draft_title: "".to_string(),
            draft_body: "".to_string(),
            draft_time: "".to_string(),
            draft_folder: "".to_string(),
            draft_pinned: false,
            draft_tags: vec![],
            dirty: false,
            confirm_delete: false,
            dark_mode: true,
            accent_color: "indigo".to_string(),
            show_settings: false,
        };
        __self.on(NotesStoreMsg::Init);
        __self
    }
}
impl Default for NotesStore {
    fn default() -> Self { Self::new() }
}

impl Component for NotesStore {
    type Msg = NotesStoreMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            NotesStoreMsg::ApplyFilter => {
                let mut pinned = vec![];
                let mut plain = vec![];
                let mut ok = false;
                let mut idx = 0;
                for mut n in self.notes.iter_mut() { ok = false; if self.active_folder == "all".to_string() { ok = true; } else if self.active_folder == "pinned".to_string() { if n["pinned"].as_bool().unwrap_or(false) { ok = true; }; } else if n["folder"].as_str().unwrap_or_default().to_string() == self.active_folder { ok = true; }; if ok { if self.active_tag != "".to_string() { ok = false; for t in n["tags"].as_array().into_iter().flatten() { if t == self.active_tag { ok = true; }; }; }; }; if ok { if n["pinned"].as_bool().unwrap_or(false) { pinned.push(idx); } else { plain.push(idx); }; }; idx = idx + 1; };
                self.visible_pinned = pinned;
                self.visible_notes = plain;
            }
            NotesStoreMsg::ClearTag => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes(); } else { self.notes = search_notes(self.search.clone()); }; self.dirty = false; };
                self.active_tag = "".to_string();
                self.on(NotesStoreMsg::ApplyFilter);
                self.on(NotesStoreMsg::SelectFirstVisible);
            }
            NotesStoreMsg::DeleteArmed => {
                self.confirm_delete = true;
            }
            NotesStoreMsg::DeleteCancelled => {
                self.confirm_delete = false;
            }
            NotesStoreMsg::DeleteConfirmed => {
                if self.notes.len() as i32 > 0 { delete_note((self.notes[(self.active_id) as usize]["id"].as_i64().unwrap_or(0) as i32)); if self.search == "".to_string() { self.notes = list_notes(); } else { self.notes = search_notes(self.search.clone()); }; self.active_id = 0; self.on(NotesStoreMsg::LoadDraft); self.on(NotesStoreMsg::ApplyFilter); };
                self.confirm_delete = false;
            }
            NotesStoreMsg::EditBody(v) => {
                self.draft_body = v.to_string();
                self.dirty = true;
            }
            NotesStoreMsg::EditTitle(v) => {
                self.draft_title = v.to_string();
                self.dirty = true;
            }
            NotesStoreMsg::LoadDraft => {
                if self.notes.len() as i32 > 0 { self.draft_id = (self.notes[(self.active_id) as usize]["id"].as_i64().unwrap_or(0) as i32); self.draft_title = self.notes[(self.active_id) as usize]["title"].as_str().unwrap_or_default().to_string().to_string(); self.draft_body = self.notes[(self.active_id) as usize]["body"].as_str().unwrap_or_default().to_string().to_string(); self.draft_time = self.notes[(self.active_id) as usize]["time"].as_str().unwrap_or_default().to_string().to_string(); self.draft_folder = self.notes[(self.active_id) as usize]["folder"].as_str().unwrap_or_default().to_string().to_string(); self.draft_pinned = self.notes[(self.active_id) as usize]["pinned"].as_bool().unwrap_or(false); self.draft_tags = self.notes[(self.active_id) as usize]["tags"].as_str().unwrap_or_default().to_string(); } else { self.draft_id = 0; self.draft_title = "".to_string(); self.draft_body = "".to_string(); self.draft_time = "".to_string(); self.draft_folder = "".to_string(); self.draft_pinned = false; };
                self.dirty = false;
                self.confirm_delete = false;
            }
            NotesStoreMsg::NewNote => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes(); } else { self.notes = search_notes(self.search.clone()); }; };
                self.dirty = false;
                self.search = "".to_string();
                let mut f = "".to_string();
                if self.active_folder != "all".to_string() { if self.active_folder != "pinned".to_string() { f = self.active_folder; }; };
                create_note("".to_string(), "".to_string(), f);
                self.notes = list_notes();
                self.active_id = 0;
                self.on(NotesStoreMsg::LoadDraft);
                self.on(NotesStoreMsg::ApplyFilter);
            }
            NotesStoreMsg::SaveDraft => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes(); } else { self.notes = search_notes(self.search.clone()); }; };
                self.dirty = false;
                self.on(NotesStoreMsg::LoadDraft);
                self.on(NotesStoreMsg::ApplyFilter);
            }
            NotesStoreMsg::SeedVocab => {
                let mut dup = false;
                for mut n in self.notes.iter_mut() { for t in n["tags"].as_array().into_iter().flatten() { dup = false; for mut e in self.all_tags.iter_mut() { if e == t { dup = true; }; }; if dup == false { self.all_tags.push(t); }; }; if n["folder"].as_str().unwrap_or_default().to_string() != "".to_string() { dup = false; for mut e in self.all_folders.iter_mut() { if e == n["folder"].as_str().unwrap_or_default().to_string() { dup = true; }; }; if dup == false { self.all_folders.push(n["folder"].as_str().unwrap_or_default().to_string()); }; }; };
            }
            NotesStoreMsg::SelectFirstVisible => {
                if self.visible_pinned.len() as i32 > 0 { self.active_id = self.visible_pinned[0]; } else if self.visible_notes.len() as i32 > 0 { self.active_id = self.visible_notes[0]; };
                self.on(NotesStoreMsg::LoadDraft);
            }
            NotesStoreMsg::SelectFolder(f) => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes(); } else { self.notes = search_notes(self.search.clone()); }; self.dirty = false; };
                self.active_folder = f.to_string();
                self.active_tag = "".to_string();
                self.on(NotesStoreMsg::ApplyFilter);
                self.on(NotesStoreMsg::SelectFirstVisible);
            }
            NotesStoreMsg::SelectNote(i) => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes(); } else { self.notes = search_notes(self.search.clone()); }; };
                self.dirty = false;
                self.active_id = i;
                self.on(NotesStoreMsg::LoadDraft);
            }
            NotesStoreMsg::SelectTag(t) => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes(); } else { self.notes = search_notes(self.search.clone()); }; self.dirty = false; };
                self.active_tag = t.to_string();
                self.active_folder = "all".to_string();
                self.on(NotesStoreMsg::ApplyFilter);
                self.on(NotesStoreMsg::SelectFirstVisible);
            }
            NotesStoreMsg::SetAccent(name) => {
                self.accent_color = name.to_string();
            }
            NotesStoreMsg::SetMode(m) => {
                if m == "dark".to_string() { if self.dark_mode == false { self.on(NotesStoreMsg::ToggleDarkMode); }; } else { if self.dark_mode == true { self.on(NotesStoreMsg::ToggleDarkMode); }; };
            }
            NotesStoreMsg::SetSearch(q) => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes(); } else { self.notes = search_notes(self.search.clone()); }; self.dirty = false; };
                self.search = q.to_string();
                if self.search == "".to_string() { self.notes = list_notes(); } else { self.notes = search_notes(self.search.clone()); };
                self.active_id = 0;
                self.on(NotesStoreMsg::LoadDraft);
                self.on(NotesStoreMsg::ApplyFilter);
            }
            NotesStoreMsg::TagAdded(t) => {
                if self.notes.len() as i32 > 0 { let mut tl = vec![]; let mut exists = false; for tg in self.notes[(self.active_id) as usize]["tags"].as_array().into_iter().flatten() { if tg == t { exists = true; }; tl.push(tg); }; if exists == false { tl.push(t); }; update_tags((self.notes[(self.active_id) as usize]["id"].as_i64().unwrap_or(0) as i32), tl); let mut dup = false; for mut e in self.all_tags.iter_mut() { if e == t { dup = true; }; }; if dup == false { self.all_tags.push(t); }; if self.search == "".to_string() { self.notes = list_notes(); } else { self.notes = search_notes(self.search.clone()); }; self.on(NotesStoreMsg::LoadDraft); self.on(NotesStoreMsg::ApplyFilter); };
            }
            NotesStoreMsg::TagRemoved(t) => {
                if self.notes.len() as i32 > 0 { let mut tl = vec![]; for tg in self.notes[(self.active_id) as usize]["tags"].as_array().into_iter().flatten() { if tg.as_str().unwrap_or_default() != t.as_str() { tl.push(tg); }; }; update_tags((self.notes[(self.active_id) as usize]["id"].as_i64().unwrap_or(0) as i32), tl); if self.search == "".to_string() { self.notes = list_notes(); } else { self.notes = search_notes(self.search.clone()); }; self.on(NotesStoreMsg::LoadDraft); self.on(NotesStoreMsg::ApplyFilter); };
            }
            NotesStoreMsg::ToggleDarkMode => {
                self.dark_mode = !(self.dark_mode);
            }
            NotesStoreMsg::TogglePin(idx) => {
                if idx < self.notes.len() as i32 { let mut cur = self.notes[(idx) as usize]["pinned"].as_bool().unwrap_or(false); self.notes[(idx)as usize]["pinned"] = serde_json::json!(!(as usize]["pinned"].as_bool().unwrap_or(false))); };
            }
            NotesStoreMsg::TogglePinActive => {
                if self.notes.len() as i32 > 0 { if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); self.dirty = false; }; toggle_pin((self.notes[(self.active_id) as usize]["id"].as_i64().unwrap_or(0) as i32)); if self.search == "".to_string() { self.notes = list_notes(); } else { self.notes = search_notes(self.search.clone()); }; self.on(NotesStoreMsg::LoadDraft); self.on(NotesStoreMsg::ApplyFilter); };
            }
            NotesStoreMsg::ToggleSettings => {
                self.show_settings = !(self.show_settings);
            }
            NotesStoreMsg::Init => {
                self.search = "".to_string();
                self.active_folder = "all".to_string();
                self.active_tag = "".to_string();
                self.notes = list_notes();
                self.on(NotesStoreMsg::SeedVocab);
                self.active_id = 0;
                self.on(NotesStoreMsg::LoadDraft);
                self.on(NotesStoreMsg::ApplyFilter);
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().build()
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("active_id".to_string(), auto_lang::ui::auto_val::Value::Int(self.active_id));
        m.insert("search".to_string(), auto_lang::ui::auto_val::Value::str(&self.search));
        m.insert("active_folder".to_string(), auto_lang::ui::auto_val::Value::str(&self.active_folder));
        m.insert("active_tag".to_string(), auto_lang::ui::auto_val::Value::str(&self.active_tag));
        m.insert("draft_id".to_string(), auto_lang::ui::auto_val::Value::Int(self.draft_id));
        m.insert("draft_title".to_string(), auto_lang::ui::auto_val::Value::str(&self.draft_title));
        m.insert("draft_body".to_string(), auto_lang::ui::auto_val::Value::str(&self.draft_body));
        m.insert("draft_time".to_string(), auto_lang::ui::auto_val::Value::str(&self.draft_time));
        m.insert("draft_folder".to_string(), auto_lang::ui::auto_val::Value::str(&self.draft_folder));
        m.insert("draft_pinned".to_string(), auto_lang::ui::auto_val::Value::Bool(self.draft_pinned));
        m.insert("dirty".to_string(), auto_lang::ui::auto_val::Value::Bool(self.dirty));
        m.insert("confirm_delete".to_string(), auto_lang::ui::auto_val::Value::Bool(self.confirm_delete));
        m.insert("dark_mode".to_string(), auto_lang::ui::auto_val::Value::Bool(self.dark_mode));
        m.insert("accent_color".to_string(), auto_lang::ui::auto_val::Value::str(&self.accent_color));
        m.insert("show_settings".to_string(), auto_lang::ui::auto_val::Value::Bool(self.show_settings));
        m
    }
}


pub type Note = serde_json::Value;

pub type Folder = serde_json::Value;



// API functions (auto-generated, in-process merged mode — no HTTP)

use std::sync::{LazyLock, Mutex};
use serde_json::Value;

static API_DATA: LazyLock<Mutex<Vec<Value>>> = LazyLock::new(|| {
    Mutex::new(vec![serde_json::json!({"id": 0, "title": "Welcome", "body": "This is your notes app. Click on any note to view it.", "time": "Just now", "pinned": true, "tags": ["intro"], "folder": ""}), serde_json::json!({"id": 1, "title": "Shopping List", "body": "Milk, Eggs, Bread, Cheese", "time": "2 hours ago", "pinned": false, "tags": ["home"], "folder": "personal"}), serde_json::json!({"id": 2, "title": "Meeting Notes", "body": "Q3 roadmap discussion with the team", "time": "Yesterday", "pinned": false, "tags": ["work"], "folder": "work"})])
});
static API_NEXT_ID: LazyLock<Mutex<i64>> = LazyLock::new(|| Mutex::new(100));

fn list_notes() -> Vec<Value> {
    API_DATA.lock().unwrap().clone()
}

fn get_note(id: i32) -> Option<Value> {
    API_DATA.lock().unwrap().iter().find(|n| n["id"].as_i64() == Some(id as i64)).cloned()
}

fn create_note(title: String, body: String, folder: String) -> Value {
    let mut data = API_DATA.lock().unwrap();
    let id = { let mut next = API_NEXT_ID.lock().unwrap(); *next += 1; *next };
    let item = serde_json::json!({"id": id, "title": serde_json::Value::from(title.clone()), "body": serde_json::Value::from(body.clone()), "folder": serde_json::Value::from(folder.clone())});
    data.push(item.clone());
    item
}

fn update_note(id: i32, title: String, body: String) {
    let mut data = API_DATA.lock().unwrap();
    if let Some(item) = data.iter_mut().find(|n| n["id"].as_i64() == Some(id as i64)) {
        item["title"] = serde_json::Value::from(title.clone()); item["body"] = serde_json::Value::from(body.clone());
    }
}

fn delete_note(id: i32) {
    let mut data = API_DATA.lock().unwrap();
    data.retain(|n| n["id"].as_i64() != Some(id as i64));
}

fn toggle_pin(id: i32) {
    let mut data = API_DATA.lock().unwrap();
    if let Some(item) = data.iter_mut().find(|n| n["id"].as_i64() == Some(id as i64)) {
        ;
    }
}

fn update_tags(id: i32, tags: Vec<String>) {
    let mut data = API_DATA.lock().unwrap();
    if let Some(item) = data.iter_mut().find(|n| n["id"].as_i64() == Some(id as i64)) {
        item["tags"] = serde_json::Value::Array(tags.iter().map(|s| serde_json::Value::from(s.clone())).collect());
    }
}

fn search_notes(id: i32) -> Option<Value> {
    API_DATA.lock().unwrap().iter().find(|n| n["id"].as_i64() == Some(id as i64)).cloned()
}

fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        if std::env::var("AUTO_VM_TITLE").is_err() {
            std::env::set_var("AUTO_VM_TITLE", "备忘录");
        }
        // Plan 020 T-05：孵化参数在册 → native 协议 client 臂（返回即走）；
        // 无标记 → 独立窗（下行 iced_entry 现行行为零变化）。
        let __autodesk_args: Vec<String> = std::env::args().collect();
        let __has_client = __autodesk_args.iter().any(|a| a.starts_with("--autodesk-client="));
        let __has_incubate = __autodesk_args.iter().any(|a| a == "--autodesk-incubate");
        if __has_client || __has_incubate {
            let mut __pipe: Option<String> = None;
            let mut __broker = auto_lang::ui::desktop_protocol::broker::BROKER_PIPE.to_string();
            let mut __render: Option<String> = None;
            let mut __app_name = "notes".to_string();
            for __a in &__autodesk_args {
                if let Some(v) = __a.strip_prefix("--autodesk-client=") {
                    __pipe = Some(v.to_string());
                } else if let Some(v) = __a.strip_prefix("--autodesk-broker=") {
                    __broker = v.to_string();
                } else if let Some(v) = __a.strip_prefix("--autodesk-render=") {
                    __render = Some(v.to_string());
                } else if let Some(v) = __a.strip_prefix("--app386=") {
                    __app_name = v.to_string();
                }
            }
            if let Some(arg) = __render.as_deref() {
                if auto_lang::ui::desktop_protocol::coverage::RenderMode::parse(arg).is_none() {
                    eprintln!("[autodesk-client] 未知 --autodesk-render={arg}（auto|queue|independent），回退 auto");
                }
            }
            let __mode = auto_lang::ui::desktop_protocol::coverage::RenderMode::resolve(
                __render.as_deref(),
                None,
            );
            // PLAN-026 T-06 翻转：组件先行构造（auto 裁决 = 覆盖扫描制
            // ——queue 优先 + NotCovered 降级 independent 留痕）。
            let __component = App::default();
            let (__frame_mode, __downgraded, __log) =
                auto_lang::ui::desktop_protocol::client_entry::resolve_native_frame_mode(
                    __mode,
                    "App",
                    &__component.view(),
                );
            if let Some(__l) = &__log {
                eprintln!("[autodesk-client] {__l}");
            }
            let __rqhost = __autodesk_args.iter().any(|a| a == "--autodesk-rqhost");
            let __target = if __rqhost {
                // PLAN-031：rqhost 采纳（well-known rendezvous + exit-on-EOF）。
                auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Rqhost {
                    wellknown: __broker.clone(),
                    app_name: __app_name.clone(),
                }
            } else { match __pipe {
                Some(p) => auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Direct(p),
                None => auto_lang::ui::desktop_protocol::client_entry::ClientTarget::Broker {
                    broker_pipe: __broker,
                },
            } };
            let __opts = auto_lang::ui::desktop_protocol::client_entry::ClientOpts {
                app_name: __app_name.clone(),
                title: __app_name,
                width: 480.0,
                height: 320.0,
                frame_mode: __frame_mode,
                auto_downgraded: __downgraded,
            };
            return auto_lang::ui::desktop_protocol::client_entry::run_native_client(
                __component,
                __opts,
                __target,
            )
            .map_err(Into::into);
        }
        println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app_devtools::<App>();
    }
    #[cfg(feature = "ui-gpui")]
    {
        // Plan 020 §5.5：GPUI 臂不接桌面孵化客户端——参数在册报错退出留痕
        //（v1 限 iced；防静默直跑开窗与孵化预期背离）。
        if std::env::args().any(|a| a == "--autodesk-incubate" || a.starts_with("--autodesk-client=")) {
            return Err("native GPUI 臂不接桌面孵化客户端（Plan 020 v1 限 iced）".into());
        }
        println!("Running with GPUI backend");
        return auto_lang::ui::gpui::run_app::<App>("notes");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}

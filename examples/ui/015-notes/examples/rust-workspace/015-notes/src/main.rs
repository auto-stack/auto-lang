// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum AppMsg {
    Init,
    NewNote,
    NewNoteInFolder(String),
    SelectNote(i32),
    SearchChanged,
    TogglePin(i32),
    DeleteActive,
    ToggleDarkMode,
    TagsChanged,
    SelectAll,
    SelectPinned,
    SelectRecent,
    SelectTag(String),
    ClearTag,
    NavTree(NavTreeMsg),
    EditorPanel(EditorPanelMsg),
}

#[derive(Debug)]
pub struct App {
    pub search: String,
    pub store: NotesStore,
}

impl App {
    pub fn new() -> Self {
        let mut __self = Self {
            search: "".to_string(),
            store: NotesStore::new(),
        };
        __self.on(AppMsg::Init);
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
            AppMsg::SelectNote(i) => {
                self.store.active_id = i
            }
            AppMsg::SelectPinned => {
                self.store.active_folder = "pinned".to_string();
                self.store.active_tag = "".to_string()
            }
            AppMsg::NewNoteInFolder(f) => {
                self.store.on(NotesStoreMsg::NewNoteInFolder(f))
            }
            AppMsg::TogglePin(i) => {
                self.store.on(NotesStoreMsg::TogglePin(self.store.active_id))
            }
            AppMsg::ToggleDarkMode => {
                self.store.on(NotesStoreMsg::ToggleDarkMode)
            }
            AppMsg::SelectTag(t) => {
                self.store.active_tag = t
            }
            AppMsg::SearchChanged => {
                self.search = self.search.clone()
            }
            AppMsg::SelectRecent => {
                self.store.active_folder = "recent".to_string();
                self.store.active_tag = "".to_string()
            }
            AppMsg::NewNote => {
                self.store.on(NotesStoreMsg::NewNote)
            }
            AppMsg::ClearTag => {
                self.store.active_tag = "".to_string()
            }
            AppMsg::TagsChanged => {
                self.store.notes = list_notes()
            }
            AppMsg::SelectAll => {
                self.store.active_folder = "all".to_string();
                self.store.active_tag = "".to_string()
            }
            AppMsg::DeleteActive => {
                delete_note(self.store.notes[self.store.active_id as usize]["id"].as_i64().unwrap_or(0) as i32);
                self.store.notes = list_notes();
                if self.store.notes.len() as i32 > 0 { self.store.active_id = 0 }
            }
            AppMsg::Init => {
                self.store.on(NotesStoreMsg::Init)
            }
            AppMsg::NavTree(inner) => {
                let mut __child = NavTree::new(self.store.active_folder.clone(), self.store.active_id.clone(), self.store.active_tag.clone(), self.search.clone());
                __child.search = self.search.clone();
                __child.on(inner);
                self.search = __child.search;
            }
            AppMsg::EditorPanel(inner) => {
                let mut __child = EditorPanel::new(self.store.notes[self.store.active_id as usize].clone());
                __child.search = self.search.clone();
                __child.on(inner);
                self.search = __child.search;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("w-full h-screen flex-col bg-muted/30 p-3 gap-3").child(View::row().style("items-center justify-between px-5 py-3 bg-card rounded-xl shadow-sm").child(View::row().style("items-center gap-2").child(View::text_styled("📝".to_string(), "text-xl")).child(View::text_styled("Notes".to_string(), "text-3xl font-bold text-lg font-bold text-foreground")).build()).child(View::button("New").style("px-4 py-2 bg-primary text-primary-foreground rounded-full text-sm font-medium shadow-sm hover:bg-primary/90 transition-colors").on_click(|_| AppMsg::NewNote).build()).build()).child(View::row().style("flex-1 gap-3 min-h-0").child(NavTree::new(self.store.active_folder.clone(), self.store.active_id.clone(), self.store.active_tag.clone(), self.search.clone()).view().map_msg(|m| AppMsg::NavTree(m))).child(View::col().style("flex-1 min-h-0").child(if self.store.notes.len ( ) > 0 { EditorPanel::new(self.store.notes[self.store.active_id as usize].clone()).view().map_msg(|m| AppMsg::EditorPanel(m)) } else { View::col().style("flex-1 items-center justify-center bg-card rounded-xl shadow-sm").child(View::text_styled("📝".to_string(), "text-6xl mb-4 opacity-40")).child(View::text_styled("No notes yet".to_string(), "text-lg text-muted-foreground")).child(View::button("Create your first note").style("mt-4 px-6 py-2 bg-primary text-primary-foreground rounded-full text-sm font-medium shadow-sm hover:bg-primary/90 transition-colors").on_click(|_| AppMsg::NewNote).build()).build() }).build()).build()).build()
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("search".to_string(), auto_lang::ui::auto_val::Value::str(&self.search));
        m
    }
}


// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum EditorPanelMsg {
    Init,
    Edit,
    Save,
    Cancel,
    EditBody(String),
    EditTitle(String),
    Delete,
    TogglePin,
    AddTag,
    EditTagInput,
    RemoveTag(String),
    ShowTagInput,
}

#[derive(Debug)]
pub struct EditorPanel {
    pub note: Note,
    pub editing: bool,
    pub edit_title: String,
    pub edit_body: String,
    pub tag_input: String,
    pub show_tag_input: bool,
    pub store: NotesStore,
    pub search: String,
}

impl EditorPanel {
    pub fn new(note: Note) -> Self {
        let mut __self = Self {
            note: note,
            editing: false,
            edit_title: "".to_string(),
            edit_body: "".to_string(),
            tag_input: "".to_string(),
            show_tag_input: false,
            search: "".to_string(),
            store: NotesStore::new(),
        };
        __self.on(EditorPanelMsg::Init);
        __self
    }
}

impl Component for EditorPanel {
    type Msg = EditorPanelMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            EditorPanelMsg::Cancel => {
                self.editing = false
            }
            EditorPanelMsg::Edit => {
                self.edit_title = self.note["title"].as_str().unwrap_or_default().to_string().to_string();
                self.edit_body = self.note["body"].as_str().unwrap_or_default().to_string().to_string();
                self.tag_input = "".to_string();
                self.editing = true
            }
            EditorPanelMsg::EditBody(md) => {
                self.edit_body = md.to_string()
            }
            EditorPanelMsg::AddTag => {
                if self.tag_input != "".to_string() { { let mut __a = self.note["tags"].as_array().cloned().unwrap_or_default(); __a.push(serde_json::json!(self.tag_input.clone())); self.note["tags"] = serde_json::Value::Array(__a); }; update_tags(self.note["id"].as_i64().unwrap_or(0) as i32, self.note["tags"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>()).unwrap_or_default()); self.tag_input = "".to_string(); self.show_tag_input = false; () }
            }
            EditorPanelMsg::EditTitle(t) => {
                self.edit_title = t.to_string()
            }
            EditorPanelMsg::EditTagInput => {
                
            }
            EditorPanelMsg::TogglePin => {
                ()
            }
            EditorPanelMsg::Save => {
                self.note["title"] = serde_json::json!(self.edit_title);
                self.note["body"] = serde_json::json!(self.edit_body);
                update_note(self.note["id"].as_i64().unwrap_or(0) as i32, self.edit_title.clone(), self.edit_body.clone());
                self.editing = false
            }
            EditorPanelMsg::ShowTagInput => {
                self.show_tag_input = true
            }
            EditorPanelMsg::Delete => {
                ()
            }
            EditorPanelMsg::RemoveTag(t) => {
                let mut new_tags = vec![];
                for tg in self.note["tags"].as_array().into_iter().flatten() { if tg.as_str().unwrap_or_default() != t.as_str() { new_tags.push(tg) } };
                self.note["tags"] = serde_json::json!(new_tags);
                update_tags(self.note["id"].as_i64().unwrap_or(0) as i32, self.note["tags"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<String>>()).unwrap_or_default());
                ()
            }
            EditorPanelMsg::Init => {
                if self.note["title"].as_str().unwrap_or_default().to_string() == "".to_string() { self.edit_title = "".to_string(); self.edit_body = "".to_string(); self.editing = true }
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("flex-1 flex-col bg-card rounded-xl shadow-sm overflow-hidden min-h-0").child(View::col().style("px-8 pt-8 pb-2 flex-1 overflow-y-auto").child(if self.editing == false { View::row().style("group items-center gap-2").child(View::text_styled(self.note["title"].as_str().unwrap_or_default().to_string(), "text-xl font-bold text-foreground")).child(if self.note["pinned"].as_bool().unwrap_or(false) { View::button("📌").style("text-base cursor-pointer opacity-100").on_click(|_| EditorPanelMsg::TogglePin).build() } else { View::Empty }).child(if ! self.note["pinned"].as_bool().unwrap_or(false) { View::button("📌").style("text-base cursor-pointer opacity-0 group-hover:opacity-30 hover:opacity-60 transition-opacity").on_click(|_| EditorPanelMsg::TogglePin).build() } else { View::Empty }).build() } else { View::input("Note title...").style("text-xl font-bold text-foreground bg-transparent border-b border-border outline-none w-full focus:border-primary p-1 transition-colors").on_change(EditorPanelMsg::EditTitle("".to_string())).build() }).child(View::text_styled(self.note["time"].as_str().unwrap_or_default().to_string(), "text-xs text-muted-foreground mt-1")).child(View::row().style("group gap-1 mt-2 flex-wrap items-center").child(View::col().children(self.note["tags"].as_array().unwrap_or(&Vec::new()).iter().map(|t| { View::row().style("group/tag items-center rounded-full bg-primary/10 pl-2.5 pr-1").child(View::text_styled(format!("{}", t), "text-xs text-primary font-medium")).child(View::button("×").style("text-xs text-primary/60 hover:text-destructive w-4 h-4 flex items-center justify-center rounded-full hover:bg-destructive/10 opacity-0 group-hover/tag:opacity-100 transition-opacity ml-0.5").on_click(|_| EditorPanelMsg::RemoveTag(t.to_string())).build()).build() }).collect::<Vec<_>>()).build()).child(if self.show_tag_input == false { View::button("+ tag").style("text-xs px-2 py-0.5 text-muted-foreground hover:text-foreground hover:bg-accent rounded-full opacity-0 group-hover:opacity-100 transition-opacity").on_click(|_| EditorPanelMsg::ShowTagInput).build() } else { View::Empty }).child(if self.show_tag_input == true { View::col().child(View::input("tag name...").style("text-xs px-2.5 py-0.5 border border-border rounded-full w-24 focus:border-primary outline-none bg-background").on_change(EditorPanelMsg::EditTagInput).build()).child(View::button("✓").style("text-xs w-5 h-5 flex items-center justify-center bg-primary text-primary-foreground rounded-full hover:bg-primary/90 transition-colors").on_click(|_| EditorPanelMsg::AddTag).build()).build() } else { View::Empty }).build()).child(if self.editing == true { View::col().style("flex-1 min-h-64 mt-4 border border-border rounded-lg overflow-hidden").child(View::text_styled(self.edit_body.clone(), "text-sm text-foreground whitespace-pre-wrap")).build() } else { View::col().style("flex-1 min-h-64 mt-4").child(View::text_styled(self.note["body"].as_str().unwrap_or_default().to_string().clone(), "text-sm text-foreground whitespace-pre-wrap")).build() }).build()).child(View::row().style("p-4 border-t border-border bg-muted/30").child(if self.editing == false { View::button("Edit").style("px-4 py-2 text-sm bg-primary text-primary-foreground rounded-lg shadow-sm hover:bg-primary/90 font-medium transition-colors").on_click(|_| EditorPanelMsg::Edit).build() } else { View::row().style("gap-2").child(View::button("Save").style("px-4 py-2 text-sm bg-primary text-primary-foreground rounded-lg shadow-sm hover:bg-primary/90 font-medium transition-colors").on_click(|_| EditorPanelMsg::Save).build()).child(View::button("Cancel").style("px-4 py-2 text-sm text-muted-foreground hover:bg-accent rounded-lg transition-colors").on_click(|_| EditorPanelMsg::Cancel).build()).build() }).child(View::button("Delete").style("ml-auto px-4 py-2 text-sm text-destructive hover:bg-destructive/10 rounded-lg transition-colors").on_click(|_| EditorPanelMsg::Delete).build()).build()).build()
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("editing".to_string(), auto_lang::ui::auto_val::Value::Bool(self.editing));
        m.insert("edit_title".to_string(), auto_lang::ui::auto_val::Value::str(&self.edit_title));
        m.insert("edit_body".to_string(), auto_lang::ui::auto_val::Value::str(&self.edit_body));
        m.insert("tag_input".to_string(), auto_lang::ui::auto_val::Value::str(&self.tag_input));
        m.insert("show_tag_input".to_string(), auto_lang::ui::auto_val::Value::Bool(self.show_tag_input));
        m
    }
}


// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum NotesStoreMsg {
    Init,
    Refresh,
    SelectNote(i32),
    NewNote,
    NewNoteInFolder(String),
    DeleteNote(i32),
    TogglePin(i32),
    SetSort(String),
    Search(String),
    UpdateTags(i32),
    ToggleDarkMode,
    SetAccent(String),
    SelectFolder(String),
    SelectTag(String),
    MoveNote(i32),
}

#[derive(Debug)]
pub struct NotesStore {
    pub notes: Vec<Note>,
    pub active_id: i32,
    pub sort_mode: String,
    pub search: String,
    pub loading: bool,
    pub dark_mode: bool,
    pub active_folder: String,
    pub active_tag: String,
    pub accent_color: String,
}

impl NotesStore {
    pub fn new() -> Self {
        let mut __self = Self {
            notes: vec![],
            active_id: 0,
            sort_mode: "updated".to_string(),
            search: "".to_string(),
            loading: false,
            dark_mode: false,
            active_folder: "all".to_string(),
            active_tag: "".to_string(),
            accent_color: "indigo".to_string(),
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
            NotesStoreMsg::MoveNote(id) => {
                let mut note = None;
                for mut n in self.notes.iter_mut() { if n["id"].as_i64().unwrap_or(0) as i32 == id { note = Some(n) } };
                if note.is_some() { update_note(id, note.as_ref().and_then(|n| n.get("title")).and_then(|v| v.as_str()).unwrap_or_default().to_string(), note.as_ref().and_then(|n| n.get("body")).and_then(|v| v.as_str()).unwrap_or_default().to_string()) } else {};
                self.notes = list_notes()
            }
            NotesStoreMsg::SetAccent(name) => {
                self.accent_color = name.to_string()
            }
            NotesStoreMsg::ToggleDarkMode => {
                self.dark_mode = !(self.dark_mode)
            }
            NotesStoreMsg::Refresh => {
                self.notes = list_notes()
            }
            NotesStoreMsg::SetSort(mode) => {
                self.sort_mode = mode.to_string()
            }
            NotesStoreMsg::TogglePin(idx) => {
                if idx < self.notes.len() as i32 {self.notes[idx as usize]["pinned"] = serde_json::json!(!(self.notes[idx as usize]["pinned"].as_bool().unwrap_or(false))) }
            }
            NotesStoreMsg::SelectTag(t) => {
                self.active_tag = t.to_string()
            }
            NotesStoreMsg::NewNoteInFolder(folder) => {
                create_note("".to_string(), "".to_string(), folder);
                self.notes = list_notes();
                self.active_id = self.notes.len() as i32 - 1
            }
            NotesStoreMsg::UpdateTags(id) => {
                self.notes = list_notes()
            }
            NotesStoreMsg::Search(q) => {
                self.search = q.to_string()
            }
            NotesStoreMsg::NewNote => {
                create_note("".to_string(), "".to_string(), "".to_string());
                self.notes = list_notes();
                self.active_id = self.notes.len() as i32 - 1
            }
            NotesStoreMsg::DeleteNote(id) => {
                delete_note(id);
                self.notes = list_notes();
                if self.notes.len() as i32 > 0 { self.active_id = 0 }
            }
            NotesStoreMsg::SelectFolder(folder) => {
                self.active_folder = folder.to_string();
                self.active_tag = "".to_string()
            }
            NotesStoreMsg::SelectNote(id) => {
                self.active_id = id
            }
            NotesStoreMsg::Init => {
                self.loading = true;
                self.notes = list_notes();
                self.loading = false
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().build()
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("active_id".to_string(), auto_lang::ui::auto_val::Value::Int(self.active_id));
        m.insert("sort_mode".to_string(), auto_lang::ui::auto_val::Value::str(&self.sort_mode));
        m.insert("search".to_string(), auto_lang::ui::auto_val::Value::str(&self.search));
        m.insert("loading".to_string(), auto_lang::ui::auto_val::Value::Bool(self.loading));
        m.insert("dark_mode".to_string(), auto_lang::ui::auto_val::Value::Bool(self.dark_mode));
        m.insert("active_folder".to_string(), auto_lang::ui::auto_val::Value::str(&self.active_folder));
        m.insert("active_tag".to_string(), auto_lang::ui::auto_val::Value::str(&self.active_tag));
        m.insert("accent_color".to_string(), auto_lang::ui::auto_val::Value::str(&self.accent_color));
        m
    }
}

impl NotesStore {
    pub fn pinned_notes(&self) -> Vec<serde_json::Value> {
        self.notes.iter().filter(|n| n["pinned"].as_bool().unwrap_or(false)).cloned().collect::<Vec<_>>()
    }

    pub fn all_tags(&self) -> Vec<serde_json::Value> {
        vec![]
    }

}


// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum NavTreeMsg {
    SelectAll,
    SelectPinned,
    SelectRecent,
    SelectTag(String),
    SelectNote(i32),
    NewNote,
    NewNoteInFolder(String),
    ToggleDarkMode,
    SetAccent(String),
}

#[derive(Debug)]
pub struct NavTree {
    pub active_folder: String,
    pub active_id: i32,
    pub active_tag: String,
    pub search: String,
    pub store: NotesStore,
}

impl NavTree {
    pub fn new(active_folder: String, active_id: i32, active_tag: String, search: String) -> Self {
        Self {
            active_folder: active_folder,
            active_id: active_id,
            active_tag: active_tag,
            search: search,
            store: NotesStore::new(),
        }
    }
}

impl Component for NavTree {
    type Msg = NavTreeMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            NavTreeMsg::SelectNote(i) => {
                self.store.active_id = i
            }
            NavTreeMsg::ToggleDarkMode => {
                self.store.on(NotesStoreMsg::ToggleDarkMode)
            }
            NavTreeMsg::SelectAll => {
                self.store.active_folder = "all".to_string();
                self.store.active_tag = "".to_string()
            }
            NavTreeMsg::SelectTag(t) => {
                self.store.active_tag = t
            }
            NavTreeMsg::SetAccent(name) => {
                self.store.on(NotesStoreMsg::SetAccent(name))
            }
            NavTreeMsg::SelectRecent => {
                self.store.active_folder = "recent".to_string();
                self.store.active_tag = "".to_string()
            }
            NavTreeMsg::SelectPinned => {
                self.store.active_folder = "pinned".to_string();
                self.store.active_tag = "".to_string()
            }
            NavTreeMsg::NewNote => {
                self.store.on(NotesStoreMsg::NewNote)
            }
            NavTreeMsg::NewNoteInFolder(f) => {
                self.store.on(NotesStoreMsg::NewNoteInFolder(f))
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("w-80 bg-card rounded-xl shadow-sm flex flex-col h-full overflow-hidden flex-shrink-0").child(View::row().style("gap-1 m-3 p-1 bg-muted rounded-lg").child(View::button("All").style(if self.active_folder == "all".to_string() { "flex-1 px-2 py-1 text-xs font-medium rounded-md bg-card text-card-foreground shadow-sm".to_string() } else { "flex-1 px-2 py-1 text-xs rounded-md text-muted-foreground hover:text-foreground transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectAll).build()).child(View::button("Pinned").style(if self.active_folder == "pinned".to_string() { "flex-1 px-2 py-1 text-xs font-medium rounded-md bg-card text-card-foreground shadow-sm".to_string() } else { "flex-1 px-2 py-1 text-xs rounded-md text-muted-foreground hover:text-foreground transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectPinned).build()).child(View::button("Recent").style(if self.active_folder == "recent".to_string() { "flex-1 px-2 py-1 text-xs font-medium rounded-md bg-card text-card-foreground shadow-sm".to_string() } else { "flex-1 px-2 py-1 text-xs rounded-md text-muted-foreground hover:text-foreground transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectRecent).build()).build()).child(View::row().style("mx-3 mb-2 items-center gap-2 px-3 py-1.5 bg-muted rounded-lg").child(View::text_styled("🔍".to_string(), "text-sm opacity-50")).child(View::input("Search notes...").style("flex-1 bg-transparent text-sm text-foreground outline-none placeholder:text-muted-foreground").build()).build()).child(View::row().style("flex-wrap gap-1 px-3 mb-2").child(View::col().children(self.store.all_tags().iter().map(|t| { View::button(format!("{}", t)).style(if self.store.active_tag == t.as_str().unwrap_or_default() { "px-2 py-0.5 text-xs rounded-full bg-primary/10 text-primary font-medium".to_string() } else { "px-2 py-0.5 text-xs rounded-full bg-muted text-muted-foreground hover:bg-accent transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectTag(t.to_string())).build() }).collect::<Vec<_>>()).build()).build()).child(View::col().style("flex-1 overflow-y-auto px-2 gap-0.5").child(if self.active_folder == "all" { View::col().child(View::row().style("px-2 py-1 items-center").child(View::text_styled("📁 Notes".to_string(), "text-xs font-bold text-muted-foreground uppercase flex-1 tracking-wide")).child(View::button("+").style("text-xs text-muted-foreground hover:text-primary w-5 h-5 flex items-center justify-center rounded hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::NewNote).build()).build()).child(View::col().children(self.store.notes.iter().enumerate().map(|(i, note)| { let i = i as i32; if note["folder"].as_str().unwrap_or_default().to_string() == "" { if self.store.active_tag == "" { View::button(format!("{}\n{}", note["title"].as_str().unwrap_or_default().to_string(), note["time"].as_str().unwrap_or_default().to_string())).style(if i == self.store.active_id { "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { if note["tags"].as_str().unwrap_or_default().to_string().contains(self.store.active_tag.as_str()) { View::button(format!("{}\n{}", note["title"].as_str().unwrap_or_default().to_string(), note["time"].as_str().unwrap_or_default().to_string())).style(if i == self.store.active_id { "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { View::Empty } } } else { View::Empty } }).collect::<Vec<_>>()).build()).child(View::row().style("px-2 py-1 items-center mt-2").child(View::text_styled("📁 Work".to_string(), "text-xs font-bold text-muted-foreground uppercase flex-1 tracking-wide")).child(View::button("+").style("text-xs text-muted-foreground hover:text-primary w-5 h-5 flex items-center justify-center rounded hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::NewNoteInFolder("work".to_string())).build()).build()).child(View::col().children(self.store.notes.iter().enumerate().map(|(i, note)| { let i = i as i32; if note["folder"].as_str().unwrap_or_default().to_string() == "work" { if self.store.active_tag == "" { View::button(format!("{}\n{}", note["title"].as_str().unwrap_or_default().to_string(), note["time"].as_str().unwrap_or_default().to_string())).style(if i == self.store.active_id { "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { if note["tags"].as_str().unwrap_or_default().to_string().contains(self.store.active_tag.as_str()) { View::button(format!("{}\n{}", note["title"].as_str().unwrap_or_default().to_string(), note["time"].as_str().unwrap_or_default().to_string())).style(if i == self.store.active_id { "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { View::Empty } } } else { View::Empty } }).collect::<Vec<_>>()).build()).child(View::row().style("px-2 py-1 items-center mt-2").child(View::text_styled("📁 Personal".to_string(), "text-xs font-bold text-muted-foreground uppercase flex-1 tracking-wide")).child(View::button("+").style("text-xs text-muted-foreground hover:text-primary w-5 h-5 flex items-center justify-center rounded hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::NewNoteInFolder("personal".to_string())).build()).build()).child(View::col().children(self.store.notes.iter().enumerate().map(|(i, note)| { let i = i as i32; if note["folder"].as_str().unwrap_or_default().to_string() == "personal" { if self.store.active_tag == "" { View::button(format!("{}\n{}", note["title"].as_str().unwrap_or_default().to_string(), note["time"].as_str().unwrap_or_default().to_string())).style(if i == self.store.active_id { "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { if note["tags"].as_str().unwrap_or_default().to_string().contains(self.store.active_tag.as_str()) { View::button(format!("{}\n{}", note["title"].as_str().unwrap_or_default().to_string(), note["time"].as_str().unwrap_or_default().to_string())).style(if i == self.store.active_id { "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { View::Empty } } } else { View::Empty } }).collect::<Vec<_>>()).build()).build() } else { if self.active_folder == "pinned" { View::col().children(self.store.notes.iter().enumerate().map(|(i, note)| { let i = i as i32; if note["pinned"].as_bool().unwrap_or(false) { if self.store.active_tag == "" { View::button(format!("{}\n{}", note["title"].as_str().unwrap_or_default().to_string(), note["time"].as_str().unwrap_or_default().to_string())).style(if i == self.store.active_id { "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { if note["tags"].as_str().unwrap_or_default().to_string().contains(self.store.active_tag.as_str()) { View::button(format!("{}\n{}", note["title"].as_str().unwrap_or_default().to_string(), note["time"].as_str().unwrap_or_default().to_string())).style(if i == self.store.active_id { "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { View::Empty } } } else { View::Empty } }).collect::<Vec<_>>()).build() } else { if self.active_folder == "recent" { View::col().children(self.store.notes.iter().enumerate().map(|(i, note)| { let i = i as i32; if self.store.active_tag == "" { View::button(format!("{}\n{}", note["title"].as_str().unwrap_or_default().to_string(), note["time"].as_str().unwrap_or_default().to_string())).style(if i == self.store.active_id { "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { if note["tags"].as_str().unwrap_or_default().to_string().contains(self.store.active_tag.as_str()) { View::button(format!("{}\n{}", note["title"].as_str().unwrap_or_default().to_string(), note["time"].as_str().unwrap_or_default().to_string())).style(if i == self.store.active_id { "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { View::Empty } } }).collect::<Vec<_>>()).build() } else { View::Empty } } }).build()).child(View::col().style("border-t border-border").build()).child(View::row().style("items-center gap-2 m-3 mb-1").child(View::text_styled("Theme".to_string(), "text-xs text-muted-foreground font-medium mr-1")).child(View::button("").style(if self.store.accent_color == "indigo".to_string() { "w-5 h-5 rounded-full bg-indigo-500 ring-2 ring-offset-2 ring-offset-card ring-primary".to_string() } else { "w-5 h-5 rounded-full bg-indigo-500 hover:scale-110 transition-transform".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("indigo".to_string())).build()).child(View::button("").style(if self.store.accent_color == "coral".to_string() { "w-5 h-5 rounded-full bg-rose-500 ring-2 ring-offset-2 ring-offset-card ring-primary".to_string() } else { "w-5 h-5 rounded-full bg-rose-500 hover:scale-110 transition-transform".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("coral".to_string())).build()).child(View::button("").style(if self.store.accent_color == "ocean".to_string() { "w-5 h-5 rounded-full bg-blue-500 ring-2 ring-offset-2 ring-offset-card ring-primary".to_string() } else { "w-5 h-5 rounded-full bg-blue-500 hover:scale-110 transition-transform".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("ocean".to_string())).build()).child(View::button("").style(if self.store.accent_color == "sage".to_string() { "w-5 h-5 rounded-full bg-emerald-500 ring-2 ring-offset-2 ring-offset-card ring-primary".to_string() } else { "w-5 h-5 rounded-full bg-emerald-500 hover:scale-110 transition-transform".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("sage".to_string())).build()).child(View::button("").style(if self.store.accent_color == "amber".to_string() { "w-5 h-5 rounded-full bg-amber-500 ring-2 ring-offset-2 ring-offset-card ring-primary".to_string() } else { "w-5 h-5 rounded-full bg-amber-500 hover:scale-110 transition-transform".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("amber".to_string())).build()).build()).child(if self.store.dark_mode { View::button("☀ Light").style("mx-3 mb-3 px-3 py-1.5 text-xs rounded-lg text-muted-foreground hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::ToggleDarkMode).build() } else { View::button("🌙 Dark").style("mx-3 mb-3 px-3 py-1.5 text-xs rounded-lg text-muted-foreground hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::ToggleDarkMode).build() }).build()
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("active_folder".to_string(), auto_lang::ui::auto_val::Value::str(&self.active_folder));
        m.insert("active_id".to_string(), auto_lang::ui::auto_val::Value::Int(self.active_id));
        m.insert("active_tag".to_string(), auto_lang::ui::auto_val::Value::str(&self.active_tag));
        m.insert("search".to_string(), auto_lang::ui::auto_val::Value::str(&self.search));
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

fn toggle_pin() {}

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
        println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app_devtools::<App>();
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

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
            AppMsg::SelectPinned => {
                self.store.active_folder = "pinned".to_string();
                self.store.active_tag = "".to_string()
            }
            AppMsg::SelectNote(i) => {
                self.store.active_id = i
            }
            AppMsg::DeleteActive => {
                delete_note(self.store.notes[self.store.active_id as usize]["id"].as_i64().unwrap_or(0) as i32);
                self.store.notes = list_notes();
                if self.store.notes.len() as i32 > 0 { self.store.active_id = 0 }
            }
            AppMsg::SelectRecent => {
                self.store.active_folder = "recent".to_string();
                self.store.active_tag = "".to_string()
            }
            AppMsg::NewNoteInFolder(f) => {
                self.store.on(NotesStoreMsg::NewNoteInFolder(f))
            }
            AppMsg::TagsChanged => {
                self.store.notes = list_notes()
            }
            AppMsg::SelectAll => {
                self.store.active_folder = "all".to_string();
                self.store.active_tag = "".to_string()
            }
            AppMsg::ToggleDarkMode => {
                self.store.on(NotesStoreMsg::ToggleDarkMode)
            }
            AppMsg::TogglePin(i) => {
                self.store.on(NotesStoreMsg::TogglePin(self.store.active_id))
            }
            AppMsg::NewNote => {
                self.store.on(NotesStoreMsg::NewNote)
            }
            AppMsg::SearchChanged => {
                self.search = self.search.clone()
            }
            AppMsg::ClearTag => {
                self.store.active_tag = "".to_string()
            }
            AppMsg::SelectTag(t) => {
                self.store.active_tag = t
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
                let mut __child = EditorPanel::new(self.store.active_folder.clone(), self.store.active_id.clone(), self.store.active_tag.clone(), self.search.clone());
                __child.search = self.search.clone();
                __child.on(inner);
                self.search = __child.search;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("w-full h-screen flex-col bg-muted/30 p-3 gap-3").child(View::row().style("items-center justify-between px-5 py-3 bg-card rounded-xl shadow-sm").child(View::row().style("items-center gap-2").child(View::text_styled("📝".to_string(), "text-xl")).child(View::text_styled("Notes".to_string(), "text-3xl font-bold text-lg font-bold text-foreground")).build()).child(View::button("New").style("px-4 py-2 bg-primary text-primary-foreground rounded-full text-sm font-medium shadow-sm hover:bg-primary/90 transition-colors").on_click(|_| AppMsg::NewNote).build()).build()).child(View::row().style("flex-1 gap-3 min-h-0").child(NavTree::new(self.store.active_folder.clone(), self.store.active_id.clone(), self.store.active_tag.clone(), self.search.clone()).view().map_msg(|m| AppMsg::NavTree(m))).child(View::col().style("flex-1 min-h-0").child(if self.store.notes.len ( ) > 0 { EditorPanel::new(self.store.notes[self.store.active_id as usize].clone()).view().map_msg(|m| AppMsg::EditorPanel(m)) } else { View::col().style("flex-1 items-center justify-center bg-card rounded-xl shadow-sm").child(View::text_styled("📝".to_string(), "text-6xl mb-4 opacity-40")).child(View::text_styled("No notes yet".to_string(), "text-lg text-muted-foreground")).child(View::button("Create your first note").style("mt-4 px-6 py-2 bg-primary text-primary-foreground rounded-full text-sm font-medium shadow-sm hover:bg-primary/90 transition-colors").on_click(|_| AppMsg::NewNote).build()).build() }).build()).build()).build()
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
            EditorPanelMsg::AddTag => {
                if self.tag_input != "".to_string() { self.note["tags"].as_str().unwrap_or_default().to_string().push(self.tag_input.clone()); update_tags(self.note["id"].as_i64().unwrap_or(0) as i32, self.note["tags"].as_str().unwrap_or_default().to_string()); self.tag_input = "".to_string(); self.show_tag_input = false; () }
            }
            EditorPanelMsg::TogglePin => {
                ()
            }
            EditorPanelMsg::EditBody(md) => {
                self.edit_body = md.to_string()
            }
            EditorPanelMsg::EditTitle(t) => {
                self.edit_title = t.to_string()
            }
            EditorPanelMsg::Cancel => {
                self.editing = false
            }
            EditorPanelMsg::EditTagInput => {
                
            }
            EditorPanelMsg::ShowTagInput => {
                self.show_tag_input = true
            }
            EditorPanelMsg::RemoveTag(t) => {
                let mut new_tags = vec![];
                for tg in self.note["tags"].as_str().unwrap_or_default().to_string().iter() { if tg != t { new_tags.push(tg) } };
                self.note["tags"] = serde_json::json!(new_tags);
                update_tags(self.note["id"].as_i64().unwrap_or(0) as i32, self.note["tags"].as_str().unwrap_or_default().to_string());
                ()
            }
            EditorPanelMsg::Delete => {
                ()
            }
            EditorPanelMsg::Edit => {
                self.edit_title = self.note["title"].as_str().unwrap_or_default().to_string().to_string();
                self.edit_body = self.note["body"].as_str().unwrap_or_default().to_string().to_string();
                self.tag_input = "".to_string();
                self.editing = true
            }
            EditorPanelMsg::Save => {
                self.note["title"] = serde_json::json!(self.edit_title);
                self.note["body"] = serde_json::json!(self.edit_body);
                update_note(self.note["id"].as_i64().unwrap_or(0) as i32, self.edit_title.clone(), self.edit_body.clone());
                self.editing = false
            }
            EditorPanelMsg::Init => {
                if self.note["title"].as_str().unwrap_or_default().to_string() == "".to_string() { self.edit_title = "".to_string(); self.edit_body = "".to_string(); self.editing = true }
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("flex-1 flex-col bg-card rounded-xl shadow-sm overflow-hidden min-h-0").child(View::col().style("px-8 pt-8 pb-2 flex-1 overflow-y-auto").child(if self.editing == false { View::row().style("group items-center gap-2").child(View::text_styled(self.note["title"].as_str().unwrap_or_default().to_string(), "text-xl font-bold text-foreground")).child(if self.note["pinned"].as_bool().unwrap_or(false) { View::button("📌").style("text-base cursor-pointer opacity-100").on_click(|_| EditorPanelMsg::TogglePin).build() } else { View::Empty }).child(if ! self.note["pinned"].as_bool().unwrap_or(false) { View::button("📌").style("text-base cursor-pointer opacity-0 group-hover:opacity-30 hover:opacity-60 transition-opacity").on_click(|_| EditorPanelMsg::TogglePin).build() } else { View::Empty }).build() } else { View::input("Note title...").style("text-xl font-bold text-foreground bg-transparent border-b border-border outline-none w-full focus:border-primary p-1 transition-colors").on_change(EditorPanelMsg::EditTitle("".to_string())).build() }).child(View::text_styled(self.note["time"].as_str().unwrap_or_default().to_string(), "text-xs text-muted-foreground mt-1")).child(View::row().style("group gap-1 mt-2 flex-wrap items-center").child(View::col().children(self.note["tags"].iter().map(|t| { View::row().style("group/tag items-center rounded-full bg-primary/10 pl-2.5 pr-1").child(View::text_styled(format!("{}", t), "text-xs text-primary font-medium")).child(View::button("×").style("text-xs text-primary/60 hover:text-destructive w-4 h-4 flex items-center justify-center rounded-full hover:bg-destructive/10 opacity-0 group-hover/tag:opacity-100 transition-opacity ml-0.5").on_click(|_| EditorPanelMsg::RemoveTag(t)).build()).build() }).collect::<Vec<_>>()).build()).child(if self.show_tag_input == false { View::button("+ tag").style("text-xs px-2 py-0.5 text-muted-foreground hover:text-foreground hover:bg-accent rounded-full opacity-0 group-hover:opacity-100 transition-opacity").on_click(|_| EditorPanelMsg::ShowTagInput).build() } else { View::Empty }).child(if self.show_tag_input == true { View::row().child(View::input("tag name...").style("text-xs px-2.5 py-0.5 border border-border rounded-full w-24 focus:border-primary outline-none bg-background").on_change(EditorPanelMsg::EditTagInput).build()).child(View::button("✓").style("text-xs w-5 h-5 flex items-center justify-center bg-primary text-primary-foreground rounded-full hover:bg-primary/90 transition-colors").on_click(|_| EditorPanelMsg::AddTag).build()).build() } else { View::Empty }).build()).child(if self.editing == true { View::col().style("flex-1 min-h-64 mt-4 border border-border rounded-lg overflow-hidden").build() } else { View::col().style("flex-1 min-h-64 mt-4").build() }).build()).child(View::row().style("p-4 border-t border-border bg-muted/30").child(if self.editing == false { View::button("Edit").style("px-4 py-2 text-sm bg-primary text-primary-foreground rounded-lg shadow-sm hover:bg-primary/90 font-medium transition-colors").on_click(|_| EditorPanelMsg::Edit).build() } else { View::row().style("gap-2").child(View::button("Save").style("px-4 py-2 text-sm bg-primary text-primary-foreground rounded-lg shadow-sm hover:bg-primary/90 font-medium transition-colors").on_click(|_| EditorPanelMsg::Save).build()).child(View::button("Cancel").style("px-4 py-2 text-sm text-muted-foreground hover:bg-accent rounded-lg transition-colors").on_click(|_| EditorPanelMsg::Cancel).build()).build() }).child(View::button("Delete").style("ml-auto px-4 py-2 text-sm text-destructive hover:bg-destructive/10 rounded-lg transition-colors").on_click(|_| EditorPanelMsg::Delete).build()).build()).build()
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
    pub note: Note,
    pub is_active: bool,
    pub store: NotesStore,
    pub search: String,
}

impl NoteItem {
    pub fn new(note: Note, is_active: bool) -> Self {
        Self {
            note: note,
            is_active: is_active,
            search: "".to_string(),
            store: NotesStore::new(),
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
            NotesStoreMsg::Refresh => {
                self.notes = list_notes()
            }
            NotesStoreMsg::TogglePin(idx) => {
                if idx < self.notes.len() as i32 { self.notes[idx as usize]["pinned"].as_bool().unwrap_or(false) = !(self.notes[idx as usize]["pinned"].as_bool().unwrap_or(false)) }
            }
            NotesStoreMsg::SetAccent(name) => {
                self.accent_color = name.to_string()
            }
            NotesStoreMsg::SetSort(mode) => {
                self.sort_mode = mode.to_string()
            }
            NotesStoreMsg::NewNote => {
                create_note("".to_string(), "".to_string(), "".to_string());
                self.notes = list_notes();
                self.active_id = self.notes.len() as i32 - 1
            }
            NotesStoreMsg::Search(q) => {
                self.search = q.to_string()
            }
            NotesStoreMsg::SelectFolder(folder) => {
                self.active_folder = folder.to_string();
                self.active_tag = "".to_string()
            }
            NotesStoreMsg::SelectTag(t) => {
                self.active_tag = t.to_string()
            }
            NotesStoreMsg::DeleteNote(id) => {
                delete_note(id);
                self.notes = list_notes();
                if self.notes.len() as i32 > 0 { self.active_id = 0 }
            }
            NotesStoreMsg::NewNoteInFolder(folder) => {
                create_note("".to_string(), "".to_string(), folder);
                self.notes = list_notes();
                self.active_id = self.notes.len() as i32 - 1
            }
            NotesStoreMsg::MoveNote(id) => {
                let mut note = None;
                for mut n in self.notes.iter_mut() { if n["id"].as_i64().unwrap_or(0) as i32 == id { note = Some(n) } };
                if note != None { update_note(id, note.title, note.body) };
                self.notes = list_notes()
            }
            NotesStoreMsg::UpdateTags(id) => {
                self.notes = list_notes()
            }
            NotesStoreMsg::SelectNote(id) => {
                self.active_id = id
            }
            NotesStoreMsg::ToggleDarkMode => {
                self.dark_mode = !(self.dark_mode)
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
}

impl NotesStore {
    pub fn pinned_notes(&self) -> Vec<serde_json::Value> {
        self.notes.iter().filter(|n| n["pinned"].as_bool().unwrap_or(false)).collect::<Vec<_>>()
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
            NavTreeMsg::SelectAll => {
                self.store.active_folder = "all".to_string();
                self.store.active_tag = "".to_string()
            }
            NavTreeMsg::NewNoteInFolder(f) => {
                self.store.on(NotesStoreMsg::NewNoteInFolder(f))
            }
            NavTreeMsg::SelectRecent => {
                self.store.active_folder = "recent".to_string();
                self.store.active_tag = "".to_string()
            }
            NavTreeMsg::SelectTag(t) => {
                self.store.active_tag = t
            }
            NavTreeMsg::SelectNote(i) => {
                self.store.active_id = i
            }
            NavTreeMsg::ToggleDarkMode => {
                self.store.on(NotesStoreMsg::ToggleDarkMode)
            }
            NavTreeMsg::SetAccent(name) => {
                self.store.on(NotesStoreMsg::SetAccent(name))
            }
            NavTreeMsg::NewNote => {
                self.store.on(NotesStoreMsg::NewNote)
            }
            NavTreeMsg::SelectPinned => {
                self.store.active_folder = "pinned".to_string();
                self.store.active_tag = "".to_string()
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("w-80 bg-card rounded-xl shadow-sm flex flex-col h-full overflow-hidden flex-shrink-0").child(View::row().style("gap-1 m-3 p-1 bg-muted rounded-lg").child(View::button("All").style(if self.active_folder == "all".to_string() { "flex-1 px-2 py-1 text-xs font-medium rounded-md bg-card text-card-foreground shadow-sm".to_string() } else { "flex-1 px-2 py-1 text-xs rounded-md text-muted-foreground hover:text-foreground transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectAll).build()).child(View::button("Pinned").style(if self.active_folder == "pinned".to_string() { "flex-1 px-2 py-1 text-xs font-medium rounded-md bg-card text-card-foreground shadow-sm".to_string() } else { "flex-1 px-2 py-1 text-xs rounded-md text-muted-foreground hover:text-foreground transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectPinned).build()).child(View::button("Recent").style(if self.active_folder == "recent".to_string() { "flex-1 px-2 py-1 text-xs font-medium rounded-md bg-card text-card-foreground shadow-sm".to_string() } else { "flex-1 px-2 py-1 text-xs rounded-md text-muted-foreground hover:text-foreground transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectRecent).build()).build()).child(View::row().style("mx-3 mb-2 items-center gap-2 px-3 py-1.5 bg-muted rounded-lg").child(View::text_styled("🔍".to_string(), "text-sm opacity-50")).child(View::input("Search notes...").style("flex-1 bg-transparent text-sm text-foreground outline-none placeholder:text-muted-foreground").build()).build()).child(View::row().style("flex-wrap gap-1 px-3 mb-2").child(View::col().children(self.store.all_tags().iter().map(|t| { View::button(format!("{}", self.t)).style(if self.store.active_tag == t { "px-2 py-0.5 text-xs rounded-full bg-primary/10 text-primary font-medium".to_string() } else { "px-2 py-0.5 text-xs rounded-full bg-muted text-muted-foreground hover:bg-accent transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectTag(t)).build() }).collect::<Vec<_>>()).build()).build()).child(View::col().style("flex-1 overflow-y-auto px-2 gap-0.5").child(if self.active_folder == "all" { View::row().child(View::row().style("px-2 py-1 items-center").child(View::text_styled("📁 Notes".to_string(), "text-xs font-bold text-muted-foreground uppercase flex-1 tracking-wide")).child(View::button("+").style("text-xs text-muted-foreground hover:text-primary w-5 h-5 flex items-center justify-center rounded hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::NewNote).build()).build()).child(View::col().children(self.store.notes.iter().enumerate().map(|(i, note)| { let i = i as i32; if note["folder"].as_str().unwrap_or_default().to_string() == "" { if self.store.active_tag == "" { View::button(()).style(if active { "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).child(View::text_styled(note["title"].as_str().unwrap_or_default().to_string(), "block truncate")).child(View::text_styled(note["time"].as_str().unwrap_or_default().to_string(), "block text-xs text-muted-foreground mt-0.5")).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { if note["tags"].as_str().unwrap_or_default().to_string().contains(self.store.active_tag.as_str()) { View::button(()).style(if active { "w-full text-left py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).child(View::text_styled(note["title"].as_str().unwrap_or_default().to_string(), "block truncate")).child(View::text_styled(note["time"].as_str().unwrap_or_default().to_string(), "block text-xs text-muted-foreground mt-0.5")).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { View::Empty } } } else { View::Empty } }).collect::<Vec<_>>()).build()).child(View::row().style("px-2 py-1 items-center mt-2").child(View::text_styled("📁 Work".to_string(), "text-xs font-bold text-muted-foreground uppercase flex-1 tracking-wide")).child(View::button("+").style("text-xs text-muted-foreground hover:text-primary w-5 h-5 flex items-center justify-center rounded hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::NewNoteInFolder("work".to_string())).build()).build()).child(View::col().children(self.store.notes.iter().enumerate().map(|(i, note)| { let i = i as i32; if note["folder"].as_str().unwrap_or_default().to_string() == "work" { if self.store.active_tag == "" { View::button(()).style(if i == self.store.active_id { "w-full text-left px-5 py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left px-5 py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).child(View::text_styled(note["title"].as_str().unwrap_or_default().to_string(), "block truncate")).child(View::text_styled(note["time"].as_str().unwrap_or_default().to_string(), "block text-xs text-muted-foreground mt-0.5")).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { if note["tags"].as_str().unwrap_or_default().to_string().contains(self.store.active_tag.as_str()) { View::button(()).style(if i == self.store.active_id { "w-full text-left px-5 py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left px-5 py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).child(View::text_styled(note["title"].as_str().unwrap_or_default().to_string(), "block truncate")).child(View::text_styled(note["time"].as_str().unwrap_or_default().to_string(), "block text-xs text-muted-foreground mt-0.5")).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { View::Empty } } } else { View::Empty } }).collect::<Vec<_>>()).build()).child(View::row().style("px-2 py-1 items-center mt-2").child(View::text_styled("📁 Personal".to_string(), "text-xs font-bold text-muted-foreground uppercase flex-1 tracking-wide")).child(View::button("+").style("text-xs text-muted-foreground hover:text-primary w-5 h-5 flex items-center justify-center rounded hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::NewNoteInFolder("personal".to_string())).build()).build()).child(View::col().children(self.store.notes.iter().enumerate().map(|(i, note)| { let i = i as i32; if note["folder"].as_str().unwrap_or_default().to_string() == "personal" { if self.store.active_tag == "" { View::button(()).style(if i == self.store.active_id { "w-full text-left px-5 py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left px-5 py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).child(View::text_styled(note["title"].as_str().unwrap_or_default().to_string(), "block truncate")).child(View::text_styled(note["time"].as_str().unwrap_or_default().to_string(), "block text-xs text-muted-foreground mt-0.5")).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { if note["tags"].as_str().unwrap_or_default().to_string().contains(self.store.active_tag.as_str()) { View::button(()).style(if i == self.store.active_id { "w-full text-left px-5 py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left px-5 py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).child(View::text_styled(note["title"].as_str().unwrap_or_default().to_string(), "block truncate")).child(View::text_styled(note["time"].as_str().unwrap_or_default().to_string(), "block text-xs text-muted-foreground mt-0.5")).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { View::Empty } } } else { View::Empty } }).collect::<Vec<_>>()).build()).build() } else { View::Empty }).child(if self.active_folder == "pinned" { View::col().children(self.store.notes.iter().enumerate().map(|(i, note)| { let i = i as i32; if note["pinned"].as_bool().unwrap_or(false) { View::row().child(if self.store.active_tag == "" { View::button(()).style(if i == self.store.active_id { "w-full text-left px-3 py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left px-3 py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).child(View::text_styled(note["title"].as_str().unwrap_or_default().to_string(), "block truncate")).child(View::text_styled(note["time"].as_str().unwrap_or_default().to_string(), "block text-xs text-muted-foreground mt-0.5")).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { View::Empty }).child(if self.store.active_tag != "" { if note["tags"].as_str().unwrap_or_default().to_string().contains(self.store.active_tag.as_str()) { View::button(()).style(if i == self.store.active_id { "w-full text-left px-3 py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left px-3 py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).child(View::text_styled(note["title"].as_str().unwrap_or_default().to_string(), "block truncate")).child(View::text_styled(note["time"].as_str().unwrap_or_default().to_string(), "block text-xs text-muted-foreground mt-0.5")).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { View::Empty } } else { View::Empty }).build() } else { View::Empty } }).collect::<Vec<_>>()).build() } else { View::Empty }).child(if self.active_folder == "recent" { View::col().children(self.store.notes.iter().enumerate().map(|(i, note)| { let i = i as i32; if self.store.active_tag == "" { View::button(()).style(if i == self.store.active_id { "w-full text-left px-3 py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left px-3 py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).child(View::text_styled(note["title"].as_str().unwrap_or_default().to_string(), "block truncate")).child(View::text_styled(note["time"].as_str().unwrap_or_default().to_string(), "block text-xs text-muted-foreground mt-0.5")).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { View::Empty }
if self.store.active_tag != "" { if note["tags"].as_str().unwrap_or_default().to_string().contains(self.store.active_tag.as_str()) { View::button(()).style(if i == self.store.active_id { "w-full text-left px-3 py-2 rounded-lg bg-accent text-accent-foreground".to_string() } else { "w-full text-left px-3 py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors".to_string() }.as_str()).child(View::text_styled(note["title"].as_str().unwrap_or_default().to_string(), "block truncate")).child(View::text_styled(note["time"].as_str().unwrap_or_default().to_string(), "block text-xs text-muted-foreground mt-0.5")).on_click(|_| NavTreeMsg::SelectNote(i)).build() } else { View::Empty } } else { View::Empty } }).collect::<Vec<_>>()).build() } else { View::Empty }).build()).child(View::col().style("border-t border-border").build()).child(View::row().style("items-center gap-2 m-3 mb-1").child(View::text_styled("Theme".to_string(), "text-xs text-muted-foreground font-medium mr-1")).child(View::button("").style(if self.store.accent_color == "indigo".to_string() { "w-5 h-5 rounded-full bg-indigo-500 ring-2 ring-offset-2 ring-offset-card ring-primary".to_string() } else { "w-5 h-5 rounded-full bg-indigo-500 hover:scale-110 transition-transform".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("indigo".to_string())).build()).child(View::button("").style(if self.store.accent_color == "coral".to_string() { "w-5 h-5 rounded-full bg-rose-500 ring-2 ring-offset-2 ring-offset-card ring-primary".to_string() } else { "w-5 h-5 rounded-full bg-rose-500 hover:scale-110 transition-transform".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("coral".to_string())).build()).child(View::button("").style(if self.store.accent_color == "ocean".to_string() { "w-5 h-5 rounded-full bg-blue-500 ring-2 ring-offset-2 ring-offset-card ring-primary".to_string() } else { "w-5 h-5 rounded-full bg-blue-500 hover:scale-110 transition-transform".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("ocean".to_string())).build()).child(View::button("").style(if self.store.accent_color == "sage".to_string() { "w-5 h-5 rounded-full bg-emerald-500 ring-2 ring-offset-2 ring-offset-card ring-primary".to_string() } else { "w-5 h-5 rounded-full bg-emerald-500 hover:scale-110 transition-transform".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("sage".to_string())).build()).child(View::button("").style(if self.store.accent_color == "amber".to_string() { "w-5 h-5 rounded-full bg-amber-500 ring-2 ring-offset-2 ring-offset-card ring-primary".to_string() } else { "w-5 h-5 rounded-full bg-amber-500 hover:scale-110 transition-transform".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("amber".to_string())).build()).build()).child(if self.store.dark_mode { View::button("☀ Light").style("mx-3 mb-3 px-3 py-1.5 text-xs rounded-lg text-muted-foreground hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::ToggleDarkMode).build() } else { View::button("🌙 Dark").style("mx-3 mb-3 px-3 py-1.5 text-xs rounded-lg text-muted-foreground hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::ToggleDarkMode).build() }).build()
    }
}


pub type Note = serde_json::Value;

pub type Folder = serde_json::Value;



// API functions (auto-generated, in-process merged mode — no HTTP)

use std::sync::{LazyLock, Mutex};
use serde_json::Value;

static API_DATA: LazyLock<Mutex<Vec<Value>>> = LazyLock::new(|| {
    Mutex::new(vec![serde_json::json!({"id": 0, "title": "Welcome", "body": "This is your notes app. Click on any note to view it.", "time": "Just now", "pinned": false, "tags": "Sample", "folder": "Sample"}), serde_json::json!({"id": 1, "title": "Shopping List", "body": "Milk, Eggs, Bread, Cheese", "time": "2 hours ago", "pinned": false, "tags": "Sample", "folder": "Sample"}), serde_json::json!({"id": 2, "title": "Meeting Notes", "body": "Q3 roadmap discussion with the team", "time": "Yesterday", "pinned": false, "tags": "Sample", "folder": "Sample"})])
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

fn update_note(id: i32, title: String, body: String) -> Option<Value> {
    let mut data = API_DATA.lock().unwrap();
    if let Some(item) = data.iter_mut().find(|n| n["id"].as_i64() == Some(id as i64)) {
        item["title"] = serde_json::Value::from(title.clone()); item["body"] = serde_json::Value::from(body.clone());
        return Some(item.clone());
    }
    None
}

fn delete_note(id: i32) {
    let mut data = API_DATA.lock().unwrap();
    data.retain(|n| n["id"].as_i64() != Some(id as i64));
}

fn toggle_pin() {}

fn update_tags(id: i32, tags: String) -> Option<Value> {
    let mut data = API_DATA.lock().unwrap();
    if let Some(item) = data.iter_mut().find(|n| n["id"].as_i64() == Some(id as i64)) {
        item["tags"] = serde_json::Value::from(tags.clone());
        return Some(item.clone());
    }
    None
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

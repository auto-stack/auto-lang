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
                self.store.on(NotesStoreMsg::NewNote)
            }
            AppMsg::SetAccent(name) => {
                self.store.on(NotesStoreMsg::SetAccent(name));
                self.accent_color = self.store.accent_color.to_string()
            }
            AppMsg::SetMode(m) => {
                self.store.on(NotesStoreMsg::SetMode(m));
                self.dark_mode = self.store.dark_mode
            }
            AppMsg::ToggleDarkMode => {
                self.store.on(NotesStoreMsg::ToggleDarkMode);
                self.dark_mode = self.store.dark_mode
            }
            AppMsg::ToggleSettings => {
                self.store.on(NotesStoreMsg::ToggleSettings)
            }
            AppMsg::Init => {
                self.store.on(NotesStoreMsg::Init)
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
        View::col().style("w-full h-screen flex-col bg-background text-foreground overflow-hidden").child(View::row().style("h-14 shrink-0 items-center justify-between gap-3 px-4 border-b border-border").child(View::row().style("w-auto items-center gap-2").child(View::icon().style("text-primary").build()).child(View::text_styled("Notes".to_string(), "text-sm font-semibold tracking-tight")).build()).child(View::row().style("w-auto items-center gap-1").child(View::button("").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-8 w-8 border-0 flex items-center justify-center rounded-md bg-transparent text-muted-foreground hover:text-foreground hover:bg-accent transition-colors").on_click(|_| AppMsg::ToggleDarkMode).build()).child(View::button(format!("{}", "New note".to_string())).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-9 w-auto border-0 flex items-center gap-1.5 px-3 rounded-md bg-primary text-primary-foreground text-sm font-medium shadow-sm hover:bg-primary/90 transition-colors").on_click(|_| AppMsg::NewNote).build()).build()).build()).child(View::row().style("flex-1 min-h-0").child({ let mut __c = self.nav_tree.clone(); __c.store = self.store.clone(); __c.view().map_msg(|m| AppMsg::NavTree(m)) }).child(View::col().style("flex-1 min-h-0").child(if self.store.notes.len ( ) > 0 { { let mut __c = self.editor_panel.clone(); __c.store = self.store.clone(); __c.view().map_msg(|m| AppMsg::EditorPanel(m)) } } else { View::col().style("flex-1 items-center justify-center gap-3").child(View::icon().style("text-muted-foreground/40").build()).child(View::text_styled("No notes yet".to_string(), "text-sm text-muted-foreground")).child(View::button(format!("{}", "Create your first note".to_string())).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-9 w-auto border-0 flex items-center gap-1.5 px-3 rounded-md bg-primary text-primary-foreground text-sm font-medium shadow-sm hover:bg-primary/90 transition-colors").on_click(|_| AppMsg::NewNote).build()).build() }).build()).build()).build()
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
pub enum EditorPanelMsg {
    EditTitle(String),
    EditBody(String),
    ShowTagInput,
    TagInputChanged(String),
    AddTag,
    RemoveTag(String),
    SaveDraft,
    TogglePinActive,
    DeleteArmed,
    DeleteCancelled,
    DeleteConfirmed,
}

#[derive(Clone, Debug)]
pub struct EditorPanel {
    pub tag_input: String,
    pub show_tag_input: bool,
    pub store: NotesStore,
    pub dark_mode: bool,
    pub accent_color: String,
}

impl EditorPanel {
    pub fn new() -> Self {
        Self {
            tag_input: "".to_string(),
            show_tag_input: false,
            dark_mode: false,
            accent_color: "".to_string(),
            store: NotesStore::new(),
        }
    }
}
impl Default for EditorPanel {
    fn default() -> Self { Self::new() }
}

impl Component for EditorPanel {
    type Msg = EditorPanelMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            EditorPanelMsg::AddTag => {
                if self.tag_input != "".to_string() { self.store.on(NotesStoreMsg::TagAdded(self.tag_input.clone())); self.tag_input = "".to_string(); self.show_tag_input = false }
            }
            EditorPanelMsg::DeleteArmed => {
                self.store.on(NotesStoreMsg::DeleteArmed)
            }
            EditorPanelMsg::DeleteCancelled => {
                self.store.on(NotesStoreMsg::DeleteCancelled)
            }
            EditorPanelMsg::DeleteConfirmed => {
                self.store.on(NotesStoreMsg::DeleteConfirmed)
            }
            EditorPanelMsg::EditBody(v) => {
                self.store.on(NotesStoreMsg::EditBody(v))
            }
            EditorPanelMsg::EditTitle(v) => {
                self.store.on(NotesStoreMsg::EditTitle(v))
            }
            EditorPanelMsg::RemoveTag(t) => {
                self.store.on(NotesStoreMsg::TagRemoved(t))
            }
            EditorPanelMsg::SaveDraft => {
                self.store.on(NotesStoreMsg::SaveDraft)
            }
            EditorPanelMsg::ShowTagInput => {
                self.show_tag_input = true
            }
            EditorPanelMsg::TagInputChanged(v) => {
                let _text = auto_lang::ui::iced::last_input_text();
                self.tag_input = _text;
            }
            EditorPanelMsg::TogglePinActive => {
                self.store.on(NotesStoreMsg::TogglePinActive)
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("h-full flex-1 flex-col min-h-0 bg-background").child(View::col().style("shrink-0 gap-2 px-8 pt-6 pb-3 border-b border-border").child(View::row().style("items-center gap-2").child(View::input("Untitled").style("flex-1 text-2xl font-semibold tracking-tight bg-transparent border-0 outline-none text-foreground").on_change(EditorPanelMsg::EditTitle("".to_string())).on_submit(EditorPanelMsg::SaveDraft).build()).child(if self.store.dirty { View::col().child(View::text_styled("Unsaved changes".to_string(), "text-xs text-muted-foreground")).child(View::button(format!("{}", "Save".to_string())).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-8 w-auto border-0 flex items-center gap-1.5 px-3 rounded-md bg-primary text-primary-foreground text-xs font-medium shadow-sm hover:bg-primary/90 transition-colors").on_click(|_| EditorPanelMsg::SaveDraft).build()).build() } else { View::text_styled("Saved".to_string(), "text-xs text-muted-foreground/70") }).child(if self.store.draft_pinned { View::button(format!("{}", "Unpin".to_string())).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-8 w-auto border-0 flex items-center gap-1.5 px-2.5 rounded-full bg-primary/10 text-xs text-primary font-medium hover:bg-primary/20 transition-colors").on_click(|_| EditorPanelMsg::TogglePinActive).build() } else { View::button(format!("{}", "Pin note".to_string())).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-8 w-auto border-0 flex items-center gap-1.5 px-2.5 rounded-md bg-transparent text-xs text-muted-foreground hover:text-foreground hover:bg-accent transition-colors").on_click(|_| EditorPanelMsg::TogglePinActive).build() }).child(if self.store.confirm_delete { View::col().child(View::button(format!("{}", "Cancel".to_string())).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-8 w-auto border-0 flex items-center gap-1.5 px-2.5 rounded-md bg-transparent text-xs text-muted-foreground hover:text-foreground hover:bg-accent transition-colors").on_click(|_| EditorPanelMsg::DeleteCancelled).build()).child(View::button(format!("{}", "Delete".to_string())).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-8 w-auto border-0 flex items-center gap-1.5 px-2.5 rounded-md bg-transparent text-xs text-destructive hover:bg-destructive/10 transition-colors").on_click(|_| EditorPanelMsg::DeleteConfirmed).build()).build() } else { View::button(format!("{}", "Delete".to_string())).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-8 w-auto border-0 flex items-center gap-1.5 px-2.5 rounded-md bg-transparent text-xs text-destructive hover:bg-destructive/10 transition-colors").on_click(|_| EditorPanelMsg::DeleteArmed).build() }).build()).child(View::row().style("items-center gap-1.5").child(View::icon().style("text-muted-foreground").build()).child(View::text_styled(format!("{}", self.store.draft_time), "text-xs text-muted-foreground")).child(if self.store.draft_folder != "" { View::text_styled(format!("{}", self.store.draft_folder), "text-xs text-muted-foreground") } else { View::Empty }).build()).child(View::row().style("items-center gap-1.5").child(View::col().children(self.store.draft_tags.iter().map(|t| { View::row().style("h-6 w-auto border-0 flex items-center gap-1 pl-2 pr-1 rounded-full bg-primary/10").child(View::text_styled(format!("{}", t), "text-xs text-primary font-medium")).child(View::button("").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-4 w-4 border-0 flex items-center justify-center rounded-full bg-transparent hover:bg-destructive/10").on_click(|_| EditorPanelMsg::RemoveTag(t.to_string())).build()).build() }).collect::<Vec<_>>()).build()).child(if self.show_tag_input { View::col().child(View::input("tag name").style("h-6 w-28 rounded-full border border-border bg-background px-2 text-xs outline-none text-foreground").on_change(EditorPanelMsg::TagInputChanged("".to_string())).on_submit(EditorPanelMsg::AddTag).build()).child(View::button("").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-6 w-6 border-0 flex items-center justify-center rounded-full bg-primary text-primary-foreground hover:bg-primary/90 transition-colors").on_click(|_| EditorPanelMsg::AddTag).build()).build() } else { View::button(format!("{}", "Tag".to_string())).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-8 w-auto border-0 flex items-center gap-1.5 px-2.5 rounded-md bg-transparent text-xs text-muted-foreground hover:text-foreground hover:bg-accent transition-colors").on_click(|_| EditorPanelMsg::ShowTagInput).build() }).build()).build()).child(View::col().style("flex-1 min-h-0 px-8 py-4").child(View::textarea("Start writing...").style("w-full h-full max-w-2xl text-base leading-7 bg-transparent border-0 outline-none resize-none text-foreground").on_change(EditorPanelMsg::EditBody("".to_string())).build()).build()).build()
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        m.insert("tag_input".to_string(), auto_lang::ui::auto_val::Value::str(&self.tag_input));
        m.insert("show_tag_input".to_string(), auto_lang::ui::auto_val::Value::Bool(self.show_tag_input));
        for (k, v) in self.store.state_snapshot() { m.insert(format!("{}.{}", "store", k), v); }
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
                for mut n in self.notes.iter_mut() { ok = false; if self.active_folder == "all".to_string() { ok = true } else if self.active_folder == "pinned".to_string() { if n["pinned"].as_bool().unwrap_or(false) { ok = true } } else if n["folder"].as_str().unwrap_or_default().to_string() == self.active_folder { ok = true }; if ok { if self.active_tag != "".to_string() { ok = false; for t in n["tags"].as_array().into_iter().flatten() { if t == self.active_tag { ok = true } } } }; if ok { if n["pinned"].as_bool().unwrap_or(false) { pinned.push(idx) } else { plain.push(idx) } }; idx = idx + 1 };
                self.visible_pinned = pinned;
                self.visible_notes = plain
            }
            NotesStoreMsg::ClearTag => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes() } else { self.notes = search_notes(self.search.clone()) }; self.dirty = false };
                self.active_tag = "".to_string();
                self.ApplyFilter();
                self.SelectFirstVisible()
            }
            NotesStoreMsg::DeleteArmed => {
                self.confirm_delete = true
            }
            NotesStoreMsg::DeleteCancelled => {
                self.confirm_delete = false
            }
            NotesStoreMsg::DeleteConfirmed => {
                if self.notes.len() as i32 > 0 { delete_note((self.notes[(self.active_id) as usize]["id"].as_i64().unwrap_or(0) as i32)); if self.search == "".to_string() { self.notes = list_notes() } else { self.notes = search_notes(self.search.clone()) }; self.active_id = 0; self.LoadDraft(); self.ApplyFilter() };
                self.confirm_delete = false
            }
            NotesStoreMsg::EditBody(v) => {
                self.draft_body = v.to_string();
                self.dirty = true
            }
            NotesStoreMsg::EditTitle(v) => {
                self.draft_title = v.to_string();
                self.dirty = true
            }
            NotesStoreMsg::LoadDraft => {
                if self.notes.len() as i32 > 0 { self.draft_id = (self.notes[(self.active_id) as usize]["id"].as_i64().unwrap_or(0) as i32); self.draft_title = self.notes[(self.active_id) as usize]["title"].as_str().unwrap_or_default().to_string().to_string(); self.draft_body = self.notes[(self.active_id) as usize]["body"].as_str().unwrap_or_default().to_string().to_string(); self.draft_time = self.notes[(self.active_id) as usize]["time"].as_str().unwrap_or_default().to_string().to_string(); self.draft_folder = self.notes[(self.active_id) as usize]["folder"].as_str().unwrap_or_default().to_string().to_string(); self.draft_pinned = self.notes[(self.active_id) as usize]["pinned"].as_bool().unwrap_or(false); self.draft_tags = self.notes[(self.active_id) as usize]["tags"].as_str().unwrap_or_default().to_string() } else { ; self.draft_id = 0; self.draft_title = "".to_string(); self.draft_body = "".to_string(); self.draft_time = "".to_string(); self.draft_folder = "".to_string(); self.draft_pinned = false };
                self.dirty = false;
                self.confirm_delete = false
            }
            NotesStoreMsg::NewNote => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes() } else { self.notes = search_notes(self.search.clone()) } };
                self.dirty = false;
                ;
                self.search = "".to_string();
                let mut f = "".to_string();
                if self.active_folder != "all".to_string() { if self.active_folder != "pinned".to_string() { f = self.active_folder } };
                create_note("".to_string(), "".to_string(), f);
                self.notes = list_notes();
                self.active_id = 0;
                self.LoadDraft();
                self.ApplyFilter()
            }
            NotesStoreMsg::SaveDraft => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes() } else { self.notes = search_notes(self.search.clone()) } };
                self.dirty = false;
                self.LoadDraft();
                self.ApplyFilter()
            }
            NotesStoreMsg::SeedVocab => {
                let mut dup = false;
                for mut n in self.notes.iter_mut() { for t in n["tags"].as_array().into_iter().flatten() { dup = false; for mut e in self.all_tags.iter_mut() { if e == t { dup = true } }; if dup == false { self.all_tags.push(t) } }; if n["folder"].as_str().unwrap_or_default().to_string() != "".to_string() { dup = false; for mut e in self.all_folders.iter_mut() { if e == n["folder"].as_str().unwrap_or_default().to_string() { dup = true } }; if dup == false { self.all_folders.push(n["folder"].as_str().unwrap_or_default().to_string()) } } }
            }
            NotesStoreMsg::SelectFirstVisible => {
                if self.visible_pinned.len() as i32 > 0 { self.active_id = self.visible_pinned[0] } else if self.visible_notes.len() as i32 > 0 { self.active_id = self.visible_notes[0] };
                self.LoadDraft()
            }
            NotesStoreMsg::SelectFolder(f) => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes() } else { self.notes = search_notes(self.search.clone()) }; self.dirty = false };
                self.active_folder = f.to_string();
                self.active_tag = "".to_string();
                self.ApplyFilter();
                self.SelectFirstVisible()
            }
            NotesStoreMsg::SelectNote(i) => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes() } else { self.notes = search_notes(self.search.clone()) } };
                self.dirty = false;
                self.active_id = i;
                self.LoadDraft()
            }
            NotesStoreMsg::SelectTag(t) => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes() } else { self.notes = search_notes(self.search.clone()) }; self.dirty = false };
                self.active_tag = t.to_string();
                self.active_folder = "all".to_string();
                self.ApplyFilter();
                self.SelectFirstVisible()
            }
            NotesStoreMsg::SetAccent(name) => {
                self.accent_color = name.to_string()
            }
            NotesStoreMsg::SetMode(m) => {
                if m == "dark".to_string() { if self.dark_mode == false { self.ToggleDarkMode() } } else { if self.dark_mode == true { self.ToggleDarkMode() } }
            }
            NotesStoreMsg::SetSearch(q) => {
                if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); if self.search == "".to_string() { self.notes = list_notes() } else { self.notes = search_notes(self.search.clone()) }; self.dirty = false };
                self.search = q.to_string();
                if self.search == "".to_string() { self.notes = list_notes() } else { self.notes = search_notes(self.search.clone()) };
                self.active_id = 0;
                self.LoadDraft();
                self.ApplyFilter()
            }
            NotesStoreMsg::TagAdded(t) => {
                if self.notes.len() as i32 > 0 { let mut tl = vec![]; let mut exists = false; for tg in self.notes[(self.active_id) as usize]["tags"].as_array().into_iter().flatten() { if tg == t { exists = true }; tl.push(tg) }; if exists == false { tl.push(t) }; update_tags((self.notes[(self.active_id) as usize]["id"].as_i64().unwrap_or(0) as i32), tl); let mut dup = false; for mut e in self.all_tags.iter_mut() { if e == t { dup = true } }; if dup == false { self.all_tags.push(t) }; if self.search == "".to_string() { self.notes = list_notes() } else { self.notes = search_notes(self.search.clone()) }; self.LoadDraft(); self.ApplyFilter() }
            }
            NotesStoreMsg::TagRemoved(t) => {
                if self.notes.len() as i32 > 0 { let mut tl = vec![]; for tg in self.notes[(self.active_id) as usize]["tags"].as_array().into_iter().flatten() { if tg.as_str().unwrap_or_default() != t.as_str() { tl.push(tg) } }; update_tags((self.notes[(self.active_id) as usize]["id"].as_i64().unwrap_or(0) as i32), tl); if self.search == "".to_string() { self.notes = list_notes() } else { self.notes = search_notes(self.search.clone()) }; self.LoadDraft(); self.ApplyFilter() }
            }
            NotesStoreMsg::ToggleDarkMode => {
                self.dark_mode = !(self.dark_mode)
            }
            NotesStoreMsg::TogglePin(idx) => {
                if idx < self.notes.len() as i32 { ; let mut cur = self.notes[(idx) as usize]["pinned"].as_bool().unwrap_or(false); self.notes[(idx)as usize]["pinned"] = serde_json::json!(!(as usize]["pinned"].as_bool().unwrap_or(false))) }
            }
            NotesStoreMsg::TogglePinActive => {
                if self.notes.len() as i32 > 0 { if self.dirty { update_note(self.draft_id, self.draft_title.clone(), self.draft_body.clone()); self.dirty = false }; toggle_pin((self.notes[(self.active_id) as usize]["id"].as_i64().unwrap_or(0) as i32)); if self.search == "".to_string() { self.notes = list_notes() } else { self.notes = search_notes(self.search.clone()) }; self.LoadDraft(); self.ApplyFilter() }
            }
            NotesStoreMsg::ToggleSettings => {
                self.show_settings = !(self.show_settings)
            }
            NotesStoreMsg::Init => {
                self.search = "".to_string();
                self.active_folder = "all".to_string();
                self.active_tag = "".to_string();
                self.notes = list_notes();
                self.SeedVocab();
                self.active_id = 0;
                self.LoadDraft();
                self.ApplyFilter()
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


// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum NavTreeMsg {
    SelectNote(i32),
    SearchChanged(String),
    SelectFolder(String),
    SelectTag(String),
    ClearTag,
    ToggleDarkMode,
    SetMode(String),
    SetAccent(String),
    ToggleSettings,
}

#[derive(Clone, Debug)]
pub struct NavTree {
    pub store: NotesStore,
    pub dark_mode: bool,
    pub accent_color: String,
}

impl NavTree {
    pub fn new() -> Self {
        Self {
            dark_mode: false,
            accent_color: "".to_string(),
            store: NotesStore::new(),
        }
    }
}
impl Default for NavTree {
    fn default() -> Self { Self::new() }
}

impl Component for NavTree {
    type Msg = NavTreeMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            NavTreeMsg::ClearTag => {
                self.store.on(NotesStoreMsg::ClearTag)
            }
            NavTreeMsg::SearchChanged(q) => {
                self.store.on(NotesStoreMsg::SetSearch(q))
            }
            NavTreeMsg::SelectFolder(f) => {
                self.store.on(NotesStoreMsg::SelectFolder(f))
            }
            NavTreeMsg::SelectNote(i) => {
                self.store.on(NotesStoreMsg::SelectNote(i))
            }
            NavTreeMsg::SelectTag(t) => {
                self.store.on(NotesStoreMsg::SelectTag(t))
            }
            NavTreeMsg::SetAccent(name) => {
                ()
            }
            NavTreeMsg::SetMode(m) => {
                ()
            }
            NavTreeMsg::ToggleDarkMode => {
                ()
            }
            NavTreeMsg::ToggleSettings => {
                self.store.on(NotesStoreMsg::ToggleSettings)
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().style("w-auto min-h-0 flex").child(View::col().style("w-80 shrink-0 h-full flex flex-col overflow-hidden bg-card border-r border-border").child(View::row().style("h-9 mx-3 mt-3 mb-2 shrink-0 items-center gap-2 rounded-md border border-border bg-background px-2.5").child(View::icon().style("text-muted-foreground").build()).child(View::input("Search notes").style("flex-1 text-sm bg-transparent border-0 outline-none").on_change(NavTreeMsg::SearchChanged("".to_string())).build()).build()).child(View::row().style("shrink-0 gap-1 px-3 pb-1").child(View::button("All").style(if self.store.active_folder == "all".to_string() { "h-7 w-auto flex items-center gap-1 px-2.5 text-xs rounded-full border-0 bg-primary/10 text-primary font-medium".to_string() } else { "h-7 w-auto flex items-center gap-1 px-2.5 text-xs rounded-full border-0 bg-muted text-muted-foreground hover:bg-accent hover:text-foreground transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectFolder("all".to_string())).build()).child(View::button("Pinned").style(if self.store.active_folder == "pinned".to_string() { "h-7 w-auto flex items-center gap-1 px-2.5 text-xs rounded-full border-0 bg-primary/10 text-primary font-medium".to_string() } else { "h-7 w-auto flex items-center gap-1 px-2.5 text-xs rounded-full border-0 bg-muted text-muted-foreground hover:bg-accent hover:text-foreground transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectFolder("pinned".to_string())).build()).child(View::col().children(self.store.all_folders.iter().map(|f| { View::button(format!("{}", f)).style(if self.store.active_folder == f { "h-7 w-auto flex items-center gap-1 px-2.5 text-xs rounded-full border-0 bg-primary/10 text-primary font-medium".to_string() } else { "h-7 w-auto flex items-center gap-1 px-2.5 text-xs rounded-full border-0 bg-muted text-muted-foreground hover:bg-accent hover:text-foreground transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectFolder(f)).build() }).collect::<Vec<_>>()).build()).build()).child(View::row().style("shrink-0 gap-1 px-3 pb-2").child(View::col().children(self.store.all_tags.iter().map(|t| { View::button(format!("{}\n{}", "#".to_string(), t)).style(if self.store.active_tag == t.as_str().unwrap_or_default() { "h-7 w-auto flex items-center gap-1 px-2.5 text-xs rounded-full border-0 bg-primary/10 text-primary font-medium".to_string() } else { "h-7 w-auto flex items-center gap-1 px-2.5 text-xs rounded-full border-0 bg-muted text-muted-foreground hover:bg-accent hover:text-foreground transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SelectTag(t.to_string())).build() }).collect::<Vec<_>>()).build()).child(if self.store.active_tag != "" { View::button("").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-6 w-6 border-0 flex items-center justify-center rounded-full bg-transparent text-muted-foreground hover:text-foreground hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::ClearTag).build() } else { View::Empty }).build()).child(View::col().style("px-2 pb-2").child(if self.store.visible_pinned.len ( ) > 0 { View::col().child(View::col().child(View::text("Pinned".to_string())).build()).child(View::col().child(View::col().child(View::col().children(self.store.visible_pinned.iter().map(|k| { View::col().child(View::col().style("h-auto py-1.5 note-row").child(View::col().style("flex-1 min-w-0 items-start gap-0.5").child(View::text_styled(format!("{}", self.store.notes[(k) as usize]["title"].as_str().unwrap_or_default().to_string()), "text-sm truncate w-full")).child(View::text_styled(format!("{}", self.store.notes[(k) as usize]["time"].as_str().unwrap_or_default().to_string()), "text-xs text-muted-foreground truncate w-full")).build()).on_click(|_| NavTreeMsg::SelectNote(k)).build()).build() }).collect::<Vec<_>>()).build()).build()).build()).build() } else { View::Empty }).child(if self.store.visible_notes.len ( ) > 0 { View::col().child(View::col().child(View::text("Notes".to_string())).build()).child(View::col().child(View::col().child(View::col().children(self.store.visible_notes.iter().map(|k| { View::col().child(View::col().style("h-auto py-1.5 note-row").child(View::col().style("flex-1 min-w-0 items-start gap-0.5").child(View::text_styled(format!("{}", self.store.notes[(k) as usize]["title"].as_str().unwrap_or_default().to_string()), "text-sm truncate w-full")).child(View::text_styled(format!("{}", self.store.notes[(k) as usize]["time"].as_str().unwrap_or_default().to_string()), "text-xs text-muted-foreground truncate w-full")).build()).on_click(|_| NavTreeMsg::SelectNote(k)).build()).build() }).collect::<Vec<_>>()).build()).build()).build()).build() } else { View::Empty }).child(if self.store.visible_pinned.len ( ) == 0 { if self.store.visible_notes.len ( ) == 0 { View::col().style("items-center gap-2 py-8").child(View::icon().style("text-muted-foreground/50").build()).child(View::text_styled("No matching notes".to_string(), "text-xs text-muted-foreground")).build() } else { View::Empty } } else { View::Empty }).build()).child(if self.store.show_settings { View::col().style("shrink-0 gap-2 mx-3 mb-2 p-3 rounded-lg border border-border bg-background").child(View::row().style("w-auto items-center justify-between").child(View::text_styled("Appearance".to_string(), "text-xs font-medium text-foreground")).child(View::button("").style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-6 w-6 border-0 flex items-center justify-center rounded-full bg-transparent text-muted-foreground hover:text-foreground hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::ToggleSettings).build()).build()).child(View::row().style("w-auto items-center gap-1").child(View::button(format!("{}", "Dark".to_string())).style(if self.store.dark_mode { "h-7 w-auto border-0 flex-1 items-center justify-center rounded-md bg-primary text-primary-foreground text-xs font-medium transition-colors".to_string() } else { "h-7 w-auto border-0 flex-1 items-center justify-center rounded-md bg-transparent text-xs text-muted-foreground hover:text-foreground transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetMode("dark".to_string())).build()).child(View::button(format!("{}", "Light".to_string())).style(if self.store.dark_mode { "h-7 w-auto border-0 flex-1 items-center justify-center rounded-md bg-transparent text-xs text-muted-foreground hover:text-foreground transition-colors".to_string() } else { "h-7 w-auto border-0 flex-1 items-center justify-center rounded-md bg-primary text-primary-foreground text-xs font-medium transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetMode("light".to_string())).build()).build()).child(View::row().style("w-auto items-center gap-2.5").child(View::button(()).style(if self.store.accent_color == "indigo".to_string() { "w-5 h-5 border-0 rounded-full p-0 bg-indigo-500 ring-2 ring-offset-2 ring-offset-background ring-primary".to_string() } else { "w-5 h-5 border-0 rounded-full p-0 bg-indigo-500".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("indigo".to_string())).build()).child(View::button(()).style(if self.store.accent_color == "coral".to_string() { "w-5 h-5 border-0 rounded-full p-0 bg-rose-500 ring-2 ring-offset-2 ring-offset-background ring-primary".to_string() } else { "w-5 h-5 border-0 rounded-full p-0 bg-rose-500".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("coral".to_string())).build()).child(View::button(()).style(if self.store.accent_color == "ocean".to_string() { "w-5 h-5 border-0 rounded-full p-0 bg-sky-500 ring-2 ring-offset-2 ring-offset-background ring-primary".to_string() } else { "w-5 h-5 border-0 rounded-full p-0 bg-sky-500".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("ocean".to_string())).build()).child(View::button(()).style(if self.store.accent_color == "sage".to_string() { "w-5 h-5 border-0 rounded-full p-0 bg-emerald-500 ring-2 ring-offset-2 ring-offset-background ring-primary".to_string() } else { "w-5 h-5 border-0 rounded-full p-0 bg-emerald-500".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("sage".to_string())).build()).child(View::button(()).style(if self.store.accent_color == "amber".to_string() { "w-5 h-5 border-0 rounded-full p-0 bg-amber-500 ring-2 ring-offset-2 ring-offset-background ring-primary".to_string() } else { "w-5 h-5 border-0 rounded-full p-0 bg-amber-500".to_string() }.as_str()).on_click(|_| NavTreeMsg::SetAccent("amber".to_string())).build()).build()).build() } else { View::Empty }).child(View::row().style("w-auto shrink-0 items-center gap-2 px-3 py-2 border-t border-border").child(View::button(format!("{}", "Settings".to_string())).style(if self.store.show_settings { "h-7 w-auto border-0 flex items-center gap-1.5 px-2.5 rounded-md bg-accent text-foreground transition-colors".to_string() } else { "h-7 w-auto border-0 flex items-center gap-1.5 px-2.5 rounded-md bg-transparent text-muted-foreground hover:text-foreground hover:bg-accent transition-colors".to_string() }.as_str()).on_click(|_| NavTreeMsg::ToggleSettings).build()).child(View::button(format!("{}", "Theme".to_string())).style("bg-muted border border-border text-foreground font-medium rounded-md hover:bg-muted/70 h-10 px-4 h-7 w-auto border-0 flex items-center gap-1.5 px-2.5 rounded-md bg-transparent text-muted-foreground hover:text-foreground hover:bg-accent transition-colors").on_click(|_| NavTreeMsg::ToggleDarkMode).build()).build()).build()).build()
    }

    fn state_snapshot(&self) -> std::collections::HashMap<String, auto_lang::ui::auto_val::Value> {
        let mut m = std::collections::HashMap::new();
        for (k, v) in self.store.state_snapshot() { m.insert(format!("{}.{}", "store", k), v); }
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

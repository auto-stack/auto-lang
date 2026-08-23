// Plan 418 Phase 2: declarative action/binding configuration.
//
// Auto-atom file (declared in pac.at as `ui_config: "auto-edit.at"`):
//
//   auto-edit {
//       action { id : "file.new" handler : ".ActNew" title : "新建"
//                icon : "file-plus" shortcut : "Ctrl+N" }
//       menubar { menu { id : "file" title : "文件"
//                        item { action : "file.new" }  sep {} } }
//       toolbar { item { action : "file.new" }  sep {} }
//   }
//
// Architecture (plan 418 §2.3): Action = declaration layer (data), Event =
// execution layer. A configured action's `handler` is an EXISTING `on {}`
// event name (e.g. ".ActSave") — triggering dispatches into the unchanged
// VM handler pipeline. Bad files degrade gracefully: parse/validation
// errors are logged and the config is ignored (auto-os-config drop-in
// philosophy), so the app falls back to its DSL-declared bindings.
//!
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// One semantic action: identity + dispatch target + presentation metadata.
#[derive(Debug, Clone)]
pub struct ActionDef {
    /// Dotted lowercase id (`file.new`). Unique within the config.
    pub id: String,
    /// Dispatch target: an existing `on {}` event name (`.ActNew`).
    pub handler: String,
    /// Menu/tooltip label.
    pub title: String,
    /// Lucide icon name for toolbar/menubar use.
    pub icon: Option<String>,
    /// Shortcut in display form (`Ctrl+N`); normalized for key lookup.
    pub shortcut: Option<String>,
    /// Menu check-state var reference (`.console_open`), simple form only.
    pub checked_if: Option<String>,
    /// Enable-state var reference; absent = always enabled.
    pub enabled_if: Option<String>,
}

/// A menubar/toolbar entry: an action reference or a separator.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItem {
    Action(String),
    Separator,
}

/// One dropdown menu in the menubar.
#[derive(Debug, Clone)]
pub struct MenuDef {
    pub id: String,
    pub title: String,
    pub items: Vec<MenuItem>,
}

/// The full parsed + validated config.
#[derive(Debug, Clone, Default)]
pub struct UiActionConfig {
    pub actions: Vec<ActionDef>,
    pub menus: Vec<MenuDef>,
    pub toolbar: Vec<MenuItem>,
    /// shortcut lookup key (`Ctrl+n`) → handler event name.
    shortcut_bindings: Vec<(String, String)>,
}

impl UiActionConfig {
    /// Parse from an auto-atom document. Returns the config plus a list of
    /// non-fatal validation warnings (empty = clean).
    pub fn parse(doc: &str) -> Result<(UiActionConfig, Vec<String>), String> {
        let atom = auto_atom::AtomParser::parse(doc)
            .map_err(|e| format!("action config parse error: {e}"))?;
        let node = match atom {
            auto_atom::Atom::Node(n) => n,
            _ => return Err("action config: document root is not a node".into()),
        };

        let mut cfg = UiActionConfig::default();
        let mut warnings = Vec::new();

        for (_, kid) in node.kids_iter() {
            if let auto_val::Kid::Node(n) = kid {
                match n.name.as_str() {
                    "action" => {
                        let a = Self::parse_action(n);
                        if a.handler.is_empty() {
                            warnings.push(format!(
                                "action {:?}: missing handler — entry skipped",
                                a.id
                            ));
                        } else {
                            cfg.actions.push(a);
                        }
                    }
                    "menubar" => {
                        for (_, mk) in n.kids_iter() {
                            if let auto_val::Kid::Node(menu) = mk {
                                if menu.name == "menu" {
                                    cfg.menus.push(Self::parse_menu(menu));
                                }
                            }
                        }
                    }
                    "toolbar" => {
                        cfg.toolbar = Self::parse_items(n);
                    }
                    other => {
                        warnings.push(format!("unknown block `{}` ignored", other));
                    }
                }
            }
        }

        // Validation: unique ids, menu/toolbar refs resolve, shortcuts normalize.
        let mut seen = HashMap::new();
        for a in &cfg.actions {
            if a.id.is_empty() {
                warnings.push("action with empty id — skipped".into());
            } else if seen.insert(a.id.clone(), ()).is_some() {
                warnings.push(format!("duplicate action id {:?} — later wins", a.id));
            }
        }
        let exists = |id: &str| seen.contains_key(id);
        for m in &cfg.menus {
            for item in &m.items {
                if let MenuItem::Action(id) = item {
                    if !exists(id) {
                        warnings.push(format!("menu {:?} references unknown action {:?}", m.id, id));
                    }
                }
            }
        }
        for item in &cfg.toolbar {
            if let MenuItem::Action(id) = item {
                if !exists(id) {
                    warnings.push(format!("toolbar references unknown action {:?}", id));
                }
            }
        }

        // Shortcut bindings: normalized key → handler (first action wins on
        // collision, warning recorded).
        for w in cfg.rebuild_shortcut_bindings() {
            warnings.push(w);
        }

        Ok((cfg, warnings))
    }

    /// Rebuild `shortcut_bindings` from the actions' current `shortcut`
    /// fields (normalized key → handler; first wins, collision warns).
    /// Plan 423 P2: re-run after the OS keymap layer mutates shortcuts.
    fn rebuild_shortcut_bindings(&mut self) -> Vec<String> {
        let mut bindings = Vec::new();
        let mut warnings = Vec::new();
        for a in &self.actions {
            if let Some(sc) = &a.shortcut {
                let key = normalize_shortcut(sc);
                if key.is_empty() {
                    warnings.push(format!("action {:?}: unparseable shortcut {:?}", a.id, sc));
                    continue;
                }
                if bindings.iter().any(|(k, _)| *k == key) {
                    warnings.push(format!("shortcut {key:?} bound twice — first wins"));
                } else {
                    bindings.push((key, a.handler.clone()));
                }
            }
        }
        self.shortcut_bindings = bindings;
        warnings
    }

    fn parse_action(n: &auto_val::Node) -> ActionDef {
        let prop = |k: &str| n.get_prop(k).to_astr().trim().to_string();
        let opt = |k: &str| {
            let v = prop(k);
            (!v.is_empty()).then_some(v)
        };
        ActionDef {
            id: prop("id"),
            handler: prop("handler"),
            title: prop("title"),
            icon: opt("icon"),
            shortcut: opt("shortcut"),
            checked_if: opt("checked-if"),
            enabled_if: opt("enabled-if"),
        }
    }

    fn parse_menu(n: &auto_val::Node) -> MenuDef {
        let prop = |k: &str| n.get_prop(k).to_astr().trim().to_string();
        MenuDef {
            id: prop("id"),
            title: prop("title"),
            items: Self::parse_items(n),
        }
    }

    /// `item { action : "..." }` kids → MenuItems; `sep {}` → Separator.
    fn parse_items(n: &auto_val::Node) -> Vec<MenuItem> {
        let mut out = Vec::new();
        for (_, kid) in n.kids_iter() {
            if let auto_val::Kid::Node(item) = kid {
                match item.name.as_str() {
                    "item" => {
                        let id = item.get_prop("action").to_astr().trim().to_string();
                        if id.is_empty() {
                            out.push(MenuItem::Separator); // degrade: bare item ~ sep
                        } else {
                            out.push(MenuItem::Action(id));
                        }
                    }
                    "sep" | "separator" => out.push(MenuItem::Separator),
                    _ => {}
                }
            }
        }
        out
    }

    /// Handler for a normalized shortcut key, if configured.
    pub fn handler_for_key(&self, key: &str) -> Option<&str> {
        self.shortcut_bindings
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, h)| h.as_str())
    }

    pub fn shortcut_bindings(&self) -> &[(String, String)] {
        &self.shortcut_bindings
    }

    pub fn action_by_id(&self, id: &str) -> Option<&ActionDef> {
        self.actions.iter().find(|a| a.id == id)
    }

    /// Does `handler` match any configured action? (Used by menubar close-
    /// on-activate.)
    pub fn is_configured_handler(&self, handler: &str) -> bool {
        self.actions.iter().any(|a| a.handler == handler)
    }
}

/// Normalize a display shortcut (`Ctrl+N`, `alt+f4`) to the iced keyboard
/// listener's lookup form: `Ctrl+`/`Alt+` prefixes + key. Single alpha
/// chars are lowercased (the OS reports the base character with Ctrl/Alt
/// held); named keys (F4, Enter…) pass through as-is.
pub fn normalize_shortcut(s: &str) -> String {
    let mut ctrl = false;
    let mut alt = false;
    let mut key = String::new();
    for part in s.split('+') {
        let raw = part.trim();
        match raw.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => ctrl = true,
            "alt" => alt = true,
            "shift" => {} // Shift is expressed by the shifted character itself
            "" => {}
            _ => key = raw.to_string(),
        }
    }
    if key.is_empty() {
        return String::new();
    }
    if key.len() == 1 {
        key = key.to_lowercase();
    }
    let mut out = String::new();
    if ctrl {
        out.push_str("Ctrl+");
    }
    if alt {
        out.push_str("Alt+");
    }
    out.push_str(&key);
    out
}

static ACTION_CONFIG: std::sync::RwLock<Option<std::sync::Arc<UiActionConfig>>> =
    std::sync::RwLock::new(None);

/// Resolved once from AUTO_VM_ACTION_CONFIG (injected by `auto run` from
/// pac.at `ui_config:`); hot reloads re-read this path.
static CONFIG_PATH: OnceLock<Option<String>> = OnceLock::new();

/// App identity for the OS keymap layer: file stem of the config path
/// (auto-edit.at → "auto-edit").
static CONFIG_APP_ID: OnceLock<String> = OnceLock::new();

/// (mtime, len) of the app config + OS keymap layer at the last successful
/// load — the mtime-poll change detector compares against this.
static CONFIG_STAMP: Mutex<Option<(std::time::SystemTime, u64)>> = Mutex::new(None);

/// Bumped on every successful reload. The renderer's update closure compares
/// its last-seen value on each message (heartbeat included) to force a
/// view rebuild after a config swap (Plan 423 P1).
static CONFIG_GENERATION: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

pub fn config_generation() -> u64 {
    CONFIG_GENERATION.load(std::sync::atomic::Ordering::SeqCst)
}

/// Plan 423 P5:最近一次成功重载的摘要(action_config_reload 工具响应用)。
static LAST_RELOAD_INFO: Mutex<Option<String>> = Mutex::new(None);

pub fn last_reload_info() -> Option<String> {
    LAST_RELOAD_INFO.lock().unwrap().clone()
}

/// Plan 418 P2-3: which synthesized menubar is open (menu id), if any.
/// Renderer-side local UI state (same pattern as preview-card states) —
/// kept here so the builder (view) and the renderer (update) share it
/// without threading through DynamicState.
static MENUBAR_OPEN: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

pub fn menubar_open() -> Option<String> {
    MENUBAR_OPEN.lock().unwrap().clone()
}

pub fn set_menubar_open(v: Option<String>) {
    *MENUBAR_OPEN.lock().unwrap() = v;
}

/// Plan 422 P4: which DSL popover (self-managed form, no `open` prop) is
/// open, keyed by instance slot id (`pv_<path>`; stable across rebuilds).
/// Same renderer-local pattern as MENUBAR_OPEN.
static POPOVER_OPEN: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

pub fn popover_open() -> Option<String> {
    POPOVER_OPEN.lock().unwrap().clone()
}

pub fn set_popover_open(v: Option<String>) {
    *POPOVER_OPEN.lock().unwrap() = v;
}

/// Process-wide config. None when AUTO_VM_ACTION_CONFIG is unset/unreadable/
/// invalid — callers fall back to DSL-declared bindings.
///
/// Plan 423 P1: hot-swappable — first access loads from disk, later calls
/// hit the read-lock fast path (Arc clone, no parse). Reloads swap the Arc
/// under the write lock; a failed reload keeps the previous value.
pub fn action_config() -> Option<std::sync::Arc<UiActionConfig>> {
    if let Some(cfg) = ACTION_CONFIG.read().unwrap().clone() {
        return Some(cfg);
    }
    reload_action_config()
}

/// Force a (re)load from disk. Degradation semantics (plan 423 P1): unreadable
/// or unparseable input KEEPS the previously loaded config (logged); only a
/// clean parse swaps it in and bumps the generation.
pub fn reload_action_config() -> Option<std::sync::Arc<UiActionConfig>> {
    let path = CONFIG_PATH
        .get_or_init(|| {
            std::env::var("AUTO_VM_ACTION_CONFIG")
                .ok()
                .filter(|p| !p.is_empty())
        })
        .clone();
    let Some(path) = path else {
        return None; // no config wired — DSL-declared bindings only
    };
    let _ = CONFIG_APP_ID.set(
        std::path::Path::new(&path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
    );
    let doc = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[ACTION-CONFIG] cannot read {path}: {e} — keeping previous config");
            return ACTION_CONFIG.read().unwrap().clone();
        }
    };
    match UiActionConfig::parse(&doc) {
        Ok((mut cfg, warnings)) => {
            for w in &warnings {
                eprintln!("[ACTION-CONFIG] warning: {w}");
            }
            // Plan 423 P2: OS user keymap layer (by action id, bindings only).
            let os_overrides = apply_os_keymap_layer(&mut cfg);
            eprintln!(
                "[ACTION-CONFIG] loaded {}: {} actions, {} menus, {} toolbar items, {} OS keymap overrides",
                path,
                cfg.actions.len(),
                cfg.menus.len(),
                cfg.toolbar.len(),
                os_overrides
            );
            let arc = std::sync::Arc::new(cfg);
            *ACTION_CONFIG.write().unwrap() = Some(arc.clone());
            *CONFIG_STAMP.lock().unwrap() = config_stamp(&path);
            *LAST_RELOAD_INFO.lock().unwrap() = Some(format!(
                "{} actions, {} menus, {} toolbar items, {} OS keymap overrides",
                arc.actions.len(),
                arc.menus.len(),
                arc.toolbar.len(),
                os_overrides
            ));
            CONFIG_GENERATION.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Some(arc)
        }
        Err(e) => {
            eprintln!("[ACTION-CONFIG] {e} — keeping previous config ({path})");
            ACTION_CONFIG.read().unwrap().clone()
        }
    }
}

/// (mtime, len) of the config file + OS keymap layer, combined — a change in
/// either re-triggers a reload.
fn config_stamp(path: &str) -> Option<(std::time::SystemTime, u64)> {
    let mut mtime = None;
    let mut len = 0u64;
    if let Ok(meta) = std::fs::metadata(path) {
        mtime = meta.modified().ok();
        len += meta.len();
    }
    if let Some(app_id) = CONFIG_APP_ID.get() {
        if let Some(os) = os_keymap_path(app_id) {
            if let Ok(meta) = std::fs::metadata(&os) {
                if meta.modified().ok() > mtime || mtime.is_none() {
                    mtime = meta.modified().ok();
                }
                len = len.wrapping_add(meta.len());
            }
        }
    }
    mtime.map(|m| (m, len))
}

/// Plan 423 P1: mtime poll (called from the renderer's tick/heartbeat cadence,
/// throttled by the caller). Returns true when a reload actually happened.
/// Writers should swap atomically (temp file + rename); a half-written file
/// is caught by the parse-failure keep-previous path.
pub fn check_action_config_changed() -> bool {
    let Some(path) = CONFIG_PATH.get().cloned().flatten() else {
        return false;
    };
    let stamp = config_stamp(&path);
    let changed = stamp != *CONFIG_STAMP.lock().unwrap();
    if !changed {
        return false;
    }
    let before = config_generation();
    reload_action_config();
    config_generation() != before
}

/// Plan 423 P2: OS user-layer keymap path —
/// `%APPDATA%/auto/keymaps/<app>.at` (Windows) or `~/.auto/keymaps/<app>.at`.
fn os_keymap_path(app_id: &str) -> Option<std::path::PathBuf> {
    if app_id.is_empty() {
        return None;
    }
    let base = std::env::var("APPDATA")
        .ok()
        .map(|d| std::path::PathBuf::from(d).join("auto"))
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| std::path::PathBuf::from(h).join(".auto"))
        })?;
    Some(base.join("keymaps").join(format!("{app_id}.at")))
}

/// Apply the OS user keymap layer: same action schema but ONLY `id` +
/// `shortcut` participate — matching action ids get their shortcut (and
/// therefore the normalized key binding) overridden; handlers/menus are
/// never copied. A bad OS layer logs and is ignored (app layer stands).
/// Returns the number of overrides applied.
fn apply_os_keymap_layer(cfg: &mut UiActionConfig) -> usize {
    let Some(app_id) = CONFIG_APP_ID.get() else {
        return 0;
    };
    let Some(path) = os_keymap_path(app_id) else {
        return 0;
    };
    let Ok(doc) = std::fs::read_to_string(&path) else {
        return 0;
    };
    match parse_keymap_overrides(&doc) {
        Ok(overrides) => {
            let mut count = 0usize;
            for a in cfg.actions.iter_mut() {
                if let Some(sc) = overrides.get(&a.id) {
                    a.shortcut = Some(sc.clone());
                    count += 1;
                }
            }
            let warnings = cfg.rebuild_shortcut_bindings();
            for w in warnings {
                eprintln!("[ACTION-CONFIG] warning (after OS keymap): {w}");
            }
            count
        }
        Err(e) => {
            eprintln!("[ACTION-CONFIG] OS keymap {} ignored: {e}", path.display());
            0
        }
    }
}

/// Parse an OS keymap document: `action { id : "..." shortcut : "..." }`
/// entries → id → shortcut (display form). Handler-less by design.
fn parse_keymap_overrides(doc: &str) -> Result<HashMap<String, String>, String> {
    let atom = auto_atom::AtomParser::parse(doc)
        .map_err(|e| format!("keymap parse error: {e}"))?;
    let node = match atom {
        auto_atom::Atom::Node(n) => n,
        _ => return Err("keymap: document root is not a node".into()),
    };
    let mut out = HashMap::new();
    for (_, kid) in node.kids_iter() {
        if let auto_val::Kid::Node(n) = kid {
            if n.name == "action" {
                let id = n.get_prop("id").to_astr().trim().to_string();
                let sc = n.get_prop("shortcut").to_astr().trim().to_string();
                if !id.is_empty() && !sc.is_empty() {
                    out.insert(id, sc);
                }
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
auto-edit {
    action { id : "file.new"  handler : ".ActNew"  title : "新建" icon : "file-plus" shortcut : "Ctrl+N" }
    action { id : "file.save" handler : ".ActSave" title : "保存" shortcut : "Ctrl+S" }
    action { id : "view.console" handler : ".ActConsole" title : "切换 Console" checked-if : ".console_open" }
    menubar {
        menu { id : "file" title : "文件"  item { action : "file.new" }  sep {}  item { action : "file.save" } }
    }
    toolbar { item { action : "file.new" }  sep {} }
}
"#;

    #[test]
    fn parse_sample_actions_menus_toolbar() {
        let (cfg, warnings) = UiActionConfig::parse(SAMPLE).unwrap();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(cfg.actions.len(), 3);
        assert_eq!(cfg.menus.len(), 1);
        assert_eq!(cfg.menus[0].items.len(), 3);
        assert_eq!(cfg.menus[0].items[0], MenuItem::Action("file.new".into()));
        assert_eq!(cfg.menus[0].items[1], MenuItem::Separator);
        assert_eq!(cfg.toolbar.len(), 2);
        let a = cfg.action_by_id("view.console").unwrap();
        assert_eq!(a.checked_if.as_deref(), Some(".console_open"));
    }

    #[test]
    fn shortcut_normalization_matches_keyboard_lookup() {
        assert_eq!(normalize_shortcut("Ctrl+N"), "Ctrl+n");
        assert_eq!(normalize_shortcut("ctrl+s"), "Ctrl+s");
        assert_eq!(normalize_shortcut("Alt+F4"), "Alt+F4");
        assert_eq!(normalize_shortcut("Ctrl+Shift+Z"), "Ctrl+z");
        assert_eq!(normalize_shortcut("Enter"), "Enter");
        assert_eq!(normalize_shortcut("Ctrl"), "");
    }

    #[test]
    fn handler_lookup_and_warnings() {
        let (cfg, warnings) = UiActionConfig::parse(SAMPLE).unwrap();
        assert_eq!(cfg.handler_for_key("Ctrl+n"), Some(".ActNew"));
        assert_eq!(cfg.handler_for_key("Ctrl+s"), Some(".ActSave"));
        assert_eq!(cfg.handler_for_key("Ctrl+x"), None);
        assert!(cfg.is_configured_handler(".ActConsole"));
        assert!(!cfg.is_configured_handler(".Nope"));

        let (bad, warnings) = UiActionConfig::parse(
            "x { action { id : \"a\" title : \"t\" } item_ref_unknown { } }",
        )
        .unwrap();
        assert!(bad.actions.is_empty()); // missing handler → skipped
        assert!(!warnings.is_empty());
    }
}

#[cfg(test)]
mod plan423_tests {
    use super::*;

    const SAMPLE: &str = r#"
auto-edit {
    action { id : "file.new"  handler : ".ActNew"  title : "新建" icon : "file-plus" shortcut : "Ctrl+N" }
    action { id : "file.save" handler : ".ActSave" title : "保存" shortcut : "Ctrl+S" }
}
"#;

    /// Plan 423 P2:OS keymap 层按 action id 覆盖 shortcut,重建绑定表。
    #[test]
    fn os_keymap_overrides_apply_by_action_id() {
        let (mut cfg, _) = UiActionConfig::parse(SAMPLE).unwrap();
        assert_eq!(cfg.handler_for_key("Ctrl+n"), Some(".ActNew"));

        let doc = r#"
k {
    action { id : "file.new"  shortcut : "Ctrl+Shift+`" }
    action { id : "no.such"   shortcut : "Ctrl+F9" }
}"#;
        let overrides = parse_keymap_overrides(doc).unwrap();
        assert_eq!(overrides.len(), 2);

        for a in cfg.actions.iter_mut() {
            if let Some(sc) = overrides.get(&a.id) {
                a.shortcut = Some(sc.clone());
            }
        }
        let warnings = cfg.rebuild_shortcut_bindings();
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(cfg.handler_for_key("Ctrl+n"), None);
        assert_eq!(cfg.handler_for_key("Ctrl+`"), Some(".ActNew"));

        assert!(parse_keymap_overrides("not a node").is_err());
    }

    /// Plan 423 P1 降级语义:坏文档解析失败(reload 侧保旧 Arc)。
    #[test]
    fn hot_reload_bad_doc_fails_parse() {
        assert!(UiActionConfig::parse("auto-edit { action { id : \"x\"").is_err());
    }
}

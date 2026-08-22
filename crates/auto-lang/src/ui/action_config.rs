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
use std::sync::OnceLock;

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
        for a in &cfg.actions {
            if let Some(sc) = &a.shortcut {
                let key = normalize_shortcut(sc);
                if key.is_empty() {
                    warnings.push(format!("action {:?}: unparseable shortcut {:?}", a.id, sc));
                    continue;
                }
                if cfg.shortcut_bindings.iter().any(|(k, _)| *k == key) {
                    warnings.push(format!("shortcut {key:?} bound twice — first wins"));
                } else {
                    cfg.shortcut_bindings.push((key, a.handler.clone()));
                }
            }
        }

        Ok((cfg, warnings))
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

static ACTION_CONFIG: OnceLock<Option<UiActionConfig>> = OnceLock::new();

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

/// Process-wide config (loaded once from AUTO_VM_ACTION_CONFIG, injected by
/// `auto run` from pac.at `ui_config:`). None when unset/unreadable/invalid
/// — callers fall back to DSL-declared bindings.
pub fn action_config() -> Option<&'static UiActionConfig> {
    ACTION_CONFIG
        .get_or_init(|| {
            let path = std::env::var("AUTO_VM_ACTION_CONFIG").ok()?;
            let doc = match std::fs::read_to_string(&path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("[ACTION-CONFIG] cannot read {path}: {e}");
                    return None;
                }
            };
            match UiActionConfig::parse(&doc) {
                Ok((cfg, warnings)) => {
                    for w in &warnings {
                        eprintln!("[ACTION-CONFIG] warning: {w}");
                    }
                    eprintln!(
                        "[ACTION-CONFIG] loaded {}: {} actions, {} menus, {} toolbar items",
                        path,
                        cfg.actions.len(),
                        cfg.menus.len(),
                        cfg.toolbar.len()
                    );
                    Some(cfg)
                }
                Err(e) => {
                    eprintln!("[ACTION-CONFIG] {e} — config ignored ({path})");
                    None
                }
            }
        })
        .as_ref()
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

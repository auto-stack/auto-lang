//! PLAN-009 P1: native `terminal` component — the Auto-side true self of
//! auto-term's TermGrid control (single-truth migration; the Rust widget.rs
//! freezes as reference oracle once this lands).
//!
//! Layering (code_editor paradigm, Plan 413 §3.1):
//!   mod.rs   ① backend-neutral core — grid state + feed accessors, NO iced
//!            imports (headless tests assert against this layer);
//!   iced/    ② adapter — the only iced dependency point
//!            (T2: placeholder rect; T3: grid glyphs + damage gating;
//!            T4: selection/IME/scroll/menu).
//!
//! Data plane — 形态甲 (props-feed), T2 ruling per the pre-authorized rule:
//! the app feeds grid rows through `View::Terminal` props every frame
//! (code_editor §5.4 姿势: props in → internal state diffed, events out);
//! the component itself stays pure-render. Downgrade to 形态乙 (opaque
//! adapter session handle) only if the T3 2000-line streaming benchmark
//! exceeds the frame budget — measured data goes to the execution record.
//!
//! The engine (PTY + emulator) stays behind the auto-term cdylib adapter
//! (PLAN-009 P2): FFI at runtime, zero Cargo edge auto-lang → auto-term
//! (无环铁律).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

// ② iced adapter — the only iced dependency point of the terminal.
#[cfg(feature = "ui-iced")]
pub mod iced;

pub use iced::{Terminal, TerminalState};

/// Backend-neutral terminal state, registered per `key` (mirrors
/// CODE_EDITORS: leaked storage, interior mutability, widget rebuilt freely
/// every frame).
pub struct TerminalCore {
    pub key: String,
    /// Grid geometry in cells.
    pub cols: u16,
    pub rows: u16,
    /// Grid text buffer, one entry per row (padded/truncated to `rows` by
    /// the feed). Cell-level styles arrive with T3 damage gating.
    lines: Mutex<Vec<String>>,
    /// Bumped on every feed that changes content — T3 damage gating seed.
    generation: AtomicU64,
}

impl TerminalCore {
    fn new(key: &str, cols: u16, rows: u16) -> Self {
        Self {
            key: key.to_owned(),
            cols,
            rows,
            lines: Mutex::new(vec![String::new(); rows as usize]),
            generation: AtomicU64::new(0),
        }
    }

    /// Current content generation (damage epoch).
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }

    /// Read one row (cloned); `None` when out of range.
    pub fn line(&self, row: usize) -> Option<String> {
        self.lines.lock().unwrap().get(row).cloned()
    }

    /// Snapshot the full buffer (tests, iced draw).
    pub fn lines(&self) -> Vec<String> {
        self.lines.lock().unwrap().clone()
    }
}

static TERMINALS: Mutex<Option<HashMap<String, &'static TerminalCore>>> = Mutex::new(None);

/// Get-or-create the terminal state for `key`, diffing geometry in.
/// Geometry changes grow/shrink the row buffer, preserving existing rows.
pub fn terminal(key: &str, cols: u16, rows: u16) -> &'static TerminalCore {
    let mut map = TERMINALS.lock().unwrap();
    let map = map.get_or_insert_with(HashMap::new);
    if let Some(core) = map.get(key) {
        let core: &TerminalCore = core;
        if core.cols != cols || core.rows != rows {
            // Only the fresh-create path can resize (the leaked struct is
            // shared); re-register a replacement under the same key.
            let fresh: &'static TerminalCore =
                Box::leak(Box::new(TerminalCore::new(key, cols, rows)));
            let mut carry = fresh.lines.lock().unwrap();
            let old = core.lines.lock().unwrap();
            let n = core.rows as usize;
            let cloned: Vec<String> = old.iter().take(n).cloned().collect();
            for (i, line) in cloned.into_iter().enumerate() {
                if i < carry.len() {
                    carry[i] = line;
                }
            }
            drop(old);
            drop(carry);
            fresh
                .generation
                .store(core.generation() + 1, Ordering::Relaxed);
            map.insert(key.to_owned(), fresh);
            return fresh;
        }
        return core;
    }
    let core: &'static TerminalCore = Box::leak(Box::new(TerminalCore::new(key, cols, rows)));
    map.insert(key.to_owned(), core);
    core
}

/// 形态甲 feed: replace the buffer with the app-supplied rows (padded to
/// `rows` with empty lines, truncated beyond). Bumps the generation only
/// when content actually changed — a no-op feed must not invalidate frames.
pub fn terminal_feed(core: &TerminalCore, lines: &[String]) {
    let mut buf = core.lines.lock().unwrap();
    let rows = core.rows as usize;
    let mut changed = buf.len() != rows;
    for i in 0..rows {
        let incoming = lines.get(i).map(|s| s.as_str()).unwrap_or("");
        if buf.get(i).map(String::as_str) != Some(incoming) {
            changed = true;
        }
    }
    if !changed {
        return;
    }
    buf.clear();
    for i in 0..rows {
        buf.push(lines.get(i).cloned().unwrap_or_default());
    }
    drop(buf);
    core.generation.fetch_add(1, Ordering::Relaxed);
}

/// Dispose the state for `key` (component unmount / test teardown).
pub fn terminal_dispose(key: &str) {
    if let Ok(mut guard) = TERMINALS.lock() {
        if let Some(map) = guard.as_mut() {
            map.remove(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_get_or_create_and_feed() {
        terminal_dispose("t2-core-1");
        let core = terminal("t2-core-1", 80, 24);
        assert_eq!(core.cols, 80);
        assert_eq!(core.rows, 24);
        assert_eq!(core.lines().len(), 24);
        let gen0 = core.generation();

        // Feed two rows; the rest stays blank.
        terminal_feed(core, &["ready.".to_owned(), "second".to_owned()]);
        assert_eq!(core.line(0).as_deref(), Some("ready."));
        assert_eq!(core.line(1).as_deref(), Some("second"));
        assert_eq!(core.line(2).as_deref(), Some(""));
        assert!(core.generation() > gen0);

        // Identical feed is a no-op (generation stable — no frame churn).
        let gen1 = core.generation();
        terminal_feed(core, &["ready.".to_owned(), "second".to_owned()]);
        assert_eq!(core.generation(), gen1);

        terminal_dispose("t2-core-1");
    }

    #[test]
    fn terminal_geometry_change_preserves_rows() {
        terminal_dispose("t2-core-2");
        let core = terminal("t2-core-2", 40, 4);
        terminal_feed(core, &["a".into(), "b".into(), "c".into(), "d".into()]);
        let core2 = terminal("t2-core-2", 40, 6);
        assert_eq!(core2.rows, 6);
        assert_eq!(core2.line(0).as_deref(), Some("a"));
        assert_eq!(core2.line(3).as_deref(), Some("d"));
        assert_eq!(core2.line(4).as_deref(), Some(""));
        terminal_dispose("t2-core-2");
    }

    /// PLAN-009 T2: 最小 `.at` 示例挂载 + headless 断言。
    ///
    /// `test/ui/terminal_min/src/front/app.at` 挂 `<terminal/>`(占位矩形);
    /// 本测沿 view-builder 全链(parse → extract → VmBridge → build)断言
    /// 产出 `View::Terminal`,再经 headless 管线(view_to_vtree)断言占位
    /// 节点存在(几何/键入 prop 可见)。
    #[test]
    fn minimal_at_example_mounts_terminal_placeholder() {
        use crate::aura::extract::extract_widget_from_decl;
        use crate::ast::Stmt;
        use crate::parser::Parser;
        use crate::session::CompilerSession;
        use crate::ui::aura_view_builder::AuraViewBuilder;
        use crate::ui::vm_bridge::VmBridge;
        use crate::ui::vnode_converter::view_to_vtree;
        use crate::ui::View;

        let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test/ui/terminal_min/src/front/app.at");
        let src = std::fs::read_to_string(&at)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", at.display()));

        let session = CompilerSession::ui();
        let mut parser = Parser::from(src.as_str()).with_session(session);
        let ast = parser.parse().expect("parse");
        let decl = ast
            .stmts
            .iter()
            .find_map(|s| match s {
                Stmt::WidgetDecl(d) => Some(d),
                _ => None,
            })
            .expect("widget decl");
        let widget = extract_widget_from_decl(decl).expect("extract");

        let bridge = VmBridge::new(&widget).unwrap();
        let builder = AuraViewBuilder::new(&bridge, "TermApp");
        let view = builder.build(&widget.view_tree);

        // The tree must carry the terminal element with its geometry.
        fn find_terminal<M: Clone + std::fmt::Debug>(v: &View<M>) -> Option<String> {
            match v {
                View::Terminal { key, cols, rows, lines, .. } => {
                    Some(format!("{key}/{cols}/{rows}/{}", lines.len()))
                }
                _ => None,
            }
        }
        let hit = find_terminal(&view).expect("View::Terminal missing from built view tree");
        assert_eq!(hit, "main/40/10/0", "terminal geometry props must round-trip");

        // Headless: the placeholder node exists in the VTree (占位矩形在案)。
        let vtree = view_to_vtree(view);
        let dump = format!("{vtree:?}");
        assert!(
            dump.contains("terminal key=main cols=40 rows=10"),
            "headless VTree must expose the terminal placeholder node, got:\n{dump}"
        );
    }
}

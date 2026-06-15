//! Integration test for `ls` table rendering: icon column + directory-name coloring.
//!
//! Plan 309 / ls UX. Uses the public `render_table` API (does not touch the
//! lib's internal `#[cfg(test)]` modules, which have pre-existing compile errors
//! unrelated to this feature).
//!
//! Note: `buffer_to_ansi` emits ANSI styling per buffer cell, so multi-char text
//! is NOT contiguous in the raw output (each glyph is wrapped in its own escape
//! sequence). Strip ANSI before asserting on text content; assert on raw output
//! only for single-glyph icons and presence of specific style codes.

use auto_shell::frontend::renderer::render_table;
use auto_val::{Array, Obj, Value};

/// Strip CSI ANSI escape sequences (`ESC[ ... letter`) so text-content
/// assertions work despite per-cell styling.
fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            while let Some(csi) = chars.next() {
                if csi.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn file_obj(name: &str, ty: &str) -> Value {
    let mut o = Obj::new();
    o.set("name", Value::str(name));
    o.set("type", Value::str(ty));
    Value::Obj(o)
}

#[test]
fn file_listing_renders_dir_and_file_icons() {
    let arr = Array::from_vec(vec![file_obj("src", "dir"), file_obj("main.rs", "file")]);
    let out = render_table(&Value::Array(arr), 60).expect("should render");
    let plain = strip_ansi(&out);
    assert!(out.contains('📁'), "dir icon missing:\n{out}");
    assert!(out.contains('📄'), "file icon missing:\n{out}");
    assert!(plain.contains("src"), "dir name missing:\n{plain}");
    assert!(plain.contains("main.rs"), "file name missing:\n{plain}");
}

#[test]
fn non_file_listing_has_no_icon_column() {
    // No `type` column → not a file listing → no icon column.
    let mut o = Obj::new();
    o.set("name", Value::str("widget"));
    o.set("value", Value::Int(7));
    let arr = Array::from_vec(vec![Value::Obj(o)]);
    let out = render_table(&Value::Array(arr), 60).expect("should render");
    assert!(!out.contains('📁'));
    assert!(!out.contains('📄'));
}

#[test]
fn directory_name_and_icon_share_blue_style() {
    // A directory row: the icon and name are both LightBlue (ANSI bright blue = `94m`).
    let arr = Array::from_vec(vec![file_obj("docs", "dir")]);
    let out = render_table(&Value::Array(arr), 60).expect("should render");
    assert!(out.contains("94m"), "expected light-blue styling for dir row:\n{out}");
}

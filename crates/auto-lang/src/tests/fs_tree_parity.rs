//! PLAN-681 (P670-D1 缺口④ / AC-05): dual-track byte parity for `fs.tree`.
//!
//! The AutoVM native (`shim_fs_tree`, backed by `fs_tree_walk` in
//! vm/native.rs) and the a2r-std mirror (`a2r_std::fs::tree`) must emit
//! BYTE-IDENTICAL JSON for the same directory + depth: same TreeView node
//! schema `{id,label,children,kind,icon,is_leaf,badge}`, same skip list,
//! same dirs-first + case-insensitive ordering, same depth clamp (1..=8)
//! and same empty results (`[]`). The auto-edit front parses this JSON via
//! `json.to_value`, so any divergence forks the app across tracks.

use std::fs;
use std::path::Path;

fn vm_track(root: &Path, depth: usize) -> String {
    let mut buf = String::new();
    crate::vm::native::fs_tree_walk(root, root, depth, &mut buf);
    if buf.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", buf)
    }
}

fn a2r_track(root: &Path, depth: i32) -> String {
    a2r_std::fs::tree(&root.to_string_lossy(), depth)
}

#[test]
fn fs_tree_dual_track_byte_parity() {
    let dir = std::env::temp_dir().join(format!("auto_lang_fs_tree_parity_681_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let src = dir.join("ws");
    // Fixture mirrors plan626_fs_tree_nested_json_schema_and_order plus a
    // case-differing pair (B_Dir vs a_dir) to pin case-insensitive ordering.
    fs::create_dir_all(src.join("b_dir")).unwrap();
    fs::create_dir_all(src.join("B_Dir2")).unwrap();
    fs::create_dir_all(src.join("a_dir").join("nested")).unwrap();
    fs::create_dir_all(src.join("a_dir").join("nested").join("deeper")).unwrap();
    fs::write(src.join("z.txt"), "z").unwrap();
    fs::write(src.join("m.md"), "m").unwrap();
    fs::write(src.join("a_dir").join("f.at"), "fn main() {}").unwrap();
    fs::write(src.join("a_dir").join("nested").join("deep.md"), "# d").unwrap();
    fs::write(src.join("a_dir").join("nested").join("deeper").join("x.rs"), "").unwrap();
    fs::create_dir_all(src.join(".git")).unwrap();
    fs::write(src.join(".git").join("HEAD"), "ref").unwrap();
    fs::create_dir_all(src.join("target")).unwrap();
    fs::create_dir_all(src.join("node_modules")).unwrap();

    // Every depth level through BOTH tracks must agree byte-for-byte.
    for depth in 1..=8 {
        let vm = vm_track(&src, depth);
        let a2r = a2r_track(&src, depth as i32);
        assert_eq!(vm, a2r, "track divergence at depth {depth}");
    }

    // Depth clamp: VM clamps via pop_arg_i32().clamp(1, 8); the a2r host
    // clamps identically, so 0 / negative / oversized depths agree too.
    for depth in [0, -3, 99] {
        let a2r = a2r_track(&src, depth);
        let clamped = a2r_track(&src, depth.clamp(1, 8));
        assert_eq!(a2r, clamped, "depth {depth} must clamp to 1..=8");
    }

    // Shape spot-checks on the a2r track (schema + ordering + skip list).
    let a2r = a2r_track(&src, 4);
    let parsed: serde_json::Value = serde_json::from_str(&a2r).unwrap();
    let items = parsed.as_array().unwrap();
    // Dirs first (a_dir, b_dir, B_Dir2), then files case-insensitive (m.md, z.txt);
    // .git/target/node_modules skipped.
    let labels: Vec<&str> = items
        .iter()
        .map(|n| n["label"].as_str().unwrap())
        .collect();
    assert_eq!(labels, vec!["a_dir", "b_dir", "B_Dir2", "m.md", "z.txt"], "dirs-first + case-insensitive: {a2r}");
    assert_eq!(items[0]["kind"], "dir");
    assert_eq!(items[0]["icon"], "folder");
    assert_eq!(items[0]["is_leaf"], false);
    assert_eq!(items[0]["id"], "a_dir");
    assert_eq!(items[4]["kind"], "file");
    assert_eq!(items[4]["icon"], "file-text");
    assert_eq!(items[4]["is_leaf"], true);

    // Empty results: missing root and an empty directory both yield `[]`.
    let missing = dir.join("definitely_missing");
    assert_eq!(vm_track(&missing, 3), "[]");
    assert_eq!(a2r_track(&missing, 3), "[]");
    let empty = dir.join("empty_dir");
    fs::create_dir_all(&empty).unwrap();
    assert_eq!(vm_track(&empty, 3), "[]");
    assert_eq!(a2r_track(&empty, 3), "[]");

    fs::remove_dir_all(&dir).ok();
}

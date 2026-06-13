//! Godot `.tscn` scene generator (Plan 306).
//!
//! Walks a parsed `SceneDecl` (produced by `Parser::parse_scene_decl`) and
//! emits a Godot 4.x scene file. The output mirrors the hand-authored
//! `.tscn` files in `examples/godot/` so it can be opened directly in the
//! Godot editor.
//!
//! Generated structure:
//! ```text
//! [gd_scene load_steps=N format=3 uid="uid://..."]
//! [ext_resource type="Script" path="res://foo.gd" id="1"]
//! [ext_resource type="PackedScene" path="res://bar.tscn" id="2"]
//! [node name="Root" type="Type"]
//! script = ExtResource("1")
//! [node name="Child" type="Timer" parent="."]
//! wait_time = 0.5
//! [connection signal="timeout" from="Child" to="." method="on_timeout"]
//! ```

use crate::ast::{Code, Expr, SceneDecl, SceneNode, SceneProp, SceneSubResource, SceneValue, Stmt};
use auto_val::AutoStr;

/// External resource reference collected from the scene (scripts, packed
/// scenes, and `load()` calls).
#[derive(Debug, Clone)]
struct ExtResource {
    /// Godot resource type, e.g. "Script", "PackedScene", "Texture2D".
    res_type: String,
    /// `res://`-prefixed path.
    path: String,
}

/// An inline sub-resource collected for a `[sub_resource]` section.
///
/// `id` is its 1-based position in `TscnGenerator::sub_resources` (separate
/// from the ext_resource id space, matching Godot 4's `.tscn` format).
struct SubResourceRec {
    /// Godot resource type, e.g. "CapsuleShape2D".
    res_type: String,
    /// Cloned property list, rendered in a second pass once all ids are known.
    props: Vec<SceneProp>,
}

/// A flattened, ordered node ready for `[node ...]` emission.
struct FlatNode {
    name: String,
    /// `Some(type)` for typed nodes; `None` for instances (which use `instance=`).
    node_type: Option<String>,
    /// `parent` value, or empty for the scene root (no parent field emitted).
    parent: String,
    /// `[name = rendered-value, ...]` property lines.
    prop_lines: Vec<(String, String)>,
    /// If this node is an instance, the ext_resource id string it references.
    instance_id: Option<String>,
}

/// The scene generator. Collects resources during a first walk, then emits.
pub struct TscnGenerator {
    /// Ordered list of external resources (id = index + 1).
    ext_resources: Vec<ExtResource>,
    /// Dedup map: `res://` path → ext_resource id (1-based, as string).
    ext_path_to_id: std::collections::HashMap<String, String>,
    /// Ordered list of inline sub-resources (id = index + 1).
    sub_resources: Vec<SubResourceRec>,
    /// Map from a sub-resource's AST address → its 1-based id string.
    /// Lets node prop rendering reference a sub-resource by identity.
    sub_ptr_to_id: std::collections::HashMap<usize, String>,
}

impl TscnGenerator {
    pub fn new() -> Self {
        Self {
            ext_resources: Vec::new(),
            ext_path_to_id: std::collections::HashMap::new(),
            sub_resources: Vec::new(),
            sub_ptr_to_id: std::collections::HashMap::new(),
        }
    }

    /// Generate a `.tscn` string from a parsed scene declaration.
    pub fn generate(&mut self, scene: &SceneDecl) -> String {
        // Phase 1: collect external resources and inline sub-resources.
        // Script is always id 1 when present, then root props, instances and
        // load() calls are appended in document order.
        if let Some(script) = &scene.script {
            self.add_ext_resource("Script".into(), self.res_path(script));
        }
        self.collect_from_props(&scene.props);
        self.collect_ext_from_nodes(&scene.children);

        // Phase 2: flatten the node tree (assigns parent paths) and emit.
        let mut nodes = Vec::new();
        self.flatten_node(
            &scene.name.to_string(),
            Some(scene.node_type.to_string()),
            &scene.props,
            "",
            None,
            &mut nodes,
        );
        for child in &scene.children {
            self.flatten_children(child, ".", &mut nodes);
        }

        self.render(scene, &nodes)
    }

    // ---- resource collection ------------------------------------------------

    fn add_ext_resource(&mut self, res_type: String, path: String) -> String {
        if let Some(id) = self.ext_path_to_id.get(&path) {
            return id.clone();
        }
        let id = (self.ext_resources.len() + 1).to_string();
        self.ext_path_to_id.insert(path.clone(), id.clone());
        self.ext_resources.push(ExtResource { res_type, path });
        id
    }

    /// Normalise a path to a `res://` path (prepend if not already prefixed).
    fn res_path(&self, raw: &AutoStr) -> String {
        let s = raw.to_string();
        if s.starts_with("res://") || s.starts_with("uid://") || s.starts_with("user://") {
            s
        } else {
            format!("res://{}", s)
        }
    }

    /// Infer a Godot resource type from a `res://` path's extension.
    fn infer_type_from_path(path: &str) -> String {
        let lower = path.to_lowercase();
        let ext = lower.rsplit('.').next().unwrap_or("");
        match ext {
            "gd" => "Script",
            "png" | "jpg" | "jpeg" | "svg" | "webp" | "bmp" => "Texture2D",
            "wav" | "ogg" | "mp3" | "flac" => "AudioStream",
            "ttf" | "otf" => "FontFile",
            "tscn" => "PackedScene",
            "tres" => "Resource",
            _ => "Resource",
        }
        .into()
    }

    fn collect_ext_from_nodes(&mut self, nodes: &[SceneNode]) {
        for node in nodes {
            match node {
                SceneNode::Instance { path, .. } => {
                    let p = self.res_path(path);
                    self.add_ext_resource("PackedScene".into(), p);
                }
                SceneNode::Node { props, children, .. } => {
                    self.collect_from_props(props);
                    self.collect_ext_from_nodes(children);
                }
            }
        }
    }

    /// Walk a property list, collecting loads (ext) and inline sub-resources.
    fn collect_from_props(&mut self, props: &[SceneProp]) {
        for prop in props {
            self.collect_value(&prop.value);
        }
    }

    /// Dispatch collection by value kind.
    fn collect_value(&mut self, value: &SceneValue) {
        match value {
            SceneValue::Expr(e) => self.collect_loads(e),
            SceneValue::SubResource(sr) => self.add_sub_resource(sr),
        }
    }

    /// Register an inline sub-resource, assigning it a 1-based id (separate
    /// from ext_resource ids), then recurse into its own properties.
    fn add_sub_resource(&mut self, sr: &SceneSubResource) {
        let key = std::ptr::from_ref(sr) as *const SceneSubResource as usize;
        if self.sub_ptr_to_id.contains_key(&key) {
            return;
        }
        let id = (self.sub_resources.len() + 1).to_string();
        self.sub_ptr_to_id.insert(key, id);
        self.sub_resources.push(SubResourceRec {
            res_type: sr.res_type.to_string(),
            props: sr.props.clone(),
        });
        // Nested loads / sub-resources inside this sub-resource.
        self.collect_from_props(&sr.props);
    }

    /// Scan an expression tree for `load("res://...")` calls and register them.
    fn collect_loads(&mut self, expr: &Expr) {
        match expr {
            Expr::Call(call) => {
                if let Expr::Ident(name) = call.name.as_ref() {
                    if name.as_str() == "load" {
                        if let Some(crate::ast::Arg::Pos(Expr::Str(path))) = call.args.args.first() {
                            let p = self.res_path(path);
                            let t = Self::infer_type_from_path(&p);
                            self.add_ext_resource(t, p);
                        }
                    }
                }
                for arg in &call.args.args {
                    if let crate::ast::Arg::Pos(e) = arg {
                        self.collect_loads(e);
                    }
                }
            }
            _ => {}
        }
    }

    // ---- node flattening ----------------------------------------------------

    /// Push a single flattened node (root or a collected node).
    #[allow(clippy::too_many_arguments)]
    fn flatten_node(
        &mut self,
        name: &str,
        node_type: Option<String>,
        props: &[crate::ast::SceneProp],
        parent: &str,
        instance_id: Option<String>,
        out: &mut Vec<FlatNode>,
    ) {
        let prop_lines: Vec<(String, String)> = props
            .iter()
            .map(|p| (p.name.to_string(), self.render_scene_value(&p.value)))
            .collect();
        out.push(FlatNode {
            name: name.to_string(),
            node_type,
            parent: parent.to_string(),
            prop_lines,
            instance_id,
        });
    }

    /// Recurse into a child node, computing its parent path.
    fn flatten_children(&mut self, node: &SceneNode, parent: &str, out: &mut Vec<FlatNode>) {
        match node {
            SceneNode::Instance { name, path, .. } => {
                let p = self.res_path(path);
                let id = self
                    .ext_path_to_id
                    .get(&p)
                    .cloned()
                    .unwrap_or_else(|| "1".into());
                self.flatten_node(name, None, &[], parent, Some(id), out);
            }
            SceneNode::Node {
                node_type,
                name,
                props,
                children,
            } => {
                let effective_name = name
                    .clone()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| node_type.to_string());
                self.flatten_node(
                    &effective_name,
                    Some(node_type.to_string()),
                    props,
                    parent,
                    None,
                    out,
                );
                // Children of this node get a parent path relative to the root.
                let child_parent = if parent == "." {
                    effective_name.clone()
                } else {
                    format!("{}/{}", parent, effective_name)
                };
                for child in children {
                    self.flatten_children(child, &child_parent, out);
                }
            }
        }
    }

    // ---- value rendering ----------------------------------------------------

    /// Render a scene property value to its `.tscn` text form.
    fn render_scene_value(&self, value: &SceneValue) -> String {
        match value {
            SceneValue::Expr(e) => self.render_value(e),
            SceneValue::SubResource(sr) => {
                let key = std::ptr::from_ref(sr) as *const SceneSubResource as usize;
                let id = self
                    .sub_ptr_to_id
                    .get(&key)
                    .cloned()
                    .unwrap_or_else(|| "1".into());
                format!("SubResource(\"{}\")", id)
            }
        }
    }

    fn render_value(&self, expr: &Expr) -> String {
        match expr {
            Expr::Int(i) => i.to_string(),
            Expr::Uint(u) => u.to_string(),
            Expr::I64(i) => i.to_string(),
            Expr::U64(u) => u.to_string(),
            Expr::I8(i) => i.to_string(),
            Expr::U8(i) => i.to_string(),
            Expr::Byte(b) => b.to_string(),
            Expr::Float(_, text) | Expr::Double(_, text) => {
                let t = text.to_string();
                if t.contains('.') || t.contains('e') || t.contains('E') {
                    t
                } else {
                    // Godot prefers floats to carry a decimal point.
                    format!("{}.0", t)
                }
            }
            Expr::Bool(b) => {
                if *b {
                    "true".into()
                } else {
                    "false".into()
                }
            }
            Expr::Str(s) => format!("\"{}\"", escape_tscn_str(s)),
            Expr::Call(call) => self.render_call(call),
            // Fallback: best-effort literal rendering.
            _ => format!("\"{}\"", escape_tscn_str(&format!("{:?}", expr))),
        }
    }

    fn render_call(&self, call: &crate::ast::Call) -> String {
        if let Expr::Ident(name) = call.name.as_ref() {
            match name.as_str() {
                "load" => {
                    if let Some(crate::ast::Arg::Pos(Expr::Str(path))) = call.args.args.first() {
                        let p = if path.starts_with("res://") {
                            path.to_string()
                        } else {
                            format!("res://{}", path)
                        };
                        if let Some(id) = self.ext_path_to_id.get(&p) {
                            return format!("ExtResource(\"{}\")", id);
                        }
                    }
                    return "null".into();
                }
                // Constructor-style values are emitted verbatim: Vector2, Color, Rect2, ...
                _ => {
                    let args = call
                        .args
                        .args
                        .iter()
                        .filter_map(|a| match a {
                            crate::ast::Arg::Pos(e) => Some(self.render_value(e)),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    return format!("{}({})", name, args);
                }
            }
        }
        format!("\"{}\"", escape_tscn_str(&format!("{:?}", call)))
    }

    // ---- final rendering ----------------------------------------------------

    fn render(&self, scene: &SceneDecl, nodes: &[FlatNode]) -> String {
        let mut out = String::new();
        let load_steps = 1 + self.ext_resources.len() + self.sub_resources.len();

        // Header
        out.push_str(&format!(
            "[gd_scene load_steps={} format=3 uid=\"{}\"]\n",
            load_steps,
            make_uid(&scene.name.to_string())
        ));

        // External resources
        for (i, res) in self.ext_resources.iter().enumerate() {
            out.push_str(&format!(
                "\n[ext_resource type=\"{}\" path=\"{}\" id=\"{}\"]\n",
                res.res_type,
                res.path,
                i + 1
            ));
        }

        // Sub-resources (inline typed values, e.g. shapes, materials)
        for (i, sub) in self.sub_resources.iter().enumerate() {
            out.push_str(&format!(
                "\n[sub_resource type=\"{}\" id=\"{}\"]\n",
                sub.res_type,
                i + 1
            ));
            for prop in &sub.props {
                out.push_str(&format!(
                    "{} = {}\n",
                    prop.name,
                    self.render_scene_value(&prop.value)
                ));
            }
        }

        // Nodes
        for node in nodes {
            out.push('\n');
            // Header line
            if let Some(t) = &node.node_type {
                if node.parent.is_empty() {
                    out.push_str(&format!("[node name=\"{}\" type=\"{}\"]\n", node.name, t));
                } else {
                    out.push_str(&format!(
                        "[node name=\"{}\" type=\"{}\" parent=\"{}\"]\n",
                        node.name, t, node.parent
                    ));
                }
            } else if let Some(id) = &node.instance_id {
                out.push_str(&format!(
                    "[node name=\"{}\" parent=\"{}\" instance=ExtResource(\"{}\")]\n",
                    node.name, node.parent, id
                ));
            }
            // Properties
            for (k, v) in &node.prop_lines {
                out.push_str(&format!("{} = {}\n", k, v));
            }
            // Script attachment (root only, when present)
            if node.parent.is_empty() && scene.script.is_some() {
                let id = self
                    .ext_path_to_id
                    .get(&self.res_path(scene.script.as_ref().unwrap()))
                    .cloned()
                    .unwrap_or_else(|| "1".into());
                out.push_str(&format!("script = ExtResource(\"{}\")\n", id));
            }
        }

        // Signal connections
        for conn in &scene.connections {
            out.push('\n');
            out.push_str(&format!(
                "[connection signal=\"{}\" from=\"{}\" to=\"{}\" method=\"{}\"]\n",
                conn.signal, conn.from, conn.to, conn.method
            ));
        }

        out
    }
}

/// Escape a string for inclusion in a double-quoted `.tscn` value.
fn escape_tscn_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

/// Deterministically derive a Godot-compatible `uid://` id from a scene name.
///
/// Uses FNV-1a over the name, encoded as 12 base32 chars (a-z, 2-7), so the
/// output is stable across runs and valid for Godot's uid format.
fn make_uid(name: &str) -> String {
    let mut hash: u64 = 0x811c9dc5;
    for b in name.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    const ALPHA: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";
    let mut out = String::with_capacity(12);
    for i in 0..12usize {
        let idx = ((hash >> (i * 5)) & 0x1f) as usize;
        out.push(ALPHA[idx] as char);
    }
    format!("uid://{}", out)
}

// ---- public entry points ----------------------------------------------------

/// Generate a `.tscn` string from a single scene declaration.
pub fn generate_scene(scene: &SceneDecl) -> String {
    let mut gen = TscnGenerator::new();
    gen.generate(scene)
}

/// Find the first `SceneDecl` in an AST and generate its `.tscn` text.
/// Returns `None` when the AST contains no scene declaration.
pub fn generate_from_ast(ast: &Code) -> Option<String> {
    for stmt in &ast.stmts {
        if let Stmt::SceneDecl(scene) = stmt {
            return Some(generate_scene(scene));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use crate::trans::gdscript::GDScriptTrans;
    use crate::trans::{Sink, Trans};
    use std::fs::read_to_string;
    use std::path::PathBuf;

    /// Parse `test/a2gd/tscn/{case}/{name}.at`, generate `.tscn`, and compare
    /// against `{case}/{name}.expected.tscn`. Reuses the a2gd test layout.
    fn test_a2tscn(case: &str) -> Result<(), Box<dyn std::error::Error>> {
        let last_segment = case.rsplit('/').next().unwrap_or(case);
        let parts: Vec<&str> = last_segment.split('_').collect();
        let name = parts[1..].join("_");

        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = d.join(format!("test/a2gd/tscn/{}/{}.at", case, name));
        let src = read_to_string(&src_path)?;

        let mut parser = Parser::from(src.as_str());
        let ast = parser.parse()?;
        let generated = generate_from_ast(&ast).ok_or("no SceneDecl found in input")?;

        let expected_path = d.join(format!("test/a2gd/tscn/{}/{}.expected.tscn", case, name));
        let wrong_path = d.join(format!("test/a2gd/tscn/{}/{}.wrong.tscn", case, name));

        let expected = match read_to_string(&expected_path) {
            Ok(content) => content,
            Err(_) => {
                // Bootstrap: expected file doesn't exist yet. Write the generated
                // output to .wrong.tscn for review, then copy to .expected.tscn.
                std::fs::write(&wrong_path, &generated)?;
                panic!(
                    "expected file missing for {} — wrote generated output to {} for review",
                    case,
                    wrong_path.display()
                );
            }
        };

        if generated != expected {
            std::fs::write(&wrong_path, &generated)?;
            panic!(
                "tscn mismatch for {}\n--- expected ---\n{}\n--- generated ---\n{}\n(wrote {})",
                case, expected, generated, wrong_path.display()
            );
        }
        // Touch the sink import so it is not flagged unused across configurations.
        let _ = Sink::new("tscn".into());
        Ok(())
    }

    #[test]
    fn test_tscn_001_hello() {
        test_a2tscn("001_hello").unwrap();
    }

    #[test]
    fn test_tscn_002_player() {
        test_a2tscn("002_player").unwrap();
    }

    #[test]
    fn test_tscn_003_timers() {
        test_a2tscn("003_timers").unwrap();
    }

    #[test]
    fn test_tscn_004_nested() {
        test_a2tscn("004_nested").unwrap();
    }

    #[test]
    fn test_tscn_005_subresource() {
        test_a2tscn("005_subresource").unwrap();
    }

    // Plan 308: reverse-translated Godot demo scenes.
    #[test]
    fn test_godot_demo_instancing_ball_scene() {
        test_a2tscn("godot_demos/instancing/001_ball").unwrap();
    }

    /// Plan 306 Phase 2b: one .at file carries both a `scene` (→ .tscn) and
    /// functions (→ .gd). The .tscn comes from the scene; the GDScript pass
    /// must skip the SceneDecl without erroring and keep the functions.
    #[test]
    fn test_tscn_006_combined_with_gd() {
        let src = r#"
scene Counter : Control {
    script = "counter.gd"
    node Label "Count" { text = "0" }
}

fn increment(n int) int { n + 1 }
"#;
        let mut parser = Parser::from(src);
        let ast = parser.parse().unwrap();

        // .tscn side: scene present with its script reference.
        let tscn = generate_from_ast(&ast).expect("scene present");
        assert!(
            tscn.contains(r#"[node name="Counter" type="Control"]"#),
            "tscn root node: {}",
            tscn
        );
        assert!(
            tscn.contains(r#"script = ExtResource("1")"#),
            "tscn script ref: {}",
            tscn
        );

        // .gd side: GDScriptTrans consumes the moved AST and skips SceneDecl.
        let mut sink = Sink::new("counter".into());
        let mut trans = GDScriptTrans::new("counter".into());
        trans
            .trans(ast, &mut sink)
            .expect("gd transpile must skip SceneDecl");
        let gd = String::from_utf8_lossy(sink.done().unwrap()).into_owned();
        assert!(gd.contains("func increment"), "gd keeps functions: {}", gd);
        assert!(
            !gd.to_lowercase().contains("scene"),
            "gd must not leak the scene declaration: {}",
            gd
        );
        // The script extends the scene's root node type, not the Node default.
        assert!(
            gd.contains("extends Control"),
            "gd extends scene root type: {}",
            gd
        );
    }

    #[test]
    fn test_tscn_uid_deterministic() {
        assert_eq!(make_uid("Player"), make_uid("Player"));
        let uid = make_uid("Player");
        assert!(uid.starts_with("uid://"));
        assert_eq!(uid.len(), "uid://".len() + 12);
    }
}

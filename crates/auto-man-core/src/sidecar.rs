//! Sidecar 声明解析纯规则（PLAN-809 R2-F02 / T01 contract §6.4）。
//!
//! `rust_sidecar { modules: [...]; deps: [...] }` 的节点/obj/嵌套三形态
//! 提形与条目校验（合法 Rust ident、`C:/` 路径防误拆）单源于此；
//! AutoConfig 求值与文件复制/Cargo 注入留 auto-man adapter（sidecar.rs）。

use auto_val::{Node, Value};

/// 声明解析结果（字符串形态；路径→PathBuf 由 adapter 转）。
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SidecarDecl {
    /// `<mod名>:<工程相对源路径>`
    pub modules: Vec<(String, String)>,
    /// `<crate>:<版本>`
    pub deps: Vec<(String, String)>,
}

impl SidecarDecl {
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty() && self.deps.is_empty()
    }
}

/// 从求值 root 提取 rust_sidecar 声明（缺省空——生成流程不因侧车
/// 缺省分叉）。主形态：root 的具名子节点；兜底：obj/嵌套 value 形态。
pub fn sidecar_decl_from_root(root: &Node) -> SidecarDecl {
    let (modules_v, deps_v) = match root.get_nodes("rust_sidecar").into_iter().next() {
        Some(node) => (node.get_prop("modules"), node.get_prop("deps")),
        None => match root.get_prop("rust_sidecar") {
            Value::Obj(obj) => (
                obj.get("modules").map(|v| v.clone()).unwrap_or(Value::Nil),
                obj.get("deps").map(|v| v.clone()).unwrap_or(Value::Nil),
            ),
            Value::Node(node) => (node.get_prop("modules"), node.get_prop("deps")),
            _ => (Value::Nil, Value::Nil),
        },
    };
    SidecarDecl {
        modules: parse_modules(&modules_v),
        deps: parse_deps(&deps_v),
    }
}

/// modules 彐形态解析（ident 校验与原文逐字：首字符**小写**字母或 '_'，
/// 防 "C:/..." 被误拆）。
fn parse_modules(v: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in str_list(v) {
        if let Some((name, path)) = entry.split_once(':') {
            let name = name.trim();
            let valid_ident = name
                .chars()
                .next()
                .map(|c| c.is_ascii_lowercase() || c == '_')
                .unwrap_or(false)
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            if valid_ident && !path.trim().is_empty() {
                out.push((name.to_string(), path.trim().to_string()));
            }
        }
    }
    out
}

/// deps 彐形态解析（含 PLAN-025 `name:path:REL` → `{ path = "REL" }`
/// path 依赖展开）。
fn parse_deps(v: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in str_list(v) {
        if let Some((name, version)) = entry.split_once(':') {
            let name = name.trim();
            if !name.is_empty() && !version.trim().is_empty() {
                if let Some(rel) = version.strip_prefix("path:") {
                    let rel = rel.trim();
                    if !rel.is_empty() {
                        out.push((name.to_string(), format!("{{ path = \"{rel}\" }}")));
                        continue;
                    }
                }
                out.push((name.to_string(), version.trim().to_string()));
            }
        }
    }
    out
}

fn str_list(v: &Value) -> Vec<String> {
    match v {
        Value::Array(arr) => arr
            .values
            .iter()
            .map(|s| s.to_astr().trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        Value::Str(s) => {
            let s = s.trim();
            (!s.is_empty())
                .then(|| vec![s.to_string()])
                .unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node_with(module_line: &str, dep_line: &str) -> Node {
        let mut n = Node::new("rust_sidecar");
        n.set_prop(
            "modules",
            Value::str_array(vec![AutoStr::from(module_line)]),
        );
        n.set_prop("deps", Value::str_array(vec![AutoStr::from(dep_line)]));
        let mut root = Node::new("root");
        root.add_kid(n);
        root
    }

    #[test]
    fn node_form_parse_and_ident_guard() {
        let d = sidecar_decl_from_root(&node_with("term:term.rs", "libloading:0.8"));
        assert_eq!(d.modules, vec![("term".to_string(), "term.rs".to_string())]);
        assert_eq!(d.deps, vec![("libloading".to_string(), "0.8".to_string())]);
        // C:/ 路径不被误拆（首字符大写非合法 ident）
        let d2 = sidecar_decl_from_root(&node_with("C:/abs/path.rs", "x:1"));
        assert!(d2.modules.is_empty(), "非 ident 名拒收");
        // PLAN-025 path:REL 展开
        let d3 = sidecar_decl_from_root(&node_with("m:m.rs", "local:path:../rel"));
        assert_eq!(
            d3.deps,
            vec![("local".to_string(), "{ path = \"../rel\" }".to_string())]
        );
        // 缺省空
        let empty = Node::new("root");
        assert!(sidecar_decl_from_root(&empty).is_empty());
        // R3-02：Block 列表形态与 Array 同值对拍（旧支持域保留）
        let mut nb = Node::new("rust_sidecar");
        nb.set_prop(
            "modules",
            Value::Block(auto_val::Array::from_vec(vec![auto_val::Value::Str(
                AutoStr::from("m:extra.rs"),
            )])),
        );
        let mut root_b = Node::new("root");
        root_b.add_kid(nb);
        let db = sidecar_decl_from_root(&root_b);
        assert_eq!(
            db.modules,
            vec![("m".to_string(), "extra.rs".to_string())],
            "Block 形态与 Array 同值同果"
        );
    }
}

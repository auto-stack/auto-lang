//! # autodown_natives — `autodown_*` VM natives（plan 019 批次八，Phase 3）
//!
//! 让 `.at` handler 可编程操作 AutoDown 文档：parse / serialize / 取文本 /
//! 查块 / Op 级编辑（applyOp）/ 模板插入——全部由 autodown-core crate 支撑
//! （feature `autodown`；无 feature 时 natives 存在但返回构建错误，保持
//! catalog 表稳定）。
//!
//! 传输形态循 `auto.fs.read_dir` 先例：复杂结构以 **JSON 字符串** 过界，
//! `.at` 侧经 `json.*` natives 导航。文档 JSON 形状（`block_to_json`）：
//!
//! ```json
//! {
//!   "id": "block-0",
//!   "kind": "Heading",                       // BlockType 变体名
//!   "attrs": [["level", {"Int": 1}]],        // 顺序稳定的 [k, v] 对
//!   "inlines": [{"text": "t", "marks": ["Strong"], "attrs": [["href", {"Str": "..."}]]}],
//!   "children": [ ... ]
//! }
//! ```
//!
//! crate `Value` 的 JSON 形态：`null`（Null）/ `{"Str": s}` / `{"Int": i}` /
//! `{"Bool": b}` / `{"List": [...]}` / `{"Attrs": [[k, v]...]}`。

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::task::AutoTask;

// ---------------------------------------------------------------- shims
// 栈序：参数自右向左弹出（与 shim_shell_exec_submit 同款约定）。

/// `autodown_parse(src, final) -> json`：markdown → 块树 JSON。
pub fn shim_autodown_parse(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let is_final = crate::vm::native::pop_arg_i32(task);
    let _stake_final = crate::vm::native::StakeGuard::new(vm, is_final as i64 as u64);
    let src = pop_string_arg(task, vm);
    let json = parse_to_json(&src, is_final != 0).map_err(VMError::RuntimeError)?;
    push_string(task, vm, &json)
}

/// `autodown_serialize(json, emit_ids) -> src`：块树 JSON → .ad 文本。
pub fn shim_autodown_serialize(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let emit_ids = crate::vm::native::pop_arg_i32(task);
    let _stake_emit = crate::vm::native::StakeGuard::new(vm, emit_ids as i64 as u64);
    let json = pop_string_arg(task, vm);
    let src = serialize_from_json(&json, emit_ids != 0).map_err(VMError::RuntimeError)?;
    push_string(task, vm, &src)
}

/// `autodown_text(json, block_id) -> str`：块内联文本（spansText）。
pub fn shim_autodown_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let block_id = pop_string_arg(task, vm);
    let json = pop_string_arg(task, vm);
    let text = text_from_json(&json, &block_id).map_err(VMError::RuntimeError)?;
    push_string(task, vm, &text)
}

/// `autodown_find_block(json, block_id) -> bool`。
pub fn shim_autodown_find_block(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let block_id = pop_string_arg(task, vm);
    let json = pop_string_arg(task, vm);
    let found = find_in_json(&json, &block_id).map_err(VMError::RuntimeError)?;
    task.ram.push_nv(auto_val::encode_bool(found));
    Ok(())
}

/// `autodown_insert_text(json, block_id, offset, text) -> json`：
/// Op::InsertText 经 applyOp（树函数式更新，返回新树 JSON）。
pub fn shim_autodown_insert_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let text = pop_string_arg(task, vm);
    let offset = crate::vm::native::pop_arg_i32(task);
    let _stake_off = crate::vm::native::StakeGuard::new(vm, offset as i64 as u64);
    let block_id = pop_string_arg(task, vm);
    let json = pop_string_arg(task, vm);
    let out = insert_text_json(&json, &block_id, offset as i64, &text)
        .map_err(VMError::RuntimeError)?;
    push_string(task, vm, &out)
}

/// `autodown_insert_template(json, md, parent_id, index) -> json`：
/// 模板 markdown 解析为块序列，拼入 parent（空串 = 文档顶层）children[index]
/// （index 越界钳到末尾，负数 = 追加）。
pub fn shim_autodown_insert_template(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let index = crate::vm::native::pop_arg_i32(task);
    let _stake_idx = crate::vm::native::StakeGuard::new(vm, index as i64 as u64);
    let parent_id = pop_string_arg(task, vm);
    let md = pop_string_arg(task, vm);
    let json = pop_string_arg(task, vm);
    let out = insert_template_json(&json, &md, &parent_id, index as i64)
        .map_err(VMError::RuntimeError)?;
    push_string(task, vm, &out)
}

fn pop_string_arg(task: &mut AutoTask, vm: &AutoVM) -> String {
    let idx = task.ram.pop_str_idx() as u32;
    vm.get_string(idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default()
}

fn push_string(task: &mut AutoTask, vm: &AutoVM, s: &str) -> Result<(), VMError> {
    let idx = vm.add_string(s.as_bytes().to_vec());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

// ------------------------------------------------- 实现（feature 门控）

#[cfg(feature = "autodown")]
mod impls {
    use autodown_core::block_model::{
        applyOp, blockFull, collapsedSel, findBlock, rng, spansText, Attr, BlockNode, BlockType,
        InlineSpan, Mark, Value,
    };
    use autodown_core::serializer::serialize;
    use autodown_core::markdown_parser::parse_blocks;
    use serde_json::{json, Value as J};

    pub fn parse_to_json(src: &str, is_final: bool) -> Result<String, String> {
        let root = parse_blocks(src, is_final);
        serde_json::to_string(&block_to_json(&root)).map_err(|e| e.to_string())
    }

    pub fn serialize_from_json(doc: &str, emit_ids: bool) -> Result<String, String> {
        let root = block_from_doc(doc)?;
        Ok(serialize(root, emit_ids))
    }

    pub fn text_from_json(doc: &str, block_id: &str) -> Result<String, String> {
        let root = block_from_doc(doc)?;
        match findBlock(root, block_id) {
            Some(b) => Ok(spansText(b.inlines)),
            None => Err(format!("autodown_text: block '{block_id}' not found")),
        }
    }

    pub fn find_in_json(doc: &str, block_id: &str) -> Result<bool, String> {
        let root = block_from_doc(doc)?;
        Ok(findBlock(root, block_id).is_some())
    }

    pub fn insert_text_json(
        doc: &str,
        block_id: &str,
        offset: i64,
        text: &str,
    ) -> Result<String, String> {
        let root = block_from_doc(doc)?;
        if findBlock(root.clone(), block_id).is_none() {
            return Err(format!("autodown_insert_text: block '{block_id}' not found"));
        }
        let op = autodown_core::block_model::Op::InsertText(
            autodown_core::block_model::InsertTextOp {
                pos: autodown_core::block_model::BlockPos {
                    blockId: block_id.to_string(),
                    offset,
                },
                text: text.to_string(),
            },
        );
        let result = applyOp(root, collapsedSel(block_id, offset), op);
        serde_json::to_string(&block_to_json(&result.tree)).map_err(|e| e.to_string())
    }

    pub fn insert_template_json(
        doc: &str,
        md: &str,
        parent_id: &str,
        index: i64,
    ) -> Result<String, String> {
        let mut root = block_from_doc(doc)?;
        let frags = parse_blocks(md, true).children;
        if frags.is_empty() {
            return Err("autodown_insert_template: template parsed to no blocks".into());
        }
        let parent = if parent_id.is_empty() {
            &mut root
        } else {
            find_mut(&mut root, parent_id).ok_or_else(|| {
                format!("autodown_insert_template: parent '{parent_id}' not found")
            })?
        };
        let idx = if index < 0 {
            parent.children.len()
        } else {
            (index as usize).min(parent.children.len())
        };
        for (i, f) in frags.into_iter().enumerate() {
            parent.children.insert(idx + i, f);
        }
        serde_json::to_string(&block_to_json(&root)).map_err(|e| e.to_string())
    }

    fn find_mut<'a>(node: &'a mut BlockNode, id: &str) -> Option<&'a mut BlockNode> {
        if node.id == id {
            return Some(node);
        }
        for c in node.children.iter_mut() {
            if let Some(found) = find_mut(c, id) {
                return Some(found);
            }
        }
        None
    }

    fn block_from_doc(doc: &str) -> Result<BlockNode, String> {
    let j: J = serde_json::from_str(doc).map_err(|e| format!("autodown: bad document json: {e}"))?;
    block_from_json(&j).ok_or_else(|| "autodown: malformed document json (block shape)".to_string())
}

fn parse_json(doc: &str) -> Result<J, String> {
        serde_json::from_str(doc).map_err(|e| format!("autodown: bad document json: {e}"))
    }

    // ---------------------------------------------------------- 封送

    fn kind_name(k: &BlockType) -> String {
        k.to_string()
    }

    fn kind_from_name(s: &str) -> Option<BlockType> {
        Some(match s {
            "Heading" => BlockType::Heading,
            "Paragraph" => BlockType::Paragraph,
            "Fence" => BlockType::Fence,
            "Blockquote" => BlockType::Blockquote,
            "ListBlock" => BlockType::ListBlock,
            "ListItem" => BlockType::ListItem,
            "Table" => BlockType::Table,
            "TableRow" => BlockType::TableRow,
            "TableCell" => BlockType::TableCell,
            "ThematicBreak" => BlockType::ThematicBreak,
            "Callout" => BlockType::Callout,
            "Details" => BlockType::Details,
            "WikilinkBlock" => BlockType::WikilinkBlock,
            "QueryBlock" => BlockType::QueryBlock,
            "BlockEmbed" => BlockType::BlockEmbed,
            "Mermaid" => BlockType::Mermaid,
            "MathBlock" => BlockType::MathBlock,
            _ => return None,
        })
    }

    fn mark_name(m: &Mark) -> &'static str {
        match m {
            Mark::Strong => "Strong",
            Mark::Em => "Em",
            Mark::Code => "Code",
            Mark::Link => "Link",
            Mark::Image => "Image",
            Mark::Del => "Del",
        }
    }

    fn mark_from_name(s: &str) -> Option<Mark> {
        Some(match s {
            "Strong" => Mark::Strong,
            "Em" => Mark::Em,
            "Code" => Mark::Code,
            "Link" => Mark::Link,
            "Image" => Mark::Image,
            "Del" => Mark::Del,
            _ => return None,
        })
    }

    fn value_to_json(v: &Value) -> J {
        match v {
            Value::Null => J::Null,
            Value::Str(s) => json!({"Str": s}),
            Value::Int(i) => json!({"Int": i}),
            Value::Bool(b) => json!({"Bool": b}),
            Value::ListV(l) => {
                json!({"List": l.iter().map(value_to_json).collect::<Vec<_>>()})
            }
            Value::AttrsV(a) => json!({"Attrs": attrs_to_json(a)}),
        }
    }

    fn value_from_json(j: &J) -> Option<Value> {
        match j {
            J::Null => Some(Value::Null),
            J::Object(m) => match m.iter().next() {
                Some((k, v)) => match k.as_str() {
                    "Str" => v.as_str().map(|s| Value::Str(s.to_string())),
                    "Int" => v.as_i64().map(Value::Int),
                    "Bool" => v.as_bool().map(Value::Bool),
                    "List" => {
                        let mut out = Vec::new();
                        for e in v.as_array()? {
                            out.push(value_from_json(e)?);
                        }
                        Some(Value::ListV(out))
                    }
                    "Attrs" => match attrs_from_json(v)? {
                        Value::AttrsV(a) => Some(Value::AttrsV(a)),
                        _ => None,
                    },
                    _ => None,
                },
                None => Some(Value::Null),
            },
            _ => None,
        }
    }

    fn attrs_to_json(attrs: &[Attr]) -> J {
        J::Array(
            attrs
                .iter()
                .map(|a| json!([a.key, value_to_json(&a.value)]))
                .collect(),
        )
    }

    fn attrs_from_json(j: &J) -> Option<Value> {
        let mut out = Vec::new();
        for pair in j.as_array()? {
            let p = pair.as_array()?;
            let key = p.first()?.as_str()?.to_string();
            let value = value_from_json(p.get(1)?)?;
            out.push(Attr { key, value });
        }
        Some(Value::AttrsV(out))
    }

    fn attrs_vec_from_json(j: &J) -> Option<Vec<Attr>> {
        match attrs_from_json(j)? {
            Value::AttrsV(a) => Some(a),
            _ => None,
        }
    }

    fn span_to_json(s: &InlineSpan) -> J {
        json!({
            "text": s.text,
            "marks": s.marks.iter().map(|m| mark_name(m)).collect::<Vec<_>>(),
            "attrs": attrs_to_json(&s.attrs),
        })
    }

    fn span_from_json(j: &J) -> Option<InlineSpan> {
        let o = j.as_object()?;
        let text = o.get("text")?.as_str()?.to_string();
        let mut marks = Vec::new();
        for m in o.get("marks").and_then(|v| v.as_array()).unwrap_or(&vec![]) {
            marks.push(mark_from_name(m.as_str()?)?);
        }
        let attrs = o
            .get("attrs")
            .and_then(attrs_vec_from_json)
            .unwrap_or_default();
        Some(InlineSpan { text, marks, attrs })
    }

    fn block_to_json(b: &BlockNode) -> J {
        json!({
            "id": b.id,
            "kind": kind_name(&b.kind),
            "attrs": attrs_to_json(&b.attrs),
            "inlines": b.inlines.iter().map(span_to_json).collect::<Vec<_>>(),
            "children": b.children.iter().map(block_to_json).collect::<Vec<_>>(),
        })
    }

    fn block_from_json(j: &J) -> Option<BlockNode> {
        let o = j.as_object()?;
        let id = o.get("id")?.as_str()?.to_string();
        let kind = kind_from_name(o.get("kind")?.as_str()?)?;
        let attrs = o.get("attrs").and_then(attrs_vec_from_json).unwrap_or_default();
        let mut inlines = Vec::new();
        if let Some(arr) = o.get("inlines").and_then(|v| v.as_array()) {
            for s in arr {
                inlines.push(span_from_json(s)?);
            }
        }
        let mut children = Vec::new();
        if let Some(arr) = o.get("children").and_then(|v| v.as_array()) {
            for c in arr {
                children.push(block_from_json(c)?);
            }
        }
        // SourceRange 零占位（parser 不产位置信息，与 crate 发射物同构）。
        Some(blockFull(&id, kind, attrs, children, inlines, rng(0, 0)))
    }
}

// 桩/实现切换：无 feature 时返回构建错误（natives 始终在表中可解析）。
macro_rules! dispatch {
    ($fname:ident($($arg:ident: $ty:ty),*) -> $retty:ty, $label:expr) => {
        #[cfg(feature = "autodown")]
        pub fn $fname($($arg: $ty),*) -> Result<$retty, String> {
            impls::$fname($($arg),*)
        }
        #[cfg(not(feature = "autodown"))]
        pub fn $fname($($arg: $ty),*) -> Result<$retty, String> {
            #[allow(unused_variables)]
            fn inner($($arg: $ty),*) -> Result<$retty, String> {
                Err($label.to_string())
            }
            inner($($arg),*)
        }
    };
}

dispatch!(parse_to_json(src: &str, is_final: bool) -> String,
    "built without the `autodown` feature: autodown_parse unavailable");
dispatch!(serialize_from_json(doc: &str, emit_ids: bool) -> String,
    "built without the `autodown` feature: autodown_serialize unavailable");
dispatch!(text_from_json(doc: &str, block_id: &str) -> String,
    "built without the `autodown` feature: autodown_text unavailable");
dispatch!(find_in_json(doc: &str, block_id: &str) -> bool,
    "built without the `autodown` feature: autodown_find_block unavailable");
dispatch!(insert_text_json(doc: &str, block_id: &str, offset: i64, text: &str) -> String,
    "built without the `autodown` feature: autodown_insert_text unavailable");
dispatch!(insert_template_json(doc: &str, md: &str, parent_id: &str, index: i64) -> String,
    "built without the `autodown` feature: autodown_insert_template unavailable");

#[cfg(all(test, feature = "autodown"))]
mod tests {
    use super::*;

    #[test]
    fn parse_serialize_roundtrip_via_json() {
        let src = "# 标题\n\n段落 **粗** 与 `码`\n\n- 甲\n- 乙\n";
        let doc = parse_to_json(src, true).unwrap();
        // JSON → BlockNode → serialize 与 crate 直接 parse+serialize 逐字节一致
        let out = serialize_from_json(&doc, false).unwrap();
        let direct = {
            use autodown_core::serializer::serialize;
            serialize(autodown_core::markdown_parser::parse_blocks(src, true), false)
        };
        assert_eq!(out, direct);
        assert_eq!(out, src);
    }

    #[test]
    fn text_and_find_block() {
        let doc = parse_to_json("# Hello\n\nworld\n", true).unwrap();
        assert!(find_in_json(&doc, "block-0").unwrap());
        assert!(!find_in_json(&doc, "no-such").unwrap());
        assert_eq!(text_from_json(&doc, "block-0").unwrap(), "Hello");
        assert!(text_from_json(&doc, "no-such").is_err());
    }

    #[test]
    fn insert_text_applies_op() {
        let doc = parse_to_json("abc\n", true).unwrap();
        let out = insert_text_json(&doc, "block-0", 1, "X").unwrap();
        assert_eq!(text_from_json(&out, "block-0").unwrap(), "aXbc");
        assert_eq!(serialize_from_json(&out, false).unwrap(), "aXbc\n");
        // 块不存在 → 错误
        assert!(insert_text_json(&doc, "nope", 0, "x").is_err());
    }

    #[test]
    fn insert_template_splices_children() {
        let doc = parse_to_json("# 头\n\n尾\n", true).unwrap();
        // 顶层 index 1 插入两个块
        let out = insert_template_json(&doc, "## 中甲\n\n## 中乙\n", "", 1).unwrap();
        let ser = serialize_from_json(&out, false).unwrap();
        assert_eq!(ser, "# 头\n\n## 中甲\n\n## 中乙\n\n尾\n");
        // 追加（index = -1）+ 不存在父块报错
        let out2 = insert_template_json(&out, "尾二\n", "", -1).unwrap();
        assert!(serialize_from_json(&out2, false).unwrap().ends_with("尾二\n"));
        assert!(insert_template_json(&out, "x\n", "nope", 0).is_err());
    }
}

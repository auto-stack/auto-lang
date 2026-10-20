//! PLAN-814 R3-04：autodown 文档引擎实装——跨仓 autodown-core 依赖与
//! `autodown` feature 同 owner 在本包；真实实现经 SDK
//! `vm::autodown_natives` 的进程默认引擎端口注册（ natives 的
//! dispatch 臂在槽未设时触发 UI 宿主装配钩子回调本包安装器）。
//! 模块整体 cfg(feature = "autodown")（ui/mod.rs 声明处门控）。

use auto_lang_core::vm::autodown_natives::AutodownEngine;
use autodown_core::block_model::{
    applyOp, blockFull, collapsedSel, findBlock, rng, spansText, Attr, BlockNode, BlockType,
    InlineSpan, Mark, Value,
};
use autodown_core::markdown_parser::parse_blocks;
use autodown_core::serializer::serialize;
use serde_json::{json, Value as J};

/// autodown-core 实装引擎（原 SDK vm/autodown_natives.rs::impls 逐字
/// 迁出，PLAN-814 T-02 迁移勘误：跨仓依赖不进 SDK）。
pub struct AutodownCoreEngine;

impl AutodownEngine for AutodownCoreEngine {
    fn parse_to_json(&self, src: &str, is_final: bool) -> Result<String, String> {
        let root = parse_blocks(src, is_final);
        serde_json::to_string(&block_to_json(&root)).map_err(|e| e.to_string())
    }

    fn serialize_from_json(&self, doc: &str, emit_ids: bool) -> Result<String, String> {
        let root = block_from_doc(doc)?;
        Ok(serialize(root, emit_ids))
    }

    fn text_from_json(&self, doc: &str, block_id: &str) -> Result<String, String> {
        let root = block_from_doc(doc)?;
        match findBlock(root, block_id) {
            Some(b) => Ok(spansText(b.inlines)),
            None => Err(format!("autodown_text: block '{block_id}' not found")),
        }
    }

    fn find_in_json(&self, doc: &str, block_id: &str) -> Result<bool, String> {
        let root = block_from_doc(doc)?;
        Ok(findBlock(root, block_id).is_some())
    }

    fn insert_text_json(
        &self,
        doc: &str,
        block_id: &str,
        offset: i64,
        text: &str,
    ) -> Result<String, String> {
        let root = block_from_doc(doc)?;
        if findBlock(root.clone(), block_id).is_none() {
            return Err(format!(
                "autodown_insert_text: block '{block_id}' not found"
            ));
        }
        let op =
            autodown_core::block_model::Op::InsertText(autodown_core::block_model::InsertTextOp {
                pos: autodown_core::block_model::BlockPos {
                    blockId: block_id.to_string(),
                    offset,
                },
                text: text.to_string(),
            });
        let result = applyOp(root, collapsedSel(block_id, offset), op);
        serde_json::to_string(&block_to_json(&result.tree)).map_err(|e| e.to_string())
    }

    fn insert_template_json(
        &self,
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

    /// 编辑壳全文（code-editor 面缺席 → None=空串语义保持）。
    fn editor_text(&self, key: &str) -> Option<String> {
        #[cfg(feature = "code-editor")]
        {
            crate::ui::autodown_editor::autodown_editor_text(key)
        }
        #[cfg(not(feature = "code-editor"))]
        {
            None
        }
    }
}

// ---- 私有 helper（文件域；trait 方法共用——原 impls 内私有面）----
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
    let j: J =
        serde_json::from_str(doc).map_err(|e| format!("autodown: bad document json: {e}"))?;
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
        Mark::Underline => "Underline",
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
        "Underline" => Mark::Underline,
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
    let attrs = o
        .get("attrs")
        .and_then(attrs_vec_from_json)
        .unwrap_or_default();
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

/// 注册进 SDK 进程默认引擎槽（幂等覆盖：显式注册权威，779 同形）。
pub fn install() {
    auto_lang_core::vm::autodown_natives::set_autodown_engine(std::sync::Arc::new(
        AutodownCoreEngine,
    ));
}

#[cfg(test)]
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
            serialize(
                autodown_core::markdown_parser::parse_blocks(src, true),
                false,
            )
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
        assert!(serialize_from_json(&out2, false)
            .unwrap()
            .ends_with("尾二\n"));
        assert!(insert_template_json(&out, "x\n", "nope", 0).is_err());
    }
}

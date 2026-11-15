//! PLAN-1058 V-01～V-06：AutoDown 混合语言增量解析与编辑定位原型测试。
//!
//! 语料出处（auto-down @ `5af11a39dd436acd9a25b523f675cec507819d2d`，只读）：
//! - `original_hardbreak_and_fence`：`packages/engine/src/rust/tests/golden/parity.ad`
//!   的 `hardbreak_and_fence` 分段逐字节保留（sha256 parity.ad =
//!   `01e920eddd2252049117d8de3e9f0a1dd511ec6ae3bb87fb3e338eeac1817686`）；
//! - 中文正文与带锚点标题派生自 `jade-garden/front/e2e/fixtures-pages/
//!   Cards Probe.ad`（sha256 `732fb665bb5b33810102ab2c9055c78190a3e043b66cd0558530069ba07aaf73`）；
//! - 带 Auto 围栏的文档为**派生例**（非原有文件），已用现版 parser 逐例确认
//!   （`D:/autostack/plan-artifacts/plan-1058/confirm_oracle.out`）。
//!
//! 手标 oracle：所有精确字节边界用 std 字符串搜索（`find`）标注在样例原文上，
//! 与被测实现独立；绝不用实现反推期望。
//!
//! 运行：`cargo test --manifest-path experimental/ace/Cargo.toml -p
//! ace-syntax-runtime --test autodown_mixed_editing`（固定组：`cargo t ace-syntax`）。

use ace_syntax_runtime::autodown::*;
use ace_syntax_runtime::snapshot::{Edit, EditRequest, PageStore, Snapshot};
use std::rc::Rc;

/// 真实 Auto 语法资产（experimental/ace/grammars/auto-core.at）。
const AUTO_CORE: &str = include_str!("../../../grammars/auto-core.at");

fn auto_grammar() -> Rc<AutoGrammar> {
    let p = ace_syntax_grammar::compile(AUTO_CORE).expect("auto-core.at must compile");
    let cs = ace_syntax_runtime::lexer::CompiledScanner::compile(&p)
        .expect("auto-core scanner must compile");
    Rc::new(AutoGrammar {
        program: Rc::new(p),
        scanner: cs,
    })
}

fn open_doc(text: &str) -> MixedDoc {
    MixedDoc::open(
        &Rc::new(PageStore::new()),
        FileId(1),
        text,
        Some(auto_grammar()),
    )
    .expect("open must succeed")
}

fn edit(doc: &mut MixedDoc, base: u32, start: u64, end: u64, insert: &str) -> u32 {
    doc.edit(&EditRequest {
        base_revision: base,
        edits: vec![Edit {
            start,
            end,
            insert: insert.as_bytes().to_vec(),
        }],
    })
    .expect("edit must succeed")
}

fn edit_force_full(doc: &mut MixedDoc, base: u32, start: u64, end: u64, insert: &str) -> u32 {
    doc.edit_force_full(&EditRequest {
        base_revision: base,
        edits: vec![Edit {
            start,
            end,
            insert: insert.as_bytes().to_vec(),
        }],
    })
    .expect("force-full edit must succeed")
}

/// 每步编辑后的等价性证明：增量结果 == 同输入从头解析（§6）。
fn assert_fresh_equivalent(doc: &MixedDoc) {
    let fresh = doc.fresh_analysis_of_current().expect("fresh analysis");
    let cur = doc.current();
    assert!(
        fresh.equivalent_to(cur),
        "incremental revision {} must equal fresh parse\nregions inc={:?}\nregions fresh={:?}",
        cur.revision,
        cur.regions
            .iter()
            .map(|r| (&r.kind, r.start, r.end))
            .collect::<Vec<_>>(),
        fresh
            .regions
            .iter()
            .map(|r| (&r.kind, r.start, r.end))
            .collect::<Vec<_>>(),
    );
}

// ---------------------------------------------------------------------------
// 样例（出处见文件头）
// ---------------------------------------------------------------------------

/// parity.ad `hardbreak_and_fence` 分段，逐字节保留。
const GOLDEN_SEGMENT: &str = "line one  \nline two\n\n```ts\nconst x = 1\n```\n";

/// 中文派生混合文档：真实标题（含锚点）+ 中文正文 + auto/ts 围栏 + 未装载
/// 标签的未闭合围栏（EOF）。
const DERIVED_MIX: &str = "# 探针清单 ^probe-list\n\n探针问题：1+1=? #card ^probe-c1\n\n\
```auto\nfn add(a: i32, b: i32) i32 {\n    return a + b\n}\n```\n\n\
```ts\nconst y = 2\n```\n\n```unknown-lang\nsome body\n";

/// V-02：两个独立 auto 围栏夹中文段落与标题。
const TWO_FENCES: &str = "# 标题 ^t1\n\n\
```auto\nfn add(a: i32, b: i32) i32 {\n    return a + b\n}\n```\n\n\
中间段落\n\n\
```auto\nfn main() int {\n    return add(1, 2)\n}\n```\n";

/// V-03：两个标题 + 两个 auto 围栏（重分类用）。
const RECLASSIFY: &str = "# 标题A ^a\n\n\
```auto\nfn add(a: i32, b: i32) i32 {\n    return a + b\n}\n```\n\n\
# 标题B ^b\n\n\
```auto\nfn main() int {\n    return add(1, 2)\n}\n```\n";

/// V-04：CRLF + 中文 + emoji 置于已标注 token（auto 围栏内 `fn`）之前。
const CRLF_EMOJI: &str = "# 标题 ^t\r\n\r\n前缀中文😀尾\r\n\r\n\
```auto\r\nfn add(a: i32) i32 { return a + 1 }\r\n```\r\n";

// ---------------------------------------------------------------------------
// V-01 基线
// ---------------------------------------------------------------------------

#[test]
fn v01_baseline_outline_labels_and_identity() {
    // --- 原始 golden 分段：ts 未装载，保留标签与正文；围栏边界手标核对 ---
    let mut doc = open_doc(GOLDEN_SEGMENT);
    let a = doc.current();
    assert_eq!(a.outline.len(), 0, "golden segment has no headings");
    assert_eq!(a.regions.len(), 2, "paragraph + ts fence");
    let fence = a.regions[1].fence().expect("second region is the ts fence");
    assert_eq!(fence.label, "ts");
    assert!(a.regions[1].embedded.is_none(), "ts is not loaded");
    // 手标：正文 = "const x = 1"，恰在两个标记行之间。
    let body_at = GOLDEN_SEGMENT.find("const x = 1").unwrap() as u64;
    assert_eq!(
        fence.content,
        (body_at, body_at + "const x = 1".len() as u64)
    );
    let close_at = GOLDEN_SEGMENT.rfind("```").unwrap() as u64;
    assert_eq!(fence.close, Some((close_at, close_at + 3)));
    // 现版 parser（confirm_oracle.out）确认 code 槽为 "const x = 1\n"：
    // 行界（含尾 \n）属父层是本层记录在案的边界裁定。
    assert_eq!(
        doc.snapshot()
            .read_str(fence.content.0, fence.content.1)
            .unwrap(),
        "const x = 1"
    );
    // 光标语言：ts 正文内 = 未装载围栏；闭栏行/行界 = 父层。
    assert_eq!(
        doc.language_at(0, body_at).unwrap(),
        LanguageAt::FenceBody {
            label: "ts".into(),
            loaded: false
        }
    );
    assert_eq!(
        doc.language_at(0, close_at).unwrap(),
        LanguageAt::Parent(ParentBlock::FenceMarker)
    );
    // 解析与查询不求值：ts 正文从未被子 parser 触碰（embedded None 已证）。

    // 在 ts 代码块内置入貌似标题的文本 → 大纲仍只含真实标题（此处为零）。
    let rev = edit(&mut doc, 0, body_at, body_at, "# not a heading\n");
    assert_eq!(rev, 1);
    assert_fresh_equivalent(&doc);
    assert_eq!(
        doc.current().outline.len(),
        0,
        "fake heading in code block is not a heading"
    );

    // --- 中文派生文档：真实标题唯一；auto 有子 token；未闭合围栏到 EOF ---
    let mut doc2 = open_doc(DERIVED_MIX);
    let a2 = doc2.current();
    assert_eq!(
        a2.outline.len(),
        1,
        "only the real heading is in the outline"
    );
    let h = &a2.outline[0];
    assert_eq!(
        (h.level, h.name.as_str(), h.anchor.as_deref()),
        (1, "探针清单", Some("probe-list"))
    );
    assert_eq!(h.start, DERIVED_MIX.find("# 探针清单").unwrap() as u64);

    let fences: Vec<&Region> = a2.regions.iter().filter(|r| r.fence().is_some()).collect();
    assert_eq!(fences.len(), 3, "auto + ts + unknown-lang fences");
    let (fauto, fts, funknown) = (fences[0], fences[1], fences[2]);
    assert_eq!(fauto.fence().unwrap().label, "auto");
    assert!(fauto.embedded.is_some(), "auto is loaded");
    assert_eq!(fts.fence().unwrap().label, "ts");
    assert!(fts.embedded.is_none());
    assert!(funknown.fence().unwrap().close.is_none(), "unclosed to EOF");
    assert_eq!(
        funknown.fence().unwrap().label,
        "unknown-lang",
        "unknown label kept verbatim"
    );
    // 未闭合围栏正文（现版：无尾随换行）。
    let ub = funknown.fence().unwrap().content;
    assert_eq!(
        doc2.snapshot().read_str(ub.0, ub.1).unwrap(),
        "some body",
        "unclosed fence body has no trailing newline"
    );

    // 已知 Auto 范围内有子 token：高亮来自真实 Auto 词法（fn 关键字等）。
    let fn_at = DERIVED_MIX.find("fn add").unwrap() as u64;
    let fn_hl = a2
        .highlights
        .iter()
        .find(|h| h.start == fn_at && h.kind == "fn")
        .expect("fn keyword highlighted inside the auto fence");
    assert_eq!(fn_hl.end - fn_hl.start, 2);
    assert!(
        a2.highlights.iter().any(|h| h.kind == "ident"),
        "ident tokens highlighted"
    );

    // auto 代码块内置入貌似标题 → 大纲不受污染；auto 子解析照常。
    let rev2 = edit(&mut doc2, 0, fn_at, fn_at, "# 卡片 #card\n");
    assert_eq!(rev2, 1);
    assert_fresh_equivalent(&doc2);
    assert_eq!(
        doc2.current().outline.len(),
        1,
        "fake heading inside auto fence excluded"
    );
    assert!(
        doc2.current().highlights.iter().any(|h| h.kind == "fn"),
        "auto tokens still highlighted"
    );

    // --- 身份：文件 + 基线摘要 + grammar 身份（§5.1）---
    let id = doc.identity();
    assert_eq!(id.file, FileId(1));
    assert!(!id.baseline_content_hash.is_empty());
    let expected_base = Snapshot::from_text(&(Rc::new(PageStore::new())), GOLDEN_SEGMENT)
        .id()
        .as_str()
        .to_string();
    assert_eq!(
        id.baseline_content_hash, expected_base,
        "baseline hash is content-bound"
    );
    assert_eq!(
        id.auto_grammar_id.as_deref(),
        Some(auto_grammar().program.grammar_id().as_str()),
        "grammar identity recorded"
    );

    // --- 空正文围栏的父子边界（§5.2 显式裁定；F4 回归钉住）---
    let empty = open_doc("# 标题 ^t\n\n```auto\n```\n");
    let f = empty
        .current()
        .regions
        .iter()
        .find_map(|r| r.fence())
        .unwrap();
    assert_eq!(f.label, "auto");
    assert_eq!(
        f.content.0, f.content.1,
        "empty fence content is zero-width"
    );
    // 零宽内部点报围栏语言（已装载空分析）；紧邻其后（闭栏行首）属父层。
    assert_eq!(
        empty.language_at(0, f.content.0).unwrap(),
        LanguageAt::FenceBody {
            label: "auto".into(),
            loaded: true
        }
    );
    let in_fence = empty
        .current()
        .regions
        .iter()
        .find(|r| r.fence().is_some())
        .unwrap();
    assert!(
        in_fence.embedded.is_some(),
        "empty auto fence still records an (empty) analysis"
    );
    assert_eq!(
        empty.current().highlights.len(),
        0,
        "no tokens to highlight in an empty body"
    );
    // 锚点字符集镜像（F2 回归）：非法字符尾段不提升，保留原名。
    let loose = open_doc("# 题 ^中文锚\n\n正文\n");
    assert_eq!(loose.current().outline.len(), 1);
    assert_eq!(
        loose.current().outline[0].name,
        "题 ^中文锚",
        "non [A-Za-z0-9_-] tail is content, not an anchor (mirrors extractAnchorBlock)"
    );
    assert!(loose.current().outline[0].anchor.is_none());
}

// ---------------------------------------------------------------------------
// V-02 局部修改
// ---------------------------------------------------------------------------

#[test]
fn v02_local_edit_reuses_subtrees_and_relocates_cursor() {
    let mut doc = open_doc(TWO_FENCES);
    assert_fresh_equivalent(&doc);

    // 两个独立围栏；改第一个围栏内的标识符（return a + b → return x + b）。
    let a_plus_b = TWO_FENCES.find("a + b").unwrap() as u64;
    let auto_fences_rev0: Vec<usize> = doc
        .current()
        .regions
        .iter()
        .enumerate()
        .filter(|(_, r)| matches!(&r.kind, RegionKind::Fence(f) if f.label == "auto"))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(auto_fences_rev0.len(), 2, "two independent auto fences");
    let fence2_idx_before = auto_fences_rev0[1];
    let fence2_emb_before = doc.current().regions[fence2_idx_before]
        .embedded
        .clone()
        .expect("fence 2 embedded analysis");

    let rev = edit(&mut doc, 0, a_plus_b, a_plus_b + 1, "x");
    assert_eq!(rev, 1);
    assert_fresh_equivalent(&doc);

    let m = &doc.current().metrics;
    // 真实复用：未改围栏的子分析 Rc 原样保留（非重算）。
    assert_eq!(m.reused_subtrees, 1, "unchanged fence subtree reused");
    assert_eq!(m.reparsed_fences, 1, "only the edited fence reparsed");
    // 区块结构未变（标题/围栏/段落/围栏）：fence 2 仍是最后一个围栏区域。
    let fence2_idx_after = doc
        .current()
        .regions
        .iter()
        .rposition(|r| r.fence().is_some())
        .expect("fence 2 region present after edit");
    assert_eq!(
        fence2_idx_after, fence2_idx_before,
        "region layout unchanged"
    );
    let fence2_emb_after = doc.current().regions[fence2_idx_after]
        .embedded
        .clone()
        .expect("fence 2 embedded after edit");
    assert!(
        Rc::ptr_eq(&fence2_emb_before, &fence2_emb_after),
        "fence 2 sub-analysis is the SAME Rc (true reuse)"
    );
    // 普通块内编辑不得用全文件重解析通过此例：重扫区间严格小于全文。
    let len = doc.snapshot().len();
    assert!(
        m.rescan_range.0 > 0 && m.rescan_range.1 < len,
        "rescan [{:?}) must be a strict sub-range of len {len}",
        m.rescan_range
    );
    assert!(
        m.fallback.is_none(),
        "local in-block edit converges without fallback"
    );
    assert!(m.grafted_regions >= 1, "unaffected tail grafted");
    // 重命名生效：x 标识符出现在高亮里，原 a 不再出现于该表达式。
    let snap = doc.snapshot().clone();
    let x_hl = doc
        .current()
        .highlights
        .iter()
        .any(|h| h.kind == "ident" && snap.read_str(h.start, h.end).as_deref() == Ok("x"));
    assert!(x_hl, "renamed identifier highlighted");

    // 在标题前插入中文 → 全部后移；后移光标定位到原 token。
    let heading_start_old = doc.current().outline[0].start;
    let fn_main_old = TWO_FENCES.find("fn main").unwrap() as u64; // fence2 首 token
    let hl_old = doc
        .current()
        .highlights
        .iter()
        .find(|h| h.start == fn_main_old && h.kind == "fn")
        .map(|h| (h.start, h.end))
        .expect("fn of fence 2 located before insert");
    let rev2 = edit(&mut doc, 1, 0, 0, "中文序言\n\n");
    let delta = "中文序言\n\n".len() as u64; // 4 汉字 ×3B + 2B 行界
    assert_eq!(delta, 14);
    assert_eq!(rev2, 2);
    assert_fresh_equivalent(&doc);

    let a2 = doc.current();
    // 标题后移且仍可导航（名字/锚点不变）。
    assert_eq!(a2.outline.len(), 1);
    assert_eq!(a2.outline[0].start, heading_start_old + delta);
    assert_eq!(a2.outline[0].anchor.as_deref(), Some("t1"));
    // 后移光标定位到原 token：同一 fn token 的旧区间 + delta 精确命中。
    let hl_new = a2
        .highlights
        .iter()
        .find(|h| h.start == hl_old.0 + delta && h.kind == "fn")
        .expect("fn token relocated at old position + delta");
    assert_eq!(hl_new.end, hl_old.1 + delta);
    assert_eq!(
        doc.snapshot().read_str(hl_new.start, hl_new.end).unwrap(),
        "fn"
    );
    // 插入在文首：首区域前的重扫从 0 开始合法，但尾部区域全部嫁接复用。
    assert_eq!(
        a2.metrics.reused_subtrees, 2,
        "both fences grafted with Rc reuse"
    );
    assert_eq!(a2.metrics.reparsed_fences, 0, "no fence content changed");

    // UTF-16 光标双向往返（后移后的 fn token；覆盖中文前缀的 UTF-16 列）。
    let (line, col) = doc.line_col_utf16(hl_new.start);
    assert_eq!(doc.offset_of_utf16(line, col), hl_new.start);
}

/// F1 回归：首区域之前（前导空行区）的编辑——从 0 重扫，增量与 fresh 等价。
#[test]
fn v02_regression_edit_before_first_region() {
    // 前导空行 + 段落（与独立复审探针同构）。
    let mut doc = open_doc("\npara\n");
    assert_fresh_equivalent(&doc);
    let rev = edit(&mut doc, 0, 0, 0, "X");
    assert_eq!(rev, 1);
    assert_fresh_equivalent(&doc);
    assert_eq!(doc.current().regions.len(), 1);
    let r = &doc.current().regions[0];
    assert!(matches!(r.kind, RegionKind::Paragraph));
    assert_eq!(
        (r.start, r.end),
        (0, 6),
        "merged paragraph starts at 0 like fresh"
    );

    // 前导空行 + 标题 + auto 围栏：在空行区插入换行（纯前导扩展）。
    let doc2_text = "\n# 标题 ^t\n\n```auto\nfn add() i32 { return 1 }\n```\n";
    let mut doc2 = open_doc(doc2_text);
    assert_fresh_equivalent(&doc2);
    let fence_idx = doc2
        .current()
        .regions
        .iter()
        .position(|r| r.fence().is_some())
        .unwrap();
    let fence_emb = doc2.current().regions[fence_idx].embedded.clone().unwrap();
    edit(&mut doc2, 0, 0, 0, "\n");
    assert_fresh_equivalent(&doc2);
    assert_eq!(doc2.current().outline.len(), 1);
    // 围栏子分析仍真实复用（嫁接尾部）。
    assert_eq!(doc2.current().metrics.reused_subtrees, 1);
    let fence_idx_after = doc2
        .current()
        .regions
        .iter()
        .position(|r| r.fence().is_some())
        .unwrap();
    let grafted_emb = doc2.current().regions[fence_idx_after]
        .embedded
        .clone()
        .unwrap();
    assert!(Rc::ptr_eq(&fence_emb, &grafted_emb));
}

// ---------------------------------------------------------------------------
// V-03 围栏重分类
// ---------------------------------------------------------------------------

#[test]
fn v03_fence_reclassification_matches_current_semantics() {
    let mut doc = open_doc(RECLASSIFY);
    assert_fresh_equivalent(&doc);
    assert_eq!(doc.current().outline.len(), 2, "both headings navigable");

    // 步骤 1：删除第一个围栏的闭栏行 → 未闭合语义：正文吞到下一个同 run
    // 闭栏（现版 parser 确认：标题B 与第二个围栏全部进入正文）。
    let brace_close = RECLASSIFY.find("}\n```\n").unwrap() as u64;
    let close_line_start = brace_close + 2;
    let rev = edit(&mut doc, 0, close_line_start, close_line_start + 4, "");
    assert_eq!(rev, 1);
    assert_fresh_equivalent(&doc);

    let a1 = doc.current();
    assert_eq!(
        a1.outline.len(),
        1,
        "heading B swallowed into the fence body"
    );
    assert_eq!(a1.outline[0].anchor.as_deref(), Some("a"));
    let fences: Vec<&FenceInfo> = a1.regions.iter().filter_map(|r| r.fence()).collect();
    assert_eq!(fences.len(), 1, "single (reclassified) fence");
    assert_eq!(fences[0].label, "auto");
    let body = doc
        .snapshot()
        .read_str(fences[0].content.0, fences[0].content.1)
        .unwrap();
    assert!(
        body.contains("# 标题B ^b") && body.contains("fn main() int {"),
        "swallowed body keeps original bytes (current-parser semantics)"
    );
    assert_eq!(
        a1.metrics.fallback,
        Some(Fallback::NoConvergence),
        "termination change re-derives to EOF, recorded honestly"
    );

    // 步骤 2：恢复闭栏行 → 后继标题再次可导航。
    let rev2 = edit(&mut doc, 1, close_line_start, close_line_start, "```\n");
    assert_eq!(rev2, 2);
    assert_fresh_equivalent(&doc);
    let a2 = doc.current();
    assert_eq!(a2.outline.len(), 2, "both headings navigable again");
    assert_eq!(a2.outline[1].anchor.as_deref(), Some("b"));
    let fences2: Vec<&FenceInfo> = a2.regions.iter().filter_map(|r| r.fence()).collect();
    assert_eq!(fences2.len(), 2);
    assert!(
        fences2.iter().all(|f| f.close.is_some()),
        "both fences closed again"
    );

    // 步骤 3：Auto 标签改为未知标签 → 不残留 Auto 高亮。
    let auto_open_at = RECLASSIFY.find("```auto").unwrap() as u64;
    let rev3 = edit(&mut doc, 2, auto_open_at + 3, auto_open_at + 7, "rust2");
    assert_eq!(rev3, 3);
    assert_fresh_equivalent(&doc);
    let a3 = doc.current();
    let f1 = a3.regions.iter().find_map(|r| r.fence()).unwrap();
    assert_eq!(f1.label, "rust2", "label changed to unknown");
    let (cs, ce) = f1.content;
    assert!(
        !a3.highlights.iter().any(|h| h.start >= cs && h.end <= ce),
        "no residual Auto highlights inside the reclassified fence"
    );
    assert!(
        a3.highlights.iter().any(|h| h.kind == "fn"),
        "the OTHER auto fence keeps its highlights"
    );
    // 光标定位：重分类围栏正文 = 未装载语言；另一 auto 围栏正文 = 已装载。
    let in_f1 = doc.language_at(3, cs).unwrap();
    assert_eq!(
        in_f1,
        LanguageAt::FenceBody {
            label: "rust2".into(),
            loaded: false
        }
    );
    let f2 = a3.regions.iter().filter_map(|r| r.fence()).nth(1).unwrap();
    assert_eq!(
        doc.language_at(3, f2.content.0).unwrap(),
        LanguageAt::FenceBody {
            label: "auto".into(),
            loaded: true
        }
    );
}

// ---------------------------------------------------------------------------
// V-04 坐标和错误
// ---------------------------------------------------------------------------

#[test]
fn v04_coordinates_crlf_utf16_and_projected_diagnostics() {
    let mut doc = open_doc(CRLF_EMOJI);
    assert_fresh_equivalent(&doc);

    let fence = doc
        .current()
        .regions
        .iter()
        .find_map(|r| r.fence())
        .unwrap();
    // CRLF 文档中围栏正文行界：内容不含尾随 \r\n，闭栏行独立。
    let fn_at = CRLF_EMOJI.find("fn add").unwrap() as u64;
    assert_eq!(
        fence.content.0, fn_at,
        "content starts exactly at the body line"
    );
    let body_end =
        CRLF_EMOJI.find("{ return a + 1 }").unwrap() as u64 + "{ return a + 1 }".len() as u64;
    assert_eq!(
        fence.content.1, body_end,
        "content ends before the trailing CRLF"
    );
    assert!(
        fence.close.is_some(),
        "close marker found across CRLF line breaks"
    );

    // 已标注 token（fn）：字节边界合法（char boundary），切片精确。
    let hl = doc
        .current()
        .highlights
        .iter()
        .find(|h| h.start == fn_at && h.kind == "fn")
        .expect("fn token annotated");
    assert!(CRLF_EMOJI.is_char_boundary(hl.start as usize));
    assert!(CRLF_EMOJI.is_char_boundary(hl.end as usize));
    assert_eq!(&CRLF_EMOJI[hl.start as usize..hl.end as usize], "fn");

    // UTF-16 双向往返：emoji（非 BMP，4B/2 单元）与中文前缀。
    let emoji_at = CRLF_EMOJI.find("😀").unwrap() as u64;
    let (l, c) = doc.line_col_utf16(emoji_at);
    assert_eq!(doc.offset_of_utf16(l, c), emoji_at, "emoji start roundtrip");
    assert_eq!(
        doc.offset_of_utf16(l, c + 1),
        emoji_at,
        "surrogate midpoint floors to scalar start"
    );
    let (fl, fc) = doc.line_col_utf16(fn_at);
    assert_eq!(
        doc.offset_of_utf16(fl, fc),
        fn_at,
        "fn token roundtrip (CRLF lines)"
    );
    // fn 前的所有字节：CRLF、中文、emoji 均在 token 之前。
    assert!(emoji_at < fn_at);

    // 在子区内制造未完整的 Auto 表达式：删去 "1"（return a + }）。
    let one_at = CRLF_EMOJI.find("a + 1 ").unwrap() as u64 + 4;
    let rev = edit(&mut doc, 0, one_at, one_at + 1, "");
    assert_eq!(rev, 1);
    assert_fresh_equivalent(&doc);

    let a1 = doc.current();
    let fence1_idx = a1
        .regions
        .iter()
        .position(|r| r.fence().is_some())
        .expect("fence region");
    let fence1 = a1.regions[fence1_idx].fence().unwrap();
    let (cs, ce) = fence1.content;
    let body = doc.snapshot().read_str(cs, ce).unwrap();
    assert!(
        body.contains("return a +"),
        "incomplete expression inside the sub-region"
    );
    assert!(
        !a1.diags.is_empty(),
        "sub diagnostics projected to the host document"
    );
    for d in &a1.diags {
        assert_eq!(
            d.fence_index,
            Some(fence1_idx),
            "diag attributed to its fence"
        );
        assert!(
            d.start >= cs && d.end <= ce,
            "diag [{},{}) must stay inside fence content [{cs},{ce})",
            d.start,
            d.end
        );
        // 标记/围栏边界属于父层：诊断切片命中正文内的真实 token 或缺失位置。
        let slice = doc.snapshot().read_str(d.start, d.end).unwrap();
        assert!(
            slice.is_empty() || body.contains(slice.as_str()),
            "diag slice {:?} must hit a real token in the original file body",
            slice
        );
        assert!(
            !slice.contains("```") && !slice.contains('#'),
            "marker/heading bytes belong to the parent layer, never to sub diags"
        );
    }
}

// ---------------------------------------------------------------------------
// V-05 编辑会话（无 UI 消费者）
// ---------------------------------------------------------------------------

#[test]
fn v05_editor_session_replay_and_revision_isolation() {
    let mut session = EditorSession::new(Some(auto_grammar()));
    let a = session.open(DERIVED_MIX).unwrap();
    let b = session.open(GOLDEN_SEGMENT).unwrap();
    assert_ne!(a, b, "each open gets its own file identity");

    // 点击标题与块内字符（修订 0）。
    let head_at = DERIVED_MIX.find("# 探针清单").unwrap() as u64;
    let auto_body_at = DERIVED_MIX.find("return a + b").unwrap() as u64;
    let ts_body_at = GOLDEN_SEGMENT.find("const x = 1").unwrap() as u64;
    assert_eq!(
        session.language_at(a, 0, head_at).unwrap(),
        LanguageAt::Parent(ParentBlock::Heading { level: 1 })
    );
    assert_eq!(
        session.language_at(a, 0, auto_body_at).unwrap(),
        LanguageAt::FenceBody {
            label: "auto".into(),
            loaded: true
        }
    );
    assert_eq!(
        session.language_at(b, 0, ts_body_at).unwrap(),
        LanguageAt::FenceBody {
            label: "ts".into(),
            loaded: false
        }
    );

    // 在途查询（修订 0 视图已构建）→ 先提交新修订 → 再交付旧结果。
    let stale = session.query(a, 0).unwrap();
    assert_eq!(
        session
            .edit(
                a,
                &EditRequest {
                    base_revision: 0,
                    edits: vec![Edit {
                        start: 0,
                        end: 0,
                        insert: "中文序言\n\n".as_bytes().to_vec(),
                    }]
                }
            )
            .unwrap(),
        1
    );
    assert_eq!(
        session.deliver(stale),
        DeliveryOutcome::Late,
        "stale rev-0 result must not override the new revision"
    );
    assert!(session.published(a).is_none());

    // 新修订结果正常发布，输出只含当前文件和修订。
    let fresh = session.query(a, 1).unwrap();
    assert_eq!(session.deliver(fresh), DeliveryOutcome::Published);
    let pa = session.published(a).unwrap().clone();
    assert_eq!((pa.file, pa.revision), (a, 1));
    assert_eq!(pa.outline.len(), 1);
    assert!(
        !pa.highlights.is_empty(),
        "auto highlights from file A itself"
    );

    // 不串文件：B 的当前修订结果独立发布，值/诊断/高亮只来自 B。
    let vb = session.query(b, 0).unwrap();
    assert_eq!(session.deliver(vb), DeliveryOutcome::Published);
    let pb = session.published(b).unwrap();
    assert_eq!(pb.revision, 0);
    assert!(
        pb.highlights.is_empty(),
        "B (ts only) never borrows A's highlights"
    );
    assert!(
        pb.outline.is_empty(),
        "B has no headings; A's outline never leaks"
    );
    assert!(!pa.outline.is_empty());

    // 同一文件重新打开 = 新 FileId；旧位置只对旧 snapshot 有效。
    let c = session.open(DERIVED_MIX).unwrap();
    assert_ne!(c, a, "re-open creates a new file identity");
    assert_eq!(session.current_revision(c).unwrap(), 0);
    // A 的旧修订查询仍合法（绑定旧快照），但不能再发布为当前结果。
    let stale2 = session.query(a, 0).unwrap();
    assert_eq!(
        session.language_at(a, 0, head_at).unwrap(),
        LanguageAt::Parent(ParentBlock::Heading { level: 1 }),
        "old positions stay valid against their own (old) snapshot"
    );
    assert_eq!(session.deliver(stale2), DeliveryOutcome::Late);
    assert_eq!(
        session.published(a).unwrap().revision,
        1,
        "published result not overwritten"
    );
    assert_eq!(
        session.late().len(),
        2,
        "both stale deliveries recorded as late"
    );

    // F5 对齐：(FileId, 修订) 键控——发布后文件继续编辑，更新修订的当前结果
    // 可替换旧发布；同修订重复交付仍首个胜出。
    session
        .edit(
            a,
            &EditRequest {
                base_revision: 1,
                edits: vec![Edit {
                    start: 0,
                    end: 0,
                    insert: b"y\n\n".to_vec(),
                }],
            },
        )
        .unwrap();
    let v2 = session.query(a, 2).unwrap();
    assert_eq!(session.deliver(v2), DeliveryOutcome::Published);
    assert_eq!(
        session.published(a).unwrap().revision,
        2,
        "newer current revision replaces the old publication"
    );
    let dup = session.query(a, 2).unwrap();
    assert_eq!(
        session.deliver(dup),
        DeliveryOutcome::Late,
        "duplicate delivery for the same revision is late (first wins)"
    );
    assert_eq!(session.late().len(), 3);
}

// ---------------------------------------------------------------------------
// V-06 反控
// ---------------------------------------------------------------------------

/// 手标 oracle 检查（与实现独立）：区间切片必须恰为期望 token 且落在标量边界。
fn annotation_matches(text: &str, start: u64, end: u64, want: &str) -> bool {
    let s = start as usize;
    let e = end as usize;
    text.is_char_boundary(s) && text.is_char_boundary(e) && text.get(s..e) == Some(want)
}

#[test]
fn v06_negative_controls() {
    // --- 1. 错误字节偏移：手标 oracle 能发现错位 ---
    let doc = open_doc(CRLF_EMOJI);
    let hl = doc
        .current()
        .highlights
        .iter()
        .find(|h| h.kind == "fn")
        .expect("annotated fn token");
    // 正确标注通过。
    assert!(annotation_matches(CRLF_EMOJI, hl.start, hl.end, "fn"));
    // 篡改 +1（错位）：oracle 拒绝（切片不再是 "fn" 或边界非法）。
    assert!(
        !annotation_matches(CRLF_EMOJI, hl.start + 1, hl.end + 1, "fn"),
        "a mis-placed annotation must be caught by the hand-marked oracle"
    );
    // 实现给出的区间与错误标注不同（区分力来自独立 oracle，非实现自证）。
    assert_ne!(hl.start, hl.start + 1);
    // emoji 起点前一位 = 中文续字节：oracle 的 char_boundary 检查同样拒绝。
    let emoji_at = CRLF_EMOJI.find("😀").unwrap() as u64;
    assert!(!annotation_matches(
        CRLF_EMOJI,
        emoji_at + 1,
        emoji_at + 4,
        "😀"
    ));

    // --- 2. 强制全量：与 fresh 等价，但不满足 V-02 的复用条件 ---
    let mut full = open_doc(TWO_FENCES);
    let a_plus_b = TWO_FENCES.find("a + b").unwrap() as u64;
    edit_force_full(&mut full, 0, a_plus_b, a_plus_b + 1, "x");
    assert_fresh_equivalent(&full);
    let mf = full.current().metrics.clone();
    assert_eq!(
        mf.rescan_range,
        (0, TWO_FENCES.len() as u64),
        "full file rescanned"
    );
    assert_eq!(
        mf.reused_subtrees, 0,
        "no subtree reuse in forced-full mode"
    );
    assert_eq!(mf.fallback, Some(Fallback::ForcedFull));
    let reuse_ok = mf.reused_subtrees >= 1 && mf.rescan_range != (0, TWO_FENCES.len() as u64);
    assert!(
        !reuse_ok,
        "forced full FAILS V-02's reuse condition (as intended)"
    );

    // 同一编辑走增量路径则满足（对照）。
    let mut inc = open_doc(TWO_FENCES);
    edit(&mut inc, 0, a_plus_b, a_plus_b + 1, "x");
    let mi = inc.current().metrics.clone();
    let reuse_ok_inc = mi.reused_subtrees >= 1 && mi.rescan_range != (0, TWO_FENCES.len() as u64);
    assert!(
        reuse_ok_inc,
        "incremental path satisfies the reuse condition"
    );

    // --- 3. 过期发布样例：迟到输出被识别 ---
    let mut session = EditorSession::new(Some(auto_grammar()));
    let f = session.open(GOLDEN_SEGMENT).unwrap();
    let v0 = session.query(f, 0).unwrap();
    session
        .edit(
            f,
            &EditRequest {
                base_revision: 0,
                edits: vec![Edit {
                    start: 0,
                    end: 0,
                    insert: b"x\n\n".to_vec(),
                }],
            },
        )
        .unwrap();
    session
        .edit(
            f,
            &EditRequest {
                base_revision: 1,
                edits: vec![Edit {
                    start: 0,
                    end: 2,
                    insert: b"".to_vec(),
                }],
            },
        )
        .unwrap();
    let v1 = session.query(f, 1).unwrap();
    assert_eq!(
        session.deliver(v0),
        DeliveryOutcome::Late,
        "two-revs-stale output identified"
    );
    assert_eq!(
        session.deliver(v1),
        DeliveryOutcome::Late,
        "one-rev-stale output identified"
    );
    assert!(session.published(f).is_none());
    let v2 = session.query(f, 2).unwrap();
    assert_eq!(session.deliver(v2), DeliveryOutcome::Published);
    assert_eq!(session.published(f).unwrap().revision, 2);
    assert_eq!(
        session.late().len(),
        2,
        "late outputs recorded, never published"
    );
}

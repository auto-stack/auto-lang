//! PLAN-1002 document-style authoring decode: a `.case.ad` file whose
//! top level carries `## Case: <key>` headings compiles into the SAME
//! `CaseSpec`/`CheckSpec` model the `test-case` contract path produces.
//!
//! The AutoDown block tree (validated fence↔raw pairing in
//! [`crate::document`]) is the structural judge; attribute lines are read
//! from the ORIGINAL bytes immediately after their heading (a blank line,
//! the next heading or a fence ends the block — the same shape inside a
//! fence is always payload). Every decode failure carries a 1-based
//! document line.
//!
//! First-batch wired vocabulary (Plan 1002 §5): `### Auto`, `### Rust`,
//! `### VM result`, `### VM diagnostic`, `### Probe <check-id>` (+`Notes`
//! prose); four-level headings are artifact paths under `Rust` and the
//! `stdout`/`stderr` streams under a console probe. Everything else in
//! the full target vocabulary is a located "not wired in this pilot"
//! error — never a silently skipped target.

use crate::admission::Limits;
use crate::atom::{self, CVal};
use crate::contract::{
    Action, Adapter, ArtifactExpect, ArtifactPayload, CaseSpec, CheckSpec, Compare, Completion,
    EntryMode, ExpectSpec, Options, PayloadSpec, ProbeAvSpec, ProbeMnSpec, ProbeStep, ResultExpect,
    SourceSpec, Stage, StreamExpect, Syntax, valid_case_id, valid_check_id,
};
use crate::document::{CaseDocument, DocumentFence};
use crate::raw_ranges::line_table;
use crate::sha256::{hex, sha256};

/// Decode failure, always located at a 1-based document line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for AuthorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for AuthorError {}

fn err<T>(line: usize, message: impl Into<String>) -> Result<T, AuthorError> {
    Err(AuthorError {
        line,
        message: message.into(),
    })
}

/// Where one check came from, for reports/inventories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocCheckSpan {
    pub id: String,
    /// 1-based line of the `###` section that declared the check.
    pub section_line: usize,
    /// Section heading text (e.g. `Probe av-console`), for humans.
    pub section: String,
}

/// One decoded case: the spec plus its document span for reporting.
#[derive(Debug, Clone, PartialEq)]
pub struct AuthorCase {
    pub spec: CaseSpec,
    /// Case key (the `## Case: <key>` name, unique per document).
    pub key: String,
    /// 1-based line of the `## Case:` heading.
    pub heading_line: usize,
    /// 1-based line that ends the case (next case heading or EOF+1).
    pub end_line: usize,
    /// Per-check document positions, in declaration order.
    pub check_spans: Vec<DocCheckSpan>,
}

const CASE_HEADING_PREFIX: &str = "Case: ";

/// Legal three-level section titles of the first wired batch.
const WIRED_SECTIONS: &str = "Auto, VM result, VM diagnostic, Rust, Probe <check-id>, Notes";

/// Headings of the full target vocabulary that are NOT wired yet (located
/// errors, never silent skips).
const UNWIRED_SECTIONS: &[&str] = &[
    "VM",
    "VM stderr",
    "VM exit",
    "C",
    "Python",
    "VM diagnostic message",
    "C diagnostic",
    "Rust diagnostic",
    "Python diagnostic",
];

/// Probe recipes wired into the document pilot (the seam contract itself
/// lives in [`crate::contract::REGISTERED_PROBE_RECIPES`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeWiring {
    AvCore,
    AvConsole,
    MnScalar,
}

fn probe_wiring(recipe: &str) -> Option<ProbeWiring> {
    match recipe {
        "ac-av/core-run-v1" => Some(ProbeWiring::AvCore),
        "ac-av/console-run-v1" => Some(ProbeWiring::AvConsole),
        "ac/mir-native-scalar-v1" => Some(ProbeWiring::MnScalar),
        _ => None,
    }
}

const WIRED_RECIPES: &str = "ac-av/core-run-v1, ac-av/console-run-v1, ac/mir-native-scalar-v1";

/// Does this document declare document-style cases (any top-level
/// `## Case: <key>` heading outside fences)?
pub fn is_document_case(doc: &CaseDocument) -> bool {
    doc.headings()
        .iter()
        .any(|h| h.level == 2 && h.text.starts_with(CASE_HEADING_PREFIX))
}

struct HeadRef<'a> {
    level: usize,
    text: &'a str,
    line: usize,
}

/// Text payload of a fence with the document-EOL rules applied: logical LF
/// lines, the final newline kept by default; `final-newline: false`
/// removes exactly the one newline adjacent to the closing fence.
fn fence_text(
    doc: &CaseDocument,
    fence: &DocumentFence,
    final_newline: bool,
) -> Result<String, AuthorError> {
    let raw = doc.payload_bytes(fence);
    let text = std::str::from_utf8(raw).map_err(|_| AuthorError {
        line: fence.open_line,
        message: "fence payload is not valid UTF-8".into(),
    })?;
    let mut s = crate::document::normalize_eols(text);
    if !final_newline && s.ends_with('\n') {
        s.pop();
    }
    Ok(s)
}

// ---------------------------------------------------------------- pieces --

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StreamKind {
    Stdout,
    Stderr,
}

struct StreamBlock {
    kind: StreamKind,
    expect: StreamExpect,
}

/// Single direct `rust` fence form of a Rust section.
#[derive(Debug, Clone)]
struct DirectRust {
    bytes: Vec<u8>,
    open_line: usize,
    close_line: Option<usize>,
}

struct RustArtifact {
    path: String,
    bytes: Vec<u8>,
    open_line: usize,
    close_line: Option<usize>,
    compare: Compare,
}

#[derive(Default)]
struct RustSection {
    section_line: usize,
    check_id: Option<String>,
    direct: Option<DirectRust>,
    subsections: Vec<RustArtifact>,
}

struct ProbeSection {
    check_id: String,
    section_line: usize,
    recipe: String,
    entry: String,
    caps: Vec<String>,
    exit: Option<i32>,
    streams: Vec<StreamBlock>,
}

/// One `key: value` attribute line.
struct Attr {
    key: String,
    value: CVal,
    line: usize,
}

fn attr_shape(line_text: &str) -> bool {
    let Some((key, _)) = line_text.split_once(':') else {
        return false;
    };
    let key = key.trim();
    !key.is_empty()
        && key.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn parse_attr(line: usize, text: &str) -> Result<Attr, AuthorError> {
    let (key, value) = text.split_once(':').ok_or_else(|| AuthorError {
        line,
        message: format!("属性行必须是 key: value 形式，得到 {:?}", text),
    })?;
    let key = key.trim().to_string();
    let value = atom::parse_contract(value.trim()).map_err(|e| AuthorError {
        line,
        message: format!("属性 {} 的值无法解析: {}", key, e),
    })?;
    Ok(Attr { key, value, line })
}

fn heading_shape(t: &str) -> bool {
    let markers = t.chars().take_while(|c| *c == '#').count();
    markers > 0
        && markers <= 6
        && (t.chars().count() == markers
            || t.chars().nth(markers) == Some(' ')
            || t.chars().nth(markers) == Some('\t'))
}

fn fence_line_shape(t: &str) -> bool {
    t.starts_with("```") || t.starts_with("~~~")
}

/// Raw text of a 1-based line (empty outside the document).
fn line_text(doc: &CaseDocument, lines: &[(usize, usize)], n: usize) -> String {
    let original = doc.original();
    match lines.get(n.wrapping_sub(1)) {
        Some(&(s, e)) if s <= e && e <= original.len() => {
            String::from_utf8_lossy(&original[s..e]).to_string()
        }
        _ => String::new(),
    }
}

/// Lines INSIDE a fence payload are always data (heading/attr/prose
/// shaped payload lines included) — same rule as the raw locator.
fn in_payload_line(doc: &CaseDocument, lines: &[(usize, usize)], n: usize) -> bool {
    match lines.get(n.wrapping_sub(1)) {
        Some(&(s, _)) => doc
            .fences()
            .iter()
            .any(|f| f.payload.start <= s && s < f.payload.end),
        None => false,
    }
}

/// The ONE attribute block directly under a heading (case level): the
/// contiguous `key: value` run starting at the next line, ending at the
/// first blank line, heading, fence line or payload line. Nothing after
/// the block belongs to the heading.
fn collect_leading_attrs(
    doc: &CaseDocument,
    lines: &[(usize, usize)],
    heading_line: usize,
    known: &[&str],
) -> Result<Vec<Attr>, AuthorError> {
    let mut out = Vec::new();
    let mut i = heading_line + 1;
    loop {
        let t = line_text(doc, lines, i).trim().to_string();
        if in_payload_line(doc, lines, i)
            || t.is_empty()
            || heading_shape(&t)
            || fence_line_shape(&t)
        {
            break;
        }
        if !attr_shape(&t) {
            return err(
                i,
                format!(
                    "属性行必须是 key: value 形式（正文段落前需要空行）：{:?}",
                    t
                ),
            );
        }
        let a = parse_attr(i, &t)?;
        if !known.contains(&a.key.as_str()) {
            return err(
                a.line,
                format!("属性 {:?} 不属于该节（可用：{}）", a.key, known.join(", ")),
            );
        }
        if out.iter().any(|x: &Attr| x.key == a.key) {
            return err(a.line, format!("重复的属性 {:?}", a.key));
        }
        out.push(a);
        i += 1;
    }
    Ok(out)
}

/// All attribute blocks in a zone, in document order. An attribute block
/// is a maximal run of consecutive non-blank lines (no headings, no fence
/// lines, not inside fences) whose FIRST line has attribute shape; every
/// line of the block must then parse. Attribute blocks that start AFTER
/// prose in the zone are located errors — attributes belong directly
/// under their heading.
fn attr_blocks(
    doc: &CaseDocument,
    lines: &[(usize, usize)],
    start_line: usize,
    end_line: usize,
) -> Result<Vec<Vec<Attr>>, AuthorError> {
    let mut blocks = Vec::new();
    let mut prose_seen = false;
    let mut i = start_line;
    while i <= end_line {
        let t = line_text(doc, lines, i).trim().to_string();
        if in_payload_line(doc, lines, i)
            || t.is_empty()
            || heading_shape(&t)
            || fence_line_shape(&t)
        {
            i += 1;
            continue;
        }
        if !attr_shape(&t) {
            prose_seen = true;
            i += 1;
            continue;
        }
        if prose_seen {
            return err(
                i,
                "属性行必须紧跟所属标题（与正文之间需要空行，且正文之后不再读属性）",
            );
        }
        let mut block = Vec::new();
        while i <= end_line {
            let t2 = line_text(doc, lines, i).trim().to_string();
            if in_payload_line(doc, lines, i)
                || t2.is_empty()
                || heading_shape(&t2)
                || fence_line_shape(&t2)
            {
                break;
            }
            if !attr_shape(&t2) {
                return err(
                    i,
                    format!(
                        "属性行必须是 key: value 形式（正文段落前需要空行）：{:?}",
                        t2
                    ),
                );
            }
            block.push(parse_attr(i, &t2)?);
            i += 1;
        }
        blocks.push(block);
    }
    Ok(blocks)
}

/// Strict attribute collection with unknown/duplicate-key rejection.
fn collect_attrs(
    doc: &CaseDocument,
    lines: &[(usize, usize)],
    start_line: usize,
    end_line: usize,
    known: &[&str],
) -> Result<Vec<Attr>, AuthorError> {
    let mut out = Vec::new();
    for block in attr_blocks(doc, lines, start_line, end_line)? {
        for a in block {
            if !known.contains(&a.key.as_str()) {
                return err(
                    a.line,
                    format!("属性 {:?} 不属于该节（可用：{}）", a.key, known.join(", ")),
                );
            }
            if out.iter().any(|x: &Attr| x.key == a.key) {
                return err(a.line, format!("重复的属性 {:?}", a.key));
            }
            out.push(a);
        }
    }
    Ok(out)
}

/// Fences whose opening line sits in `(after_line, before_line)`.
fn direct_fences(
    doc: &CaseDocument,
    after_line: usize,
    before_line: usize,
) -> Vec<&DocumentFence> {
    doc.fences()
        .iter()
        .filter(|f| f.open_line > after_line && f.open_line < before_line)
        .collect()
}

fn expect_success() -> ExpectSpec {
    ExpectSpec {
        completion: Completion::Success,
        result: None,
        stdout: None,
        stderr: None,
        artifacts: vec![],
        exit: None,
        perf: None,
        probe: None,
        av_channel: None,
        typed: None,
        trace: None,
    }
}

// ----------------------------------------------------------------- decode --

/// The document decoder. `rel_path` is the suite-relative include entry
/// (forward slashes) — the default case ID derives from it.
pub fn decode_document(
    doc: &CaseDocument,
    rel_path: &str,
    limits: &Limits,
) -> Result<Vec<AuthorCase>, AuthorError> {
    let original = doc.original();
    let lines = line_table(original);
    let total_lines = lines.len();

    // Heading sequence with raw line numbers (paired by the document load;
    // the authoring format REQUIRES positions — unpairable structure, e.g.
    // setext headings, is a located rejection here).
    if doc.headings().len() != doc.heading_lines() {
        return err(
            1,
            "标题结构无法与原文行配对（文档式文件只支持 ATX `#` 标题）",
        );
    }
    let heads: Vec<HeadRef> = doc
        .headings()
        .iter()
        .enumerate()
        .map(|(i, h)| HeadRef {
            level: h.level,
            text: &h.text,
            line: doc.heading_line(i),
        })
        .collect();

    // Level-1 title: at most one, must precede every other heading.
    let title_line = match heads.first() {
        Some(h) if h.level == 1 => Some(h.line),
        Some(_) => None,
        None => return err(1, "文档没有任何标题——至少需要一节 ## Case: <key>"),
    };
    for h in &heads {
        if h.level == 1 && Some(h.line) != title_line {
            return err(h.line, "一级标题只能有一个且必须在所有用例之前");
        }
        if h.level >= 5 {
            return err(
                h.line,
                format!("五级及更深的标题不属于本格式（得到 {:?}）", h.text),
            );
        }
    }

    // File-level attributes: after the title (or file start), before the
    // first case heading. Prose is allowed; keys are tags/links only.
    let mut file_tags: Vec<String> = Vec::new();
    let mut file_links: Vec<String> = Vec::new();
    let first_case_line = heads
        .iter()
        .find(|h| h.level == 2)
        .map(|h| h.line)
        .unwrap_or(usize::MAX);
    {
        let zone_start = title_line.map(|l| l + 1).unwrap_or(1);
        let zone_end = first_case_line.saturating_sub(1).min(total_lines);
        for a in collect_attrs(doc, &lines, zone_start, zone_end, &["tags", "links"])? {
            match a.key.as_str() {
                "tags" => merge_list(&mut file_tags, &a)?,
                "links" => merge_list(&mut file_links, &a)?,
                _ => unreachable!("known keys filtered"),
            }
        }
    }

    let case_heads: Vec<&HeadRef> = heads.iter().filter(|h| h.level == 2).collect();
    if case_heads.is_empty() {
        return err(
            heads.first().map(|h| h.line).unwrap_or(1),
            "文档没有任何 ## Case: 用例",
        );
    }
    check_limit_err(
        "max_cases_per_file",
        case_heads.len(),
        limits.max_cases_per_file,
        case_heads[0].line,
    )?;

    let mut cases = Vec::new();
    let mut seen_case_ids: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    let mut seen_keys: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for (ci, ch) in case_heads.iter().enumerate() {
        if !ch.text.starts_with(CASE_HEADING_PREFIX) {
            return err(
                ch.line,
                format!(
                    "二级标题必须是 ## Case: <key>（得到 {:?}；文档式文件不接受其他二级标题）",
                    ch.text
                ),
            );
        }
        let key = ch.text[CASE_HEADING_PREFIX.len()..].trim();
        if key.is_empty() {
            return err(ch.line, "## Case: 需要一个用例名");
        }
        if !key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            || !key
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            return err(
                ch.line,
                format!("用例名 {:?} 只能由 ASCII 小写字母、数字和连字符组成", key),
            );
        }
        if !seen_keys.insert(key.to_string()) {
            return err(ch.line, format!("重复的用例名 {:?}", key));
        }
        let end_line = case_heads
            .get(ci + 1)
            .map(|n| n.line)
            .unwrap_or(total_lines + 1);
        cases.push(decode_one_case(
            doc,
            &lines,
            &heads,
            ch,
            key,
            end_line,
            &file_tags,
            &file_links,
            rel_path,
        )?);
    }

    // Document-wide case-ID uniqueness (explicit and default IDs alike).
    for c in &cases {
        if let Some(prev_line) = seen_case_ids.get(&c.spec.id) {
            return err(
                c.heading_line,
                format!(
                    "重复的 case id {:?}（本文件第 {} 行已声明）",
                    c.spec.id, prev_line
                ),
            );
        }
        seen_case_ids.insert(c.spec.id.clone(), c.heading_line);
    }
    Ok(cases)
}

#[allow(clippy::too_many_arguments)]
fn decode_one_case(
    doc: &CaseDocument,
    lines: &[(usize, usize)],
    heads: &[HeadRef],
    case_head: &HeadRef,
    key: &str,
    end_line: usize,
    file_tags: &[String],
    file_links: &[String],
    rel_path: &str,
) -> Result<AuthorCase, AuthorError> {
    // Case attributes: the ONE block directly under the case heading.
    let mut id: Option<String> = None;
    let mut tags: Vec<String> = file_tags.to_vec();
    let mut links: Vec<String> = file_links.to_vec();
    let mut module_name: Option<String> = None;
    for a in collect_leading_attrs(
        doc,
        lines,
        case_head.line,
        &["id", "tags", "links", "module-name"],
    )? {
        match a.key.as_str() {
            "id" => {
                let v = want_str(&a)?;
                if !valid_case_id(&v) {
                    return err(
                        a.line,
                        "case id 只能由小写字母、数字、'.'、'_'、'-' 组成且以小写字母开头",
                    );
                }
                id = Some(v);
            }
            "tags" => merge_list(&mut tags, &a)?,
            "links" => merge_list(&mut links, &a)?,
            "module-name" => module_name = Some(want_str(&a)?),
            _ => unreachable!("known keys filtered"),
        }
    }

    // Sections inside the case: level-3 openers, level-4 subsections.
    let section_heads: Vec<&HeadRef> = heads
        .iter()
        .filter(|h| h.line > case_head.line && h.line < end_line)
        .collect();

    let mut auto: Option<String> = None;
    let mut auto_path: Option<String> = None;
    let mut vm_result: Option<(String, usize, Option<String>)> = None;
    let mut vm_diag: Option<(Stage, String, usize, Option<String>)> = None;
    let mut rust_section: Option<RustSection> = None;
    let mut probes: Vec<ProbeSection> = Vec::new();

    let mut current_h3: Option<&HeadRef> = None;
    for (si, sh) in section_heads.iter().enumerate() {
        let next_head_line = section_heads
            .get(si + 1)
            .map(|n| n.line)
            .unwrap_or(end_line);
        match sh.level {
            3 => {
                current_h3 = Some(sh);
                decode_h3_section(
                    doc,
                    lines,
                    sh,
                    next_head_line,
                    &mut auto,
                    &mut auto_path,
                    &mut vm_result,
                    &mut vm_diag,
                    &mut rust_section,
                    &mut probes,
                )?;
            }
            4 => {
                let parent = current_h3.ok_or_else(|| AuthorError {
                    line: sh.line,
                    message: format!(
                        "四级标题 {:?} 必须属于一个三级小节（Auto/Rust/Probe）",
                        sh.text
                    ),
                })?;
                decode_h4_subsection(
                    doc,
                    lines,
                    parent,
                    sh,
                    next_head_line,
                    &mut rust_section,
                    &mut probes,
                )?;
            }
            _ => unreachable!("levels filtered earlier"),
        }
    }

    // --- Auto is mandatory; source path defaults to <key>.at ---
    let source_text = auto.ok_or_else(|| AuthorError {
        line: case_head.line,
        message: format!("用例 {:?} 缺少 ### Auto 源码节", key),
    })?;
    let source_path = auto_path.unwrap_or_else(|| format!("{}.at", key));

    let mut checks: Vec<CheckSpec> = Vec::new();
    let mut check_spans: Vec<DocCheckSpan> = Vec::new();

    // --- VM family: one execution, success XOR failure ---
    if vm_result.is_some() && vm_diag.is_some() {
        return err(
            case_head.line,
            "同一 VM 动作同时要求成功（VM result）和失败（VM diagnostic）是格式错误",
        );
    }

    // Emit checks in DOCUMENT SECTION ORDER — the report identity follows
    // the authoring order, matching the legacy contract ordering.
    for sh in &section_heads {
        if sh.level != 3 {
            continue;
        }
        let title = sh.text.trim();
        match title {
            "Auto" | "Notes" => {}
            "VM result" => {
                if let Some((text, line, check_id)) = vm_result.clone() {
                    let id = check_id.unwrap_or_else(|| "vm".to_string());
                    let mut expect = expect_success();
                    expect.result = Some(ResultExpect {
                        format: "legacy-display".into(),
                        text,
                    });
                    push_check(
                        &mut checks,
                        &mut check_spans,
                        DocCheckSpan {
                            id: id.clone(),
                            section_line: line,
                            section: "VM result".into(),
                        },
                        CheckSpec {
                            id,
                            adapter: Adapter::Avm,
                            action: Action::Run,
                            tier: None,
                            options: Options::default(),
                            expect,
                            product_from: None,
                            skip: None,
                        },
                    )?;
                }
            }
            "VM diagnostic" => {
                if let Some((stage, code, line, check_id)) = vm_diag.clone() {
                    let id = check_id.unwrap_or_else(|| "vm".to_string());
                    let mut expect = expect_success();
                    expect.completion = Completion::Error {
                        stage,
                        code: Some(code),
                        message: None,
                    };
                    push_check(
                        &mut checks,
                        &mut check_spans,
                        DocCheckSpan {
                            id: id.clone(),
                            section_line: line,
                            section: "VM diagnostic".into(),
                        },
                        CheckSpec {
                            id,
                            adapter: Adapter::Avm,
                            action: Action::Run,
                            tier: None,
                            options: Options::default(),
                            expect,
                            product_from: None,
                            skip: None,
                        },
                    )?;
                }
            }
            "Rust" => {
                let is_this = rust_section
                    .as_ref()
                    .is_some_and(|rs| rs.section_line == sh.line);
                if is_this {
                    let rs = rust_section.take().expect("checked above");
                    let mut artifacts = Vec::new();
                    if rs.subsections.is_empty() {
                        let direct = rs.direct.ok_or_else(|| AuthorError {
                            line: rs.section_line,
                            message: "### Rust 需要一个 rust 围栏或 #### <产物路径> 小节"
                                .to_string(),
                        })?;
                        let stem = module_name
                            .clone()
                            .unwrap_or_else(|| path_stem(&source_path));
                        artifacts.push(ArtifactExpect {
                            path: format!("{}.rs", stem),
                            payload: ArtifactPayload::Inline {
                                bytes: direct.bytes,
                                open_line: direct.open_line,
                                close_line: direct.close_line,
                            },
                            compare: Compare::Bytes,
                        });
                    } else {
                        for sub in rs.subsections {
                            artifacts.push(ArtifactExpect {
                                path: sub.path,
                                payload: ArtifactPayload::Inline {
                                    bytes: sub.bytes,
                                    open_line: sub.open_line,
                                    close_line: sub.close_line,
                                },
                                compare: sub.compare,
                            });
                        }
                    }
                    let id = rs.check_id.clone().unwrap_or_else(|| "rust".to_string());
                    let options = Options {
                        module_name: module_name.clone(),
                        ..Default::default()
                    };
                    let mut expect = expect_success();
                    expect.artifacts = artifacts;
                    push_check(
                        &mut checks,
                        &mut check_spans,
                        DocCheckSpan {
                            id: id.clone(),
                            section_line: rs.section_line,
                            section: "Rust".into(),
                        },
                        CheckSpec {
                            id,
                            adapter: Adapter::A2r,
                            action: Action::Emit,
                            tier: None,
                            options,
                            expect,
                            product_from: None,
                            skip: None,
                        },
                    )?;
                }
            }
            t if t.starts_with("Probe ") => {
                let pos = probes
                    .iter()
                    .position(|p| p.section_line == sh.line)
                    .expect("probe parsed with its section line");
                let p = probes.remove(pos);
                let wiring = probe_wiring(&p.recipe).ok_or_else(|| AuthorError {
                    line: p.section_line,
                    message: format!(
                        "probe recipe {:?} 未接入文档式试点（已接通：{}）",
                        p.recipe, WIRED_RECIPES
                    ),
                })?;
                if p.entry.is_empty() {
                    return err(
                        p.section_line,
                        "Probe 节需要 entry（试点 recipe 的入口须显式声明）",
                    );
                }
                let mut options = Options {
                    probe_recipe: Some(p.recipe.clone()),
                    probe_steps: Some(vec![ProbeStep::Run]),
                    ..Default::default()
                };
                let mut expect = expect_success();
                expect.exit = p.exit;
                match wiring {
                    ProbeWiring::AvCore | ProbeWiring::AvConsole => {
                        if !p.caps.is_empty() && wiring != ProbeWiring::AvConsole {
                            return err(p.section_line, "caps 只属于 console recipe");
                        }
                        if wiring == ProbeWiring::AvConsole && p.entry != "d_main" {
                            return err(p.section_line, "console-run-v1 要求 entry 为 d_main");
                        }
                        options.probe_av = Some(ProbeAvSpec {
                            entry: p.entry.clone(),
                            caps: p.caps.clone(),
                            session: None,
                            profile: None,
                            invalidation: None,
                            resource: None,
                        });
                    }
                    ProbeWiring::MnScalar => {
                        options.probe_mn = Some(ProbeMnSpec {
                            entry: p.entry.clone(),
                            caps: vec![],
                            session: None,
                            typed: None,
                            diagnostics: None,
                            declaration: None,
                            resource: None,
                            history: None,
                            turnaround: None,
                        });
                    }
                }
                if wiring == ProbeWiring::AvConsole {
                    for st in &p.streams {
                        match st.kind {
                            StreamKind::Stdout => expect.stdout = Some(st.expect.clone()),
                            StreamKind::Stderr => expect.stderr = Some(st.expect.clone()),
                        }
                    }
                    crate::contract::validate_console_oracle(
                        expect.stdout.as_ref(),
                        expect.stderr.as_ref(),
                        expect.exit,
                        expect.av_channel.as_deref(),
                        &expect.completion,
                    )
                    .map_err(|m| AuthorError {
                        line: p.section_line,
                        message: m,
                    })?;
                } else if !p.streams.is_empty() {
                    return err(
                        p.section_line,
                        "只有 console recipe（ac-av/console-run-v1）接入输出流小节",
                    );
                }
                push_check(
                    &mut checks,
                    &mut check_spans,
                    DocCheckSpan {
                        id: p.check_id.clone(),
                        section_line: p.section_line,
                        section: format!("Probe {}", p.check_id),
                    },
                    CheckSpec {
                        id: p.check_id.clone(),
                        adapter: Adapter::Probe,
                        action: Action::Run,
                        tier: None,
                        options,
                        expect,
                        product_from: None,
                        skip: None,
                    },
                )?;
            }
            _ => unreachable!("titles classified during section decode"),
        }
    }

    if checks.is_empty() {
        return err(
            case_head.line,
            format!(
                "用例 {:?} 至少需要一项检查（VM result / VM diagnostic / Rust / Probe）",
                key
            ),
        );
    }

    let default_id = format!(
        "{}#{}",
        rel_path.strip_suffix(".case.ad").unwrap_or(rel_path),
        key
    );
    let spec = CaseSpec {
        schema: crate::contract::CASE_SCHEMA_V2.into(),
        id: id.unwrap_or(default_id),
        tags,
        links,
        source: SourceSpec {
            payload: PayloadSpec::Text(source_text),
            syntax: Syntax::Auto,
            path: source_path,
        },
        files: vec![],
        entry: EntryMode::Program,
        checks,
        status_skip: None,
    };
    Ok(AuthorCase {
        spec,
        key: key.to_string(),
        heading_line: case_head.line,
        end_line,
        check_spans,
    })
}

fn push_check(
    checks: &mut Vec<CheckSpec>,
    spans: &mut Vec<DocCheckSpan>,
    span: DocCheckSpan,
    spec: CheckSpec,
) -> Result<(), AuthorError> {
    if checks.iter().any(|c| c.id == spec.id) {
        return err(
            span.section_line,
            format!("重复的 check id {:?}（同一用例内必须唯一）", spec.id),
        );
    }
    if !valid_check_id(&spec.id) {
        return err(
            span.section_line,
            format!(
                "check id {:?} 只能由小写字母、数字、'.'、'_'、'-' 组成且以小写字母开头",
                spec.id
            ),
        );
    }
    checks.push(spec);
    spans.push(span);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn decode_h3_section(
    doc: &CaseDocument,
    lines: &[(usize, usize)],
    sh: &HeadRef,
    next_head_line: usize,
    auto: &mut Option<String>,
    auto_path: &mut Option<String>,
    vm_result: &mut Option<(String, usize, Option<String>)>,
    vm_diag: &mut Option<(Stage, String, usize, Option<String>)>,
    rust: &mut Option<RustSection>,
    probes: &mut Vec<ProbeSection>,
) -> Result<(), AuthorError> {
    let title = sh.text.trim();
    let known: &[&str] = match title {
        "Auto" => &["path", "final-newline"],
        "VM result" => &["check-id", "final-newline", "compare"],
        "VM diagnostic" => &["check-id", "stage", "code"],
        "Rust" => &["check-id"],
        "Notes" => &[],
        t if t.starts_with("Probe ") => &["recipe", "entry", "caps", "exit"],
        t if UNWIRED_SECTIONS.contains(&t) => {
            return err(
                sh.line,
                format!(
                    "标题 {:?} 属于目标格式但未在本批接通（本批接通：{}）",
                    t, WIRED_SECTIONS
                ),
            );
        }
        _ => {
            return err(
                sh.line,
                format!(
                    "未知三级标题 {:?}（本批接通：{}；区分大小写、完整匹配）",
                    title, WIRED_SECTIONS
                ),
            );
        }
    };
    if title == "Notes" {
        return Ok(()); // free prose; fences under Notes are illustrative
    }
    let attrs = collect_leading_attrs(doc, lines, sh.line, known)?;
    let fences = direct_fences(doc, sh.line, next_head_line);

    match title {
        "Auto" => {
            if fences.len() != 1 || fences[0].language != "auto" {
                return err(
                    sh.line,
                    "### Auto 需要恰好一个 auto 围栏（多文件工程属后续批次）",
                );
            }
            if auto.is_some() {
                return err(sh.line, "重复的 ### Auto 节");
            }
            let mut final_newline = true;
            for a in &attrs {
                match a.key.as_str() {
                    "path" => *auto_path = Some(want_str(a)?),
                    "final-newline" => final_newline = want_bool(a)?,
                    _ => unreachable!("known keys filtered"),
                }
            }
            *auto = Some(fence_text(doc, fences[0], final_newline)?);
            Ok(())
        }
        "VM result" => {
            if vm_result.is_some() {
                return err(sh.line, "重复的 ### VM result 节");
            }
            if fences.len() != 1 || fences[0].language != "text" {
                return err(
                    sh.line,
                    "### VM result 需要恰好一个 text 围栏（空围栏表示空串）",
                );
            }
            let mut check_id = None;
            let mut final_newline = true;
            for a in &attrs {
                match a.key.as_str() {
                    "check-id" => check_id = Some(want_str(a)?),
                    "final-newline" => final_newline = want_bool(a)?,
                    // `compare` is legal to write (payload option slot) but the
                    // legacy-display result channel compares exact text.
                    "compare" => {
                        want_compare(a)?;
                    }
                    _ => unreachable!(),
                }
            }
            let text = fence_text(doc, fences[0], final_newline)?;
            *vm_result = Some((text, sh.line, check_id));
            Ok(())
        }
        "VM diagnostic" => {
            if vm_diag.is_some() {
                return err(sh.line, "重复的 ### VM diagnostic 节");
            }
            if !fences.is_empty() {
                return err(
                    sh.line,
                    "### VM diagnostic 不接受围栏（完整消息断言属后续批次）",
                );
            }
            let mut check_id = None;
            let mut stage: Option<String> = None;
            let mut code: Option<String> = None;
            for a in &attrs {
                match a.key.as_str() {
                    "check-id" => check_id = Some(want_str(a)?),
                    "stage" => stage = Some(want_str(a)?),
                    "code" => code = Some(want_str(a)?),
                    _ => unreachable!(),
                }
            }
            let stage = stage.ok_or_else(|| AuthorError {
                line: sh.line,
                message: "### VM diagnostic 需要 stage（parse / type / run）".into(),
            })?;
            let code = code.ok_or_else(|| AuthorError {
                line: sh.line,
                message: "### VM diagnostic 需要 code（本批要求稳定的诊断码）".into(),
            })?;
            let stage = decode_vm_stage(&stage).ok_or_else(|| AuthorError {
                line: sh.line,
                message: format!("stage {:?} 不是 VM 支持的阶段（parse / type / run）", stage),
            })?;
            *vm_diag = Some((stage, code, sh.line, check_id));
            Ok(())
        }
        "Rust" => {
            if rust.is_some() {
                return err(sh.line, "重复的 ### Rust 节");
            }
            let mut section = RustSection {
                section_line: sh.line,
                ..Default::default()
            };
            for a in &attrs {
                if a.key == "check-id" {
                    section.check_id = Some(want_str(a)?);
                }
            }
            if fences.len() > 1 || (fences.len() == 1 && fences[0].language != "rust") {
                return err(sh.line, "### Rust 的直接围栏需要恰好一个 rust 围栏");
            }
            if let Some(fence) = fences.first() {
                let text = fence_text(doc, fence, true)?;
                section.direct = Some(DirectRust {
                    bytes: text.into_bytes(),
                    open_line: fence.open_line,
                    close_line: fence.close_line,
                });
            }
            *rust = Some(section);
            Ok(())
        }
        t => {
            // Probe <check-id>
            let check_id = t.strip_prefix("Probe ").unwrap_or("").trim();
            if check_id.is_empty() {
                return err(sh.line, "Probe 节需要一个 check id（### Probe <check-id>）");
            }
            if probes.iter().any(|p| p.check_id == check_id) {
                return err(sh.line, format!("重复的 Probe 节 {:?}", check_id));
            }
            let mut recipe: Option<String> = None;
            let mut entry = String::new();
            let mut caps = Vec::new();
            let mut exit = None;
            for a in &attrs {
                match a.key.as_str() {
                    "recipe" => recipe = Some(want_str(a)?),
                    "entry" => entry = want_str(a)?,
                    "caps" => caps = want_list(a)?,
                    "exit" => exit = Some(want_int(a)?),
                    _ => unreachable!(),
                }
            }
            let recipe = recipe.ok_or_else(|| AuthorError {
                line: sh.line,
                message: "Probe 节需要 recipe".into(),
            })?;
            if !fences.is_empty() {
                return err(
                    sh.line,
                    "Probe 节不接受直接围栏（输出流用 #### stdout / #### stderr 小节）",
                );
            }
            probes.push(ProbeSection {
                check_id: check_id.to_string(),
                section_line: sh.line,
                recipe,
                entry,
                caps,
                exit,
                streams: Vec::new(),
            });
            Ok(())
        }
    }
}

fn decode_vm_stage(s: &str) -> Option<Stage> {
    match s {
        "parse" => Some(Stage::Parse),
        "type" => Some(Stage::Type),
        "run" => Some(Stage::Run),
        _ => None,
    }
}

fn decode_h4_subsection(
    doc: &CaseDocument,
    lines: &[(usize, usize)],
    parent: &HeadRef,
    sh: &HeadRef,
    next_head_line: usize,
    rust: &mut Option<RustSection>,
    probes: &mut [ProbeSection],
) -> Result<(), AuthorError> {
    let parent_title = parent.text.trim();
    let fences = direct_fences(doc, sh.line, next_head_line);
    match parent_title {
        "Rust" => {
            if fences.len() != 1 || fences[0].language != "rust" {
                return err(
                    sh.line,
                    format!("#### {:?} 需要恰好一个 rust 围栏", sh.text),
                );
            }
            let attrs = collect_leading_attrs(doc, lines, sh.line, &["final-newline", "compare"])?;
            let mut final_newline = true;
            let mut compare = Compare::Bytes;
            for a in &attrs {
                match a.key.as_str() {
                    "final-newline" => final_newline = want_bool(a)?,
                    "compare" => compare = want_compare(a)?,
                    _ => unreachable!(),
                }
            }
            let text = fence_text(doc, fences[0], final_newline)?;
            let path = sh.text.trim().to_string();
            crate::admission::validate_virtual_path(&path).map_err(|e| AuthorError {
                line: sh.line,
                message: format!("产物路径 {:?}: {}", path, e),
            })?;
            let section = rust.as_mut().ok_or_else(|| AuthorError {
                line: sh.line,
                message: "四级产物小节必须紧跟 ### Rust 之后".into(),
            })?;
            if section.subsections.iter().any(|a| a.path == path) {
                return err(sh.line, format!("重复的产物路径 {:?}", path));
            }
            if section.direct.is_some() {
                return err(
                    sh.line,
                    "### Rust 不能同时携带直接围栏和 #### 产物小节（多产物全部用四级标题）",
                );
            }
            section.subsections.push(RustArtifact {
                path,
                bytes: text.into_bytes(),
                open_line: fences[0].open_line,
                close_line: fences[0].close_line,
                compare,
            });
            Ok(())
        }
        t if t.starts_with("Probe ") => {
            let check_id = t.strip_prefix("Probe ").unwrap_or("").trim();
            let kind = match sh.text.trim() {
                "stdout" => StreamKind::Stdout,
                "stderr" => StreamKind::Stderr,
                other => {
                    return err(
                        sh.line,
                        format!(
                            "Probe 下的四级标题只接受 stdout / stderr（得到 {:?}）",
                            other
                        ),
                    );
                }
            };
            let probe = probes
                .iter_mut()
                .find(|p| p.check_id == check_id)
                .ok_or_else(|| AuthorError {
                    line: sh.line,
                    message: format!("四级流小节找不到所属 Probe 节 {:?}", check_id),
                })?;
            if probe.streams.iter().any(|s| s.kind == kind) {
                return err(sh.line, format!("重复的 {:?} 流小节", sh.text));
            }
            if fences.len() != 1 || fences[0].language != "text" {
                return err(sh.line, "流小节需要恰好一个 text 围栏（空围栏表示零字节）");
            }
            let attrs = collect_leading_attrs(doc, lines, sh.line, &["final-newline", "compare"])?;
            let mut final_newline = true;
            let mut compare = Compare::Bytes;
            for a in &attrs {
                match a.key.as_str() {
                    "final-newline" => final_newline = want_bool(a)?,
                    "compare" => compare = want_compare(a)?,
                    _ => unreachable!(),
                }
            }
            let text = fence_text(doc, fences[0], final_newline)?;
            probe.streams.push(StreamBlock {
                kind,
                expect: StreamExpect { text, compare },
            });
            Ok(())
        }
        "Auto" => err(
            sh.line,
            "### Auto 下的四级标题（多文件工程）属后续批次，本批只支持单文件",
        ),
        other => err(
            sh.line,
            format!(
                "四级标题 {:?} 不属于小节 {:?} 可用的载荷形式",
                sh.text, other
            ),
        ),
    }
}

// -------------------------------------------------------------- scalars --

fn want_str(a: &Attr) -> Result<String, AuthorError> {
    match &a.value {
        CVal::Str(s) => Ok(s.clone()),
        other => err(
            a.line,
            format!("属性 {:?} 需要字符串值，得到 {}", a.key, other.type_name()),
        ),
    }
}

fn want_bool(a: &Attr) -> Result<bool, AuthorError> {
    match &a.value {
        CVal::Bool(b) => Ok(*b),
        other => err(
            a.line,
            format!("属性 {:?} 需要布尔值，得到 {}", a.key, other.type_name()),
        ),
    }
}

fn want_int(a: &Attr) -> Result<i32, AuthorError> {
    match &a.value {
        CVal::Int(i) if *i >= i32::MIN as i64 && *i <= i32::MAX as i64 => Ok(*i as i32),
        other => err(
            a.line,
            format!(
                "属性 {:?} 需要 32 位整数值，得到 {}",
                a.key,
                other.type_name()
            ),
        ),
    }
}

fn want_list(a: &Attr) -> Result<Vec<String>, AuthorError> {
    match &a.value {
        CVal::List(items) => {
            let mut out = Vec::new();
            for it in items {
                match it {
                    CVal::Str(s) => out.push(s.clone()),
                    other => {
                        return err(
                            a.line,
                            format!(
                                "属性 {:?} 的列表元素需要字符串，得到 {}",
                                a.key,
                                other.type_name()
                            ),
                        );
                    }
                }
            }
            Ok(out)
        }
        other => err(
            a.line,
            format!(
                "属性 {:?} 需要字符串列表，得到 {}",
                a.key,
                other.type_name()
            ),
        ),
    }
}

fn want_compare(a: &Attr) -> Result<Compare, AuthorError> {
    match want_str(a)?.as_str() {
        "bytes" => Ok(Compare::Bytes),
        "lf" => Ok(Compare::Lf),
        other => err(
            a.line,
            format!(
                "属性 {:?} 的 compare 只接受 \"bytes\" 或 \"lf\"（得到 {:?}）",
                a.key, other
            ),
        ),
    }
}

fn merge_list(target: &mut Vec<String>, a: &Attr) -> Result<(), AuthorError> {
    for item in want_list(a)? {
        if !target.contains(&item) {
            target.push(item);
        }
    }
    Ok(())
}

fn path_stem(path: &str) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.rsplit_once('.')
        .map(|(stem, _)| stem.to_string())
        .unwrap_or_else(|| name.to_string())
}

fn check_limit_err(name: &str, value: usize, max: usize, line: usize) -> Result<(), AuthorError> {
    if value > max {
        return err(line, format!("{} 超过上限（{} > {}）", name, value, max));
    }
    Ok(())
}

/// Fingerprint of the authoring decode identity axis: rides reports so a
/// decode-rule revision invalidates document-case evidence.
pub fn author_fingerprint() -> String {
    let source = concat!(env!("CARGO_PKG_NAME"), ":", env!("CARGO_PKG_VERSION"));
    hex(&sha256(source.as_bytes()))
}

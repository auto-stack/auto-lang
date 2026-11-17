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
    valid_case_id, valid_check_id, Action, Adapter, ArtifactExpect, ArtifactPayload, CaseSpec,
    BudgetSpec, CheckSpec, Compare, Completion, EntryMode, ExpectSpec, FileEntry, Options,
    PayloadSpec, ProbeAbiSpec, ProbeMnTypedSpec, ProbeStepExpect, SyntaxEdit, TypedExpect,
    ProbeAvInvalidationSpec, ProbeAvResourceSpec, ProbeMnDeclarationSpec, ProbeMnHistorySpec,
    ProbeMnResourceSpec, ProbeMnTurnaroundSpec,
    ProbeA2xHirSpec, ProbeAvSpec, ProbeCteSpec, ProbeMnSpec, ProbeSemaSpec, ProbeStep,
    ResultExpect, SourceSpec, Stage, StreamExpect, Syntax, MessageMatcher};
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

/// Legal three-level section titles (PLAN-1062 T-04 wires the CF-02/03
/// diagnostic-message faces and the full registered-recipe vocabulary).
const WIRED_SECTIONS: &str =
    "Auto, VM result, VM diagnostic, VM diagnostic message, Rust, Rust diagnostic, Probe <check-id>, Notes";

/// Headings of the full target vocabulary that are NOT wired yet (located
/// errors, never silent skips).
const UNWIRED_SECTIONS: &[&str] = &[
    "VM",
    "VM stderr",
    "VM exit",
    "C",
    "Python",
    "C diagnostic",
    "Python diagnostic",
];

/// Binding family of a registered probe recipe (PLAN-1062 T-04: every
/// REGISTERED_PROBE_RECIPES id is authorable; the recipe-specific binding
/// object is validated by contract admission).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeFamily {
    Av,
    AvConsole,
    Mn,
    Cte,
    Sema,
    Abi,
    A2x,
}

fn probe_family(recipe: &str) -> Option<ProbeFamily> {
    if !crate::contract::REGISTERED_PROBE_RECIPES.contains(&recipe) {
        return None;
    }
    match recipe {
        "ac-av/console-run-v1" => Some(ProbeFamily::AvConsole),
        r if r.starts_with("ac-av/") => Some(ProbeFamily::Av),
        r if r.starts_with("ac/mir-native-") || r.starts_with("ac/mn/") => Some(ProbeFamily::Mn),
        r if r.starts_with("cte/") => Some(ProbeFamily::Cte),
        r if r.starts_with("sema/") => Some(ProbeFamily::Sema),
        r if r.starts_with("abi/") => Some(ProbeFamily::Abi),
        r if r.starts_with("ac-a2r/") || r.starts_with("ac-a2c/") || r.starts_with("ac-a2py/") => {
            Some(ProbeFamily::A2x)
        }
        _ => None,
    }
}

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
    /// CF-07 mark-trace oracle (probe run steps; PLAN-914 trace channel).
    Trace,
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
    match_mode: crate::contract::ArtifactMatch,
}

#[derive(Default)]
struct RustSection {
    section_line: usize,
    check_id: Option<String>,
    tier: Option<String>,
    int_profile: Option<String>,
    resource_profile: Option<String>,
    direct: Option<DirectRust>,
    subsections: Vec<RustArtifact>,
}

/// One `### VM result` section (multiple per case, check-id distinguished).
#[derive(Clone)]
struct VmResultSection {
    text: String,
    line: usize,
    check_id: Option<String>,
    tier: Option<String>,
    has_fence: bool,
    fail_alloc_at: Option<u64>,
}

#[derive(Clone)]
struct VmDiagSection {
    stage: Stage,
    code: Option<String>,
    line: usize,
    check_id: Option<String>,
    tier: Option<String>,
    fail_alloc_at: Option<u64>,
}

#[derive(Clone)]
struct VmDiagMessageSection {
    stage: Stage,
    code: Option<String>,
    message: Option<MessageMatcher>,
    line: usize,
    check_id: Option<String>,
    tier: Option<String>,
}

struct ProbeSection {
    check_id: String,
    section_line: usize,
    recipe: String,
    entry: String,
    caps: Vec<String>,
    exit: Option<i32>,
    channel: Option<(String, usize)>,
    stage: Option<(String, usize)>,
    code: Option<(String, usize)>,
    /// Optional single-line message matchers (CF-02): exact or contains.
    message: Option<(String, usize)>,
    message_contains: Option<(String, usize)>,
    session: Option<(String, usize)>,
    profile: Option<(String, usize)>,
    diagnostics: Option<(String, usize)>,
    package: Option<String>,
    module: Option<String>,
    kind: Option<String>,
    fn_name: Option<String>,
    leg: Option<String>,
    tier: Option<String>,
    /// Ordered step list (CF-06 full vocabulary; () marks declaration).
    steps: Option<()>,
    steps_attr: Option<Attr>,
    /// CF-08 typed binding (mn typed recipe): fixture path + (kind,bits) args.
    typed_fixture: Option<(String, usize)>,
    typed_args: Option<Vec<(String, String)>>,
    /// CF-08 typed oracle: expected {kind, bits}.
    typed_expect: Option<(String, String)>,
    /// CF-07 AV nested oracles (raw CVal kept; wired per family).
    invalidation: Option<CVal>,
    av_resource: Option<CVal>,
    /// CF-07 MN nested oracles.
    mn_resource: Option<CVal>,
    declaration: Option<CVal>,
    history: Option<CVal>,
    turnaround: Option<CVal>,
    artifacts: Vec<RustArtifact>,
    streams: Vec<StreamBlock>,
    /// CF-07 per-step expectations (#### Observation N, 1-based order).
    observations: Vec<ObservationSection>,
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
fn direct_fences(doc: &CaseDocument, after_line: usize, before_line: usize) -> Vec<&DocumentFence> {
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
    let mut entry_mode = EntryMode::Program;
    for a in collect_leading_attrs(
        doc,
        lines,
        case_head.line,
        &["id", "tags", "links", "module-name", "entry-mode"],
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
            "entry-mode" => {
                entry_mode = match want_str(&a)?.as_str() {
                    "program" => EntryMode::Program,
                    "script" => EntryMode::Script,
                    "expression" => EntryMode::Expression,
                    "module" => EntryMode::Module,
                    other => {
                        return err(
                            a.line,
                            format!(
                                "entry-mode 仅支持 \"program\" / \"script\" / \"expression\" / \"module\"，收到 {:?}",
                                other
                            ),
                        )
                    }
                };
            }
            _ => unreachable!("known keys filtered"),
        }
    }

    // Sections inside the case: level-3 openers, level-4 subsections.
    let section_heads: Vec<&HeadRef> = heads
        .iter()
        .filter(|h| h.line > case_head.line && h.line < end_line)
        .collect();

    let mut auto: Option<String> = None;
    let mut auto_syntax: Option<Syntax> = None;
    let mut auto_path: Option<String> = None;
    let mut vm_result: Vec<VmResultSection> = Vec::new();
    let mut vm_diag: Vec<VmDiagSection> = Vec::new();
    let mut rust_section: Option<RustSection> = None;
    let mut probes: Vec<ProbeSection> = Vec::new();
    let mut acs: Vec<AcSection> = Vec::new();
    let mut syntaxes: Vec<SyntaxSection> = Vec::new();
    let mut vm_streams: Vec<(usize, StreamBlock)> = Vec::new();
    let mut auto_files: Vec<(String, String)> = Vec::new();
    let mut vm_diag_message: Vec<VmDiagMessageSection> = Vec::new();
    let mut rust_diag: Option<(
        Stage,
        Option<String>,
        Option<MessageMatcher>,
        usize,
        Option<String>,
        Option<String>,
    )> = None;

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
                    &mut auto_syntax,
                    &mut vm_result,
                    &mut vm_diag,
                    &mut vm_diag_message,
                    &mut rust_section,
                    &mut rust_diag,
                    &mut probes,
                    &mut acs,
                    &mut syntaxes,
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
                    &mut acs,
                    &mut syntaxes,
                    &mut vm_streams,
                    &mut auto_files,
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
    // Per check-id success XOR failure (PLAN-1062 T-10: a case may carry
    // multiple VM checks, each either a result or a diagnostic).
    let result_ids: std::collections::BTreeSet<Option<String>> =
        vm_result.iter().map(|v| v.check_id.clone()).collect();
    for fail in vm_diag
        .iter()
        .map(|d| (&d.check_id, d.line))
        .chain(vm_diag_message.iter().map(|m| (&m.check_id, m.line)))
    {
        if result_ids.contains(fail.0) {
            return err(
                fail.1,
                format!(
                    "同一 VM 检查 {:?} 同时要求成功（VM result）和失败（VM diagnostic）是格式错误",
                    fail.0.clone().unwrap_or_else(|| "vm".into())
                ),
            );
        }
    }
    for msg in vm_diag_message.iter() {
        if vm_diag.iter().any(|d| d.check_id == msg.check_id) {
            return err(
                msg.line,
                "VM diagnostic 与 VM diagnostic message 互斥（后者是前者的完整消息形态）",
            );
        }
    }
    if let Some(rust_section) = rust_section.as_ref() {
        if rust_section.direct.is_some() && rust_diag.is_some() {
            return err(
                case_head.line,
                "同一 Rust 检查同时要求金样（Rust）和诊断（Rust diagnostic）是格式错误",
            );
        }
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
            "VM diagnostic message" => {
                let pos = vm_diag_message
                    .iter()
                    .position(|v| v.line == sh.line)
                    .expect("vm diag message parsed with its line");
                if let Some(VmDiagMessageSection {
                    stage,
                    code,
                    message,
                    line,
                    check_id,
                    tier,
                }) = vm_diag_message.clone().into_iter().nth(pos)
                {
                    let id = check_id.unwrap_or_else(|| "vm".to_string());
                    let mut expect = expect_success();
                    for (_, st) in vm_streams.iter().filter(|(l, _)| *l == line) {
                        match st.kind {
                            StreamKind::Stdout => expect.stdout = Some(st.expect.clone()),
                            StreamKind::Stderr => expect.stderr = Some(st.expect.clone()),
                            StreamKind::Trace => unreachable!("VM streams never carry trace"),
                        }
                    }
                    expect.completion = Completion::Error {
                        stage,
                        code,
                        message,
                    };
                    push_check(
                        &mut checks,
                        &mut check_spans,
                        DocCheckSpan {
                            id: id.clone(),
                            section_line: line,
                            section: "VM diagnostic message".into(),
                        },
                        CheckSpec {
                            id,
                            adapter: Adapter::Avm,
                            action: Action::Run,
                            tier,
                            options: Options::default(),
                            expect,
                            product_from: None,
                            skip: None,
                        },
                    )?;
                }
            }
            "Rust diagnostic" => {
                if let Some((stage, code, message, line, check_id, tier)) = rust_diag.clone() {
                    let id = check_id.unwrap_or_else(|| "rust".to_string());
                    let options = Options {
                        module_name: module_name.clone(),
                        ..Default::default()
                    };
                    let mut expect = expect_success();
                    expect.completion = Completion::Error {
                        stage,
                        code,
                        message,
                    };
                    push_check(
                        &mut checks,
                        &mut check_spans,
                        DocCheckSpan {
                            id: id.clone(),
                            section_line: line,
                            section: "Rust diagnostic".into(),
                        },
                        CheckSpec {
                            id,
                            adapter: Adapter::A2r,
                            action: Action::Emit,
                            tier,
                            options,
                            expect,
                            product_from: None,
                            skip: None,
                        },
                    )?;
                }
            }
            "VM result" => {
                let pos = vm_result
                    .iter()
                    .position(|v| v.line == sh.line)
                    .expect("vm result parsed with its line");
                if let Some(VmResultSection {
                    text,
                    line,
                    check_id,
                    tier,
                    has_fence,
                    fail_alloc_at,
                }) = vm_result.clone().into_iter().nth(pos)
                {
                    let id = check_id.unwrap_or_else(|| "vm".to_string());
                    let mut expect = expect_success();
                    for (_, st) in vm_streams.iter().filter(|(l, _)| *l == line) {
                        match st.kind {
                            StreamKind::Stdout => expect.stdout = Some(st.expect.clone()),
                            StreamKind::Stderr => expect.stderr = Some(st.expect.clone()),
                            StreamKind::Trace => unreachable!("VM streams never carry trace"),
                        }
                    }
                    if has_fence {
                        expect.result = Some(ResultExpect {
                            format: "legacy-display".into(),
                            text,
                        });
                    }
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
                            tier,
                            options: Options {
                                avm_fail_alloc_at: fail_alloc_at,
                                ..Default::default()
                            },
                            expect,
                            product_from: None,
                            skip: None,
                        },
                    )?;
                }
            }
            "VM diagnostic" => {
                let pos = vm_diag
                    .iter()
                    .position(|v| v.line == sh.line)
                    .expect("vm diag parsed with its line");
                if let Some(VmDiagSection {
                    stage,
                    code,
                    line,
                    check_id,
                    tier,
                    fail_alloc_at,
                }) = vm_diag.clone().into_iter().nth(pos)
                {
                    let id = check_id.unwrap_or_else(|| "vm".to_string());
                    let mut expect = expect_success();
                    for (_, st) in vm_streams.iter().filter(|(l, _)| *l == line) {
                        match st.kind {
                            StreamKind::Stdout => expect.stdout = Some(st.expect.clone()),
                            StreamKind::Stderr => expect.stderr = Some(st.expect.clone()),
                            StreamKind::Trace => unreachable!("VM streams never carry trace"),
                        }
                    }
                    expect.completion = Completion::Error {
                        stage,
                        code,
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
                            tier,
                            options: Options {
                                avm_fail_alloc_at: fail_alloc_at,
                                ..Default::default()
                            },
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
                    let mut rs = rust_section.take().expect("checked above");
                    let mut artifacts = Vec::new();
                    if rs.subsections.is_empty() {
                        let direct = match rs.direct {
                            Some(d) => d,
                            None => DirectRust {
                                bytes: Vec::new(),
                                open_line: 0,
                                close_line: None,
                            },
                        };
                        if !direct.bytes.is_empty() {
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
                                match_mode: crate::contract::ArtifactMatch::Bytes,
                            });
                        }
                    } else {
                        for sub in std::mem::take(&mut rs.subsections) {
                            artifacts.push(ArtifactExpect {
                                path: sub.path,
                                payload: ArtifactPayload::Inline {
                                    bytes: sub.bytes,
                                    open_line: sub.open_line,
                                    close_line: sub.close_line,
                                },
                                compare: sub.compare,
                                match_mode: sub.match_mode,
                            });
                        }
                    }
                    let id = rs.check_id.clone().unwrap_or_else(|| "rust".to_string());
                    let options = Options {
                        module_name: module_name.clone(),
                        a2r_int_profile: rs.int_profile.clone(),
                        a2r_resource_profile: rs.resource_profile.clone(),
                        ..Default::default()
                    };
                    let mut expect = expect_success();
                    // PLAN-1062 T-15: a fence-less Rust section is the
                    // legacy smoke face (transpile must succeed; no golden).
                    if artifacts.is_empty() && rs.subsections.is_empty() {
                        expect.completion = Completion::Success;
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
                                tier: rs.tier.clone(),
                                options,
                                expect,
                                product_from: None,
                                skip: None,
                            },
                        )?;
                        continue;
                    }
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
                            tier: rs.tier.clone(),
                            options,
                            expect,
                            product_from: None,
                            skip: None,
                        },
                    )?;
                }
            }
            t if t.starts_with("AC ") => {
                let pos = acs.iter().position(|a| a.section_line == sh.line).expect("ac parsed");
                let a = acs.remove(pos);
                let mut options = Options::default();
                if let Some((v, _)) = &a.profile {
                    options.ac_profile = Some(if v == "core" {
                        crate::contract::AcProfile::Core
                    } else {
                        crate::contract::AcProfile::Scalar
                    });
                }
                if let Some((v, _)) = &a.target {
                    options.ac_target = Some(v.clone());
                }
                if let Some((v, _)) = &a.entry {
                    options.ac_entry = Some(v.clone());
                }
                if let Some((v, _)) = &a.cache {
                    options.ac_cache = Some(match v.as_str() {
                        "fresh" => crate::contract::CacheMode::Fresh,
                        "read-only" => crate::contract::CacheMode::ReadOnly,
                        _ => crate::contract::CacheMode::Reuse,
                    });
                }
                options.ac_budget = a.budget.clone();
                if !a.caps.is_empty() {
                    options.ac_capabilities = Some(a.caps.clone());
                }
                if let Some((v, _)) = &a.support_lib {
                    options.ac_support_lib = Some(v.clone());
                }
                let mut expect = error_expectation_from(
                    a.section_line, &a.stage, &a.code, &a.message, &a.message_contains,
                    decode_ac_stage, " AC", a.exit,
                )?;
                for st in &a.streams {
                    match st.kind {
                        StreamKind::Stdout => expect.stdout = Some(st.expect.clone()),
                        StreamKind::Stderr => expect.stderr = Some(st.expect.clone()),
                        StreamKind::Trace => return err(a.section_line, "AC 不接受 trace 流"),
                    }
                }
                if !a.artifacts.is_empty() {
                    expect.artifacts = a
                        .artifacts
                        .iter()
                        .map(|art| ArtifactExpect {
                            path: art.path.clone(),
                            payload: ArtifactPayload::Inline {
                                bytes: art.bytes.clone(),
                                open_line: art.open_line,
                                close_line: art.close_line,
                            },
                            compare: art.compare,
                            match_mode: art.match_mode.clone(),
                        })
                        .collect();
                }
                let action = match a.action.as_str() {
                    "check" => Action::Check,
                    "build" => Action::Build,
                    _ => Action::Run,
                };
                push_check(
                    &mut checks,
                    &mut check_spans,
                    DocCheckSpan {
                        id: a.check_id.clone(),
                        section_line: a.section_line,
                        section: format!("AC {}", a.check_id),
                    },
                    CheckSpec {
                        id: a.check_id.clone(),
                        adapter: Adapter::Ac,
                        action,
                        tier: a.tier.clone(),
                        options,
                        expect,
                        product_from: a.product_from.as_ref().map(|(v, _)| v.clone()),
                        skip: None,
                    },
                )?;
            }
            t if t.starts_with("Syntax ") => {
                let pos = syntaxes.iter().position(|x| x.section_line == sh.line).expect("syntax parsed");
                let x = syntaxes.remove(pos);
                let mut options = Options::default();
                if let Some((g, _)) = &x.grammar {
                    options.syntax_grammar = Some(g.clone());
                }
                options.syntax_edits = x.edits.clone();
                let mut expect = error_expectation_from(
                    x.section_line, &x.stage, &x.code, &x.message, &x.message_contains,
                    |s| if s == "parse" { Some(Stage::Parse) } else { None },
                    " Syntax（parse）", None,
                )?;
                for st in &x.streams {
                    match st.kind {
                        StreamKind::Stdout => expect.stdout = Some(st.expect.clone()),
                        StreamKind::Stderr => expect.stderr = Some(st.expect.clone()),
                        StreamKind::Trace => return err(x.section_line, "Syntax 不接受 trace 流"),
                    }
                }
                if !x.artifacts.is_empty() {
                    expect.artifacts = x
                        .artifacts
                        .iter()
                        .map(|a| ArtifactExpect {
                            path: a.path.clone(),
                            payload: ArtifactPayload::Inline {
                                bytes: a.bytes.clone(),
                                open_line: a.open_line,
                                close_line: a.close_line,
                            },
                            compare: a.compare,
                            match_mode: a.match_mode.clone(),
                        })
                        .collect();
                }
                let action = if x.action == "parse" { Action::Parse } else { Action::Edit };
                push_check(
                    &mut checks,
                    &mut check_spans,
                    DocCheckSpan {
                        id: x.check_id.clone(),
                        section_line: x.section_line,
                        section: format!("Syntax {}", x.check_id),
                    },
                    CheckSpec {
                        id: x.check_id.clone(),
                        adapter: Adapter::Syntax,
                        action,
                        tier: x.tier.clone(),
                        options,
                        expect,
                        product_from: None,
                        skip: None,
                    },
                )?;
            }
            t if t.starts_with("Probe ") => {
                let pos = probes
                    .iter()
                    .position(|p| p.section_line == sh.line)
                    .expect("probe parsed with its section line");
                let p = probes.remove(pos);
                let family = probe_family(&p.recipe).ok_or_else(|| AuthorError {
                    line: p.section_line,
                    message: format!(
                        "probe recipe {:?} 未注册（合法集合见 contract::REGISTERED_PROBE_RECIPES）",
                        p.recipe
                    ),
                })?;
                let steps = match &p.steps {
                    None => vec![ProbeStep::Run],
                    Some(_) => want_steps(p.steps_attr.as_ref().expect("raw attr kept"))?,
                };
                let mut options = Options {
                    probe_recipe: Some(p.recipe.clone()),
                    probe_steps: Some(steps),
                    ..Default::default()
                };
                let mut expect = expect_success();
                expect.exit = p.exit;
                match family {
                    ProbeFamily::Av | ProbeFamily::AvConsole => {
                        if p.entry.is_empty() {
                            return err(p.section_line, "Probe 节需要 entry（av recipe 入口须显式）");
                        }
                        if family == ProbeFamily::AvConsole && p.entry != "d_main" {
                            return err(p.section_line, "console-run-v1 要求 entry 为 d_main");
                        }
                        if let Some((_, l)) = &p.profile {
                            if family == ProbeFamily::AvConsole {
                                return err(*l, "console-run-v1 不接受 profile 覆盖");
                            }
                        }
                        let invalidation = match &p.invalidation {
                            None => None,
                            Some(CVal::Obj(pairs)) => {
                                obj_keys_ok(pairs, &["stable_defs", "rebuilt_defs"], "invalidation", p.section_line)?;
                                let stable = obj_str_list(pairs, "stable_defs", p.section_line)?;
                                let rebuilt = obj_str_list(pairs, "rebuilt_defs", p.section_line)?;
                                if stable.is_empty() && rebuilt.is_empty() {
                                    return err(p.section_line, "invalidation 需要非空 stable_defs/rebuilt_defs");
                                }
                                Some(ProbeAvInvalidationSpec { stable_defs: stable, rebuilt_defs: rebuilt })
                            }
                            Some(_) => return err(p.section_line, "invalidation 需要对象值"),
                        };
                        let av_resource = match &p.av_resource {
                            None => None,
                            Some(CVal::Obj(pairs)) => {
                                obj_keys_ok(pairs, &["profile", "max_steps", "wall_ms", "cancel_after_ms"], "av-resource", p.section_line)?;
                                let profile = obj_str(pairs, "profile", p.section_line)?
                                    .ok_or_else(|| AuthorError {
                                        line: p.section_line,
                                        message: "av-resource 需要 profile".into(),
                                    })?;
                                Some(ProbeAvResourceSpec {
                                    profile,
                                    max_steps: obj_u64(pairs, "max_steps", p.section_line)?,
                                    wall_ms: obj_u64(pairs, "wall_ms", p.section_line)?,
                                    cancel_after_ms: obj_u64(pairs, "cancel_after_ms", p.section_line)?,
                                })
                            }
                            Some(_) => return err(p.section_line, "av-resource 需要对象值"),
                        };
                        options.probe_av = Some(ProbeAvSpec {
                            entry: p.entry.clone(),
                            caps: p.caps.clone(),
                            session: p.session.as_ref().map(|(v, _)| v.clone()),
                            profile: p.profile.as_ref().map(|(v, _)| v.clone()),
                            invalidation,
                            resource: av_resource,
                        });
                    }
                    ProbeFamily::Mn => {
                        if p.entry.is_empty() {
                            return err(p.section_line, "Probe 节需要 entry（mn recipe 入口须显式）");
                        }
                        if let Some((v, l)) = &p.diagnostics {
                            if v != "source-v1" {
                                return err(*l, "diagnostics 仅支持 source-v1（mn.source-diagnostic/1）");
                            }
                        }
                        if (p.typed_fixture.is_some() || p.typed_args.is_some())
                            && p.recipe != "ac/mir-native-typed-v1"
                        {
                            return err(
                                p.section_line,
                                "typed-fixture/typed-args 仅属于 ac/mir-native-typed-v1",
                            );
                        }
                        if (p.typed_fixture.is_some()) != p.typed_args.is_some() {
                            return err(
                                p.section_line,
                                "typed-fixture 与 typed-args 需成对出现（args 可为空列表）",
                            );
                        }
                        let mn_resource = match &p.mn_resource {
                            None => None,
                            Some(CVal::Obj(pairs)) => {
                                obj_keys_ok(pairs, &["budget_bytes", "cancel_after_work", "fail_nth", "fail_seam", "fail_text_nth", "fail_ledger_nth"], "mn-resource", p.section_line)?;
                                let fail_seam = 'seam: {
                                    for (k, v) in pairs {
                                        if k == "fail_seam" {
                                            let CVal::List(list) = v else {
                                                return err(p.section_line, "mn-resource.fail_seam 需要 [seam, nth] 对");
                                            };
                                            if list.len() != 2 {
                                                return err(p.section_line, "mn-resource.fail_seam 需要 [seam, nth] 对");
                                            }
                                            let (CVal::Int(a), CVal::Int(b)) = (&list[0], &list[1]) else {
                                                return err(p.section_line, "mn-resource.fail_seam 元素需要整数");
                                            };
                                            break 'seam Some((*a as u32, *b as u64));
                                        }
                                    }
                                    None
                                };
                                Some(ProbeMnResourceSpec {
                                    budget_bytes: obj_u64(pairs, "budget_bytes", p.section_line)?,
                                    cancel_after_work: obj_u64(pairs, "cancel_after_work", p.section_line)?,
                                    fail_nth: obj_u64(pairs, "fail_nth", p.section_line)?,
                                    fail_seam,
                                    fail_text_nth: obj_u64(pairs, "fail_text_nth", p.section_line)?,
                                    fail_ledger_nth: obj_u64(pairs, "fail_ledger_nth", p.section_line)?,
                                })
                            }
                            Some(_) => return err(p.section_line, "mn-resource 需要对象值"),
                        };
                        let declaration = match &p.declaration {
                            None => None,
                            Some(CVal::Obj(pairs)) => {
                                obj_keys_ok(pairs, &["min_frontend_invalidations", "max_codegen_units", "stable_defs"], "declaration", p.section_line)?;
                                let min_inv = obj_u64(pairs, "min_frontend_invalidations", p.section_line)?
                                    .ok_or_else(|| AuthorError {
                                        line: p.section_line,
                                        message: "declaration 需要 min_frontend_invalidations".into(),
                                    })?;
                                Some(ProbeMnDeclarationSpec {
                                    min_frontend_invalidations: min_inv,
                                    max_codegen_units: obj_u64(pairs, "max_codegen_units", p.section_line)?,
                                    stable_defs: obj_str_list(pairs, "stable_defs", p.section_line)?,
                                })
                            }
                            Some(_) => return err(p.section_line, "declaration 需要对象值"),
                        };
                        let history = match &p.history {
                            None => None,
                            Some(CVal::Obj(pairs)) => {
                                obj_keys_ok(pairs, &["min_cache_hit_coff"], "history", p.section_line)?;
                                let min_hits = obj_u64(pairs, "min_cache_hit_coff", p.section_line)?
                                    .ok_or_else(|| AuthorError {
                                        line: p.section_line,
                                        message: "history 需要 min_cache_hit_coff".into(),
                                    })?;
                                Some(ProbeMnHistorySpec { min_cache_hit_coff: min_hits })
                            }
                            Some(_) => return err(p.section_line, "history 需要对象值"),
                        };
                        let turnaround = match &p.turnaround {
                            None => None,
                            Some(CVal::Obj(pairs)) => {
                                obj_keys_ok(pairs, &["p90_max_ms", "min_edits"], "turnaround", p.section_line)?;
                                let p90 = obj_u64(pairs, "p90_max_ms", p.section_line)?
                                    .ok_or_else(|| AuthorError {
                                        line: p.section_line,
                                        message: "turnaround 需要 p90_max_ms".into(),
                                    })?;
                                let min_edits = obj_u64(pairs, "min_edits", p.section_line)?
                                    .ok_or_else(|| AuthorError {
                                        line: p.section_line,
                                        message: "turnaround 需要 min_edits".into(),
                                    })?;
                                Some(ProbeMnTurnaroundSpec { p90_max_ms: p90, min_edits })
                            }
                            Some(_) => return err(p.section_line, "turnaround 需要对象值"),
                        };
                        options.probe_mn = Some(ProbeMnSpec {
                            entry: p.entry.clone(),
                            caps: p.caps.clone(),
                            session: p.session.as_ref().map(|(v, _)| v.clone()),
                            typed: p.typed_fixture.as_ref().map(|(fixture, _)| {
                                ProbeMnTypedSpec {
                                    fixture: fixture.clone(),
                                    args: p.typed_args.clone().unwrap_or_default(),
                                }
                            }),
                            diagnostics: p.diagnostics.as_ref().map(|(v, _)| v.clone()),
                            declaration,
                            resource: mn_resource,
                            history,
                            turnaround,
                        });
                    }
                    ProbeFamily::Cte => {
                        if p.entry.is_empty() {
                            return err(p.section_line, "Probe 节需要 entry（cte recipe 入口须显式）");
                        }
                        options.probe_cte = Some(ProbeCteSpec {
                            entry: p.entry.clone(),
                            caps: p.caps.clone(),
                            session: p.session.as_ref().map(|(v, _)| v.clone()),
                        });
                    }
                    ProbeFamily::Sema => {
                        if !p.entry.is_empty() || !p.caps.is_empty() {
                            return err(
                                p.section_line,
                                "sema binding 不接受 entry/caps（用 package/module/kind/fn）",
                            );
                        }
                        let package = p.package.clone().ok_or_else(|| AuthorError {
                            line: p.section_line,
                            message: "sema recipe 需要 package".into(),
                        })?;
                        let module = p.module.clone().ok_or_else(|| AuthorError {
                            line: p.section_line,
                            message: "sema recipe 需要 module".into(),
                        })?;
                        let kind = p.kind.clone().ok_or_else(|| AuthorError {
                            line: p.section_line,
                            message: "sema recipe 需要 kind（v1 冻结 infer-body）".into(),
                        })?;
                        let fn_name = p.fn_name.clone().ok_or_else(|| AuthorError {
                            line: p.section_line,
                            message: "sema recipe 需要 fn".into(),
                        })?;
                        options.probe_sema = Some(ProbeSemaSpec {
                            package,
                            module,
                            kind,
                            fn_name,
                        });
                    }
                    ProbeFamily::Abi => {
                        if !p.entry.is_empty() || !p.caps.is_empty() {
                            return err(
                                p.section_line,
                                "abi binding 不接受 entry/caps（用 leg）",
                            );
                        }
                        let leg = p.leg.clone().ok_or_else(|| AuthorError {
                            line: p.section_line,
                            message: "abi recipe 需要 leg（a / b / c）".into(),
                        })?;
                        options.probe_abi = Some(ProbeAbiSpec { leg });
                    }
                    ProbeFamily::A2x => {
                        if p.entry.is_empty() {
                            return err(p.section_line, "Probe 节需要 entry（emit recipe 入口须显式）");
                        }
                        let profile = p.profile.as_ref().ok_or_else(|| AuthorError {
                            line: p.section_line,
                            message: "emit recipe 需要 profile".into(),
                        })?;
                        options.probe_a2x_hir = Some(ProbeA2xHirSpec {
                            profile: profile.0.clone(),
                            entry: p.entry.clone(),
                            caps: p.caps.clone(),
                        });
                    }
                }
                // CF-07: nested oracle attrs are family-gated — a declared
                // but unwired oracle would silently drop the expectation.
                if family != ProbeFamily::Av && family != ProbeFamily::AvConsole {
                    if p.invalidation.is_some() {
                        return err(p.section_line, "invalidation 仅属于 av 系 recipe");
                    }
                    if p.av_resource.is_some() {
                        return err(p.section_line, "av-resource 仅属于 av 系 recipe");
                    }
                }
                if family != ProbeFamily::Mn {
                    if p.mn_resource.is_some() {
                        return err(p.section_line, "mn-resource 仅属于 mn 系 recipe");
                    }
                    if p.declaration.is_some() {
                        return err(p.section_line, "declaration 仅属于 mn 系 recipe");
                    }
                    if p.history.is_some() {
                        return err(p.section_line, "history 仅属于 mn 系 recipe");
                    }
                    if p.turnaround.is_some() {
                        return err(p.section_line, "turnaround 仅属于 mn 系 recipe");
                    }
                }
                // CF-02: typed error expectations (stage/code + optional
                // message) are legal for every probe family; the value/unit
                // result channel is legal on av recipes generally.
                if !matches!(family, ProbeFamily::Av | ProbeFamily::AvConsole) {
                    if let Some((_, l)) = &p.channel {
                        return err(*l, "channel 仅允许用于 av 系 recipe（value/unit 结果通道）");
                    }
                }
                match (&p.stage, &p.code) {
                    (None, None) => {}
                    (None, Some((_, cl))) => {
                        return err(*cl, "Probe 的 code 需要与 stage 成对出现（缺少 stage）");
                    }
                    (Some((st, sl)), Some((c, cl))) => {
                        let stage = decode_probe_stage(st).ok_or_else(|| AuthorError {
                            line: *sl,
                            message: format!(
                                "Probe stage 仅支持 parse / resolve / typecheck / verify / run，收到 {:?}",
                                st
                            ),
                        })?;
                        if c.trim().is_empty() {
                            return err(*cl, "Probe code 不能为空");
                        }
                        if expect.exit.is_some() {
                            return err(p.section_line, "Probe 错误预期不能同时声明 exit");
                        }
                        if family == ProbeFamily::AvConsole && p.channel.is_some() {
                            return err(p.section_line, "console Probe 错误预期不能同时声明 channel");
                        }
                        // Non-console av keeps the legacy contract: an error
                        // expectation and the value/unit channel may coexist
                        // only when the channel itself declares the error
                        // result; admission validates the combination.
                        let message = p
                            .message
                            .as_ref()
                            .map(|(t, _)| MessageMatcher::Exact(t.clone()))
                            .or_else(|| {
                                p.message_contains
                                    .as_ref()
                                    .map(|(t, _)| MessageMatcher::Contains(t.clone()))
                            });
                        expect.completion = Completion::Error {
                            stage,
                            code: Some(c.clone()),
                            message,
                        };
                    }
                    (Some((st, sl)), None) => {
                        let stage = decode_probe_stage(st).ok_or_else(|| AuthorError {
                            line: *sl,
                            message: format!(
                                "Probe stage 仅支持 parse / resolve / typecheck / verify / run，收到 {:?}",
                                st
                            ),
                        })?;
                        if expect.exit.is_some() {
                            return err(p.section_line, "Probe 错误预期不能同时声明 exit");
                        }
                        let message = p
                            .message
                            .as_ref()
                            .map(|(t, _)| MessageMatcher::Exact(t.clone()))
                            .or_else(|| {
                                p.message_contains
                                    .as_ref()
                                    .map(|(t, _)| MessageMatcher::Contains(t.clone()))
                            });
                        expect.completion = Completion::Error { stage, code: None, message };
                    }
                }
                if let Some((ch, ch_line)) = &p.channel {
                    if ch != "value" && ch != "unit" {
                        return err(
                            *ch_line,
                            format!(
                                "console Probe channel 仅支持 'value' 或 'unit'，收到 {:?}",
                                ch
                            ),
                        );
                    }
                    expect.av_channel = Some(ch.clone());
                }
                for st in &p.streams {
                    match st.kind {
                        StreamKind::Stdout => expect.stdout = Some(st.expect.clone()),
                        StreamKind::Stderr => expect.stderr = Some(st.expect.clone()),
                        StreamKind::Trace => expect.trace = Some(st.expect.clone()),
                    }
                }
                if family == ProbeFamily::AvConsole {
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
                }
                if let Some((kind, bits)) = &p.typed_expect {
                    expect.typed = Some(TypedExpect {
                        kind: kind.clone(),
                        bits: bits.clone(),
                    });
                }
                if !p.observations.is_empty() {
                    let mut ordered = p.observations.clone();
                    ordered.sort_by_key(|o| o.order);
                    for (i, o) in ordered.iter().enumerate() {
                        if o.order != i + 1 {
                            return err(
                                o.section_line,
                                format!("Observation 序号必须从 1 连续（第 {} 项是 {}）", i + 1, o.order),
                            );
                        }
                    }
                    expect.probe = Some(
                        ordered
                            .iter()
                            .map(|o| ProbeStepExpect {
                                cache: o.cache.clone(),
                                compute_calls: o.compute_calls,
                                bindings_file: None,
                                bindings_text: o.bindings_text.clone(),
                                stages_include: o.stages_include.clone(),
                                pe_exit: o.pe_exit,
                            })
                            .collect(),
                    );
                }
                if !p.artifacts.is_empty() {
                    expect.artifacts = p
                        .artifacts
                        .iter()
                        .map(|a| ArtifactExpect {
                            path: a.path.clone(),
                            payload: ArtifactPayload::Inline {
                                bytes: a.bytes.clone(),
                                open_line: a.open_line,
                                close_line: a.close_line,
                            },
                            compare: a.compare,
                            match_mode: a.match_mode.clone(),
                        })
                        .collect();
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
                        tier: p.tier.clone(),
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
            syntax: auto_syntax.unwrap_or(Syntax::Auto),
            path: source_path,
        },
        files: auto_files
            .iter()
            .map(|(path, text)| FileEntry {
                path: path.clone(),
                payload: PayloadSpec::Text(text.clone()),
            })
            .collect(),
        entry: entry_mode,
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
/// CF-07 per-step expectation: `#### Observation N` under a Probe section.
#[derive(Clone)]
struct ObservationSection {
    order: usize,
    section_line: usize,
    cache: Option<String>,
    compute_calls: Option<u32>,
    stages_include: Vec<String>,
    pe_exit: Option<i32>,
    /// Inline bindings payload (optional json fence).
    bindings_text: Option<String>,
}

// ---- PLAN-1062 T-06 (CF-01/CF-04): AC and Syntax author sections ----

/// AC failure stages (contract::Stage::supported_for(Adapter::Ac)).
fn decode_ac_stage(s: &str) -> Option<Stage> {
    match s {
        "parse" => Some(Stage::Parse),
        "resolve" => Some(Stage::Resolve),
        "typecheck" => Some(Stage::Typecheck),
        "bind" => Some(Stage::Bind),
        "verify" => Some(Stage::Verify),
        "capability" => Some(Stage::Capability),
        "backend" => Some(Stage::Backend),
        "link" => Some(Stage::Link),
        "run" => Some(Stage::Run),
        _ => None,
    }
}

struct AcSection {
    check_id: String,
    section_line: usize,
    action: String,
    profile: Option<(String, usize)>,
    target: Option<(String, usize)>,
    entry: Option<(String, usize)>,
    cache: Option<(String, usize)>,
    budget: Option<BudgetSpec>,
    caps: Vec<String>,
    support_lib: Option<(String, usize)>,
    product_from: Option<(String, usize)>,
    tier: Option<String>,
    exit: Option<i32>,
    stage: Option<(String, usize)>,
    code: Option<(String, usize)>,
    message: Option<(String, usize)>,
    message_contains: Option<(String, usize)>,
    artifacts: Vec<RustArtifact>,
    streams: Vec<StreamBlock>,
}

struct SyntaxSection {
    check_id: String,
    section_line: usize,
    action: String,
    grammar: Option<(String, usize)>,
    edits: Option<Vec<SyntaxEdit>>,
    tier: Option<String>,
    stage: Option<(String, usize)>,
    code: Option<(String, usize)>,
    message: Option<(String, usize)>,
    message_contains: Option<(String, usize)>,
    artifacts: Vec<RustArtifact>,
    streams: Vec<StreamBlock>,
}

fn want_budget(a: &Attr) -> Result<BudgetSpec, AuthorError> {
    let v = a.value.as_obj().ok_or_else(|| AuthorError {
        line: a.line,
        message: "budget 需要对象值 {build_ms: N, run_ms: N, total_ms: N}".into(),
    })?;
    let mut spec = BudgetSpec::default();
    for (k, val) in v {
        let CVal::Int(n) = val else {
            return err(a.line, format!("budget.{} 需要整数", k));
        };
        let n = *n as u64;
        match k.as_str() {
            "build_ms" => spec.build_ms = Some(n),
            "run_ms" => spec.run_ms = Some(n),
            "total_ms" => spec.total_ms = Some(n),
            other => {
                return err(a.line, format!("budget.{} 未知（build_ms/run_ms/total_ms）", other))
            }
        }
    }
    Ok(spec)
}

fn want_edits(a: &Attr) -> Result<Vec<SyntaxEdit>, AuthorError> {
    let CVal::List(list) = &a.value else {
        return err(a.line, "edits 需要 [{start: N, end: N, insert: \"…\"}] 列表值");
    };
    let mut out = Vec::new();
    for item in list {
        let Some(obj) = item.as_obj() else {
            return err(a.line, "edits 每项需要 {start: N, end: N, insert: \"…\"}");
        };
        let mut start = None;
        let mut end = None;
        let mut insert = None;
        for (k, v) in obj {
            match k.as_str() {
                "start" => {
                    let CVal::Int(n) = v else {
                        return err(a.line, "edits.start 需要整数");
                    };
                    start = Some(*n)
                }
                "end" => {
                    let CVal::Int(n) = v else {
                        return err(a.line, "edits.end 需要整数");
                    };
                    end = Some(*n)
                }
                "insert" => {
                    let CVal::Str(t) = v else {
                        return err(a.line, "edits.insert 需要字符串");
                    };
                    insert = Some(t.clone())
                }
                other => return err(a.line, format!("edits.{} 未知（start/end/insert）", other)),
            }
        }
        let (Some(start), Some(end), Some(insert)) = (start, end, insert) else {
            return err(a.line, "edits 每项需要 start/end/insert 三键齐全");
        };
        if end < start {
            return err(a.line, format!("edits 区间非法：end {} < start {}", end, start));
        }
        out.push(SyntaxEdit { start: start as u64, end: end as u64, insert });
    }
    if out.is_empty() {
        return err(a.line, "edits 需要至少一项（空编辑序列无意义）");
    }
    Ok(out)
}

fn error_expectation_from(
    section_line: usize,
    stage: &Option<(String, usize)>,
    code: &Option<(String, usize)>,
    message: &Option<(String, usize)>,
    message_contains: &Option<(String, usize)>,
    decode_stage: fn(&str) -> Option<Stage>,
    stage_label: &str,
    exit: Option<i32>,
) -> Result<ExpectSpec, AuthorError> {
    let mut expect = expect_success();
    expect.exit = exit;
    if message.is_some() && message_contains.is_some() {
        return err(section_line, "message 与 message-contains 互斥（同一检查只声明一种匹配）");
    }
    if (message.is_some() || message_contains.is_some()) && stage.is_none() {
        return err(section_line, "message / message-contains 需要与 stage 成对（诊断错误预期）");
    }
    match (stage, code) {
        (None, None) => {
            if message.is_some() || message_contains.is_some() {
                unreachable!("guarded above")
            }
        }
        (None, Some((_, cl))) => {
            return err(*cl, "code 需要与 stage 成对出现（缺少 stage）");
        }
        (Some((st, sl)), code_opt) => {
            let stage = decode_stage(st).ok_or_else(|| AuthorError {
                line: *sl,
                message: format!("stage {:?} 不是{}支持的阶段", st, stage_label),
            })?;
            if expect.exit.is_some() {
                return err(section_line, "错误预期不能同时声明 exit");
            }
            let message = message
                .as_ref()
                .map(|(t, _)| MessageMatcher::Exact(t.clone()))
                .or_else(|| {
                    message_contains
                        .as_ref()
                        .map(|(t, _)| MessageMatcher::Contains(t.clone()))
                });
            let code = code_opt.as_ref().map(|(c, _)| c.clone());
            expect.completion = Completion::Error { stage, code, message };
        }
    }
    Ok(expect)
}

fn decode_h3_section(
    doc: &CaseDocument,
    lines: &[(usize, usize)],
    sh: &HeadRef,
    next_head_line: usize,
    auto: &mut Option<String>,
    auto_path: &mut Option<String>,
    auto_syntax: &mut Option<Syntax>,
    vm_result: &mut Vec<VmResultSection>,
    vm_diag: &mut Vec<VmDiagSection>,
    vm_diag_message: &mut Vec<VmDiagMessageSection>,
    rust: &mut Option<RustSection>,
    rust_diag: &mut Option<(
        Stage,
        Option<String>,
        Option<MessageMatcher>,
        usize,
        Option<String>,
        Option<String>,
    )>,
    probes: &mut Vec<ProbeSection>,
    acs: &mut Vec<AcSection>,
    syntaxes: &mut Vec<SyntaxSection>,
) -> Result<(), AuthorError> {
    let title = sh.text.trim();
    let known: &[&str] = match title {
        "Auto" => &["path", "final-newline", "eol", "syntax"],
        "VM result" => &["check-id", "final-newline", "compare", "tier", "fail-alloc-at"],
        "VM diagnostic" => &["check-id", "stage", "code", "tier", "fail-alloc-at"],
        "VM diagnostic message" => &["check-id", "stage", "code", "contains", "final-newline", "tier"],
        "Rust" => &["check-id", "tier", "int-profile", "resource-profile"],
        "Rust diagnostic" => &["check-id", "stage", "code", "contains", "final-newline", "tier"],
        "Notes" => &[],
        t if t.starts_with("Probe ") => &[
            "recipe", "entry", "caps", "exit", "channel", "stage", "code",
            "message", "message-contains", "session", "profile", "diagnostics",
            "package", "module", "kind", "fn", "leg", "tier", "steps",
            "typed-fixture", "typed-args", "typed",
            "invalidation", "av-resource", "mn-resource", "declaration",
            "history", "turnaround",
        ],
        t if t.starts_with("AC ") => &[
            "action", "profile", "target", "entry", "cache", "budget", "caps", "support-lib",
            "product-from", "tier", "exit", "stage", "code", "message", "message-contains",
        ],
        t if t.starts_with("Syntax ") => &[
            "action", "grammar", "edits", "tier", "stage", "code", "message", "message-contains",
        ],
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
            let mut eol = "lf".to_string();
            for a in &attrs {
                match a.key.as_str() {
                    "path" => *auto_path = Some(want_str(a)?),
                    "final-newline" => final_newline = want_bool(a)?,
                    "syntax" => {
                        // PLAN-1062 CF-01: non-default source syntax is an
                        // explicit pair; default (auto) stays implicit.
                        let v = want_str(a)?;
                        *auto_syntax = Some(match v.as_str() {
                            "auto-script" => Syntax::AutoScript,
                            "hir-atom" => Syntax::HirAtom,
                            "typed-fixture" => Syntax::TypedFixture,
                            "atom-data" => Syntax::AtomData,
                            other => {
                                return err(a.line, format!(
                                    "syntax {:?} 未注册（auto-script / hir-atom / typed-fixture / atom-data）",
                                    other))
                            }
                        });
                    }
                    "eol" => {
                        eol = want_str(a)?.to_string();
                        if eol != "crlf" {
                            return err(a.line, "eol 仅支持 \"crlf\"（默认 lf 不需声明）");
                        }
                    }
                    _ => unreachable!("known keys filtered"),
                }
            }
            let mut text = fence_text(doc, fences[0], final_newline)?;
            if eol.as_str() == "crlf" {
                // PLAN-1062 §5.1 payload restoration: the fence body is
                // logical-LF; uniform-CRLF originals round-trip through this
                // explicit attribute (mixed endings need named-asset admission).
                text = text.replace('\n', "\r\n");
            }
            *auto = Some(text);
            Ok(())
        }
        "VM result" => {
            let id_ref = check_id_of(&attrs)?;
            if let Some(id) = &id_ref {
                if vm_result.iter().any(|v| v.check_id.as_deref() == Some(id.as_str())) {
                    return err(sh.line, format!("重复的 ### VM result 节（check id {:?}）", id));
                }
            }
            if fences.len() > 1 || (fences.len() == 1 && fences[0].language != "text") {
                return err(
                    sh.line,
                    "### VM result 至多一个 text 围栏（空围栏=空串；无围栏=仅流断言）",
                );
            }
            let check_id = id_ref;
            let mut tier = None;
            let mut fail_alloc_at = None;
            let mut final_newline = true;
            for a in &attrs {
                if a.key == "check-id" {
                    continue; // consumed by check_id_of above
                }
                match a.key.as_str() {
                    "tier" => tier = Some(want_tier(a)?),
                    "fail-alloc-at" => {
                        let n = want_int(a)?;
                        if n < 1 {
                            return err(a.line, "fail-alloc-at 需要 ≥1 的分配序号");
                        }
                        fail_alloc_at = Some(n as u64);
                    }
                    "final-newline" => final_newline = want_bool(a)?,
                    // `compare` is legal to write (payload option slot) but the
                    // legacy-display result channel compares exact text.
                    "compare" => {
                        want_compare(a)?;
                    }
                    _ => unreachable!(),
                }
            }
            let (text, has_fence) = match fences.first() {
                Some(fence) => (fence_text(doc, fence, final_newline)?, true),
                None => (String::new(), false),
            };
            vm_result.push(VmResultSection {
                text,
                line: sh.line,
                check_id,
                tier,
                has_fence,
                fail_alloc_at,
            });
            Ok(())
        }
        "VM diagnostic" => {
            let id_ref = check_id_of(&attrs)?;
            if let Some(id) = &id_ref {
                if vm_diag.iter().any(|v| v.check_id.as_deref() == Some(id.as_str())) {
                    return err(sh.line, format!("重复的 ### VM diagnostic 节（check id {:?}）", id));
                }
            }
            if !fences.is_empty() {
                return err(
                    sh.line,
                    "### VM diagnostic 不接受围栏（完整消息断言属后续批次）",
                );
            }
            let check_id = id_ref;
            let mut tier = None;
            let mut fail_alloc_at = None;
            let mut stage: Option<String> = None;
            let mut code: Option<String> = None;
            for a in &attrs {
                if a.key == "check-id" {
                    continue; // consumed by check_id_of above
                }
                match a.key.as_str() {
                    "tier" => tier = Some(want_tier(a)?),
                    "fail-alloc-at" => {
                        let n = want_int(a)?;
                        if n < 1 {
                            return err(a.line, "fail-alloc-at 需要 ≥1 的分配序号");
                        }
                        fail_alloc_at = Some(n as u64);
                    }
                    "stage" => stage = Some(want_str(a)?),
                    "code" => code = Some(want_str(a)?),
                    _ => unreachable!(),
                }
            }
            let stage = stage.ok_or_else(|| AuthorError {
                line: sh.line,
                message: "### VM diagnostic 需要 stage（parse / type / run）".into(),
            })?;
            // PLAN-1062: the contract keeps completion.code optional; a
            // diagnostic may ride stage alone (e.g. message-only oracles).
            let code = code.unwrap_or_default();
            if code.trim().is_empty() && code.is_empty() {
                // absent code is legal (code: None)
            }
            let stage = decode_vm_stage(&stage).ok_or_else(|| AuthorError {
                line: sh.line,
                message: format!("stage {:?} 不是 VM 支持的阶段（parse / type / run）", stage),
            })?;
            let code_opt = if code.is_empty() { None } else { Some(code) };
            vm_diag.push(VmDiagSection {
                stage,
                code: code_opt,
                line: sh.line,
                check_id,
                tier,
                fail_alloc_at,
            });
            Ok(())
        }
        "VM diagnostic message" | "Rust diagnostic" => {
            // CF-02: typed stage/code plus an optional exact/contains
            // message matcher riding one text fence.
            let is_vm = title == "VM diagnostic message";
            if is_vm {
                let id_ref = check_id_of(&attrs)?;
                if let Some(id) = &id_ref {
                    if vm_diag_message
                        .iter()
                        .any(|v| v.check_id.as_deref() == Some(id.as_str()))
                    {
                        return err(
                            sh.line,
                            format!("重复的 ### VM diagnostic message 节（check id {:?}）", id),
                        );
                    }
                }
            } else if rust_diag.is_some() {
                return err(sh.line, "重复的 ### Rust diagnostic 节");
            }
            if fences.len() > 1 || (fences.len() == 1 && fences[0].language != "text") {
                return err(
                    sh.line,
                    format!("### {} 至多一个 text 围栏（完整消息文本；无围栏=无消息断言）", title),
                );
            }
            let check_id = check_id_of(&attrs)?;
            let mut tier = None;
            let mut stage = None;
            let mut code = None;
            let mut contains = false;
            let mut final_newline = false;
            for a in &attrs {
                if a.key == "check-id" {
                    continue; // consumed by check_id_of above
                }
                match a.key.as_str() {
                    "tier" => tier = Some(want_tier(a)?),
                    "stage" => stage = Some(want_str(a)?),
                    "code" => code = Some(want_str(a)?),
                    "contains" => contains = want_bool(a)?,
                    "final-newline" => final_newline = want_bool(a)?,
                    _ => unreachable!(),
                }
            }
            let stage_text = stage.ok_or_else(|| AuthorError {
                line: sh.line,
                message: format!("### {} 需要 stage", title),
            })?;
            let code = code.unwrap_or_default();
            // Both VM and a2r failures use the same parse/type/run set
            // (contract::Stage::supported_for(Avm | A2r)).
            let stage = decode_vm_stage(&stage_text).ok_or_else(|| AuthorError {
                line: sh.line,
                message: format!(
                    "stage {:?} 不是 VM/Rust 诊断支持的阶段（parse / type / run）",
                    stage_text
                ),
            })?;
            let _ = &code;
            let message = if let Some(fence) = fences.first() {
                // A diagnostic message is a single logical line by default;
                // the closing fence's newline is an artifact unless kept.
                let text = fence_text(doc, fence, final_newline)?;
                Some(if contains {
                    MessageMatcher::Contains(text)
                } else {
                    MessageMatcher::Exact(text)
                })
            } else {
                None
            };
            let code_opt = if code.is_empty() { None } else { Some(code) };
            if is_vm {
                vm_diag_message.push(VmDiagMessageSection {
                    stage,
                    code: code_opt,
                    message,
                    line: sh.line,
                    check_id,
                    tier,
                });
            } else {
                *rust_diag = Some((stage, code_opt, message, sh.line, check_id, tier));
            }
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
                if a.key == "tier" {
                    section.tier = Some(want_tier(a)?);
                }
                if a.key == "int-profile" {
                    section.int_profile = Some(want_str(a)?.to_string());
                }
                if a.key == "resource-profile" {
                    section.resource_profile = Some(want_str(a)?.to_string());
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
        t if t.starts_with("AC ") => {
            let check_id = t.strip_prefix("AC ").unwrap_or("").trim();
            if check_id.is_empty() {
                return err(sh.line, "AC 节需要一个 check id（### AC <check-id>）");
            }
            if acs.iter().any(|a| a.check_id == check_id) {
                return err(sh.line, format!("重复的 AC 节 {:?}", check_id));
            }
            let mut action = None;
            let mut profile = None;
            let mut target = None;
            let mut entry = None;
            let mut cache = None;
            let mut budget = None;
            let mut caps = Vec::new();
            let mut support_lib = None;
            let mut product_from = None;
            let mut tier = None;
            let mut exit = None;
            let mut stage = None;
            let mut code = None;
            let mut message = None;
            let mut message_contains = None;
            for a in &attrs {
                match a.key.as_str() {
                    "action" => action = Some(want_str(a)?.to_string()),
                    "profile" => profile = Some((want_str(a)?.to_string(), a.line)),
                    "target" => target = Some((want_str(a)?.to_string(), a.line)),
                    "entry" => entry = Some((want_str(a)?.to_string(), a.line)),
                    "cache" => cache = Some((want_str(a)?.to_string(), a.line)),
                    "budget" => budget = Some(want_budget(a)?),
                    "caps" => caps = want_list(a)?,
                    "support-lib" => support_lib = Some((want_str(a)?.to_string(), a.line)),
                    "product-from" => product_from = Some((want_str(a)?.to_string(), a.line)),
                    "tier" => tier = Some(want_tier(a)?),
                    "exit" => exit = Some(want_int(a)?),
                    "stage" => stage = Some((want_str(a)?.to_string(), a.line)),
                    "code" => code = Some((want_str(a)?.to_string(), a.line)),
                    "message" => message = Some((want_str(a)?.to_string(), a.line)),
                    "message-contains" => message_contains = Some((want_str(a)?.to_string(), a.line)),
                    _ => unreachable!(),
                }
            }
            let action = action.ok_or_else(|| AuthorError {
                line: sh.line,
                message: "AC 节需要 action（check / build / run）".into(),
            })?;
            if !matches!(action.as_str(), "check" | "build" | "run") {
                return err(sh.line, format!("AC action 仅支持 check/build/run，收到 {:?}", action));
            }
            if let Some((v, l)) = &profile {
                if v != "core" && v != "scalar" {
                    return err(*l, format!("AC profile 仅支持 core/scalar，收到 {:?}", v));
                }
            }
            if let Some((v, l)) = &cache {
                if !matches!(v.as_str(), "fresh" | "reuse" | "read-only") {
                    return err(*l, format!("AC cache 仅支持 fresh/reuse/read-only，收到 {:?}", v));
                }
            }
            if !fences.is_empty() {
                return err(sh.line, "AC 节不接受直接围栏（输出流用 #### stdout / #### stderr 小节）");
            }
            acs.push(AcSection {
                check_id: check_id.to_string(),
                section_line: sh.line,
                action,
                profile,
                target,
                entry,
                cache,
                budget,
                caps,
                support_lib,
                product_from,
                tier,
                exit,
                stage,
                code,
                message,
                message_contains,
                artifacts: Vec::new(),
                streams: Vec::new(),
            });
            Ok(())
        }
        t if t.starts_with("Syntax ") => {
            let check_id = t.strip_prefix("Syntax ").unwrap_or("").trim();
            if check_id.is_empty() {
                return err(sh.line, "Syntax 节需要一个 check id（### Syntax <check-id>）");
            }
            if syntaxes.iter().any(|x| x.check_id == check_id) {
                return err(sh.line, format!("重复的 Syntax 节 {:?}", check_id));
            }
            let mut action = None;
            let mut grammar = None;
            let mut edits = None;
            let mut tier = None;
            let mut stage = None;
            let mut code = None;
            let mut message = None;
            let mut message_contains = None;
            for a in &attrs {
                match a.key.as_str() {
                    "action" => action = Some(want_str(a)?.to_string()),
                    "grammar" => grammar = Some((want_str(a)?.to_string(), a.line)),
                    "edits" => edits = Some(want_edits(a)?),
                    "tier" => tier = Some(want_tier(a)?),
                    "stage" => stage = Some((want_str(a)?.to_string(), a.line)),
                    "code" => code = Some((want_str(a)?.to_string(), a.line)),
                    "message" => message = Some((want_str(a)?.to_string(), a.line)),
                    "message-contains" => message_contains = Some((want_str(a)?.to_string(), a.line)),
                    _ => unreachable!(),
                }
            }
            let action = action.ok_or_else(|| AuthorError {
                line: sh.line,
                message: "Syntax 节需要 action（parse / edit）".into(),
            })?;
            if !matches!(action.as_str(), "parse" | "edit") {
                return err(sh.line, format!("Syntax action 仅支持 parse/edit，收到 {:?}", action));
            }
            if action == "edit" && edits.is_none() {
                return err(sh.line, "Syntax edit 需要 edits（有序编辑序列）");
            }
            if action == "parse" && edits.is_some() {
                return err(sh.line, "Syntax parse 不接受 edits");
            }
            if !fences.is_empty() {
                return err(sh.line, "Syntax 节不接受直接围栏");
            }
            syntaxes.push(SyntaxSection {
                check_id: check_id.to_string(),
                section_line: sh.line,
                action,
                grammar,
                edits,
                tier,
                stage,
                code,
                message,
                message_contains,
                artifacts: Vec::new(),
                streams: Vec::new(),
            });
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
            let mut channel = None;
            let mut stage = None;
            let mut code = None;
            let mut message = None;
            let mut message_contains = None;
            let mut session = None;
            let mut profile = None;
            let mut diagnostics = None;
            let mut package = None;
            let mut module = None;
            let mut kind = None;
            let mut fn_name = None;
            let mut leg = None;
            let mut tier = None;
            let mut steps = None;
            let mut steps_attr = None;
            let mut typed_fixture = None;
            let mut typed_args = None;
            let mut typed_expect = None;
            let mut invalidation = None;
            let mut av_resource = None;
            let mut mn_resource = None;
            let mut declaration = None;
            let mut history = None;
            let mut turnaround = None;
            for a in &attrs {
                match a.key.as_str() {
                    "recipe" => recipe = Some(want_str(a)?),
                    "entry" => entry = want_str(a)?,
                    "caps" => caps = want_list(a)?,
                    "exit" => exit = Some(want_int(a)?),
                    "channel" => channel = Some((want_str(a)?, a.line)),
                    "stage" => stage = Some((want_str(a)?, a.line)),
                    "code" => code = Some((want_str(a)?, a.line)),
                    "message" => message = Some((want_str(a)?, a.line)),
                    "message-contains" => message_contains = Some((want_str(a)?, a.line)),
                    "session" => session = Some((want_str(a)?, a.line)),
                    "profile" => profile = Some((want_str(a)?, a.line)),
                    "diagnostics" => diagnostics = Some((want_str(a)?, a.line)),
                    "package" => package = Some(want_str(a)?),
                    "module" => module = Some(want_str(a)?),
                    "kind" => kind = Some(want_str(a)?),
                    "fn" => fn_name = Some(want_str(a)?),
                    "leg" => leg = Some(want_str(a)?),
                    "tier" => tier = Some(want_str(a)?),
                    "steps" => {
                        steps = Some(());
                        steps_attr = Some(Attr {
                            key: a.key.clone(),
                            value: a.value.clone(),
                            line: a.line,
                        });
                    }
                    "typed-fixture" => typed_fixture = Some((want_str(a)?.to_string(), a.line)),
                    "typed-args" => typed_args = Some(want_typed_args(a)?),
                    "typed" => typed_expect = Some(want_typed_expect(a)?),
                    "invalidation" => invalidation = Some(a.value.clone()),
                    "av-resource" => av_resource = Some(a.value.clone()),
                    "mn-resource" => mn_resource = Some(a.value.clone()),
                    "declaration" => declaration = Some(a.value.clone()),
                    "history" => history = Some(a.value.clone()),
                    "turnaround" => turnaround = Some(a.value.clone()),
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
            if message.is_some() && message_contains.is_some() {
                return err(
                    sh.line,
                    "message 与 message-contains 互斥（同一检查只声明一种匹配）",
                );
            }
            if (message.is_some() || message_contains.is_some()) && stage.is_none() {
                return err(
                    sh.line,
                    "message / message-contains 需要与 stage 成对（诊断错误预期）",
                );
            }
            probes.push(ProbeSection {
                check_id: check_id.to_string(),
                section_line: sh.line,
                recipe,
                entry,
                caps,
                exit,
                channel,
                stage,
                code,
                message,
                message_contains,
                session,
                profile,
                diagnostics,
                package,
                module,
                kind,
                fn_name,
                leg,
                tier,
                steps,
                steps_attr,
                typed_fixture,
                typed_args,
                typed_expect,
                invalidation,
                av_resource,
                mn_resource,
                declaration,
                history,
                turnaround,
                artifacts: Vec::new(),
                streams: Vec::new(),
                observations: Vec::new(),
            });
            Ok(())
        }
    }
}

fn obj_str_list(obj: &[(String, CVal)], key: &str, line: usize) -> Result<Vec<String>, AuthorError> {
    for (k, v) in obj {
        if k == key {
            let CVal::List(list) = v else {
                return err(line, format!("{}.{} 需要字符串列表", key, key));
            };
            let mut out = Vec::new();
            for item in list {
                let CVal::Str(t) = item else {
                    return err(line, format!("{}.{} 元素需要字符串", key, key));
                };
                out.push(t.clone());
            }
            return Ok(out);
        }
    }
    Ok(Vec::new())
}

fn obj_u64(obj: &[(String, CVal)], key: &str, line: usize) -> Result<Option<u64>, AuthorError> {
    for (k, v) in obj {
        if k == key {
            let CVal::Int(n) = v else {
                return err(line, format!("{} 需要整数", key));
            };
            if *n < 0 {
                return err(line, format!("{} 需要非负整数", key));
            }
            return Ok(Some(*n as u64));
        }
    }
    Ok(None)
}

fn obj_str(obj: &[(String, CVal)], key: &str, line: usize) -> Result<Option<String>, AuthorError> {
    for (k, v) in obj {
        if k == key {
            let CVal::Str(t) = v else {
                return err(line, format!("{} 需要字符串", key));
            };
            return Ok(Some(t.clone()));
        }
    }
    Ok(None)
}

fn obj_keys_ok(obj: &[(String, CVal)], allowed: &[&str], label: &str, line: usize) -> Result<(), AuthorError> {
    for (k, _) in obj {
        if !allowed.contains(&k.as_str()) {
            return err(line, format!("{}.{} 未知（允许：{}）", label, k, allowed.join("/")));
        }
    }
    Ok(())
}

fn want_steps(a: &Attr) -> Result<Vec<ProbeStep>, AuthorError> {
    let CVal::List(items) = &a.value else {
        return err(a.line, "steps 需要有序步骤列表");
    };
    let mut out = Vec::new();
    for item in items {
        let Some(obj) = item.as_obj() else {
            // bare "run"/"query" strings stay legal shorthand
            if let CVal::Str(op) = item {
                match op.as_str() {
                    "run" => {
                        out.push(ProbeStep::Run);
                        continue;
                    }
                    "query" => {
                        out.push(ProbeStep::Query);
                        continue;
                    }
                    "restart" => {
                        out.push(ProbeStep::Restart);
                        continue;
                    }
                    "resident-apply" => {
                        out.push(ProbeStep::ResidentApply);
                        continue;
                    }
                    "resident-finish" => {
                        out.push(ProbeStep::ResidentFinish);
                        continue;
                    }
                    other => {
                        return err(a.line, format!(
                            "steps 短形式仅接受 run/query/restart/resident-apply/resident-finish，收到 {:?}（带参步骤用对象）",
                            other))
                    }
                }
            }
            return err(a.line, "steps 每项需要 {op: …} 对象或合法短字符串");
        };
        let mut op = None;
        let mut from = None;
        let mut to = None;
        let mut defs = Vec::new();
        let mut slices = None;
        for (k, v) in obj {
            match k.as_str() {
                "op" => {
                    let CVal::Str(t) = v else {
                        return err(a.line, "steps.op 需要字符串");
                    };
                    op = Some(t.clone());
                }
                "from" | "to" => {
                    let CVal::Str(t) = v else {
                        return err(a.line, format!("steps.{} 需要字符串", k));
                    };
                    if k == "from" {
                        from = Some(t.clone());
                    } else {
                        to = Some(t.clone());
                    }
                }
                "defs" => {
                    let CVal::List(list) = v else {
                        return err(a.line, "steps.defs 需要字符串列表");
                    };
                    for d in list {
                        let CVal::Str(t) = d else {
                            return err(a.line, "steps.defs 元素需要字符串");
                        };
                        defs.push(t.clone());
                    }
                }
                "slices" => {
                    let CVal::Int(n) = v else {
                        return err(a.line, "steps.slices 需要整数");
                    };
                    if *n < 1 {
                        return err(a.line, "steps.slices 需要正整数");
                    }
                    slices = Some(*n as u32);
                }
                other => return err(a.line, format!("steps.{} 未知", other)),
            }
        }
        let op = op.ok_or_else(|| AuthorError {
            line: a.line,
            message: "steps 对象需要 op".into(),
        })?;
        let need_pair = |from: &Option<String>, to: &Option<String>| -> Result<(String, String), AuthorError> {
            match (from, to) {
                (Some(f), Some(t)) => Ok((f.clone(), t.clone())),
                _ => err(a.line, format!("steps op {:?} 需要 from/to 成对", op)),
            }
        };
        let step = match op.as_str() {
            "run" => ProbeStep::Run,
            "query" => ProbeStep::Query,
            "restart" => ProbeStep::Restart,
            "resident-apply" => ProbeStep::ResidentApply,
            "resident-finish" => ProbeStep::ResidentFinish,
            "resident-slice" => {
                let Some(n) = slices else {
                    return err(a.line, "resident-slice 需要 slices");
                };
                ProbeStep::ResidentSlice { slices: n }
            }
            "edit-source" => {
                let (f, t) = need_pair(&from, &to)?;
                ProbeStep::EditSource { from: f, to: t }
            }
            "declaration-invalidate" => {
                let (f, t) = need_pair(&from, &to)?;
                ProbeStep::DeclarationInvalidate { from: f, to: t }
            }
            "partial-rebuild" => {
                let (f, t) = need_pair(&from, &to)?;
                if defs.is_empty() {
                    return err(a.line, "partial-rebuild 需要非空 defs");
                }
                ProbeStep::PartialRebuild { from: f, to: t, defs }
            }
            other => {
                return err(a.line, format!("steps.op {:?} 未知", other))
            }
        };
        out.push(step);
    }
    if out.is_empty() {
        return err(a.line, "steps 需要至少一步");
    }
    Ok(out)
}

fn check_id_of(attrs: &[Attr]) -> Result<Option<String>, AuthorError> {
    for a in attrs {
        if a.key == "check-id" {
            return Ok(Some(want_str(a)?.to_string()));
        }
    }
    Ok(None)
}

fn want_typed_args(a: &Attr) -> Result<Vec<(String, String)>, AuthorError> {
    let CVal::List(items) = &a.value else {
        return err(a.line, "typed-args 需要 [[kind, bits], …] 列表值");
    };
    let mut out = Vec::new();
    for item in items {
        let CVal::List(pair) = item else {
            return err(a.line, "typed-args 每项需要 [kind, bits] 二元组");
        };
        if pair.len() != 2 {
            return err(a.line, "typed-args 每项需要 [kind, bits] 恰好两元素");
        }
        let (CVal::Str(kind), CVal::Str(bits)) = (&pair[0], &pair[1]) else {
            return err(a.line, "typed-args 元素需要字符串对");
        };
        out.push((kind.clone(), bits.clone()));
    }
    Ok(out)
}

fn want_typed_expect(a: &Attr) -> Result<(String, String), AuthorError> {
    let Some(obj) = a.value.as_obj() else {
        return err(a.line, "typed 需要 {kind: \"i64\", bits: \"十六进制\"} 对象值");
    };
    let mut kind = None;
    let mut bits = None;
    for (k, v) in obj {
        match k.as_str() {
            "kind" => {
                let CVal::Str(t) = v else {
                    return err(a.line, "typed.kind 需要字符串");
                };
                kind = Some(t.clone());
            }
            "bits" => {
                let CVal::Str(t) = v else {
                    return err(a.line, "typed.bits 需要完整十六进制字符串");
                };
                bits = Some(t.clone());
            }
            other => return err(a.line, format!("typed.{} 未知（kind/bits）", other)),
        }
    }
    match (kind, bits) {
        (Some(kind), Some(bits)) => Ok((kind, bits)),
        _ => err(a.line, "typed 需要 kind 与 bits 两键齐全"),
    }
}

fn want_tier(a: &Attr) -> Result<String, AuthorError> {
    let v = want_str(a)?;
    if v.trim().is_empty() {
        return err(a.line, "tier 不能为空");
    }
    Ok(v)
}

fn decode_probe_stage(s: &str) -> Option<Stage> {
    if !Stage::supported_for(Adapter::Probe).contains(&s) {
        return None;
    }
    match s {
        "parse" => Some(Stage::Parse),
        "resolve" => Some(Stage::Resolve),
        "typecheck" => Some(Stage::Typecheck),
        "verify" => Some(Stage::Verify),
        "run" => Some(Stage::Run),
        _ => None,
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

fn decode_artifact_subsection(
    doc: &CaseDocument,
    lines: &[(usize, usize)],
    sh: &HeadRef,
    fences: &[&crate::document::DocumentFence],
    sink: &mut Vec<RustArtifact>,
) -> Result<(), AuthorError> {
    // PLAN-1066 T-05: an artifact is judged either by full bytes (`#### <path>`
    // + one fence) or by finite `exists`/`contains` assertions (no fence). The
    // two forms are mutually exclusive; unknown keys, empty/duplicate match
    // sets and `exists: false` are located errors, never silent skips.
    let attrs = collect_leading_attrs(
        doc,
        lines,
        sh.line,
        &["final-newline", "compare", "exists", "contains"],
    )?;
    let mut final_newline = true;
    let mut compare = Compare::Bytes;
    let mut exists: Option<usize> = None;
    let mut contains: Option<(Vec<String>, usize)> = None;
    for a in &attrs {
        match a.key.as_str() {
            "final-newline" => final_newline = want_bool(a)?,
            "compare" => compare = want_compare(a)?,
            "exists" => {
                if !want_bool(a)? {
                    return err(a.line, "exists 只接受 true（不存在的产物不可声明为期望）");
                }
                exists = Some(a.line);
            }
            "contains" => {
                let value = want_list(a)?;
                if value.is_empty() {
                    return err(a.line, "contains 需要非空字符串数组（空匹配集不是断言）");
                }
                let mut seen = std::collections::BTreeSet::new();
                for item in &value {
                    if item.is_empty() {
                        return err(a.line, "contains 不接受空字符串断言");
                    }
                    if !seen.insert(item.clone()) {
                        return err(a.line, format!("重复的 contains 断言 {:?}", item));
                    }
                }
                contains = Some((value, a.line));
            }
            _ => unreachable!(),
        }
    }
    let path = sh.text.trim().to_string();
    crate::admission::validate_virtual_path(&path).map_err(|e| AuthorError {
        line: sh.line,
        message: format!("产物路径 {:?}: {}", path, e),
    })?;
    if sink.iter().any(|a| a.path == path) {
        return err(sh.line, format!("重复的产物路径 {:?}", path));
    }
    let (bytes, open_line, close_line, match_mode) = if exists.is_some() || contains.is_some() {
        if !fences.is_empty() {
            return err(
                sh.line,
                "exists/contains 断言与全文围栏互斥（同一产物只能选一种判据）",
            );
        }
        if attrs
            .iter()
            .any(|a| matches!(a.key.as_str(), "final-newline" | "compare"))
        {
            return err(
                sh.line,
                "final-newline/compare 只属于全文金样；exists/contains 断言不接受载荷选项",
            );
        }
        let list = contains.as_ref().map(|(list, _)| list.clone()).unwrap_or_default();
        let match_mode = if exists.is_some() {
            crate::contract::ArtifactMatch::Exists { contains: list }
        } else {
            crate::contract::ArtifactMatch::Contains { contains: list }
        };
        (
            Vec::new(),
            contains.as_ref().map(|(_, line)| *line).or(exists).unwrap_or(sh.line),
            None,
            match_mode,
        )
    } else {
        if fences.len() != 1 {
            return err(
                sh.line,
                format!("产物小节 {:?} 需要恰好一个围栏（或 exists/contains 断言）", sh.text),
            );
        }
        (
            fence_text(doc, fences[0], final_newline)?.into_bytes(),
            fences[0].open_line,
            fences[0].close_line,
            crate::contract::ArtifactMatch::Bytes,
        )
    };
    sink.push(RustArtifact {
        path,
        bytes,
        open_line,
        close_line,
        compare,
        match_mode,
    });
    Ok(())
}

fn decode_h4_subsection(
    doc: &CaseDocument,
    lines: &[(usize, usize)],
    parent: &HeadRef,
    sh: &HeadRef,
    next_head_line: usize,
    rust: &mut Option<RustSection>,
    probes: &mut [ProbeSection],
    acs: &mut [AcSection],
    syntaxes: &mut [SyntaxSection],
    vm_streams: &mut Vec<(usize, StreamBlock)>,
    auto_files: &mut Vec<(String, String)>,
) -> Result<(), AuthorError> {
    let parent_title = parent.text.trim();
    let fences = direct_fences(doc, sh.line, next_head_line);
    match parent_title {
        "Rust" => {
            // PLAN-1062 T-14: artifact subsections carry any payload
            // language (rust/c/h/python/ts/…). PLAN-1066 T-05: finite
            // `exists`/`contains` assertions replace the byte oracle for
            // project products and are mutually exclusive with a fence.
            let attrs = collect_leading_attrs(
                doc,
                lines,
                sh.line,
                &["final-newline", "compare", "exists", "contains"],
            )?;
            let mut final_newline = true;
            let mut compare = Compare::Bytes;
            let mut exists = false;
            let mut contains: Vec<String> = Vec::new();
            for a in &attrs {
                match a.key.as_str() {
                    "final-newline" => final_newline = want_bool(a)?,
                    "compare" => compare = want_compare(a)?,
                    "exists" => {
                        if !want_bool(a)? {
                            return err(a.line, "exists 只接受 true（不存在的产物不可声明为期望）");
                        }
                        exists = true;
                    }
                    "contains" => {
                        contains = want_list(a)?;
                        if contains.is_empty() {
                            return err(a.line, "contains 需要非空字符串数组（空匹配集不是断言）");
                        }
                        let mut seen = std::collections::BTreeSet::new();
                        for item in &contains {
                            if item.is_empty() {
                                return err(a.line, "contains 不接受空字符串断言");
                            }
                            if !seen.insert(item.clone()) {
                                return err(a.line, format!("重复的 contains 断言 {:?}", item));
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
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
            if exists || !contains.is_empty() {
                if !fences.is_empty() {
                    return err(
                        sh.line,
                        "exists/contains 断言与全文围栏互斥（同一产物只能选一种判据）",
                    );
                }
                if attrs
                    .iter()
                    .any(|a| matches!(a.key.as_str(), "final-newline" | "compare"))
                {
                    return err(
                        sh.line,
                        "final-newline/compare 只属于全文金样；exists/contains 断言不接受载荷选项",
                    );
                }
                let match_mode = if exists {
                    crate::contract::ArtifactMatch::Exists { contains }
                } else {
                    crate::contract::ArtifactMatch::Contains { contains }
                };
                section.subsections.push(RustArtifact {
                    path,
                    bytes: Vec::new(),
                    open_line: sh.line,
                    close_line: None,
                    compare,
                    match_mode,
                });
                return Ok(());
            }
            if fences.len() != 1 {
                return err(
                    sh.line,
                    format!("#### {:?} 需要恰好一个围栏（或 exists/contains 断言）", sh.text),
                );
            }
            let text = fence_text(doc, fences[0], final_newline)?;
            section.subsections.push(RustArtifact {
                path,
                bytes: text.into_bytes(),
                open_line: fences[0].open_line,
                close_line: fences[0].close_line,
                compare,
                match_mode: crate::contract::ArtifactMatch::Bytes,
            });
            Ok(())
        }
        t if t.starts_with("Probe ") => {
            let check_id = t.strip_prefix("Probe ").unwrap_or("").trim();
            let obs_title = sh.text.trim();
            if let Some(num) = obs_title.strip_prefix("Observation ") {
                // CF-07 per-step expectation: attrs + optional json fence
                // carrying the inline bindings payload.
                let order: usize = num.trim().parse().map_err(|_| AuthorError {
                    line: sh.line,
                    message: format!("Observation 序号需要正整数，收到 {:?}", num),
                })?;
                if order == 0 {
                    return err(sh.line, "Observation 序号从 1 开始");
                }
                let attrs = collect_leading_attrs(
                    doc, lines, sh.line,
                    &["cache", "compute-calls", "stages-include", "pe-exit", "final-newline"],
                )?;
                let mut cache = None;
                let mut compute_calls = None;
                let mut stages_include = Vec::new();
                let mut pe_exit = None;
                let mut obs_final_newline = true;
                for a in &attrs {
                    match a.key.as_str() {
                        "final-newline" => obs_final_newline = want_bool(a)?,
                        "cache" => {
                            let v = want_str(a)?.to_string();
                            if !matches!(v.as_str(), "hit" | "miss" | "stale" | "published") {
                                return err(a.line, format!(
                                    "Observation.cache 仅支持 hit/miss/stale/published，收到 {:?}", v));
                            }
                            cache = Some(v);
                        }
                        "compute-calls" => {
                            let n = want_int(a)?;
                            if n < 0 {
                                return err(a.line, "compute-calls 需要非负整数");
                            }
                            compute_calls = Some(n as u32);
                        }
                        "stages-include" => stages_include = want_list(a)?,
                        "pe-exit" => {
                            let n = want_int(a)?;
                            if !(0..=255).contains(&n) {
                                return err(a.line, "pe-exit 需要 0..=255");
                            }
                            pe_exit = Some(n as i32);
                        }
                        _ => unreachable!(),
                    }
                }
                let bindings_text = if fences.len() == 1 {
                    let text = fence_text(doc, &fences[0], obs_final_newline)?;
                    if text.trim().is_empty() {
                        return err(sh.line, "Observation 围栏需要非空 bindings 载荷");
                    }
                    Some(text)
                } else if fences.is_empty() {
                    None
                } else {
                    return err(sh.line, "Observation 至多一个围栏（内联 bindings）");
                };
                let probe = probes.iter_mut().find(|x| x.check_id == check_id).ok_or_else(|| AuthorError {
                    line: sh.line,
                    message: format!("Observation 找不到所属 Probe 节 {:?}", check_id),
                })?;
                if probe.observations.iter().any(|o| o.order == order) {
                    return err(sh.line, format!("重复的 Observation {}", order));
                }
                probe.observations.push(ObservationSection {
                    order,
                    section_line: sh.line,
                    cache,
                    compute_calls,
                    stages_include,
                    pe_exit,
                    bindings_text,
                });
                let _ = &obs_final_newline;
                return Ok(());
            }
            let kind = match sh.text.trim() {
                "stdout" => StreamKind::Stdout,
                "stderr" => StreamKind::Stderr,
                "trace" => StreamKind::Trace,
                other => {
                    let _ = other;
                    // PLAN-1062 T-08 (CF-05): emit recipes carry multi-file
                    // artifact oracles (rs/c/h/json sets).
                    let check_id = parent_title.strip_prefix("Probe ").unwrap_or("").trim();
                    return decode_artifact_subsection(
                        doc, lines, sh, &fences,
                        &mut probes.iter_mut()
                            .find(|x| x.check_id == check_id)
                            .expect("probe section parsed")
                            .artifacts,
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
        _t if parent_title.starts_with("AC ") || parent_title.starts_with("Syntax ") => {
            let kind = match sh.text.trim() {
                "stdout" => StreamKind::Stdout,
                "stderr" => StreamKind::Stderr,
                other => {
                    let _ = other;
                    if parent_title.starts_with("Syntax ") {
                        // PLAN-1062 T-08 (CF-04): syntax oracle artifacts
                        // (lossless reconstruction / outline / canonical).
                        return decode_artifact_subsection(
                            doc, lines, sh, &fences,
                            &mut syntaxes.iter_mut()
                                .find(|x| x.check_id == parent_title.strip_prefix("Syntax ").unwrap_or("").trim())
                                .expect("syntax section parsed")
                                .artifacts,
                        );
                    }
                    if parent_title.starts_with("AC ") {
                        // PLAN-1062 T-08: AC build/run product oracles (e.g.
                        // the intentionally-wrong golden of ac.red).
                        let check_id = parent_title.strip_prefix("AC ").unwrap_or("").trim();
                        return decode_artifact_subsection(
                            doc, lines, sh, &fences,
                            &mut acs.iter_mut()
                                .find(|x| x.check_id == check_id)
                                .expect("ac section parsed")
                                .artifacts,
                        );
                    }
                    return err(
                        sh.line,
                        format!("AC 下的四级标题只接受 stdout / stderr（得到 {:?}）", other),
                    );
                }
            };
            let target_streams: &mut Vec<StreamBlock> = if parent_title.starts_with("AC ") {
                let check_id = parent_title.strip_prefix("AC ").unwrap_or("").trim();
                &mut acs.iter_mut()
                    .find(|a| a.check_id == check_id)
                    .ok_or_else(|| AuthorError {
                        line: sh.line,
                        message: format!("四级流小节找不到所属 AC 节 {:?}", check_id),
                    })?
                    .streams
            } else {
                let check_id = parent_title.strip_prefix("Syntax ").unwrap_or("").trim();
                &mut syntaxes.iter_mut()
                    .find(|x| x.check_id == check_id)
                    .ok_or_else(|| AuthorError {
                        line: sh.line,
                        message: format!("四级流小节找不到所属 Syntax 节 {:?}", check_id),
                    })?
                    .streams
            };
            if target_streams.iter().any(|s| s.kind == kind) {
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
            target_streams.push(StreamBlock {
                kind,
                expect: StreamExpect { text, compare },
            });
            Ok(())
        }
        _t if matches!(
            parent_title,
            "VM result" | "VM diagnostic" | "VM diagnostic message"
        ) =>
        {
            // PLAN-1062 T-05: VM checks may carry byte-stream oracles
            // (stdout-only success pairs in the vm-corpus tier).
            let kind = match sh.text.trim() {
                "stdout" => StreamKind::Stdout,
                "stderr" => StreamKind::Stderr,
                other => {
                    return err(
                        sh.line,
                        format!("VM 下的四级标题只接受 stdout / stderr（得到 {:?}）", other),
                    );
                }
            };
            if vm_streams
                .iter()
                .any(|(line, s)| *line == parent.line && s.kind == kind)
            {
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
            vm_streams.push((
                parent.line,
                StreamBlock {
                    kind,
                    expect: StreamExpect { text, compare },
                },
            ));
            Ok(())
        }
        "Auto" => {
            // PLAN-1062 T-08 (CF-01): additional project files ride ordered
            // #### <virtual-path> subsections, one fence each.
            let path = sh.text.trim().to_string();
            crate::admission::validate_virtual_path(&path).map_err(|e| AuthorError {
                line: sh.line,
                message: format!("附加源路径 {:?}: {}", path, e),
            })?;
            if auto_files.iter().any(|f| f.0 == path) {
                return err(sh.line, format!("重复的附加源路径 {:?}", path));
            }
            if fences.len() != 1 {
                return err(sh.line, "附加源小节需要恰好一个围栏（载荷文本）");
            }
            let attrs = collect_leading_attrs(doc, lines, sh.line, &["final-newline", "eol"])?;
            let mut final_newline = true;
            let mut eol = "lf".to_string();
            for a in &attrs {
                match a.key.as_str() {
                    "final-newline" => final_newline = want_bool(a)?,
                    "eol" => {
                        eol = want_str(a)?.to_string();
                        if eol != "crlf" {
                            return err(a.line, "eol 仅支持 \"crlf\"（默认 lf 不需声明）");
                        }
                    }
                    _ => unreachable!(),
                }
            }
            let mut text = fence_text(doc, fences[0], final_newline)?;
            if eol == "crlf" {
                text = text.replace('\n', "\r\n");
            }
            auto_files.push((path, text));
            Ok(())
        }
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

//! AutoDown 混合语言文档层（PLAN-1058 库级原型）。
//!
//! 现版 AutoDown parser（auto-down @ `5af11a39dd436acd9a25b523f675cec507819d2d`，
//! `packages/engine/auto/parser/markdown_parser.at`，sha256
//! `43299345fa5b0f961aa936f27692ffa9f144c45c36d958b497cdd67b68c2ffea`）为只读
//! 格式参照。本模块在其上叠加混合语言编辑：AutoDown 划定标题、正文与围栏
//! （父层），围栏正文交给按标签选择的子 parser（现装载 `auto`）。
//!
//! 支持子集（样例驱动，逐条对齐现版行为，差异如实记录）：
//! - ATX 标题 `#{1,6}`（后随空格或行尾；尾随 ` ##` 剥离；` ^anchor` 提升为
//!   锚点字段并从标题名剥离——锚点为非空白、不含 `^` 的尾段）；
//! - 段落（连续非空行合并；列表行/`key:: value` 行按段落正文处理）；
//! - 围栏 ```/`~~~`：缩进 <4 开栏；信息串 = 去标记后的整行（trim = 语言标签）；
//!   闭栏 = 同字符、同 run 长度的独立行（缩进 <4，长 run 不闭栏）；
//!   EOF 未闭合 = 正文吞到结尾（尾部全标记字符行剥离；`stripTrailingSpaceLine`
//!   语义：仅"末行全空格且前有换行"时剥空格）；与完整文档模式（isFinal=true）
//!   一致——流式 loading 标记不在本层；
//! - setext 标题、thematic break、数学块、$ 组件、表格、frontmatter、列表
//!   结构**不在子集**：这些行按段落正文处理（`---` 分隔线等行为差异已在
//!   PLAN-1058 记录，派生样例不使用）；
//! - 换行归一化语义与 `normalizeNewlines` 对齐（`\r\n`/孤立 `\r` 均为行界，
//!   `\r\n` 的 `\r` 属行界不属内容）——行区间直接落在原文字节上，围栏正文
//!   逐段 1:1 映射（无去缩进）。
//!
//! 坐标契约（§5.1）：所有对外位置 = (文件 FileId, 修订号, 半开 UTF-8 字节
//! 区间)，原文快照坐标；UTF-16 光标经 [`crate::position::LineIndex`] 换算。
//! 位置句柄不是内容 NodeId；无出版 OccurrenceId。
//!
//! 边界规则（§5.2，父子层归属）：围栏开栏行、闭栏行、行界与空行属父层；
//! 正文区间 `[content_start, content_end)` 属围栏语言；光标落在 `content_end`
//! （紧邻闭栏标记）属父层闭栏；空正文（`content_start == content_end`）的
//! 零宽内部点报围栏语言（显式裁定）。未装载标签（如 `ts`）保留标签与正文
//! 区间，查询仍返回父层数据。
//!
//! 增量（§5.3）：编辑后父层从「包含编辑首行的行起点的最近区域」重扫；若该
//! 区域是段落且编辑行紧邻其后（无空行分隔），再向前多重导出一个区域（段落
//! 吸收只向后达一行）。越过编辑区后，一旦新扫描位置与旧区域起点按总位移
//! 对齐（该旧区域完全位于未触尾部 ⇒ 后续字节逐字节相同），即嫁接未变尾部；
//! 未变围栏的子解析产物以 `Rc` 原样复用（真实子树复用，非重算）。到 EOF
//! 仍未对齐 = 全量重扫（按实记录于 [`Fallback::NoConvergence`]）。

use crate::cst::CstTree;
use crate::lexer::{CompiledScanner, LexDiag, Token};
use crate::parse::{parse_snapshot, ParseOutput};
use crate::position::LineIndex;
use crate::program::{Budget, GrammarProgram, RtError};
use crate::snapshot::{Document, EditRequest, PageStore, Snapshot};
use std::collections::HashMap;
use std::rc::Rc;

/// 装载的子语言（现版仅 `auto` 标签映射到此）。
pub struct AutoGrammar {
    pub program: Rc<GrammarProgram>,
    pub scanner: CompiledScanner,
}

/// 会话内文件身份（打开即分配；重开 = 新 FileId，旧位置只对旧 snapshot 有效）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FileId(pub u64);

/// 文档身份：文件 + 基线内容摘要 + 子语法身份（§5.1）。
#[derive(Clone, Debug)]
pub struct DocIdentity {
    pub file: FileId,
    pub baseline_content_hash: String,
    pub auto_grammar_id: Option<String>,
}

// ---------------------------------------------------------------------------
// 父层区域
// ---------------------------------------------------------------------------

/// 围栏结构信息（全部原文半开字节区间）。
#[derive(Clone, Debug, PartialEq)]
pub struct FenceInfo {
    /// 开栏行（含缩进）的行内容区间（不含行界）。
    pub open: (u64, u64),
    /// 信息串（trim 行去 run 后、行尾前的原文区间；可为零宽）。
    pub info: (u64, u64),
    /// 语言标签 = 信息串 trim（对齐现版 `language = info.trim()`）。
    pub label: String,
    /// 正文区间（首正文行起点 .. 末正文行内容末尾）。
    pub content: (u64, u64),
    /// 闭栏行区间（存在 = 闭合围栏）。
    pub close: Option<(u64, u64)>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RegionKind {
    /// ATX 标题：级别、标题名（已剥 ` ##` 与 ` ^anchor`）、锚点。
    Heading {
        level: u8,
        name: String,
        anchor: Option<String>,
    },
    /// 段落（连续非空行）。
    Paragraph,
    /// 围栏（语言可能已装载或未装载）。
    Fence(FenceInfo),
}

/// 父层区域：块级结构 + 区间（区域不含尾随行界；区域间空行不属于任何区域）。
#[derive(Clone, Debug)]
pub struct Region {
    pub kind: RegionKind,
    pub start: u64,
    pub end: u64,
    /// 围栏子解析产物（仅 `auto` 且已装载；`Rc` 复用 = 真实子树复用）。
    pub embedded: Option<Rc<EmbeddedAnalysis>>,
    /// 区域字节摘要（嫁接对齐与等价比较用；派生时计算）。
    pub content_hash: String,
}

impl Region {
    /// 结构等价（忽略 embedded 的 Rc 身份；比较可观察结构与内容摘要）。
    fn structural_eq(&self, other: &Region) -> bool {
        self.kind == other.kind
            && self.start == other.start
            && self.end == other.end
            && self.content_hash == other.content_hash
            && match (&self.embedded, &other.embedded) {
                (None, None) => true,
                (Some(a), Some(b)) => a.content_hash == b.content_hash,
                _ => false,
            }
    }

    /// 围栏信息（非围栏区域 None）。
    pub fn fence(&self) -> Option<&FenceInfo> {
        match &self.kind {
            RegionKind::Fence(f) => Some(f),
            _ => None,
        }
    }
}

/// 围栏内子语言解析产物（子快照坐标；投影由持有者完成）。
#[derive(Debug)]
pub struct EmbeddedAnalysis {
    pub tree: CstTree,
    pub tokens: Vec<Token>,
    pub lex_diags: Vec<LexDiag>,
    pub parse_diags: Vec<crate::cst::SynDiag>,
    /// 子快照内正文长度（= content 区间长度）。
    pub content_len: u64,
    /// 正文内容摘要（复用判定：内容相同即同一分析可复用）。
    pub content_hash: String,
}

// ---------------------------------------------------------------------------
// 查询产物
// ---------------------------------------------------------------------------

/// 标题大纲项（仅真实标题；代码块内的"貌似标题"永不进入）。
#[derive(Clone, Debug, PartialEq)]
pub struct OutlineEntry {
    pub level: u8,
    pub name: String,
    pub anchor: Option<String>,
    pub start: u64,
    pub end: u64,
}

/// 高亮范围（非 trivia 的子 token；原文坐标；带围栏来源）。
#[derive(Clone, Debug, PartialEq)]
pub struct Highlight {
    pub fence_index: usize,
    pub kind: String,
    pub start: u64,
    pub end: u64,
    pub error: bool,
}

/// 混合文档诊断：父层或子层（投影回原文坐标）。
#[derive(Clone, Debug, PartialEq)]
pub struct MixedDiag {
    pub fence_index: Option<usize>,
    pub message: String,
    pub start: u64,
    pub end: u64,
}

/// 光标语言定位（§5.2 边界规则见模块文档）。
#[derive(Clone, Debug, PartialEq)]
pub enum LanguageAt {
    /// 父层（AutoDown 正文）：标题 / 段落 / 空行 / 围栏标记行 / 行界。
    Parent(ParentBlock),
    /// 围栏正文内：标签 + 是否已装载子 parser。
    FenceBody { label: String, loaded: bool },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParentBlock {
    Heading { level: u8 },
    Paragraph,
    Blank,
    FenceMarker,
}

/// 编辑计量（§5.3：真实扫描区间、重解析、复用与回退理由，按实记录）。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EditMetrics {
    /// 父层实际重扫字节区间（起点 = 重扫区域起点，终点 = 收敛点或 EOF）。
    pub rescan_range: (u64, u64),
    pub rederived_regions: usize,
    pub grafted_regions: usize,
    /// 未变围栏的子分析 Rc 原样复用数（稳定前缀 + 嫁接尾部）。
    pub reused_subtrees: usize,
    pub reparsed_fences: usize,
    pub fallback: Option<Fallback>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Fallback {
    /// 扫描至 EOF 仍未与旧结构对齐（重扫区间 = 到 EOF，如实记录）。
    NoConvergence,
    /// 显式强制全量（V-06 反控入口）。
    ForcedFull,
}

/// 一次修订的完整分析（§5.4：单修订内一致发布；不可变快照）。
#[derive(Clone, Debug)]
pub struct RevisionAnalysis {
    pub revision: u32,
    pub regions: Vec<Region>,
    pub outline: Vec<OutlineEntry>,
    pub highlights: Vec<Highlight>,
    pub diags: Vec<MixedDiag>,
    pub metrics: EditMetrics,
}

impl RevisionAnalysis {
    /// 与从头解析的等价性（忽略计量与 Rc 身份；§6 每步编辑后比对）。
    pub fn equivalent_to(&self, other: &RevisionAnalysis) -> bool {
        self.revision == other.revision
            && self.regions.len() == other.regions.len()
            && self
                .regions
                .iter()
                .zip(other.regions.iter())
                .all(|(a, b)| a.structural_eq(b))
            && self.outline == other.outline
            && self.highlights == other.highlights
            && self.diags == other.diags
    }
}

// ---------------------------------------------------------------------------
// 行扫描（normalizeNewlines 对齐：\n、\r\n、孤立 \r 均为行界）
// ---------------------------------------------------------------------------

struct PhysicalLine {
    /// 行内容区间（不含行界；`\r\n` 的 `\r` 属行界）。
    content: (u64, u64),
    /// 行界之后的位置（下一行起点或 EOF）。
    next: u64,
}

/// 从 offset 起取下一物理行（offset 必须是行起点；调用方保证）。
fn next_line(snap: &Snapshot, offset: u64) -> Result<PhysicalLine, RtError> {
    let len = snap.len();
    debug_assert!(offset <= len);
    let mut i = offset;
    while i < len {
        let b = snap.read_bytes(i, i + 1)?[0];
        if b == b'\n' {
            return Ok(PhysicalLine {
                content: (offset, i),
                next: i + 1,
            });
        }
        if b == b'\r' {
            // \r\n = 单行界；孤立 \r = 行界（normalizeNewlines 语义）。
            let brk = if i + 1 < len && snap.read_bytes(i + 1, i + 2)?[0] == b'\n' {
                2
            } else {
                1
            };
            return Ok(PhysicalLine {
                content: (offset, i),
                next: i + brk,
            });
        }
        i += 1;
    }
    Ok(PhysicalLine {
        content: (offset, len),
        next: len,
    })
}

fn is_blank(b: &[u8]) -> bool {
    b.iter().all(|c| matches!(c, b' ' | b'\t'))
}

/// 缩进宽（对齐 `indentWidth`：空格 1、tab 4、其余终止）。
fn indent_width(b: &[u8]) -> u64 {
    let mut n = 0u64;
    for &c in b {
        if c == b' ' {
            n += 1;
        } else if c == b'\t' {
            n += 4;
        } else {
            break;
        }
    }
    n
}

/// ASCII trim（行分类谓词按字节 ASCII 空白；样例正文不含 Unicode 空白差异）。
fn trim_ascii(b: &[u8]) -> &[u8] {
    let mut s = 0;
    let mut e = b.len();
    while s < e && matches!(b[s], b' ' | b'\t') {
        s += 1;
    }
    while e > s && matches!(b[e - 1], b' ' | b'\t') {
        e -= 1;
    }
    &b[s..e]
}

/// 开栏判定（缩进 <4 且 trim 后以 ``` 或 ~~~ 开头）→ 标记字符。
fn fence_marker(line: &[u8]) -> Option<u8> {
    if indent_width(line) >= 4 {
        return None;
    }
    let t = trim_ascii(line);
    if t.starts_with(b"```") {
        Some(b'`')
    } else if t.starts_with(b"~~~") {
        Some(b'~')
    } else {
        None
    }
}

/// trim 行首标记字符 run 长度（对齐 `fenceMarkerRun`）。
fn marker_run(t: &[u8]) -> usize {
    if t.is_empty() {
        return 0;
    }
    let first = t[0];
    if first != b'`' && first != b'~' {
        return 0;
    }
    t.iter().take_while(|&&c| c == first).count()
}

/// 闭栏判定：缩进 <4、trim、同字符且**长度恰等于**开栏 run。
fn is_close_fence(line: &[u8], marker: u8, run: usize) -> bool {
    if indent_width(line) >= 4 {
        return false;
    }
    let t = trim_ascii(line);
    t.len() == run && t.iter().all(|&c| c == marker)
}

/// 尾随全标记字符行（未闭合围栏正文尾部剥离；`isFenceTailRun`）。
fn is_fence_tail_run(t: &[u8]) -> bool {
    !t.is_empty() && (t.iter().all(|&c| c == b'`') || t.iter().all(|&c| c == b'~'))
}

/// ATX 标题级别（1..=6；后随空格或行尾；`#######` 拒绝）。
fn heading_level(line: &[u8]) -> Option<u8> {
    if indent_width(line) >= 4 {
        return None;
    }
    let t = trim_ascii(line);
    let mut level = 0usize;
    while level < t.len() && t[level] == b'#' {
        level += 1;
    }
    if level == 0 || level > 6 {
        return None;
    }
    if level == t.len() {
        return Some(level as u8);
    }
    if t[level] != b' ' {
        return None;
    }
    Some(level as u8)
}

/// 剥离标题尾随 ` ##` 序列（`stripTrailingHashes`）。
fn strip_trailing_hashes(t: &str) -> &str {
    let trimmed = t.trim_matches(|c: char| c == ' ' || c == '\t');
    if !trimmed.ends_with('#') {
        return trimmed;
    }
    let bytes = trimmed.as_bytes();
    let mut hs = bytes.len() - 1;
    while hs > 0 && bytes[hs - 1] == b'#' {
        hs -= 1;
    }
    let mut ss = hs;
    while ss > 0 && bytes[ss - 1] == b' ' {
        ss -= 1;
    }
    if ss == hs {
        return trimmed;
    }
    trimmed[..ss].trim_end_matches([' ', '\t'])
}

/// ` ^anchor` 提升（子集：尾段、非空、无空白、无 `^`；对齐 applyAnchorsDeep
/// 的单尾段语义）。
fn lift_anchor(content: &str) -> (String, Option<String>) {
    if let Some(pos) = content.rfind(" ^") {
        let token = &content[pos + 2..];
        if !token.is_empty() && !token.chars().any(|c| c.is_whitespace() || c == '^') {
            let name = content[..pos].trim_end_matches([' ', '\t']).to_string();
            return (name, Some(token.to_string()));
        }
    }
    (content.to_string(), None)
}

fn region_hash(snap: &Snapshot, start: u64, end: u64) -> Result<String, RtError> {
    let bytes = snap.read_bytes(start, end)?;
    Ok(crate::identity::domain_hash("ad-region", &[&bytes]))
}

// ---------------------------------------------------------------------------
// 父层块扫描
// ---------------------------------------------------------------------------

/// 从 `from`（必须是区域边界 = 行起点）扫描区域直至 EOF 或 `should_stop`
/// 在区域边界返回 true。返回（区域列表，停止位置；EOF 时 = len）。
///
/// `should_stop` 只在**区域边界**（跳过空行后、开始下一区域前）被调用——
/// 段落吸收/围栏正文期间不得停（停在中段会截断区域结构）。
fn scan_regions(
    snap: &Snapshot,
    from: u64,
    should_stop: Option<&dyn Fn(u64) -> bool>,
) -> Result<(Vec<Region>, u64), RtError> {
    let mut regions = Vec::new();
    let len = snap.len();
    let mut pos = from;
    'outer: while pos < len {
        // 空行跳过；在首个非空行前检查停止条件（此处必为区域边界候选）。
        loop {
            if pos >= len {
                break 'outer;
            }
            let line = next_line(snap, pos)?;
            let content_bytes = snap.read_bytes(line.content.0, line.content.1)?;
            if is_blank(&content_bytes) {
                pos = line.next;
                continue;
            }
            if let Some(stop) = should_stop {
                if stop(pos) {
                    return Ok((regions, pos));
                }
            }
            break;
        }
        let line = next_line(snap, pos)?;
        let content_bytes = snap.read_bytes(line.content.0, line.content.1)?;
        // --- 围栏 ---
        if let Some(marker) = fence_marker(&content_bytes) {
            let t = trim_ascii(&content_bytes);
            let run = marker_run(t);
            // info = trim 行去 run 后到行尾的原文区间（未 trim）。
            let lead_trimmed = (content_bytes.len() - t.len()) as u64;
            let info_abs = (line.content.0 + lead_trimmed + run as u64, line.content.1);
            let info_bytes = snap.read_bytes(info_abs.0, info_abs.1)?;
            let label = String::from_utf8_lossy(trim_ascii(&info_bytes)).into_owned();
            // 找闭栏（或 EOF）。
            let mut body_lines: Vec<(u64, u64)> = Vec::new();
            let mut close: Option<(u64, u64)> = None;
            let mut p = line.next;
            while p < len {
                let bl = next_line(snap, p)?;
                let bc = snap.read_bytes(bl.content.0, bl.content.1)?;
                if is_close_fence(&bc, marker, run) {
                    close = Some(bl.content);
                    break;
                }
                body_lines.push(bl.content);
                p = bl.next;
            }
            let (content_range, region_end, next_pos) = match close {
                Some(c) => {
                    let content = match body_lines.first() {
                        Some(&first) => (first.0, body_lines[body_lines.len() - 1].1),
                        None => (line.next, line.next),
                    };
                    let after = next_line(snap, c.0)?.next;
                    (content, c.1, after)
                }
                None => {
                    // 未闭合：尾部全标记字符行剥离（属正在输入的围栏）。
                    while let Some(&last) = body_lines.last() {
                        let lb = snap.read_bytes(last.0, last.1)?;
                        let lt = trim_ascii(&lb);
                        if lt.is_empty() || !is_fence_tail_run(lt) {
                            break;
                        }
                        body_lines.pop();
                    }
                    // stripTrailingSpaceLine 镜像：仅"末行全空格且其前有换行"
                    // （即前面还有正文行）时剥掉该空格行——单行全空格保留。
                    let content_end = match body_lines.last() {
                        Some(&last) => {
                            let lb = snap.read_bytes(last.0, last.1)?;
                            if body_lines.len() > 1 && trim_ascii(&lb).is_empty() {
                                last.0
                            } else {
                                last.1
                            }
                        }
                        None => line.next,
                    };
                    let content = match body_lines.first() {
                        Some(&first) => (first.0, content_end),
                        None => (line.next, line.next),
                    };
                    (content, len, len)
                }
            };
            let fence = FenceInfo {
                open: line.content,
                info: info_abs,
                label,
                content: content_range,
                close,
            };
            let content_hash = region_hash(snap, line.content.0, region_end)?;
            regions.push(Region {
                kind: RegionKind::Fence(fence),
                start: line.content.0,
                end: region_end,
                embedded: None,
                content_hash,
            });
            pos = next_pos;
            continue;
        }
        // --- ATX 标题 ---
        if let Some(level) = heading_level(&content_bytes) {
            let t = trim_ascii(&content_bytes);
            let content_slice = trim_ascii(&t[level as usize..]);
            let raw = String::from_utf8_lossy(content_slice).into_owned();
            let no_hashes = strip_trailing_hashes(&raw).to_string();
            let (name, anchor) = lift_anchor(&no_hashes);
            let content_hash = region_hash(snap, line.content.0, line.content.1)?;
            regions.push(Region {
                kind: RegionKind::Heading {
                    level,
                    name,
                    anchor,
                },
                start: line.content.0,
                end: line.content.1,
                embedded: None,
                content_hash,
            });
            pos = line.next;
            continue;
        }
        // --- 段落（连续非空行；子集：列表/组件/setext 行均入段落）---
        let mut para_end = line.content.1;
        let mut p = line.next;
        while p < len {
            let bl = next_line(snap, p)?;
            let bc = snap.read_bytes(bl.content.0, bl.content.1)?;
            if is_blank(&bc) || fence_marker(&bc).is_some() || heading_level(&bc).is_some() {
                break;
            }
            para_end = bl.content.1;
            p = bl.next;
        }
        let content_hash = region_hash(snap, line.content.0, para_end)?;
        regions.push(Region {
            kind: RegionKind::Paragraph,
            start: line.content.0,
            end: para_end,
            embedded: None,
            content_hash,
        });
        pos = p;
    }
    Ok((regions, len))
}

// ---------------------------------------------------------------------------
// 子语言解析与投影
// ---------------------------------------------------------------------------

fn is_trivia_token(p: &GrammarProgram, t: &Token) -> bool {
    t.token != u32::MAX
        && p.scanner
            .rules
            .iter()
            .any(|r| r.token == t.token && r.trivia)
}

fn parse_embedded(
    store: &Rc<PageStore>,
    auto: &AutoGrammar,
    snap: &Snapshot,
    content: (u64, u64),
) -> Result<Rc<EmbeddedAnalysis>, RtError> {
    if content.1 <= content.0 {
        // 空正文：装载语言仍记录空分析（内容摘要 = 空内容）。
        return Ok(Rc::new(EmbeddedAnalysis {
            tree: CstTree::empty(),
            tokens: Vec::new(),
            lex_diags: Vec::new(),
            parse_diags: Vec::new(),
            content_len: 0,
            content_hash: crate::identity::domain_hash("ad-fence", &[b""]),
        }));
    }
    let text = snap.read_str(content.0, content.1)?;
    let sub = Snapshot::from_text(store, &text);
    let out: ParseOutput = parse_snapshot(&auto.program, &auto.scanner, &sub, &Budget::default())?;
    let content_hash = crate::identity::domain_hash("ad-fence", &[text.as_bytes()]);
    let parse_diags = out.tree.diags.clone();
    Ok(Rc::new(EmbeddedAnalysis {
        tree: out.tree,
        tokens: out.tokens,
        lex_diags: out.lex_diags,
        parse_diags,
        content_len: sub.len(),
        content_hash,
    }))
}

/// 由区域列表构建一次修订的查询产物（大纲/高亮/诊断投影；需要语法身份
/// 填 token 名与过滤 trivia）。
fn project_analysis(
    revision: u32,
    regions: Vec<Region>,
    metrics: EditMetrics,
    auto: Option<&AutoGrammar>,
) -> RevisionAnalysis {
    let mut outline = Vec::new();
    let mut highlights = Vec::new();
    let mut diags = Vec::new();
    for (idx, r) in regions.iter().enumerate() {
        match &r.kind {
            RegionKind::Heading {
                level,
                name,
                anchor,
            } => {
                outline.push(OutlineEntry {
                    level: *level,
                    name: name.clone(),
                    anchor: anchor.clone(),
                    start: r.start,
                    end: r.end,
                });
            }
            RegionKind::Fence(f) => {
                if let Some(emb) = &r.embedded {
                    let base = f.content.0;
                    if let Some(g) = auto {
                        for t in &emb.tokens {
                            if t.token == u32::MAX || is_trivia_token(&g.program, t) {
                                continue; // 错误叶走诊断；trivia 不进高亮
                            }
                            highlights.push(Highlight {
                                fence_index: idx,
                                kind: g.program.token_name(t.token).to_string(),
                                start: base + t.start,
                                end: base + t.end,
                                error: t.error,
                            });
                        }
                    }
                    let lex = emb
                        .lex_diags
                        .iter()
                        .map(|d| (d.message.clone(), d.start, d.end));
                    let parse = emb
                        .parse_diags
                        .iter()
                        .map(|d| (d.message.clone(), d.start, d.end));
                    for (message, start, end) in lex.chain(parse) {
                        diags.push(MixedDiag {
                            fence_index: Some(idx),
                            message,
                            start: base + start,
                            end: base + end,
                        });
                    }
                }
            }
            RegionKind::Paragraph => {}
        }
    }
    highlights.sort_by_key(|h| (h.start, h.end));
    diags.sort_by_key(|d| (d.start, d.end));
    RevisionAnalysis {
        revision,
        regions,
        outline,
        highlights,
        diags,
        metrics,
    }
}

// ---------------------------------------------------------------------------
// 混合文档（修订链 + 增量分析）
// ---------------------------------------------------------------------------

pub struct MixedDoc {
    file: FileId,
    doc: Document,
    auto: Option<Rc<AutoGrammar>>,
    history: Vec<RevisionAnalysis>,
    line_index: LineIndex,
    baseline_hash: String,
    auto_grammar_id: Option<String>,
}

impl MixedDoc {
    /// 打开文档（修订 0 = 全量扫描）。`auto` 未提供时所有围栏都按未装载处理。
    pub fn open(
        store: &Rc<PageStore>,
        file: FileId,
        text: &str,
        auto: Option<Rc<AutoGrammar>>,
    ) -> Result<MixedDoc, RtError> {
        let doc = Document::from_text(store, text);
        let auto_grammar_id = auto
            .as_deref()
            .map(|a| a.program.grammar_id().as_str().to_string());
        let baseline_hash = doc.snapshot().id().as_str().to_string();
        let line_index = LineIndex::build(doc.snapshot());
        let snap_len = doc.snapshot().len();
        let mut md = MixedDoc {
            file,
            doc,
            auto,
            history: Vec::new(),
            line_index,
            baseline_hash,
            auto_grammar_id,
        };
        let regions = md.scan_and_embed(0, snap_len)?;
        let metrics = EditMetrics {
            rederived_regions: regions.len(),
            rescan_range: (0, snap_len),
            ..EditMetrics::default()
        };
        let analysis = project_analysis(0, regions, metrics, md.auto.as_deref());
        md.history.push(analysis);
        Ok(md)
    }

    pub fn identity(&self) -> DocIdentity {
        DocIdentity {
            file: self.file,
            baseline_content_hash: self.baseline_hash.clone(),
            auto_grammar_id: self.auto_grammar_id.clone(),
        }
    }

    pub fn current_revision(&self) -> u32 {
        self.doc.snapshot().revision()
    }

    pub fn analysis(&self, revision: u32) -> Option<&RevisionAnalysis> {
        self.history.get(revision as usize)
    }

    pub fn current(&self) -> &RevisionAnalysis {
        self.history.last().expect("revision 0 always present")
    }

    pub fn snapshot(&self) -> &Snapshot {
        self.doc.snapshot()
    }

    /// 光标语言定位（§5.2 边界规则）。
    pub fn language_at(&self, revision: u32, offset: u64) -> Result<LanguageAt, RtError> {
        let analysis = self
            .analysis(revision)
            .ok_or_else(|| RtError::BadHandle(format!("revision {revision} not open")))?;
        let snap_len = self.doc.snapshot().len();
        if offset > snap_len {
            return Err(RtError::BadEdit(format!(
                "offset {offset} out of bounds {snap_len}"
            )));
        }
        // 最后一个 start ≤ offset 的区域（无 = offset 在首区域之前的空行）。
        let idx = analysis.regions.partition_point(|r| r.start <= offset);
        let r = match idx.checked_sub(1).and_then(|i| analysis.regions.get(i)) {
            Some(r) => r,
            None => return Ok(LanguageAt::Parent(ParentBlock::Blank)),
        };
        match &r.kind {
            RegionKind::Fence(f) => {
                let (cs, ce) = f.content;
                if (offset >= cs && offset < ce) || (cs == ce && offset == cs) {
                    Ok(LanguageAt::FenceBody {
                        label: f.label.clone(),
                        loaded: r.embedded.is_some(),
                    })
                } else {
                    // 开/闭栏行与行界属父层。
                    Ok(LanguageAt::Parent(ParentBlock::FenceMarker))
                }
            }
            RegionKind::Heading { level, .. } => {
                if offset >= r.start && offset <= r.end {
                    Ok(LanguageAt::Parent(ParentBlock::Heading { level: *level }))
                } else {
                    Ok(LanguageAt::Parent(ParentBlock::Blank))
                }
            }
            RegionKind::Paragraph => {
                if offset >= r.start && offset <= r.end {
                    Ok(LanguageAt::Parent(ParentBlock::Paragraph))
                } else {
                    Ok(LanguageAt::Parent(ParentBlock::Blank))
                }
            }
        }
    }

    /// UTF-16 光标 (line, col) → 字节偏移（当前修订；LineIndex floor 语义）。
    pub fn offset_of_utf16(&self, line: u64, col: u64) -> u64 {
        self.line_index
            .offset_of_utf16(self.doc.snapshot(), line, col)
    }

    /// 字节偏移 → UTF-16 行列（当前修订）。
    pub fn line_col_utf16(&self, offset: u64) -> (u64, u64) {
        self.line_index.line_col_utf16(self.doc.snapshot(), offset)
    }

    /// 增量编辑：应用编辑 → 局部重扫 + 嫁接 + 子分析复用（§5.3）。
    pub fn edit(&mut self, req: &EditRequest) -> Result<u32, RtError> {
        self.edit_with(req, false)
    }

    /// 强制全量入口（V-06 反控：结果与 fresh 等价但零复用）。
    pub fn edit_force_full(&mut self, req: &EditRequest) -> Result<u32, RtError> {
        self.edit_with(req, true)
    }

    fn edit_with(&mut self, req: &EditRequest, force_full: bool) -> Result<u32, RtError> {
        let old_rev = self.current_revision();
        if req.base_revision != old_rev {
            return Err(RtError::BadEdit(format!(
                "base revision {} != current {}",
                req.base_revision, old_rev
            )));
        }
        if req.edits.is_empty() {
            return Ok(old_rev); // 无编辑 = 同修订 no-op
        }
        // 编辑前状态（旧坐标计算必须在应用编辑前完成）。
        let old_analysis = self.history.last().expect("revision 0 present");
        let old_snap = self.doc.snapshot().clone();
        let lo = req.edits.iter().map(|e| e.start).min().unwrap();
        let hi = req.edits.iter().map(|e| e.end.max(e.start)).max().unwrap();
        self.doc.edit(req)?;
        let new_rev = old_rev + 1;
        let new_snap = self.doc.snapshot().clone();
        self.line_index = LineIndex::build(&new_snap);

        let (regions, metrics) = if force_full {
            let regions = self.scan_and_embed(new_rev, new_snap.len())?;
            let m = EditMetrics {
                rederived_regions: regions.len(),
                rescan_range: (0, new_snap.len()),
                fallback: Some(Fallback::ForcedFull),
                ..EditMetrics::default()
            };
            (regions, m)
        } else {
            let (mut regions, mut metrics, middle_lo, middle_hi) =
                self.incremental_regions(old_analysis, &old_snap, lo, hi, req, &new_snap)?;
            // 重导出区域的子解析（装载语言；嫁接/前缀已带 Rc 复用）。
            let store = self.doc.store().clone();
            let mut i = middle_lo;
            while i < middle_hi {
                if let RegionKind::Fence(f) = &regions[i].kind {
                    if f.label == "auto" && self.auto.is_some() {
                        let auto = self.auto.as_deref().unwrap();
                        regions[i].embedded =
                            Some(parse_embedded(&store, auto, &new_snap, f.content)?);
                        metrics.reparsed_fences += 1;
                    }
                }
                i += 1;
            }
            (regions, metrics)
        };
        let analysis = project_analysis(new_rev, regions, metrics, self.auto.as_deref());
        self.history.push(analysis);
        Ok(new_rev)
    }

    /// 当前修订从头全量分析（等价性对照 / 审计入口）。
    pub fn fresh_analysis_of_current(&self) -> Result<RevisionAnalysis, RtError> {
        let rev = self.current_revision();
        let snap_len = self.doc.snapshot().len();
        let regions = self.scan_and_embed(rev, snap_len)?;
        let metrics = EditMetrics {
            rederived_regions: regions.len(),
            rescan_range: (0, snap_len),
            ..EditMetrics::default()
        };
        Ok(project_analysis(
            rev,
            regions,
            metrics,
            self.auto.as_deref(),
        ))
    }

    /// 全量扫描 + 装载语言子解析。
    fn scan_and_embed(&self, revision: u32, snap_len: u64) -> Result<Vec<Region>, RtError> {
        let snap = self.doc.snapshot();
        let (mut regions, stop) = scan_regions(snap, 0, None)?;
        debug_assert_eq!(stop, snap_len, "full scan runs to EOF");
        if let Some(auto) = self.auto.as_deref() {
            let store = self.doc.store();
            for r in regions.iter_mut() {
                if let RegionKind::Fence(f) = &r.kind {
                    if f.label == "auto" {
                        r.embedded = Some(parse_embedded(store, auto, snap, f.content)?);
                    }
                }
            }
        }
        let _ = revision;
        Ok(regions)
    }

    /// §5.3 增量：稳定前缀 + 重扫中段 + 对齐嫁接尾部；未变围栏子分析 Rc 复用。
    /// 返回 (区域, 计量, 重导出段区间)——重导出段无子解析，由调用方装填。
    fn incremental_regions(
        &self,
        old: &RevisionAnalysis,
        old_snap: &Snapshot,
        lo: u64,
        hi: u64,
        req: &EditRequest,
        new_snap: &Snapshot,
    ) -> Result<(Vec<Region>, EditMetrics, usize, usize), RtError> {
        let mut metrics = EditMetrics::default();
        let delta: i64 = req
            .edits
            .iter()
            .map(|e| e.insert.len() as i64 - (e.end - e.start) as i64)
            .sum();

        // 重扫起点（旧坐标）：包含 lo 的行起点处或之前最近的区域起点；
        // 若其上一区域是段落且与之紧邻（无空行分隔），再向前多取一个区域——
        // 编辑可能把分隔结构行（标题/开栏行）改成普通文本，上方段落向下
        // 吸收一行；段落吸收只向后达一行，故后退一步即充分。
        let line_start_lo = line_start_of(old_snap, lo);
        let mut rescan_idx = old.regions.partition_point(|r| r.start <= line_start_lo);
        rescan_idx = rescan_idx.saturating_sub(1);
        if rescan_idx > 0 {
            if let Some(above) = old.regions.get(rescan_idx - 1) {
                let adjacent = matches!(above.kind, RegionKind::Paragraph)
                    && next_line(old_snap, above.end)
                        .map(|n| n.next == old.regions[rescan_idx].start)
                        .unwrap_or(false);
                if adjacent {
                    rescan_idx -= 1;
                }
            }
        }
        let rescan_from = old.regions.get(rescan_idx).map(|r| r.start).unwrap_or(0);
        debug_assert!(rescan_from <= lo, "rescan start must precede the edit");

        // 稳定前缀：字节与结构原样保留（子分析 Rc 复用）。
        let mut regions: Vec<Region> = Vec::with_capacity(old.regions.len() + 4);
        for r in &old.regions[..rescan_idx] {
            if r.embedded.is_some() {
                metrics.reused_subtrees += 1;
            }
            regions.push(r.clone());
        }
        metrics.grafted_regions += rescan_idx;

        // 对齐判定（只在区域边界调用）：新位置 == 某旧区域起点 + delta，
        // 且该旧区域完全位于未触尾部（start ≥ hi ⇒ 其后字节逐字节相同）。
        let tail_starts: Vec<u64> = old
            .regions
            .iter()
            .filter(|r| r.start >= hi)
            .map(|r| r.start)
            .collect();
        let last_edit_new_end = (hi as i64 + delta) as u64;
        let converged_at = |pos: u64| -> Option<u64> {
            if pos < last_edit_new_end || tail_starts.is_empty() {
                return None;
            }
            let target = pos as i64 - delta;
            if target < 0 {
                return None;
            }
            let target = target as u64;
            let i = tail_starts.partition_point(|&s| s < target);
            if i < tail_starts.len() && tail_starts[i] == target {
                Some(target)
            } else {
                None
            }
        };

        // 新快照重扫（从 rescan_from；前缀字节相同 ⇒ 同偏移有效）。
        let (mut rederived, stop_pos) = scan_regions(
            new_snap,
            rescan_from,
            Some(&|pos: u64| converged_at(pos).is_some()),
        )?;
        let rederived_from = regions.len();
        regions.append(&mut rederived);
        let rederived_to = regions.len();
        metrics.rederived_regions = rederived_to - rederived_from;
        metrics.rescan_range = (rescan_from, stop_pos);

        if let Some(aligned_old_start) = converged_at(stop_pos) {
            // 嫁接尾部：旧区域平移 delta；子分析 Rc 原样复用。
            let j = old.regions.partition_point(|r| r.start < aligned_old_start);
            for r in &old.regions[j..] {
                let mut nr = r.clone();
                shift_region(&mut nr, delta);
                if r.embedded.is_some() {
                    metrics.reused_subtrees += 1;
                }
                regions.push(nr);
                metrics.grafted_regions += 1;
            }
        } else {
            // 到 EOF 未对齐：重扫区间即到 EOF（如实记录；区域已全部重导出）。
            metrics.fallback = Some(Fallback::NoConvergence);
        }
        Ok((regions, metrics, rederived_from, rederived_to))
    }

    pub fn line_index(&self) -> &LineIndex {
        &self.line_index
    }
}

/// 行起点（offset 所在行；LineIndex 基于 `\n`，与行扫描语义一致）。
fn line_start_of(snap: &Snapshot, offset: u64) -> u64 {
    // 逐字节回找行界（\n 或 \r）：样例规模下可接受；LineIndex 的
    // line_col_utf8/offset_of_utf8 组合需当前修订索引，这里显式独立实现
    // 以避免新旧索引混用。
    let mut i = offset.min(snap.len());
    while i > 0 {
        let b = snap.read_bytes(i - 1, i).map(|b| b[0]).unwrap_or(0);
        if b == b'\n' || b == b'\r' {
            break;
        }
        i -= 1;
    }
    i
}

fn shift_region(r: &mut Region, delta: i64) {
    r.start = (r.start as i64 + delta) as u64;
    r.end = (r.end as i64 + delta) as u64;
    if let RegionKind::Fence(f) = &mut r.kind {
        f.open.0 = (f.open.0 as i64 + delta) as u64;
        f.open.1 = (f.open.1 as i64 + delta) as u64;
        f.info.0 = (f.info.0 as i64 + delta) as u64;
        f.info.1 = (f.info.1 as i64 + delta) as u64;
        f.content.0 = (f.content.0 as i64 + delta) as u64;
        f.content.1 = (f.content.1 as i64 + delta) as u64;
        if let Some(c) = &mut f.close {
            c.0 = (c.0 as i64 + delta) as u64;
            c.1 = (c.1 as i64 + delta) as u64;
        }
    }
}

// ---------------------------------------------------------------------------
// 无 UI 编辑会话消费者（§2 / V-05）
// ---------------------------------------------------------------------------

/// 一次查询视图：单文件、单修订的一致产物集（§5.4）。
#[derive(Clone, Debug)]
pub struct SessionView {
    pub file: FileId,
    pub revision: u32,
    pub outline: Vec<OutlineEntry>,
    pub highlights: Vec<Highlight>,
    pub diags: Vec<MixedDiag>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeliveryOutcome {
    /// 发布（该文件当前修订且首个结果；首个发布胜出）。
    Published,
    /// 迟到：目标修订已被更新修订取代 / 文件已不在 / 已有发布（不得覆盖）。
    Late,
}

/// 多文件编辑会话：打开/编辑/定位/查询 + 版本化发布屏障。
///
/// 屏障语义与 [`crate::edit::VersionedPublish`] 一致（首个当前修订结果胜出、
/// 迟到只入 late 表）；按 (FileId, 修订) 键控，值/诊断/高亮不跨文件。
pub struct EditorSession {
    store: Rc<PageStore>,
    docs: HashMap<FileId, MixedDoc>,
    next_file: u64,
    /// 每文件一个发布槽：首个该文件当前修订的结果胜出（值/诊断/高亮不串文件）。
    published: HashMap<FileId, SessionView>,
    late: Vec<SessionView>,
    auto: Option<Rc<AutoGrammar>>,
}

impl EditorSession {
    pub fn new(auto: Option<Rc<AutoGrammar>>) -> EditorSession {
        EditorSession {
            store: Rc::new(PageStore::new()),
            docs: HashMap::new(),
            next_file: 1,
            published: HashMap::new(),
            late: Vec::new(),
            auto,
        }
    }

    /// 打开文件（新 FileId；重开 = 再次 open，旧 FileId 位置仍只对旧快照有效）。
    pub fn open(&mut self, text: &str) -> Result<FileId, RtError> {
        let id = FileId(self.next_file);
        self.next_file += 1;
        let doc = MixedDoc::open(&self.store, id, text, self.auto.clone())?;
        self.docs.insert(id, doc);
        Ok(id)
    }

    pub fn edit(&mut self, file: FileId, req: &EditRequest) -> Result<u32, RtError> {
        self.doc_mut(file)?.edit(req)
    }

    pub fn edit_force_full(&mut self, file: FileId, req: &EditRequest) -> Result<u32, RtError> {
        self.doc_mut(file)?.edit_force_full(req)
    }

    /// 构建查询视图（对历史修订亦合法——视图绑定其自身快照；旧位置只对旧
    /// snapshot 有效）。
    pub fn query(&self, file: FileId, revision: u32) -> Result<SessionView, RtError> {
        let doc = self.doc(file)?;
        let a = doc
            .analysis(revision)
            .ok_or_else(|| RtError::BadHandle(format!("revision {revision} not open")))?;
        Ok(SessionView {
            file,
            revision,
            outline: a.outline.clone(),
            highlights: a.highlights.clone(),
            diags: a.diags.clone(),
        })
    }

    /// 光标语言定位（透传到对应文档的修订）。
    pub fn language_at(
        &self,
        file: FileId,
        revision: u32,
        offset: u64,
    ) -> Result<LanguageAt, RtError> {
        self.doc(file)?.language_at(revision, offset)
    }

    /// UTF-16 光标 → 字节偏移（该文件当前修订）。
    pub fn offset_of_utf16(&self, file: FileId, line: u64, col: u64) -> Result<u64, RtError> {
        Ok(self.doc(file)?.offset_of_utf16(line, col))
    }

    /// 交付查询结果：仅当（文件仍打开、目标 = 该文件当前修订、且该文件尚无
    /// 已发布结果）时发布；否则入 late（迟到不得覆盖，V-05）。
    pub fn deliver(&mut self, view: SessionView) -> DeliveryOutcome {
        let current = match self.docs.get(&view.file) {
            Some(d) => d.current_revision(),
            None => {
                self.late.push(view);
                return DeliveryOutcome::Late;
            }
        };
        if view.revision != current || self.published.contains_key(&view.file) {
            self.late.push(view);
            return DeliveryOutcome::Late;
        }
        self.published.insert(view.file, view);
        DeliveryOutcome::Published
    }

    /// 某文件的已发布视图（无 = 尚无当前修订结果发布）。
    pub fn published(&self, file: FileId) -> Option<&SessionView> {
        self.published.get(&file)
    }

    pub fn late(&self) -> &[SessionView] {
        &self.late
    }

    pub fn current_revision(&self, file: FileId) -> Result<u32, RtError> {
        Ok(self.doc(file)?.current_revision())
    }

    pub fn doc(&self, file: FileId) -> Result<&MixedDoc, RtError> {
        self.docs
            .get(&file)
            .ok_or_else(|| RtError::BadHandle(format!("file {file:?} not open")))
    }

    fn doc_mut(&mut self, file: FileId) -> Result<&mut MixedDoc, RtError> {
        self.docs
            .get_mut(&file)
            .ok_or_else(|| RtError::BadHandle(format!("file {file:?} not open")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap_text(text: &str) -> Snapshot {
        Snapshot::from_text(&Rc::new(PageStore::new()), text)
    }

    #[test]
    fn line_split_crlf_and_lone_cr() {
        let s = snap_text("a\r\nb\rc\nd");
        let mut pos = 0;
        let mut lines = Vec::new();
        loop {
            let l = next_line(&s, pos).unwrap();
            lines.push(s.read_str(l.content.0, l.content.1).unwrap());
            pos = l.next;
            if pos >= s.len() {
                break;
            }
        }
        assert_eq!(lines, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn fence_predicates_mirror_current_parser() {
        assert_eq!(fence_marker(b"```ts"), Some(b'`'));
        assert_eq!(fence_marker(b"  ~~~x"), Some(b'~'));
        assert_eq!(fence_marker(b"    ```"), None, "indent >= 4");
        assert_eq!(fence_marker(b"``"), None, "run < 3");
        assert!(is_close_fence(b"```", b'`', 3));
        assert!(!is_close_fence(b"````", b'`', 3), "longer run no close");
        assert!(!is_close_fence(b"```x", b'`', 3));
        assert!(!is_close_fence(b"    ```", b'`', 3));
        assert!(is_close_fence(b"~~~", b'~', 3));
        assert_eq!(heading_level(b"# a").unwrap(), 1);
        assert_eq!(heading_level(b"###### x").unwrap(), 6);
        assert_eq!(heading_level(b"####### x"), None, "7 hashes");
        assert_eq!(heading_level(b"#x"), None, "no space");
        assert_eq!(heading_level(b"###"), Some(3), "bare hashes");
        assert_eq!(strip_trailing_hashes("Title ##"), "Title");
        assert_eq!(strip_trailing_hashes("Title ###  "), "Title");
        assert_eq!(strip_trailing_hashes("A # B"), "A # B");
        let (name, anchor) = lift_anchor("Cards Probe ^cards-probe");
        assert_eq!(
            (name.as_str(), anchor.as_deref()),
            ("Cards Probe", Some("cards-probe"))
        );
    }

    #[test]
    fn scan_fences_unclosed_tail_pop() {
        // 未闭合围栏：尾部全标记行剥离属围栏，正文不含它。
        let s = snap_text("```auto\nfn a() i32 { return 1 }\n``\n");
        let (regions, _) = scan_regions(&s, 0, None).unwrap();
        assert_eq!(regions.len(), 1);
        let f = regions[0].fence().unwrap();
        assert_eq!(f.label, "auto");
        assert!(f.close.is_none());
        let body = s.read_str(f.content.0, f.content.1).unwrap();
        assert_eq!(body, "fn a() i32 { return 1 }");
    }
}

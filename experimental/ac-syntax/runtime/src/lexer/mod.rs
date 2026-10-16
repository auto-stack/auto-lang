//! 模式驱动 scanner：消费分页 snapshot，产出无损 token 流 + checkpoint。
//!
//! 匹配语义（T-01 冻结）：候选规则中取**最长匹配**；同长取**声明序**（显式
//! 优先级，不依赖 HashMap 顺序）。零长度匹配由编译期拒绝。无匹配字节 →
//! 单字节 ERROR 叶（流不中断，保持无损）。trivia（空白/注释）保留原拼写范围。
//!
//! checkpoint 含完整 mode/未闭合 token 状态（PLAN-787 §5.1：不能只有 offset）。

use crate::program::{GrammarProgram, Matcher, RtError, TokenId};
use crate::snapshot::Snapshot;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    /// token id（grammar 命名空间；trivia 也是 token，is_trivia 查 grammar）。
    pub token: TokenId,
    pub start: u64,
    pub end: u64,
    /// token 起始行（1-based，按 `\n` 计；CRLF 中 `\r` 属行尾内容）。
    pub line: u32,
    /// 词法错误（坏 token 仍占字节，进入 CST error 叶）。
    pub error: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LexDiag {
    pub message: String,
    pub start: u64,
    pub end: u64,
}

/// 未闭合 token 的跨块状态（携带 token 原始起点，恢复时产出完整 token）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidToken {
    Str {
        start: u64,
        quote: u8,
        escape_pending: bool,
    },
    Block {
        start: u64,
        depth: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScanCheckpoint {
    pub offset: u64,
    pub mode: u32,
    pub mid: Option<MidToken>,
    pub line: u32,
}

#[derive(Clone, Debug, Default)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub diags: Vec<LexDiag>,
    /// 实际扫描字节数（重扫计量，V-06）。
    pub scanned_bytes: u64,
}

/// 按首字节分桶的规则索引（编译期一次性构建，运行期确定性遍历）。
pub struct CompiledScanner {
    /// first_byte(0..=255) -> 规则索引（literal 按长度降序在前，其余声明序）。
    buckets: Vec<Vec<u32>>,
}

impl CompiledScanner {
    pub fn compile(p: &GrammarProgram) -> Result<CompiledScanner, RtError> {
        let mut buckets = vec![Vec::new(); 256];
        for (i, r) in p.scanner.rules.iter().enumerate() {
            let name = p.token_name(r.token);
            match &r.matcher {
                Matcher::Literal(s) => {
                    let Some(&b0) = s.as_bytes().first() else {
                        return Err(RtError::InvalidProgram(format!(
                            "scan rule #{} (`{name}`): empty literal",
                            i
                        )));
                    };
                    buckets[b0 as usize].push(i as u32);
                }
                Matcher::Charset { first, .. } => {
                    if first.non_ascii {
                        return Err(RtError::InvalidProgram(format!(
                            "scan rule #{} (`{name}`): non-ASCII charset not supported (ASCII-only language surface)",
                            i
                        )));
                    }
                    let mut placed = false;
                    for b in 0u16..128 {
                        if first.contains(b as u8) {
                            buckets[b as usize].push(i as u32);
                            placed = true;
                        }
                    }
                    if !placed {
                        return Err(RtError::InvalidProgram(format!(
                            "scan rule #{} (`{name}`): charset matches nothing (non-progressing rule)",
                            i
                        )));
                    }
                }
                Matcher::LineComment(s) => {
                    let Some(&b0) = s.as_bytes().first() else {
                        return Err(RtError::InvalidProgram(format!(
                            "scan rule #{} (`{name}`): empty comment start",
                            i
                        )));
                    };
                    buckets[b0 as usize].push(i as u32);
                }
                Matcher::BlockComment { start, .. } => {
                    let Some(&b0) = start.as_bytes().first() else {
                        return Err(RtError::InvalidProgram(format!(
                            "scan rule #{} (`{name}`): empty comment start",
                            i
                        )));
                    };
                    buckets[b0 as usize].push(i as u32);
                }
                Matcher::StringLit { quote, .. } => buckets[*quote as usize].push(i as u32),
            }
        }
        // 各桶内 literal 类按长度降序稳定排序（同长保持声明序），
        // 与 charset 混排时统一按 (长度, 声明序) 裁定。
        for b in buckets.iter_mut() {
            b.sort_by(|&x, &y| {
                let lx = lit_len(&p.scanner.rules[x as usize].matcher);
                let ly = lit_len(&p.scanner.rules[y as usize].matcher);
                ly.cmp(&lx).then(x.cmp(&y))
            });
        }
        Ok(CompiledScanner { buckets })
    }
}

fn lit_len(m: &Matcher) -> usize {
    match m {
        Matcher::Literal(s) | Matcher::LineComment(s) => s.len(),
        Matcher::BlockComment { start, .. } => start.len(),
        _ => 0,
    }
}

/// 分页读取窗口：线性扫描时按需拉页（复用 snapshot 内容页）。
pub struct Window<'s> {
    snap: &'s Snapshot,
    buf: Vec<u8>,
    buf_start: u64,
    pos: u64,
    line: u32,
    /// 扫描硬限（F787-R1-03）：越过即视为 EOF——增量重扫的有界保证，
    /// 未闭合 token 的内部推进同样无法越界。None = 无限制（fresh 全量）。
    limit: Option<u64>,
}

const WINDOW_BYTES: usize = 32 * 1024;

impl<'s> Window<'s> {
    pub fn new(snap: &'s Snapshot, offset: u64, line: u32) -> Result<Self, RtError> {
        Self::new_with_limit(snap, offset, line, None)
    }
    fn new_with_limit(
        snap: &'s Snapshot,
        offset: u64,
        line: u32,
        limit: Option<u64>,
    ) -> Result<Self, RtError> {
        let mut w = Window {
            snap,
            buf: Vec::new(),
            buf_start: offset,
            pos: offset,
            line,
            limit,
        };
        w.fill(offset)?;
        Ok(w)
    }
    /// 设扫描硬限（含）；仅允许在扫描开始前设置。
    pub fn set_limit(&mut self, limit: u64) {
        self.limit = Some(limit);
    }
    fn fill(&mut self, at: u64) -> Result<(), RtError> {
        let hard = self.limit.unwrap_or(u64::MAX);
        let end = (at + WINDOW_BYTES as u64).min(self.snap.len()).min(hard);
        if end < at {
            return Ok(());
        }
        self.buf = self.snap.read_bytes(at, end)?;
        self.buf_start = at;
        Ok(())
    }
    #[inline]
    fn byte(&mut self, at: u64) -> Result<Option<u8>, RtError> {
        if at >= self.hard_len() {
            return Ok(None);
        }
        if at < self.buf_start || at >= self.buf_start + self.buf.len() as u64 {
            self.fill(at)?;
        }
        Ok(Some(self.buf[(at - self.buf_start) as usize]))
    }
    /// 有效文档长 = min(snap.len(), limit)——limit 之后视为 EOF。
    fn hard_len(&self) -> u64 {
        self.limit.unwrap_or(u64::MAX).min(self.snap.len())
    }
    fn count_newlines(&mut self, from: u64, to: u64) -> Result<u32, RtError> {
        let mut n = 0;
        let mut i = from;
        while i < to {
            if self.byte(i)? == Some(b'\n') {
                n += 1;
            }
            i += 1;
        }
        Ok(n)
    }
}

pub struct Scanner<'p, 's> {
    p: &'p GrammarProgram,
    cs: &'p CompiledScanner,
    w: Window<'s>,
    mode: u32,
    mid: Option<MidToken>,
}

impl<'p, 's> Scanner<'p, 's> {
    pub fn start(
        p: &'p GrammarProgram,
        cs: &'p CompiledScanner,
        snap: &'s Snapshot,
        cp: &ScanCheckpoint,
    ) -> Result<Self, RtError> {
        Self::start_bounded(p, cs, snap, cp, None)
    }

    /// 带扫描硬限的启动（F787-R1-03：增量重扫有界窗口——limit 之前生效，
    /// 首次拉窗即不越界）。
    pub fn start_bounded(
        p: &'p GrammarProgram,
        cs: &'p CompiledScanner,
        snap: &'s Snapshot,
        cp: &ScanCheckpoint,
        limit: Option<u64>,
    ) -> Result<Self, RtError> {
        let w = Window::new_with_limit(snap, cp.offset, cp.line, limit)?;
        Ok(Scanner {
            p,
            cs,
            w,
            mode: cp.mode,
            mid: cp.mid.clone(),
        })
    }

    pub fn checkpoint(&self) -> ScanCheckpoint {
        ScanCheckpoint {
            offset: self.w.pos,
            mode: self.mode,
            mid: self.mid.clone(),
            line: self.w.line,
        }
    }

    /// 扫描硬限（F787-R1-03）：增量重扫的有界窗口；越界字节视为 EOF。
    pub fn set_limit(&mut self, limit: u64) {
        self.w.set_limit(limit);
    }

    /// 扫描下一项；None = EOF。
    pub fn next_token(&mut self) -> Result<Option<Token>, RtError> {
        let mid_start = self.mid.as_ref().map(|m| match m {
            MidToken::Str { start, .. } => *start,
            MidToken::Block { start, .. } => *start,
        });
        let start = mid_start.unwrap_or(self.w.pos);
        let line = self.w.line;
        let scan_at = self.w.pos;
        if scan_at >= self.snap_len() && mid_start.is_none() {
            return Ok(None);
        }
        let m = self.match_at(scan_at)?;
        let Some(m) = m else {
            // 无匹配：单字节错误叶（受控、无损、前进）。
            self.advance_past(scan_at + 1)?;
            return Ok(Some(Token {
                token: u32::MAX,
                start: scan_at,
                end: scan_at + 1,
                line,
                error: true,
            }));
        };
        let rule = &self.p.scanner.rules[m.rule as usize];
        let end = start + m.len;
        let err = m.error;
        self.mode = rule.next_mode;
        self.mid = None;
        self.advance_past(end)?;
        Ok(Some(Token {
            token: rule.token,
            start,
            end,
            line,
            error: err,
        }))
    }

    fn snap_len(&self) -> u64 {
        self.w.hard_len()
    }

    fn advance_past(&mut self, end: u64) -> Result<(), RtError> {
        let n = self.w.count_newlines(self.w.pos, end)?;
        self.w.line += n;
        self.w.pos = end;
        Ok(())
    }

    fn match_at(&mut self, at: u64) -> Result<Option<Matched>, RtError> {
        // 未闭合恢复：从 mid 状态继续扫描到闭合或 EOF。
        if let Some(mid) = self.mid.clone() {
            return self.resume_mid(mid, at);
        }
        let first = match self.w.byte(at)? {
            Some(b) => b,
            None => return Ok(None),
        };
        let mut best: Option<Matched> = None;
        for &ri in self.cs.buckets[first as usize].iter() {
            let r = &self.p.scanner.rules[ri as usize];
            if r.mode != self.mode {
                continue;
            }
            if let Some(m) = self.try_rule(ri, at)? {
                let better = match &best {
                    None => true,
                    Some(b) => m.len > b.len || (m.len == b.len && ri < b.rule),
                };
                if better {
                    best = Some(m);
                }
            }
        }
        Ok(best)
    }

    fn try_rule(&mut self, ri: u32, at: u64) -> Result<Option<Matched>, RtError> {
        let r = &self.p.scanner.rules[ri as usize];
        let matched: Option<(u64, bool)> = match &r.matcher {
            Matcher::Literal(s) => self
                .try_literal(at, s.as_bytes())?
                .map(|_| (s.len() as u64, false)),
            Matcher::Charset {
                first,
                cont,
                min_len,
            } => self
                .try_charset(at, first, cont, *min_len)?
                .map(|l| (l, false)),
            Matcher::LineComment(start) => {
                if let Some(()) = self.try_literal(at, start.as_bytes())? {
                    let mut i = at + start.len() as u64;
                    while let Some(b) = self.w.byte(i)? {
                        if b == b'\n' {
                            break;
                        }
                        i += 1;
                    }
                    Some((i - at, false))
                } else {
                    None
                }
            }
            Matcher::BlockComment { start, end, nested } => {
                if let Some(()) = self.try_literal(at, start.as_bytes())? {
                    self.scan_block(at, start, end, *nested)?
                } else {
                    None
                }
            }
            Matcher::StringLit { quote, escape } => {
                if self.w.byte(at)? == Some(*quote) {
                    self.scan_string(at + 1, *quote, *escape)?
                } else {
                    None
                }
            }
        };
        Ok(matched.map(|(len, error)| Matched {
            rule: ri,
            len,
            error,
        }))
    }

    fn try_literal(&mut self, at: u64, lit: &[u8]) -> Result<Option<()>, RtError> {
        for (k, &c) in lit.iter().enumerate() {
            if self.w.byte(at + k as u64)? != Some(c) {
                return Ok(None);
            }
        }
        Ok(Some(()))
    }

    fn try_charset(
        &mut self,
        at: u64,
        first: &crate::program::CharSet,
        cont: &crate::program::CharSet,
        min_len: u32,
    ) -> Result<Option<u64>, RtError> {
        let f = match self.w.byte(at)? {
            Some(b) if first.contains(b) || (b >= 0x80 && first.non_ascii) => b,
            _ => return Ok(None),
        };
        let mut n = 1u64;
        loop {
            match self.w.byte(at + n)? {
                Some(b) if cont.contains(b) || (b >= 0x80 && cont.non_ascii) => n += 1,
                Some(_) | None => break,
            }
        }
        if n < min_len as u64 {
            let _ = f;
            return Ok(None);
        }
        Ok(Some(n))
    }

    fn scan_block(
        &mut self,
        at: u64,
        start: &str,
        end: &str,
        nested: bool,
    ) -> Result<Option<(u64, bool)>, RtError> {
        let sb = start.as_bytes();
        let eb = end.as_bytes();
        let mut i = at + sb.len() as u64;
        let mut depth = 1u32;
        loop {
            match self.w.byte(i)? {
                None => return Ok(Some((self.snap_len() - at, true))),
                Some(_) => {
                    if self.try_literal(i, eb)? == Some(()) && depth == 1 {
                        return Ok(Some((i + eb.len() as u64 - at, false)));
                    }
                    if nested && self.try_literal(i, sb)? == Some(()) {
                        depth += 1;
                        i += sb.len() as u64;
                        continue;
                    }
                    if self.try_literal(i, eb)? == Some(()) {
                        depth -= 1;
                        i += eb.len() as u64;
                        continue;
                    }
                    i += 1;
                }
            }
        }
    }

    fn scan_string(
        &mut self,
        from: u64,
        quote: u8,
        escape: bool,
    ) -> Result<Option<(u64, bool)>, RtError> {
        let mut i = from;
        let mut pending = false;
        loop {
            match self.w.byte(i)? {
                None => return Ok(Some((self.snap_len() - from + 1, true))), // 从 quote 起整段（含 quote）
                Some(b) => {
                    if pending {
                        pending = false;
                        i += 1;
                        continue;
                    }
                    if escape && b == b'\\' {
                        pending = true;
                        i += 1;
                        continue;
                    }
                    if b == quote {
                        return Ok(Some((i + 1 - from + 1, false))); // +1 = 含起始 quote
                    }
                    i += 1;
                }
            }
        }
    }

    fn resume_mid(&mut self, mid: MidToken, at: u64) -> Result<Option<Matched>, RtError> {
        // mid 状态由 checkpoint 恢复设置：从 at 继续，直到闭合或 EOF；
        // token 覆盖 [mid.start(), 闭合点)，长度相对 at（Token.start=mid 起点，
        // 由 next_token 以 start 参数传入——见下）。
        match mid {
            MidToken::Str {
                start,
                quote,
                escape_pending,
            } => {
                let ri = self
                    .find_rule(|r| matches!(&r.matcher, Matcher::StringLit { quote: q, .. } if *q == quote))
                    .ok_or_else(|| RtError::InvalidProgram("checkpoint mid-str has no rule".into()))?;
                let escape = matches!(
                    &self.p.scanner.rules[ri as usize].matcher,
                    Matcher::StringLit { escape: true, .. }
                );
                let mut i = at;
                let mut pending = escape_pending;
                loop {
                    match self.w.byte(i)? {
                        None => {
                            let end = self.snap_len();
                            return Ok(Some(Matched {
                                rule: ri,
                                len: end - start,
                                error: true,
                            }));
                        }
                        Some(b) => {
                            if pending {
                                pending = false;
                                i += 1;
                                continue;
                            }
                            if escape && b == b'\\' {
                                pending = true;
                                i += 1;
                                continue;
                            }
                            if b == quote {
                                return Ok(Some(Matched {
                                    rule: ri,
                                    len: i + 1 - start,
                                    error: false,
                                }));
                            }
                            i += 1;
                        }
                    }
                }
            }
            MidToken::Block { start, depth } => {
                let ri = self
                    .find_rule(|r| matches!(&r.matcher, Matcher::BlockComment { .. }))
                    .ok_or_else(|| {
                        RtError::InvalidProgram("checkpoint mid-block has no rule".into())
                    })?;
                let (sb, eb, nested) = match &self.p.scanner.rules[ri as usize].matcher {
                    Matcher::BlockComment { start, end, nested } => {
                        (start.as_bytes().to_vec(), end.as_bytes().to_vec(), *nested)
                    }
                    _ => unreachable!(),
                };
                let mut i = at;
                let mut d = depth;
                loop {
                    match self.w.byte(i)? {
                        None => {
                            let end = self.snap_len();
                            return Ok(Some(Matched {
                                rule: ri,
                                len: end - start,
                                error: true,
                            }));
                        }
                        Some(_) => {
                            if self.try_literal(i, &eb)? == Some(()) {
                                d -= 1;
                                i += eb.len() as u64;
                                if d == 0 {
                                    return Ok(Some(Matched {
                                        rule: ri,
                                        len: i - start,
                                        error: false,
                                    }));
                                }
                                continue;
                            }
                            if nested && self.try_literal(i, &sb)? == Some(()) {
                                d += 1;
                                i += sb.len() as u64;
                                continue;
                            }
                            i += 1;
                        }
                    }
                }
            }
        }
    }

    fn find_rule<F: Fn(&crate::program::ScanRule) -> bool>(&self, f: F) -> Option<u32> {
        self.p
            .scanner
            .rules
            .iter()
            .position(|r| f(r) && r.mode == self.mode)
            .map(|i| i as u32)
    }
}

struct Matched {
    rule: u32,
    len: u64,
    error: bool,
}

/// 全量扫描（fresh 路径）。
pub fn lex_all(
    p: &GrammarProgram,
    cs: &CompiledScanner,
    snap: &Snapshot,
) -> Result<LexOutput, RtError> {
    lex_from(
        p,
        cs,
        snap,
        ScanCheckpoint {
            offset: 0,
            mode: 0,
            mid: None,
            line: 1,
        },
    )
}

pub fn lex_from(
    p: &GrammarProgram,
    cs: &CompiledScanner,
    snap: &Snapshot,
    cp: ScanCheckpoint,
) -> Result<LexOutput, RtError> {
    let mut out = LexOutput::default();
    let mut sc = Scanner::start(p, cs, snap, &cp)?;
    loop {
        let t = match sc.next_token()? {
            Some(t) => t,
            None => break,
        };
        if t.error {
            out.diags.push(LexDiag {
                message: "unterminated or unexpected byte".to_string(),
                start: t.start,
                end: t.end,
            });
        }
        out.tokens.push(t);
    }
    // 实际扫描跨度（重扫计量）：本次调用推进的字节数（mid 恢复时从 token 起点计）。
    let resume_start = cp
        .mid
        .as_ref()
        .map(|m| match m {
            MidToken::Str { start, .. } => *start,
            MidToken::Block { start, .. } => *start,
        })
        .unwrap_or(cp.offset);
    out.scanned_bytes = sc.w.pos.saturating_sub(resume_start);
    Ok(out)
}

/// 生成 checkpoint 列表（token 边界且 mode 回到初始、无未闭合状态）。
/// 增量词法从最近 at-rest checkpoint 恢复。
pub struct CheckpointIndex {
    pub offsets: Vec<ScanCheckpoint>,
}

impl CheckpointIndex {
    /// ≤ offset 的最近 at-rest checkpoint。
    pub fn nearest_at_rest(&self, offset: u64) -> Option<&ScanCheckpoint> {
        self.offsets
            .iter()
            .rev()
            .find(|c| c.offset <= offset && c.mode == 0 && c.mid.is_none())
    }
}

pub fn build_checkpoint_index(out: &LexOutput) -> CheckpointIndex {
    let mut offsets = Vec::with_capacity(out.tokens.len() / 4 + 1);
    offsets.push(ScanCheckpoint {
        offset: 0,
        mode: 0,
        mid: None,
        line: 1,
    });
    for t in &out.tokens {
        if !t.error {
            offsets.push(ScanCheckpoint {
                offset: t.end,
                mode: 0,
                mid: None,
                line: t.line,
            });
        }
    }
    CheckpointIndex { offsets }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::{PageStore, Snapshot};
    use crate::testkit::mini_program;
    use std::rc::Rc;

    fn lex(text: &str) -> LexOutput {
        let p = mini_program();
        let cs = CompiledScanner::compile(&p).unwrap();
        let snap = Snapshot::from_text(&Rc::new(PageStore::new()), text);
        lex_all(&p, &cs, &snap).unwrap()
    }

    fn kinds(o: &LexOutput, p: &GrammarProgram) -> Vec<String> {
        o.tokens
            .iter()
            .filter(|t| {
                !p.scanner
                    .rules
                    .iter()
                    .any(|r| r.token == t.token && r.trivia)
            })
            .map(|t| p.token_name(t.token).to_string())
            .collect()
    }

    #[test]
    fn lexer_basic_two_language_shapes() {
        let p = mini_program();
        let o = lex(r#"api add(a: i32, b: i32) -> i32; note "x\"y";"#);
        assert_eq!(
            kinds(&o, &p),
            vec![
                "api", "ident", "(", "ident", ":", "i32", ",", "ident", ":", "i32", ")", "->",
                "i32", ";", "note", "string", ";"
            ]
        );
    }

    #[test]
    fn lexer_nested_block_comment_and_string_cross_line() {
        let o = lex("/* a /* b */ c */ api x() -> i32;\nnote \"multi\nline\";");
        assert!(o.diags.is_empty(), "{:?}", o.diags);
        // 嵌套注释整体为一个 trivia；字符串跨行闭合。
        let s_tokens: Vec<_> = o.tokens.iter().map(|t| t.end - t.start).collect();
        assert!(s_tokens.iter().any(|&l| l > 10));
    }

    #[test]
    fn lexer_unterminated_string_is_error_leaf_to_eof() {
        let o = lex("note \"open");
        assert_eq!(o.tokens.last().unwrap().error, true);
        assert!(!o.diags.is_empty());
    }

    #[test]
    fn lexer_longest_match_literal_over_prefix() {
        // `->` 必须整体命中（ARROW），而非 `-`（无规则）+ `>`。
        let o = lex("->");
        let p = mini_program();
        assert_eq!(kinds(&o, &p), vec!["->"]);
    }

    #[test]
    fn lexer_unknown_byte_is_single_error_leaf() {
        let o = lex("api # x");
        let bad: Vec<_> = o.tokens.iter().filter(|t| t.error).collect();
        assert_eq!(bad.len(), 1);
        assert_eq!(bad[0].end - bad[0].start, 1);
    }

    #[test]
    fn lexer_token_lines_count_lf_only() {
        let o = lex("api a\r\nnote \"s\";");
        let first = o
            .tokens
            .iter()
            .find(|t| t.token == crate::testkit::toks::NOTE)
            .unwrap();
        assert_eq!(first.line, 2);
    }
}

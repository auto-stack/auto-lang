//! 旧词法事实层：CST 叶序列 → 旧 lexer 的致命词法诊断（首个按 token 起点）。
//!
//! 旧合同（T-01 冻结）：`tokenize_with` 全文件先行，首个词法事实致命——
//! 1. 未知 ASCII 字符（`&` `|` 单个等）→ `source.lex` "unknown character `X`"
//!    span=[pos, pos+1)；
//! 2. 未闭合字符串/字符：EOF → `source.lex` "unterminated ..." span=[start,len)；
//!    **换行前未闭合** → span=[start, 换行字节位)。text 族的字符串可跨行
//!    （值含换行），但 text 族的 char 与非 text 族的字符串/字符都不可；
//! 3. text 族的解码事实（旧 lexer 在扫描时产生）：`text.invalid-escape`
//!    （含 `\`+换行）、`text.invalid-scalar`、`text.char-empty`、
//!    `text.char-multi`——精确 span 逐字节复刻旧分支。
//!
//! 事实从 CST 叶（token 种类 + 快照字节）派生，不做源码重解析；顺序 = token
//! 起点序，与旧顺序（逐 token 扫描）一致。

use crate::atom_text::{Diagnostic, Span, Stage};
use crate::source::SourceProfile;
use ace_syntax_runtime::cst::{CstTree, FLAG_ERROR, FLAG_TRIVIA};
use ace_syntax_runtime::program::KIND_ERROR;
use ace_syntax_runtime::snapshot::Snapshot;

pub(crate) struct LeafInfo {
    pub kind: u32,
    pub start: u64,
    pub end: u64,
    pub error: bool,
}

/// 文档序收集全部非 trivia 叶（含 error 叶）。
pub(crate) fn collect_leaves(tree: &CstTree) -> Vec<LeafInfo> {
    let mut out = Vec::new();
    // 根的孩子在树级 root_edges 目录（根节点 child_count==0）——从根的孩子
    // 起种（children() 已处理跨块目录）。
    let mut stack = tree.children(tree.root);
    while let Some(n) = stack.pop() {
        let nd = tree.node(n);
        if nd.flags & FLAG_TRIVIA != 0 {
            continue;
        }
        if nd.child_count == 0 {
            // CstNode.start 是声明相对坐标——一律经 abs_range 换绝对。
            let (s0, e0) = tree.abs_range(n);
            out.push(LeafInfo {
                kind: nd.kind,
                start: s0,
                end: e0,
                error: nd.flags & FLAG_ERROR != 0,
            });
            continue;
        }
        let kids: Vec<u32> = tree.children(n);
        for c in kids.into_iter().rev() {
            stack.push(c);
        }
    }
    out
}

fn sp(s: u64, e: u64) -> Span {
    Span {
        start: s as usize,
        end: e as usize,
    }
}

fn unterminated(start: u64, end: u64, is_char: bool) -> Diagnostic {
    Diagnostic::new(
        Stage::Text,
        "source.lex",
        format!(
            "unterminated {} literal",
            if is_char { "char" } else { "string" }
        ),
        sp(start, end),
    )
}

/// str/char token 分类：跨行/未闭合 +（text 族）解码事实。
fn string_token_fact(
    snap: &Snapshot,
    start: u64,
    end: u64,
    is_char: bool,
    admits_text: bool,
) -> Option<Diagnostic> {
    let raw = snap.read_str(start, end).ok()?;
    let bytes = raw.as_bytes();
    // 换行终止扫描只适用非 text 族（旧 raw 扫描逐字节到闭引号/换行/EOF）。
    // text 族：字符串跨行合法（换行进值，解码层处理转义事实）；char 的换行
    // 分类在解码层——首体字节换行 = unterminated，其后 = char-multi，与旧
    // "先读一个标量再查闭引号"顺序一致。
    if !admits_text {
        let quote = if is_char { b'\'' } else { b'"' };
        let mut escaped = false;
        for (off, &b) in bytes.iter().enumerate().skip(1) {
            if escaped {
                // 旧 raw 扫描的转义对要求后随字符 ≠ 换行；`\`+换行 = `\` 作
                // 普通字节后撞换行 → 未闭合。
                if b == b'\n' {
                    return Some(unterminated(start, start + off as u64, is_char));
                }
                escaped = false;
                continue;
            }
            if b == b'\\' {
                escaped = true;
                continue;
            }
            if b == b'\n' {
                return Some(unterminated(start, start + off as u64, is_char));
            }
            if b == quote {
                break; // 闭合（token 到此为止）
            }
        }
    }
    if bytes.len() < 2 || bytes.last() != Some(&(if is_char { b'\'' } else { b'"' })) {
        // EOF 未闭合（扫描器 error token 覆盖 [start, len)）。
        return Some(unterminated(start, end, is_char));
    }
    if !admits_text {
        // 非 text 族：raw token 不是词法错误（旧 parser 在使用处拒绝）。
        return None;
    }
    decode_text_fact(start, &bytes[1..bytes.len() - 1], is_char)
}

/// 旧 text 解码分支（lexer.rs 文本臂的逐字节镜像；span 全部按旧终点）。
fn decode_text_fact(tok_start: u64, inner: &[u8], is_char: bool) -> Option<Diagnostic> {
    // inner 的 inner[i] 字节源码位置 = tok_start + 1 + i。
    let pos = |i: usize| tok_start + 1 + i as u64;
    if is_char {
        if inner.is_empty() {
            // 空 char `''`：闭引号即首体字节 → 旧 span [start, start+2)。
            return Some(Diagnostic::new(
                Stage::Text,
                "text.char-empty",
                "char literal must hold exactly one scalar",
                sp(tok_start, tok_start + 2),
            ));
        }
        // 首体字节为换行 → 旧 unterminated（lx.pos 停在换行位）。
        if inner[0] == b'\n' {
            return Some(unterminated(tok_start, tok_start + 1, true));
        }
        let mut i = 0usize;
        match inner[i] {
            b'\\' => {
                let esc = pos(0);
                i += 1;
                match inner.get(i) {
                    None => return None, // `'\` 闭合一类：token 边界已在上层分类
                    Some(b'n') | Some(b't') | Some(b'r') | Some(b'0') | Some(b'\\')
                    | Some(b'\'') => {
                        i += 1;
                    }
                    Some(b'u') => {
                        i += 1;
                        if inner.get(i) != Some(&b'{') {
                            // 旧：lx.pos 已过 `u` → end = esc+2。
                            return Some(Diagnostic::new(
                                Stage::Text,
                                "text.invalid-escape",
                                "escape \\u expects `{`",
                                sp(esc, esc + 2),
                            ));
                        }
                        i += 1;
                        let hex_start = i;
                        while i < inner.len() && inner[i].is_ascii_hexdigit() {
                            i += 1;
                        }
                        let hex_len = i - hex_start;
                        if inner.get(i) != Some(&b'}') || hex_len == 0 || hex_len > 6 {
                            // 旧：end = lx.pos = hex run 后 = pos(i)。
                            return Some(Diagnostic::new(
                                Stage::Text,
                                "text.invalid-escape",
                                "escape \\u{...} expects 1-6 hex digits and `}`",
                                sp(esc, pos(i)),
                            ));
                        }
                        let hex = std::str::from_utf8(&inner[hex_start..i]).ok()?;
                        let cp = u32::from_str_radix(hex, 16).ok()?;
                        if char::from_u32(cp).is_none() {
                            // 旧：已过 `}` → end = pos(i)（i 未再进）。
                            return Some(Diagnostic::new(
                                Stage::Text,
                                "text.invalid-scalar",
                                format!("\\u{{{}}} is not a Unicode scalar value", hex),
                                sp(esc, pos(i)),
                            ));
                        }
                        i += 1; // `}`
                    }
                    Some(_) => {
                        // 旧：end = (esc+2)，即越过后随字符一字节。
                        return Some(Diagnostic::new(
                            Stage::Text,
                            "text.invalid-escape",
                            "unknown escape sequence in char literal",
                            sp(esc, esc + 2),
                        ));
                    }
                }
                let scalar_len = i;
                if i + 1 < inner.len() {
                    let scalar_text = String::from_utf8_lossy(&inner[..scalar_len]).to_string();
                    return Some(Diagnostic::new(
                        Stage::Text,
                        "text.char-multi",
                        format!(
                            "char literal must hold exactly one scalar (found more after `{}`)",
                            scalar_text
                        ),
                        // 旧：end = (lx.pos+1).min(len)，lx.pos = 首标量后。
                        sp(tok_start, pos(i) + 1),
                    ));
                }
                None
            }
            _ => {
                let len = utf8_len(inner[0]);
                if 1 + len < inner.len() {
                    let scalar_text = String::from_utf8_lossy(&inner[..len]).to_string();
                    return Some(Diagnostic::new(
                        Stage::Text,
                        "text.char-multi",
                        format!(
                            "char literal must hold exactly one scalar (found more after `{}`)",
                            scalar_text
                        ),
                        sp(tok_start, pos(len) + 1),
                    ));
                }
                None
            }
        }
    } else {
        let mut i = 0usize;
        while i < inner.len() {
            match inner[i] {
                b'\n' => {
                    i += 1;
                }
                b'\\' => {
                    let esc = pos(i);
                    i += 1;
                    match inner.get(i) {
                        None => return None,
                        Some(b'n') | Some(b't') | Some(b'r') | Some(b'0') | Some(b'\\')
                        | Some(b'"') => {
                            i += 1;
                        }
                        Some(b'u') => {
                            i += 1;
                            if inner.get(i) != Some(&b'{') {
                                return Some(Diagnostic::new(
                                    Stage::Text,
                                    "text.invalid-escape",
                                    "escape \\u expects `{`",
                                    sp(esc, esc + 2),
                                ));
                            }
                            i += 1;
                            let hex_start = i;
                            while i < inner.len() && inner[i].is_ascii_hexdigit() {
                                i += 1;
                            }
                            let hex_len = i - hex_start;
                            if inner.get(i) != Some(&b'}') || hex_len == 0 || hex_len > 6 {
                                return Some(Diagnostic::new(
                                    Stage::Text,
                                    "text.invalid-escape",
                                    "escape \\u{...} expects 1-6 hex digits and `}`",
                                    sp(esc, pos(i)),
                                ));
                            }
                            let hex = std::str::from_utf8(&inner[hex_start..i]).ok()?;
                            let cp = u32::from_str_radix(hex, 16).ok()?;
                            if char::from_u32(cp).is_none() {
                                // 旧：end = 过 `}` 后位置。
                                return Some(Diagnostic::new(
                                    Stage::Text,
                                    "text.invalid-scalar",
                                    format!("\\u{{{}}} is not a Unicode scalar value", hex),
                                    sp(esc, pos(i + 1)),
                                ));
                            }
                            i += 1; // `}`
                        }
                        Some(_) => {
                            // 旧：end = (lx.pos+1)，lx.pos = 后随字符位。
                            return Some(Diagnostic::new(
                                Stage::Text,
                                "text.invalid-escape",
                                "unknown escape sequence (whitelist: \\n \\t \\r \\0 \\\\ \\\" \\u{hex})",
                                sp(esc, esc + 2),
                            ));
                        }
                    }
                }
                b if b < 0x80 => {
                    i += 1;
                }
                _ => {
                    i += utf8_len(inner[i]);
                }
            }
        }
        None
    }
}

fn utf8_len(b: u8) -> usize {
    match b {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        _ => 1,
    }
}

fn unknown_char_diag(snap: &Snapshot, start: u64) -> Diagnostic {
    let byte = snap
        .read_str(start, start + 1)
        .ok()
        .and_then(|s| s.bytes().next())
        .unwrap_or(b'?');
    Diagnostic::new(
        Stage::Text,
        "source.lex",
        format!("unknown character `{}`", byte as char),
        sp(start, start + 1),
    )
}

/// 全文件词法事实（首个按 token 起点序）。无事实 = 旧 tokenize 成功。
pub(crate) fn first_lex_fact(
    tree: &CstTree,
    snap: &Snapshot,
    k: &super::kinds::SourceKinds,
    profile: SourceProfile,
) -> Option<Diagnostic> {
    let admits_text = profile.admits_text_literals();
    let mut facts: Vec<(u64, Diagnostic)> = Vec::new();
    for leaf in collect_leaves(tree) {
        if leaf.kind == k.tok_str_lit || leaf.kind == k.tok_char_lit {
            let is_char = leaf.kind == k.tok_char_lit;
            if let Some(d) = string_token_fact(snap, leaf.start, leaf.end, is_char, admits_text) {
                facts.push((leaf.start, d));
            }
        } else if leaf.kind == KIND_ERROR && leaf.error && leaf.end == leaf.start + 1 {
            // 扫描器无匹配的单字节 error 叶 = 旧 unknown character。
            facts.push((leaf.start, unknown_char_diag(snap, leaf.start)));
        }
    }
    facts.sort_by_key(|(s, _)| *s);
    facts.into_iter().next().map(|(_, d)| d)
}

/// 解码字符串字面量的【值】（投影用；事实层已保证无解码错误）。
/// 旧合同：跨行换行按 LF 进值、CRLF 的 CR 按原字节进值、非 ASCII 原样。
pub(crate) fn decode_str_value(inner: &[u8]) -> String {
    let mut value = String::new();
    let mut i = 0usize;
    while i < inner.len() {
        match inner[i] {
            b'\n' => {
                value.push('\n');
                i += 1;
            }
            b'\\' => {
                i += 1;
                if i >= inner.len() {
                    break;
                }
                match inner[i] {
                    b'n' => {
                        value.push('\n');
                        i += 1;
                    }
                    b't' => {
                        value.push('\t');
                        i += 1;
                    }
                    b'r' => {
                        value.push('\r');
                        i += 1;
                    }
                    b'0' => {
                        value.push('\0');
                        i += 1;
                    }
                    b'\\' => {
                        value.push('\\');
                        i += 1;
                    }
                    b'"' => {
                        value.push('"');
                        i += 1;
                    }
                    b'u' => {
                        i += 1; // `{`
                        let mut hex = String::new();
                        while i < inner.len() && inner[i] != b'}' {
                            hex.push(inner[i] as char);
                            i += 1;
                        }
                        i += 1; // `}`
                        if let Some(ch) =
                            u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32)
                        {
                            value.push(ch);
                        }
                    }
                    other => {
                        value.push(other as char);
                        i += 1;
                    }
                }
            }
            b if b < 0x80 => {
                value.push(b as char);
                i += 1;
            }
            _ => {
                let len = utf8_len(inner[i]);
                let slice = &inner[i..(i + len).min(inner.len())];
                if let Ok(s) = std::str::from_utf8(slice) {
                    value.push_str(s);
                }
                i += len;
            }
        }
    }
    value
}

/// 解码字符字面量的标量值（投影用；事实层已保证恰好一个标量）。
pub(crate) fn decode_char_value(inner: &[u8]) -> u32 {
    if inner.is_empty() {
        return 0;
    }
    match inner[0] {
        b'\\' if inner.len() >= 2 => match inner[1] {
            b'n' => '\n' as u32,
            b't' => '\t' as u32,
            b'r' => '\r' as u32,
            b'0' => 0,
            b'\\' => '\\' as u32,
            b'\'' => '\'' as u32,
            b'u' => {
                let body = &inner[3..inner.len().saturating_sub(1)];
                u32::from_str_radix(&String::from_utf8_lossy(body), 16).unwrap_or(0)
            }
            _ => inner[1] as u32,
        },
        b if b < 0x80 => b as u32,
        _ => {
            let len = utf8_len(inner[0]);
            std::str::from_utf8(&inner[..len.min(inner.len())])
                .ok()
                .and_then(|s| s.chars().next())
                .map(|c| c as u32)
                .unwrap_or(0)
        }
    }
}

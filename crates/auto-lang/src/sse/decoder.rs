//! PLAN-707 T-03：字节增量 SSE decoder（live 流路径专用）。
//!
//! 与 [`crate::sse::parser`] 的一次性 helper（`parse_sse_chunk`/`SSEParser`）
//! 的关系是 **legacy 保留**：helper 消费方（`shim_sse_parse` 等）行为不变，
//! 新流生产端一律走本 decoder（707-stream-decision.md D-8 差异矩阵）。
//!
//! 规则（WHATWG HTML §9.2.5–6 子集，2026-09-29 查阅）：
//! - 流首 BOM（EF BB BF）剥离一次；行终结 = LF / CRLF / CR 三态，行尾
//!   悬置 CR 需等下一字节判别（CRLF 是一个终结符，不是终结+空行）；
//! - 行内容：首个冒号切 field/value；value 前恰一个空格移除、其余保留；
//!   `:` 开头 = 注释忽略；未知 field 忽略；
//! - `data` 多行以 \n 拼接；`event`/`id` 追加语义（同 field 多行 \n 拼接）；
//!   `id` 含 NUL 忽略；`retry` 非纯数字忽略；
//! - 空行分发：data 缓冲为空 → 不分发（含 `data:` 空值行）；否则按当前
//!   事件状态分发并复位（id 用最近一次合法 `id` —— last-event-id 语义）；
//! - EOF：未闭合事件丢弃，`[DONE]` 是数据（业务决定终止）；
//! - 预算：单行 carry 与单事件 data 超 256 KiB → `SseDecodeError::Budget`。

use crate::sse::types::SSEEvent;

/// 增量解码错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseDecodeError {
    /// 行/事件字节预算超限（kind: "line" | "event"）。
    Budget { kind: &'static str, limit: usize },
    /// 已完成行中出现非法 UTF-8（跨 chunk 拆分不会触发——字节 carry 到
    /// 行完成才解码）。
    InvalidUtf8,
}

/// 默认行 carry 预算（决策 D-10：单事件/未完成解析 256 KiB）。
pub(crate) const SSE_MAX_LINE_BYTES: usize = 256 * 1024;
/// 默认单事件 data 预算。
pub(crate) const SSE_MAX_EVENT_BYTES: usize = 256 * 1024;

pub struct SseDecoder {
    /// 未凑成完整行的原始字节（码点/行边界跨 chunk 拆分在此 carry）。
    carry: Vec<u8>,
    /// 事件构造态。
    data: String,
    event: String,
    last_event_id: Option<String>,
    retry: Option<u32>,
    bom_stripped: bool,
    max_line_bytes: usize,
    max_event_bytes: usize,
}

impl Default for SseDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl SseDecoder {
    pub fn new() -> Self {
        Self::with_limits(SSE_MAX_LINE_BYTES, SSE_MAX_EVENT_BYTES)
    }

    pub fn with_limits(max_line_bytes: usize, max_event_bytes: usize) -> Self {
        Self {
            carry: Vec::new(),
            data: String::new(),
            event: String::new(),
            last_event_id: None,
            retry: None,
            bom_stripped: false,
            max_line_bytes,
            max_event_bytes,
        }
    }

    /// 喂入一段网络字节，把已完成的事件推入 `out`。
    pub fn feed(&mut self, bytes: &[u8], out: &mut Vec<SSEEvent>) -> Result<(), SseDecodeError> {
        self.carry.extend_from_slice(bytes);
        // 流首 BOM 判定（BOM 可被 chunk 切开）：完整 BOM → 剥离；carry 是
        // BOM 真前缀 → 悬置等更多字节；否则确定无 BOM。
        if !self.bom_stripped {
            if self.carry.starts_with(&[0xEF, 0xBB, 0xBF]) {
                self.carry.drain(..3);
                self.bom_stripped = true;
            } else if self.carry.len() < 3 && [0xEFu8, 0xBB, 0xBF].starts_with(&self.carry[..]) {
                return Ok(());
            } else {
                self.bom_stripped = true;
            }
        }

        if self.carry.len() > self.max_line_bytes {
            return Err(SseDecodeError::Budget {
                kind: "line",
                limit: self.max_line_bytes,
            });
        }

        // 逐行抽取：LF 终结；CR 终结但悬置（最后字节）等下一字节判别
        // CRLF 合并。
        loop {
            let Some(term) = find_line_terminator(&self.carry) else {
                break;
            };
            match term {
                LineTerm::Found { end, next } => {
                    let line_bytes: Vec<u8> = self.carry.drain(..next).collect();
                    let line_bytes = &line_bytes[..end];
                    let line = std::str::from_utf8(line_bytes)
                        .map_err(|_| SseDecodeError::InvalidUtf8)?
                        .to_string();
                    self.parse_line(&line, out)?;
                }
                LineTerm::CrAtEnd => break, // 悬置 CR：等更多字节
            }
        }
        Ok(())
    }

    /// EOF：未闭合事件丢弃（WHATWG），不产生新事件。
    pub fn finish(&mut self) {
        self.carry.clear();
        self.data.clear();
        self.event.clear();
        self.retry = None;
    }

    /// 解析一行字段（分发空行事件）。
    fn parse_line(&mut self, line: &str, out: &mut Vec<SSEEvent>) -> Result<(), SseDecodeError> {
        if line.is_empty() {
            self.dispatch(out);
            return Ok(());
        }
        if line.starts_with(':') {
            return Ok(()); // 注释
        }
        let (field, value) = match line.split_once(':') {
            Some((f, v)) => (f, v.strip_prefix(' ').unwrap_or(v)),
            None => (line, ""),
        };
        match field {
            "data" => {
                if !self.data.is_empty() {
                    self.data.push('\n');
                }
                self.data.push_str(value);
                if self.data.len() > self.max_event_bytes {
                    return Err(SseDecodeError::Budget {
                        kind: "event",
                        limit: self.max_event_bytes,
                    });
                }
            }
            "event" => {
                if !self.event.is_empty() {
                    self.event.push('\n');
                }
                self.event.push_str(value);
            }
            "id" => {
                if !value.contains('\0') {
                    self.last_event_id = Some(value.to_string());
                }
            }
            "retry" => {
                if let Ok(ms) = value.parse::<u32>() {
                    self.retry = Some(ms);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// 空行分发：data 空 → 仅复位（不分发）；否则产出事件并复位构造态。
    fn dispatch(&mut self, out: &mut Vec<SSEEvent>) {
        if self.data.is_empty() {
            self.event.clear();
            self.retry = None;
            return;
        }
        out.push(SSEEvent {
            id: self.last_event_id.clone(),
            event: if self.event.is_empty() {
                None
            } else {
                Some(std::mem::take(&mut self.event))
            },
            data: std::mem::take(&mut self.data),
            retry: self.retry.take(),
        });
    }
}

enum LineTerm {
    /// 完整行：字节 [..end] 为行内容，消费到 next。
    Found { end: usize, next: usize },
    /// carry 以 CR 结尾——CRLF 与 CR-only 无法判别，悬置等下一字节。
    CrAtEnd,
}

fn find_line_terminator(buf: &[u8]) -> Option<LineTerm> {
    for (i, b) in buf.iter().enumerate() {
        match b {
            0x0A => {
                return Some(LineTerm::Found {
                    end: i,
                    next: i + 1,
                })
            }
            0x0D => {
                if i + 1 == buf.len() {
                    return Some(LineTerm::CrAtEnd);
                }
                let next = if buf[i + 1] == 0x0A { i + 2 } else { i + 1 };
                return Some(LineTerm::Found { end: i, next });
            }
            _ => {}
        }
    }
    None
}

/// raw（非 SSE）文本流的跨 chunk UTF-8 carry 解码：完整码点前缀立即可用，
/// 未完序列留在 carry；已判非法的字节按 lossy 约定替换（保持旧观测面——
/// 707 只修跨 chunk 拆分，不改非法字节语义）。
#[derive(Default)]
pub(crate) struct Utf8Carry {
    pending: Vec<u8>,
}

impl Utf8Carry {
    pub(crate) fn push(&mut self, bytes: &[u8]) -> String {
        self.pending.extend_from_slice(bytes);
        let mut out = String::new();
        loop {
            match std::str::from_utf8(&self.pending) {
                Ok(s) => {
                    out.push_str(s);
                    self.pending.clear();
                    break;
                }
                Err(e) => {
                    let valid = e.valid_up_to();
                    if valid > 0 {
                        out.push_str(&String::from_utf8_lossy(&self.pending[..valid]));
                        self.pending.drain(..valid);
                    }
                    match e.error_len() {
                        Some(bad) => {
                            // 非法字节：replacement（1 字节），继续消化。
                            out.push('\u{FFFD}');
                            self.pending.drain(..bad);
                        }
                        None => break, // 未完序列：留在 carry
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_all(feeds: &[&[u8]]) -> Result<Vec<SSEEvent>, SseDecodeError> {
        let mut d = SseDecoder::new();
        let mut out = Vec::new();
        for f in feeds {
            d.feed(f, &mut out)?;
        }
        d.finish();
        Ok(out)
    }

    // ---- 切分不变性：任意字节切分点结果一致（含 UTF-8 码点中点）----

    #[test]
    fn plan707_decode_byte_split_invariance_utf8_midpoint() {
        let full = "data: 中文x\n\ndata: [DONE]\n\n".as_bytes();
        let expected = decode_all(&[full]).unwrap();
        assert_eq!(expected.len(), 2);
        assert_eq!(expected[0].data, "中文x");
        // 逐字节喂（最细切分，含 E4|B8|AD 码点中点）。
        let bytewise: Vec<&[u8]> = full.iter().map(|b| std::slice::from_ref(b)).collect();
        assert_eq!(decode_all(&bytewise).unwrap(), expected);
        // 2 字节/块、3 字节/块（切在行终结处与码点中点交替）。
        for step in [2usize, 3, 5] {
            let chunks: Vec<&[u8]> = full.chunks(step).collect();
            assert_eq!(decode_all(&chunks).unwrap(), expected, "step={step}");
        }
    }

    // ---- 行终结三态：LF / CRLF / CR / 悬置 CR 跨 chunk ----

    #[test]
    fn plan707_decode_line_endings_lf_crlf_cr() {
        // CRLF 一个终结符：两行 data 同事件。
        let ev = decode_all(&[b"data: a\r\ndata: b\r\n\r\n"]).unwrap();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].data, "a\nb");

        // CR-only 行终结（分发空行用 \r\n 收尾——悬置 CR 需后续字节判别）。
        let ev = decode_all(&[b"data: a\rdata: b\r\r\n"]).unwrap();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].data, "a\nb");

        // 悬置 CR 跨 chunk：CR 与 LF 分属两 chunk，仍是一个终结。
        let ev = decode_all(&[b"data: a\r", b"\ndata: b\r", b"\n\r", b"\n"]).unwrap();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].data, "a\nb");
    }

    // ---- 字段规则：空格移除恰一个 / 多行 data / 注释 / event/id/retry ----

    #[test]
    fn plan707_decode_field_rules() {
        let ev = decode_all(&[
            b"id: 42\n",
            b"event: message\n",
            b"retry: 5000\n",
            b": comment ignored\n",
            b"data:no-space\n",
            b"data:  keeps-one-then-preserves\n",
            b"\n",
        ])
        .unwrap();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].id.as_deref(), Some("42"));
        assert_eq!(ev[0].event.as_deref(), Some("message"));
        assert_eq!(ev[0].retry, Some(5000));
        assert_eq!(ev[0].data, "no-space\n keeps-one-then-preserves");
    }

    #[test]
    fn plan707_decode_nul_id_and_bad_retry_ignored() {
        let ev = decode_all(&[b"id: a\0b\nretry: soon\ndata: x\n\n"]).unwrap();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].id, None, "含 NUL 的 id 忽略");
        assert_eq!(ev[0].retry, None, "非数字 retry 忽略");
    }

    // ---- 分发规则：空 data 不分发；event-only 不分发；[DONE] 是数据 ----

    #[test]
    fn plan707_decode_empty_data_not_dispatched_done_is_data() {
        // "data:" 空值行 + 空行 → data 缓冲空 → 不分发。
        assert!(decode_all(&[b"data:\n\ndata: [DONE]\n\n"]).unwrap().len() == 1);
        let ev = decode_all(&[b"data:\n\ndata: [DONE]\n\n"]).unwrap();
        assert_eq!(ev[0].data, "[DONE]", "[DONE] 是业务数据");
        // event-only（无 data）不分发。
        assert!(decode_all(&[b"event: ping\n\n"]).unwrap().is_empty());
    }

    #[test]
    fn plan707_decode_last_event_id_persists_and_eof_drops_unclosed() {
        // last-event-id 语义：id 只出现一次，后续事件仍携带。
        let ev = decode_all(&[b"id: 7\ndata: a\n\n\ndata: b\n\n"]).unwrap();
        assert_eq!(ev.len(), 2);
        assert_eq!(ev[0].id.as_deref(), Some("7"));
        assert_eq!(ev[1].id.as_deref(), Some("7"));

        // EOF 未闭合事件丢弃。
        let mut d = SseDecoder::new();
        let mut out = Vec::new();
        d.feed(b"data: first\n\ndata: tail", &mut out).unwrap();
        assert_eq!(out.len(), 1, "已闭合事件已产出");
        d.finish();
        assert_eq!(out.len(), 1, "EOF 未闭合事件丢弃，不伪造分发");
    }

    // ---- 预算：行与事件超限可观测终结 ----

    #[test]
    fn plan707_decode_budget_limits_are_observable() {
        let mut d = SseDecoder::with_limits(16, 64);
        let mut out = Vec::new();
        let err = d
            .feed(b"data: this-line-is-way-longer-than-sixteen\n", &mut out)
            .unwrap_err();
        assert_eq!(
            err,
            SseDecodeError::Budget {
                kind: "line",
                limit: 16
            }
        );

        let mut d = SseDecoder::with_limits(1024, 8);
        let mut out = Vec::new();
        let err = d.feed(b"data: 123456789\n\n", &mut out).unwrap_err();
        assert_eq!(
            err,
            SseDecodeError::Budget {
                kind: "event",
                limit: 8
            }
        );
    }

    // ---- Utf8Carry：raw 流跨 chunk 码点无损 + 非法字节 lossy ----

    #[test]
    fn plan707_decode_utf8_carry_split_and_lossy() {
        let mut c = Utf8Carry::default();
        assert_eq!(c.push(&[0xE4]), "");
        assert_eq!(c.push(&[0xB8]), "");
        assert_eq!(c.push(&[0xAD, 0xE6]), "中");
        assert_eq!(c.push(&[0x96, 0x87]), "文");
        assert_eq!(c.push(b"!"), "!");
        // 非法字节 → replacement，后续正常文本保留。
        let mut c = Utf8Carry::default();
        assert_eq!(c.push(&[b'a', 0xFF, b'b']), "a\u{FFFD}b");
        // 悬置多字节序列跨多次 push（"日" 的前 2 字节悬置，第 3 字节补全）。
        let ri = "日".as_bytes();
        let mut c = Utf8Carry::default();
        assert_eq!(c.push(&ri[..2]), "");
        assert_eq!(c.push(&ri[2..]), "日");
    }

    // ---- 与 legacy helper 的差异矩阵钉（D-8：legacy 保留，不无声统一）----

    #[test]
    fn plan707_decode_legacy_helper_matrix_unchanged() {
        use crate::sse::{parse_sse_chunk, sse_parser_from_bytes};
        // legacy：逐行 trim（两端全部空白）——decoder 只移除冒号后一个
        // 空格并保留其余。差异原样保留。
        let ev = parse_sse_chunk("data:  spaced \n\ndata: tail").unwrap();
        assert_eq!(ev[0].data, "spaced", "legacy trim 语义不变");
        assert_eq!(ev[1].data, "tail", "legacy 尾事件兜底不变");

        // live 流分帧差异钉住：SSEParser 分帧只认 "\n\n"，CRLF 终结的流
        // 不会中途分帧——EOF 尾事件兜底把整流合成一个事件交付，且兜底
        // 不消费 buffer（同事件重复交付 quirk，原样保留不动）。
        let mut parser = sse_parser_from_bytes(b"data: a\r\ndata: b\r\n\r\n");
        let ev = parser.next_event().unwrap().expect("EOF 尾事件兜底");
        assert_eq!(ev.data, "a\nb", "legacy 整流单事件（差异钉住）");
        let again = parser.next_event().unwrap().expect("quirk：兜底重复交付");
        assert_eq!(again.data, "a\nb", "legacy buffer 不消费（行为不变钉）");
    }
}

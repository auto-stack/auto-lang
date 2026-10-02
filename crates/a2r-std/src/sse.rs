//! PLAN-724 T-04：共享增量 SSE decoder 与跨 chunk UTF-8 carry（Rust 内核层）。
//!
//! 本模块是 auto-lang `sse::decoder`（PLAN-707 T-03 交付）的**提取宿主**：
//! 纯解码逻辑（字节增量、行终结三态、字段规则、预算）单源存放于此，两侧
//! HTTP 客户端（独立 a2r-std facade 与 auto_lang::a2r_std facade）与 VM 流
//! 生产者经 facade 复用同一实现，不再各持副本。
//!
//! 规则（WHATWG HTML §9.2.5–6 子集，707 冻结；与 legacy `sse::parser` 的
//! 差异矩阵见 auto-lang 侧 `plan707_decode_legacy_helper_matrix_unchanged`）：
//! - 流首 BOM（EF BB BF）剥离一次；行终结 = LF / CRLF / CR 三态，行尾
//!   悬置 CR 需等下一字节判别（CRLF 是一个终结符，不是终结+空行）；
//! - 行内容：首个冒号切 field/value；value 前恰一个空格移除、其余保留；
//!   `:` 开头 = 注释忽略；未知 field 忽略；
//! - `data` 多行以 \n 拼接；`event`/`id` 追加语义；`id` 含 NUL 忽略；
//!   `retry` 非纯数字忽略；
//! - 空行分发：data 缓冲为空 → 不分发（含 `data:` 空值行）；否则按当前
//!   事件状态分发并复位（id 用最近一次合法 `id` —— last-event-id 语义）；
//! - EOF：未闭合事件丢弃，`[DONE]` 是数据（业务决定终止）；
//! - 预算：单行 carry 与单事件 data 超 256 KiB → [`SseDecodeError::Budget`]。

/// 一条已解码的 SSE 事件（字段形状与 auto-lang `sse::types::SSEEvent` 一致，
/// facade 逐字段映射，零语义差）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseEvent {
    pub id: Option<String>,
    pub event: Option<String>,
    pub data: String,
    pub retry: Option<u32>,
}

/// 增量解码错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseDecodeError {
    /// 行/事件字节预算超限（kind: "line" | "event"）。
    Budget { kind: &'static str, limit: usize },
    /// 已完成行中出现非法 UTF-8（跨 chunk 拆分不会触发——字节 carry 到
    /// 行完成才解码）。
    InvalidUtf8,
}

/// 默认行 carry 预算（707 决策 D-10：单事件/未完成解析 256 KiB）。
pub const SSE_MAX_LINE_BYTES: usize = 256 * 1024;
/// 默认单事件 data 预算。
pub const SSE_MAX_EVENT_BYTES: usize = 256 * 1024;

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
    pub fn feed(&mut self, bytes: &[u8], out: &mut Vec<SseEvent>) -> Result<(), SseDecodeError> {
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
    fn parse_line(&mut self, line: &str, out: &mut Vec<SseEvent>) -> Result<(), SseDecodeError> {
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
    fn dispatch(&mut self, out: &mut Vec<SseEvent>) {
        if self.data.is_empty() {
            self.event.clear();
            self.retry = None;
            return;
        }
        out.push(SseEvent {
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
/// 未完序列留在 carry；已判非法的字节按 lossy 约定替换（保持 707 观测面）。
#[derive(Default)]
pub struct Utf8Carry {
    pending: Vec<u8>,
}

impl Utf8Carry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, bytes: &[u8]) -> String {
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

    fn decode_all(feeds: &[&[u8]]) -> Result<Vec<SseEvent>, SseDecodeError> {
        let mut d = SseDecoder::new();
        let mut out = Vec::new();
        for f in feeds {
            d.feed(f, &mut out)?;
        }
        d.finish();
        Ok(out)
    }

    #[test]
    fn plan724_sse_kernel_split_invariance_and_budget() {
        // 切分不变性（含 UTF-8 码点中点）。
        let full = "data: 中文x\n\ndata: [DONE]\n\n".as_bytes();
        let expected = decode_all(&[full]).unwrap();
        assert_eq!(expected.len(), 2);
        assert_eq!(expected[0].data, "中文x");
        let bytewise: Vec<&[u8]> = full.iter().map(|b| std::slice::from_ref(b)).collect();
        assert_eq!(decode_all(&bytewise).unwrap(), expected);

        // CRLF / 悬置 CR 跨 chunk。
        let ev = decode_all(&[b"data: a\r", b"\ndata: b\r", b"\n\r", b"\n"]).unwrap();
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].data, "a\nb");

        // 预算可观察终结。
        let mut d = SseDecoder::with_limits(16, 64);
        let mut out = Vec::new();
        assert_eq!(
            d.feed(b"data: this-line-is-way-longer-than-sixteen\n", &mut out)
                .unwrap_err(),
            SseDecodeError::Budget { kind: "line", limit: 16 }
        );

        // Utf8Carry：跨 chunk 码点无损 + 非法字节 lossy。
        let ri = "日".as_bytes();
        let mut c = Utf8Carry::new();
        assert_eq!(c.push(&ri[..2]), "");
        assert_eq!(c.push(&ri[2..]), "日");
        let mut c = Utf8Carry::new();
        assert_eq!(c.push(&[b'a', 0xFF, b'b']), "a\u{FFFD}b");
    }
}

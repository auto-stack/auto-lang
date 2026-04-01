// Plan 152: SSE 解析器
//
// Server-Sent Events (SSE) 解析实现

use std::io::{BufRead, BufReader};

use crate::sse::types::{SSEEvent, SSEError};

/// 解析 SSE 文本块
///
/// # 参数
/// - `chunk`: SSE 文本块
///
/// # 返回
/// 解析后的事件列表
pub fn parse_sse_chunk(chunk: &str) -> Result<Vec<SSEEvent>, SSEError> {
    let mut events = Vec::new();
    let mut current_event = SSEEvent::new();

    for line in chunk.lines() {
        // 跳过注释行（以 : 开头）
        if line.starts_with(':') {
            continue;
        }

        if line.is_empty() {
            // 空行表示事件结束
            if !current_event.is_empty() {
                events.push(current_event.clone());
                current_event = SSEEvent::new();
            }
            continue;
        }

        // 解析 data: 行
        if let Some(rest) = line.strip_prefix("data:") {
            let data = rest.trim();
            if !current_event.data.is_empty() {
                current_event.data.push('\n');
            }
            current_event.data.push_str(data);
            continue;
        }

        // 解析 event: 行
        if let Some(rest) = line.strip_prefix("event:") {
            current_event.event = Some(rest.trim().to_string());
            continue;
        }

        // 解析 id: 行
        if let Some(rest) = line.strip_prefix("id:") {
            current_event.id = Some(rest.trim().to_string());
            continue;
        }

        // 解析 retry: 行
        if let Some(rest) = line.strip_prefix("retry:") {
            if let Ok(retry) = rest.trim().parse::<u32>() {
                current_event.retry = Some(retry);
            }
            continue;
        }
    }

    // 最后一个事件（如果没有空行结束）
    if !current_event.is_empty() {
        events.push(current_event);
    }

    Ok(events)
}

/// SSE 流解析器
pub struct SSEParser<R: BufRead> {
    reader: R,
    buffer: String,
}

impl<R: BufRead> SSEParser<R> {
    /// 创建新的 SSE 解析器
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: String::new(),
        }
    }

    /// 解析下一个事件
    pub fn next_event(&mut self) -> Result<Option<SSEEvent>, SSEError> {
        loop {
            // 检查 buffer 中是否有完整事件
            if let Some(events) = self.parse_buffer()? {
                return Ok(Some(events));
            }

            // 读取更多数据
            let mut chunk = String::new();
            let n = self.reader.read_line(&mut chunk)?;
            if n == 0 {
                // EOF，检查是否有剩余数据
                if !self.buffer.trim().is_empty() {
                    return Ok(Some(parse_sse_chunk(&self.buffer)?.pop().unwrap()));
                }
                return Ok(None);
            }

            self.buffer.push_str(&chunk);
        }
    }

    /// 从 buffer 解析事件
    fn parse_buffer(&mut self) -> Result<Option<SSEEvent>, SSEError> {
        // 查找双换行（事件分隔符）
        while let Some(pos) = self.buffer.find("\n\n") {
            let chunk = self.buffer[..pos].to_string();
            self.buffer = self.buffer[pos + 2..].to_string();

            if !chunk.trim().is_empty() {
                let mut events = parse_sse_chunk(&chunk)?;
                if let Some(event) = events.pop() {
                    return Ok(Some(event));
                }
            }
        }

        Ok(None)
    }

    /// 解析所有剩余事件
    pub fn finish(mut self) -> Result<Vec<SSEEvent>, SSEError> {
        let mut events = Vec::new();
        while let Some(event) = self.next_event()? {
            events.push(event);
        }
        Ok(events)
    }
}

/// 从字节切片创建 SSE 解析器
///
/// 这是一个辅助函数，用于从已读取的字节数据创建解析器
pub fn sse_parser_from_bytes(bytes: &[u8]) -> SSEParser<BufReader<&[u8]>> {
    let reader = BufReader::new(bytes);
    SSEParser::new(reader)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_simple() {
        let sse_text = "data: Hello\n\n";
        let events = parse_sse_chunk(sse_text).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "Hello");
    }

    #[test]
    fn test_parse_sse_multi_line() {
        let sse_text = "data: Hello\ndata: World\n\n";
        let events = parse_sse_chunk(sse_text).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "Hello\nWorld");
    }

    #[test]
    fn test_parse_sse_multiple_events() {
        let sse_text = "data: First\n\ndata: Second\n\n";
        let events = parse_sse_chunk(sse_text).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].data, "First");
        assert_eq!(events[1].data, "Second");
    }

    #[test]
    fn test_parse_sse_with_fields() {
        let sse_text = "id: 123\nevent: message\nretry: 5000\ndata: Test\n\n";
        let events = parse_sse_chunk(sse_text).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, Some("123".to_string()));
        assert_eq!(events[0].event, Some("message".to_string()));
        assert_eq!(events[0].retry, Some(5000));
        assert_eq!(events[0].data, "Test");
    }

    #[test]
    fn test_parse_sse_done_marker() {
        let sse_text = "data: [DONE]\n\n";
        let events = parse_sse_chunk(sse_text).unwrap();
        assert_eq!(events.len(), 1);
        assert!(events[0].is_done());
    }

    #[test]
    fn test_parse_sse_with_json() {
        let sse_text = "data: {\"content\": \"Hello\"}\n\n";
        let events = parse_sse_chunk(sse_text).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "{\"content\": \"Hello\"}");
    }

    #[test]
    fn test_parse_sse_ignore_comments() {
        let sse_text = ": This is a comment\ndata: Hello\n\n";
        let events = parse_sse_chunk(sse_text).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].data, "Hello");
    }

    #[test]
    fn test_sse_event_builder() {
        let event = SSEEvent::new()
            .with_id("123".to_string())
            .with_event("message".to_string())
            .with_data("Hello".to_string())
            .with_retry(5000);

        assert_eq!(event.id, Some("123".to_string()));
        assert_eq!(event.event, Some("message".to_string()));
        assert_eq!(event.data, "Hello");
        assert_eq!(event.retry, Some(5000));
    }

    #[test]
    fn test_sse_event_is_empty() {
        let event = SSEEvent::new();
        assert!(event.is_empty());

        let event_with_data = SSEEvent::new().with_data("test".to_string());
        assert!(!event_with_data.is_empty());
    }

    #[test]
    fn test_sse_event_is_done() {
        let event = SSEEvent::new().with_data("[DONE]".to_string());
        assert!(event.is_done());

        let event_normal = SSEEvent::new().with_data("Hello".to_string());
        assert!(!event_normal.is_done());
    }
}

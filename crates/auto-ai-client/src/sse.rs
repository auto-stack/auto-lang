//! SSE (Server-Sent Events) incremental parser.
//!
//! Ported from AutoForge's `provider/sse.rs`. Parses the `text/event-stream`
//! format used by LLM streaming APIs (Anthropic, OpenAI, Zhipu, etc.).

/// Incremental SSE parser. Feed raw bytes via `push()`, get parsed events back.
pub struct SseParser {
    buffer: Vec<u8>,
}

impl SseParser {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Feed a chunk of bytes. Returns complete SSE data fields parsed so far.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.buffer.extend_from_slice(chunk);
        self.drain_data()
    }

    /// Flush remaining buffer (call when stream ends).
    pub fn finish(mut self) -> Vec<String> {
        self.buffer.extend_from_slice(b"\n\n");
        self.drain_data()
    }

    fn drain_data(&mut self) -> Vec<String> {
        let mut events = Vec::new();
        while let Some(frame) = self.next_frame() {
            if let Some(data) = Self::extract_data(&frame) {
                events.push(data);
            }
        }
        events
    }

    fn next_frame(&mut self) -> Option<String> {
        if let Some(pos) = Self::find_boundary(&self.buffer, b"\r\n\r\n") {
            let frame_bytes = self.buffer.drain(..pos).collect::<Vec<u8>>();
            self.buffer.drain(..4); // remove boundary
            return String::from_utf8(frame_bytes).ok();
        }
        if let Some(pos) = Self::find_boundary(&self.buffer, b"\n\n") {
            let frame_bytes = self.buffer.drain(..pos).collect::<Vec<u8>>();
            self.buffer.drain(..2);
            return String::from_utf8(frame_bytes).ok();
        }
        None
    }

    fn find_boundary(buf: &[u8], boundary: &[u8]) -> Option<usize> {
        buf.windows(boundary.len()).position(|w| w == boundary)
    }

    /// Extract the `data:` field from an SSE frame.
    /// Returns None for non-data events (ping, comments, etc).
    /// Returns Some("[DONE]") for stream termination.
    fn extract_data(frame: &str) -> Option<String> {
        let mut data_parts: Vec<&str> = Vec::new();
        for line in frame.lines() {
            if line.is_empty() || line.starts_with(':') {
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                if key.trim() == "data" {
                    data_parts.push(value.trim_start_matches(' '));
                }
            }
        }
        if data_parts.is_empty() {
            return None;
        }
        let data = data_parts.join("\n");
        if data == "[DONE]" {
            return None; // Stream end signal — caller knows from stream ending.
        }
        Some(data)
    }
}

impl Default for SseParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_event() {
        let mut parser = SseParser::new();
        let events = parser.push(b"data: {\"text\":\"hello\"}\n\n");
        assert_eq!(events.len(), 1);
        assert!(events[0].contains("hello"));
    }

    #[test]
    fn parse_multiple_events() {
        let mut parser = SseParser::new();
        let events = parser.push(b"data: {\"text\":\"a\"}\n\ndata: {\"text\":\"b\"}\n\n");
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn parse_partial_then_complete() {
        let mut parser = SseParser::new();
        let events = parser.push(b"data: {\"text\":\"hel");
        assert!(events.is_empty()); // incomplete
        let events = parser.push(b"lo\"}\n\n");
        assert_eq!(events.len(), 1);
        assert!(events[0].contains("hello"));
    }

    #[test]
    fn ignore_ping_and_comments() {
        let mut parser = SseParser::new();
        let events = parser.push(b": comment\nevent: ping\ndata: {}\n\n");
        assert_eq!(events.len(), 1); // only the data event
    }

    #[test]
    fn done_signal_ignored() {
        let mut parser = SseParser::new();
        let events = parser.push(b"data: [DONE]\n\n");
        assert!(events.is_empty());
    }

    #[test]
    fn crlf_boundary() {
        let mut parser = SseParser::new();
        let events = parser.push(b"data: hi\r\n\r\n");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "hi");
    }

    #[test]
    fn finish_flushes() {
        let mut parser = SseParser::new();
        let _ = parser.push(b"data: hello"); // no trailing newline
        let events = parser.finish();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "hello");
    }
}

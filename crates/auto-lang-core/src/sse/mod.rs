// Plan 152: SSE 模块
//
// Server-Sent Events (SSE) 解析模块

pub mod decoder;
pub mod parser;
pub mod types;

pub use decoder::{SseDecodeError, SseDecoder};
pub use parser::{parse_sse_chunk, sse_parser_from_bytes, SSEParser};
pub use types::{SSEError, SSEEvent, SSEResult};

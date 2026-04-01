// Plan 152: SSE 模块
//
// Server-Sent Events (SSE) 解析模块

pub mod parser;
pub mod types;

pub use parser::{parse_sse_chunk, sse_parser_from_bytes, SSEParser};
pub use types::{SSEError, SSEEvent, SSEResult};

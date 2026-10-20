//! Length-prefixed JSON framing over byte streams (PLAN-815 T-01 §1).
//!
//! One implementation shared by the tool host and every client: a frame is
//! `u32` big-endian payload length + UTF-8 JSON bytes. Frame size is
//! checked against the budget in both directions *before* payload I/O —
//! an over-limit frame is fatal for the stream because sync is lost.

use serde_json::Value as Json;
use std::io::{Read, Write};

/// Framing failure.
#[derive(Debug)]
pub enum FrameError {
    /// Underlying I/O failure.
    Io(std::io::Error),
    /// The payload exceeded the frame budget — stream sync is lost.
    TooLarge { limit: usize, observed: u64 },
    /// Truncated prefix/payload, invalid UTF-8, or non-JSON bytes.
    Malformed { reason: String },
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::Io(e) => write!(f, "io error: {e}"),
            FrameError::TooLarge { limit, observed } => {
                write!(f, "frame of {observed} bytes exceeds budget {limit}")
            }
            FrameError::Malformed { reason } => write!(f, "malformed frame: {reason}"),
        }
    }
}

impl std::error::Error for FrameError {}

impl From<std::io::Error> for FrameError {
    fn from(e: std::io::Error) -> Self {
        FrameError::Io(e)
    }
}

/// Serialize and write one frame, enforcing the frame budget on output and
/// the transmittable nesting envelope in both writer and reader (review R1
/// F-05): a frame this writer emits is always parseable by the shared
/// reader's recursion limit.
pub fn write_frame<W: Write>(
    writer: &mut W,
    message: &Json,
    max_frame_bytes: usize,
) -> Result<(), FrameError> {
    let payload = serde_json::to_vec(message).map_err(|e| FrameError::Malformed {
        reason: format!("json serialization failed: {e}"),
    })?;
    if payload.len() > max_frame_bytes {
        return Err(FrameError::TooLarge {
            limit: max_frame_bytes,
            observed: payload.len() as u64,
        });
    }
    if json_depth(message) > MAX_FRAME_JSON_DEPTH {
        return Err(FrameError::Malformed {
            reason: format!(
                "message nests deeper than the transmittable envelope ({MAX_FRAME_JSON_DEPTH} JSON levels)"
            ),
        });
    }
    let prefix = (payload.len() as u32).to_be_bytes();
    writer.write_all(&prefix)?;
    writer.write_all(&payload)?;
    writer.flush()?;
    Ok(())
}

/// Transmittable nesting envelope shared by writer and readers (review R1
/// F-05): strictly below the JSON text deserializer's recursion limit (128)
/// so any frame one side accepts is readable by the other.
/// Nesting levels the wire request codec allows for the request document
/// itself (must stay in sync with
/// `auto_project_model::wire::MAX_WIRE_JSON_DEPTH` — pinned by a protocol
/// test).
pub const WIRE_REQUEST_JSON_DEPTH: usize = 120;

/// Fixed depth overhead of the real message envelope above the wire request
/// document: envelope object (1) + body object (1). The request value sits
/// at envelope level 2, so frame depth = wire depth + 2 exactly.
pub const ENVELOPE_DEPTH_OVERHEAD: usize = 2;

/// Transmittable nesting envelope for a whole framed message, shared by
/// the writer AND both readers (review R2, F-815-R1-05): wire request
/// budget + real envelope overhead, strictly below the JSON text
/// deserializer's recursion limit (128) so any frame one side accepts is
/// readable by the other — and everything any reader accepts is within
/// the declared envelope.
pub const MAX_FRAME_JSON_DEPTH: usize = WIRE_REQUEST_JSON_DEPTH + ENVELOPE_DEPTH_OVERHEAD;

fn json_depth(json: &Json) -> usize {
    fn depth_of(json: &Json, level: usize) -> usize {
        match json {
            Json::Array(items) => items
                .iter()
                .map(|item| depth_of(item, level + 1))
                .max()
                .unwrap_or(level),
            Json::Object(map) => map
                .values()
                .map(|value| depth_of(value, level + 1))
                .max()
                .unwrap_or(level),
            _ => level,
        }
    }
    depth_of(json, 1)
}

/// Read one frame. `Ok(None)` = clean EOF before any prefix byte.
pub fn read_frame<R: Read>(
    reader: &mut R,
    max_frame_bytes: usize,
) -> Result<Option<Json>, FrameError> {
    let mut prefix = [0u8; 4];
    match read_exact_or_eof(reader, &mut prefix)? {
        ReadOutcome::Eof => return Ok(None),
        ReadOutcome::Full => {}
    }
    let len = u32::from_be_bytes(prefix) as u64;
    if len > max_frame_bytes as u64 {
        return Err(FrameError::TooLarge {
            limit: max_frame_bytes,
            observed: len,
        });
    }
    let mut payload = vec![0u8; len as usize];
    reader
        .read_exact(&mut payload)
        .map_err(|e| FrameError::Malformed {
            reason: format!("truncated payload ({} bytes expected): {e}", len),
        })?;
    let text = std::str::from_utf8(&payload).map_err(|e| FrameError::Malformed {
        reason: format!("frame payload is not UTF-8: {e}"),
    })?;
    let message: Json = serde_json::from_str(text).map_err(|e| FrameError::Malformed {
        reason: format!("frame payload is not JSON: {e}"),
    })?;
    // Reader-side envelope gate (review R2, F-815-R1-05): frames deeper
    // than the declared transmittable envelope are rejected with a named
    // reason instead of relying on the parser's own recursion limit.
    let depth = json_depth(&message);
    if depth > MAX_FRAME_JSON_DEPTH {
        return Err(FrameError::Malformed {
            reason: format!(
                "frame nests {depth} JSON levels deep, beyond the transmittable envelope {MAX_FRAME_JSON_DEPTH}"
            ),
        });
    }
    Ok(Some(message))
}

enum ReadOutcome {
    Eof,
    Full,
}

/// Result of [`parse_frame`] — incremental (buffered) frame parsing for
/// clients that pull bytes from a pipe in chunks.
#[derive(Debug)]
pub enum FrameParse {
    /// The buffer holds fewer bytes than the (partial) frame needs.
    Incomplete { needed: usize },
    /// A full frame was parsed; `consumed` bytes may be drained.
    Complete { message: Json, consumed: usize },
    /// The frame budget was exceeded by the declared length.
    TooLarge { limit: usize, observed: u64 },
    /// Truncated-looking payload that cannot be JSON, or invalid UTF-8.
    Malformed { reason: String },
}

/// Parse one frame from the front of an accumulated byte buffer. This is
/// the buffered counterpart of [`read_frame`] and shares its wire format;
/// partial data is reported as [`FrameParse::Incomplete`], never as an
/// error, so callers know to read more instead of giving up.
pub fn parse_frame(buffer: &[u8], max_frame_bytes: usize) -> FrameParse {
    if buffer.len() < 4 {
        return FrameParse::Incomplete { needed: 4 };
    }
    let len = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as u64;
    if len > max_frame_bytes as u64 {
        return FrameParse::TooLarge {
            limit: max_frame_bytes,
            observed: len,
        };
    }
    let total = 4 + len as usize;
    if buffer.len() < total {
        return FrameParse::Incomplete { needed: total };
    }
    let payload = &buffer[4..total];
    let text = match std::str::from_utf8(payload) {
        Ok(text) => text,
        Err(e) => {
            return FrameParse::Malformed {
                reason: format!("frame payload is not UTF-8: {e}"),
            }
        }
    };
    let message: Json = match serde_json::from_str(text) {
        Ok(message) => message,
        Err(e) => {
            return FrameParse::Malformed {
                reason: format!("frame payload is not JSON: {e}"),
            }
        }
    };
    // Same reader-side envelope gate as `read_frame` — the buffered and
    // streaming read paths admit exactly the same set.
    let depth = json_depth(&message);
    if depth > MAX_FRAME_JSON_DEPTH {
        return FrameParse::Malformed {
            reason: format!(
                "frame nests {depth} JSON levels deep, beyond the transmittable envelope {MAX_FRAME_JSON_DEPTH}"
            ),
        };
    }
    FrameParse::Complete {
        message,
        consumed: total,
    }
}

/// `read_exact` that distinguishes "nothing left" from "truncated".
fn read_exact_or_eof<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<ReadOutcome, FrameError> {
    let mut filled = 0;
    while filled < buf.len() {
        let n = reader.read(&mut buf[filled..])?;
        if n == 0 {
            return if filled == 0 {
                Ok(ReadOutcome::Eof)
            } else {
                Err(FrameError::Malformed {
                    reason: "truncated length prefix".into(),
                })
            };
        }
        filled += n;
    }
    Ok(ReadOutcome::Full)
}

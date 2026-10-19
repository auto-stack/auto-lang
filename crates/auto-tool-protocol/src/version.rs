//! Protocol versioning (PLAN-815 §5.3).

/// The protocol version this crate speaks. Requests carrying a different
/// version are rejected with
/// [`ErrorKind::UnknownProtocolVersion`](crate::error::ErrorKind::UnknownProtocolVersion)
/// before any work or side effect.
pub const PROTOCOL_VERSION: u32 = 1;

/// True when a request's declared version can be served by this protocol
/// implementation.
pub fn is_supported(version: u32) -> bool {
    version == PROTOCOL_VERSION
}

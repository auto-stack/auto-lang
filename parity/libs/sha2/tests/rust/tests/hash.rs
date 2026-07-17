//! Native Rust oracle tests for the sha2 (SHA-256) replication.
//!
//! These assert the same input -> hex-digest mapping that the Auto
//! implementation must match. Test names EXACTLY mirror the TAP test names in
//! `tests/auto/hash.at` so the parity framework can compare them three-way
//! (AutoVM vs a2r vs native Rust).
//!
//! The oracle is the `sha2` crate v0.10.8 (`Sha256`), the same crate the Auto
//! library is documented to replicate. Inputs are taken as UTF-8 bytes; for
//! the ASCII NIST test vectors this is identical to `input.as_bytes()` used by
//! the Auto side (`str.char_at`).

use sha2::{Digest, Sha256};

/// Compute the SHA-256 hex digest of `input` (lowercase, 64 chars).
fn sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

// ============================================================================
// hash tests (mirror tests/auto/hash.at)
// ============================================================================

#[test]
fn test_empty() {
    assert_eq!(
        sha256(""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn test_a() {
    assert_eq!(
        sha256("a"),
        "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb"
    );
}

#[test]
fn test_abc() {
    assert_eq!(
        sha256("abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn test_message_digest() {
    assert_eq!(
        sha256("message digest"),
        "f7846f55cf23e14eebeab5b4e1550cad5b509e3348fbc4efa3a1413d393cb650"
    );
}

#[test]
fn test_long() {
    assert_eq!(
        sha256("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

#[test]
fn test_123() {
    assert_eq!(
        sha256("123"),
        "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3"
    );
}

#[test]
fn test_hello() {
    assert_eq!(
        sha256("hello"),
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn test_hello_world() {
    assert_eq!(
        sha256("hello world"),
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}

#[test]
fn test_quick_fox() {
    assert_eq!(
        sha256("The quick brown fox jumps over the lazy dog"),
        "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
    );
}

#[test]
fn test_two_blocks() {
    // Same vector as test_long: 56-byte input -> exactly two padded blocks.
    assert_eq!(
        sha256("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
        "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
    );
}

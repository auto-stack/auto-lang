//! Secure randomness for the a2r stdlib `rand` module (Plan 907).
//!
//! `secure_hex(n)` yields `n` bytes from the OS CSPRNG (via the `getrandom`
//! crate: ProcessPrng on Windows, getrandom(2) on Unix) as lowercase hex —
//! the backend-track source for ≥128-bit credentials and session tokens.
//! This is deliberately NOT `uuid()` (epoch nanos) and NOT the SQLite PRNG
//! (Windows seed is timing-derived in the bundled build): both are
//! predictable and rejected for credential use by consumer audits
//! (auto-chat CHAT-002 T-00).
//!
//! Error convention follows the a2r-std sentinel pattern (see
//! `sqlite::last_error`): never panic; on entropy failure return the empty
//! string and record the reason for [`last_error`].
//!
//! Hand-copied from `stdlib/auto/rand.at` + `rand.rs.at` — keep the
//! signatures aligned (checked by the KNOWN-DEBT 396 signature-parity test).

use std::sync::Mutex;

static LAST_ERROR: Mutex<String> = Mutex::new(String::new());

fn record_error(msg: String) {
    *LAST_ERROR.lock().unwrap() = msg;
}

/// Fill `buf` with cryptographically secure random bytes from the OS.
fn os_random(buf: &mut [u8]) -> Result<(), String> {
    getrandom::getrandom(buf).map_err(|e| format!("getrandom failed: {e}"))
}

/// `n` random bytes from the OS CSPRNG as lowercase hex (`2n` chars).
/// `n <= 0` yields the empty string. On OS entropy failure, returns ""
/// and records the reason for [`last_error`].
pub fn secure_hex(n: i32) -> String {
    if n <= 0 {
        return String::new();
    }
    let n = n as usize;
    let mut buf = vec![0u8; n];
    if let Err(e) = os_random(&mut buf) {
        record_error(e);
        return String::new();
    }
    let mut out = String::with_capacity(n * 2);
    for b in buf {
        out.push_str(&format!("{:02x}", b));
    }
    out
}

/// The most recent error message recorded by this module ("" if none).
pub fn last_error() -> String {
    LAST_ERROR.lock().unwrap().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEX: &str = "0123456789abcdef";

    #[test]
    fn secure_hex_length_and_charset() {
        for n in [1i32, 8, 32, 64] {
            let s = secure_hex(n);
            assert_eq!(s.len(), (n as usize) * 2, "len mismatch for n={n}");
            assert!(s.chars().all(|c| HEX.contains(c)), "non-hex char in {s}");
        }
    }

    #[test]
    fn secure_hex_nonzero_input_never_empty() {
        for _ in 0..16 {
            assert!(!secure_hex(32).is_empty(), "32-byte draw must not fail");
        }
        assert_eq!(last_error(), "", "successful draws must not record errors");
    }

    #[test]
    fn secure_hex_unique_across_calls() {
        let a = secure_hex(32);
        let b = secure_hex(32);
        let c = secure_hex(32);
        assert_ne!(a, b);
        assert_ne!(b, c);
        assert_ne!(a, c);
    }

    #[test]
    fn secure_hex_edge_inputs() {
        assert_eq!(secure_hex(0), "");
        assert_eq!(secure_hex(-5), "");
    }
}

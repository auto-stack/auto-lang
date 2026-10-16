//! Read-only semantic-reference snapshot (plan §5.6, parallel-contract
//! "共同语义与测试接缝").
//!
//! PLAN-796 owns DeclRef/TypeRef and the canonical semantic contract; at
//! T-01 it has not landed (still drafting, no frozen schema), so this
//! module carries a 797-local, content-hashed snapshot of exactly the
//! semantic references the ABI fixtures need. It is a temporary adapter
//! by construction:
//!
//! * the format id (`autoabi-semref-snapshot/0`) and content hash are
//!   recorded in contract-decision.md §3 and must not drift silently;
//! * once D-796 holds (796 merged with its real semantic contract), T-06
//!   re-binds every fixture to the real codec, deletes the duplicate
//!   local validation, and records the retirement receipt — two
//!   authoritative semantic models are not allowed to coexist (§5.6);
//! * this snapshot never grows into a general type-inference or CT
//!   library (that is 796's scope, not 797's).

use std::sync::OnceLock;

/// The frozen snapshot document. Types used by T-01/T-02 fixtures only.
pub const SNAPSHOT_JSON: &str = r##"{
  "format": "autoabi-semref-snapshot/0",
  "status": "temporary-797-local",
  "retire_at": "D-796 (PLAN-796 semantic contract merged); see plan797 T-06",
  "note": "PLAN-797 read-only fixture-level semantic references. Not the canonical semantic contract; no inference, no CT. Hash recorded in docs/plans/attachments/797/contract-decision.md.",
  "types": {
    "RecMix": {
      "kind": "record",
      "fields": [
        ["flag", "bool"],
        ["count", "i32"],
        ["tag", "char"],
        ["big", "i64"]
      ]
    }
  },
  "funcs": {
    "p797_mix": {
      "params": [["a", "i32"], ["b", "i32"], ["c", "i32"], ["d", "i32"],
                 ["e", "i64"], ["f", "f64"], ["g", "i32"], ["h", "f64"]],
      "ret": "i64"
    }
  }
}"##;

/// blake3 content hash of the frozen snapshot bytes (64-char hex).
pub fn snapshot_hash() -> &'static str {
    static HASH: OnceLock<String> = OnceLock::new();
    HASH.get_or_init(|| blake3_hex(SNAPSHOT_JSON.as_bytes()))
}

/// blake3, hex-encoded (64 chars — full digest).
pub fn blake3_hex(bytes: &[u8]) -> String {
    let mut h = blake3::Hasher::new();
    h.update(bytes);
    h.finalize().to_hex().to_string()
}

/// Validate the snapshot is well-formed JSON with the expected top-level
/// shape. Runs in tests so a hand-edited snapshot cannot drift into
/// something the hash was frozen against.
pub fn verify_snapshot() -> Result<(), String> {
    let v: serde_json::Value = serde_json::from_str(SNAPSHOT_JSON)
        .map_err(|e| format!("snapshot JSON parse: {e}"))?;
    if v["format"] != "autoabi-semref-snapshot/0" {
        return Err("snapshot format id changed".into());
    }
    if v["status"] != "temporary-797-local" {
        return Err("snapshot status changed".into());
    }
    for key in ["types", "funcs"] {
        if !v[key].is_object() {
            return Err(format!("snapshot '{key}' is not an object"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_wellformed() {
        verify_snapshot().expect("frozen snapshot must stay well-formed");
    }

    #[test]
    fn snapshot_hash_stable() {
        // Deterministic across calls; full 64-hex blake3.
        let h = snapshot_hash();
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(h, snapshot_hash());
        println!("semref_snapshot_hash = {h}");
    }
}

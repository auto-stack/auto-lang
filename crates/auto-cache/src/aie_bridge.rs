// AIE Bridge: Integration with Auto Incremental Engine
//
// **Plan 082**: Phase 2 - Build on AIE infrastructure
//
// Features:
// - Reuse AIE's pre-computed interface hashes (L3 hashes)
// - Extract dependency information from Database
// - Extend AIE's hash caching for global cache keys
//
// This module provides bridge functions to connect AutoCache with
// the existing AIE Database and CompileSession infrastructure.

use crate::fingerprint::{Fingerprint, CompilationTarget};

/// AIE Bridge for integrating with existing AIE infrastructure
///
/// This provides compatibility layer to reuse AIE's interface hashing
/// and dependency tracking without duplicating effort.
pub struct AieBridge;

impl AieBridge {
    /// Create fingerprint from AIE interface hash
    ///
    /// This is the primary integration point with AIE.
    /// When AIE has already computed the interface hash for a fragment,
    /// we can reuse it instead of recomputing.
    ///
    /// # Arguments
    /// * `interface_hash` - Pre-computed L3 interface hash from AIE
    /// * `target` - Compilation target
    /// * `dependency_hashes` - List of dependency interface hashes
    ///
    /// # Returns
    /// Complete fingerprint for cache key
    ///
    /// # Example
    /// ```no_run
    /// use auto_cache::aie_bridge::AieBridge;
    /// use auto_cache::fingerprint::CompilationTarget;
    ///
    /// // Get interface hash from AIE Database
    /// let interface_hash = [0u8; 32];  // From AIE
    ///
    /// // Get dependency hashes from AIE
    /// let dep_hashes = vec![[0u8; 32]];
    ///
    /// let target = CompilationTarget::current();
    /// let fp = AieBridge::fingerprint_from_aie(
    ///     interface_hash,
    ///     &target,
    ///     &dep_hashes
    /// );
    /// ```
    pub fn fingerprint_from_aie(
        interface_hash: [u8; 32],
        target: &CompilationTarget,
        dependency_hashes: &[[u8; 32]],
    ) -> Fingerprint {
        let content_hash = interface_hash;  // Reuse AIE's L3 hash
        let context_hash = Fingerprint::compute_context_hash(target);
        let dependency_hash = Self::compute_dep_hash_from_aie(dependency_hashes);

        Fingerprint {
            content_hash,
            context_hash,
            dependency_hash,
        }
    }

    /// Compute dependency hash from AIE dependency list
    ///
    /// Converts AIE's dependency interface hashes into a Merkle root.
    /// This is consistent with Fingerprint's dependency hashing but
    /// works directly with AIE's hash format.
    fn compute_dep_hash_from_aie(dep_hashes: &[[u8; 32]]) -> [u8; 32] {
        if dep_hashes.is_empty() {
            return [0u8; 32];
        }

        let mut hasher = blake3::Hasher::new();

        // Sort for determinism
        let mut sorted = dep_hashes.to_vec();
        sorted.sort();  // [u8; 32] implements Ord lexicographically

        // Hash each dependency
        for dep_hash in sorted {
            hasher.update(&dep_hash);
        }

        hasher.finalize().into()
    }

    /// Create cache key from module name and interface hash
    ///
    /// This is a convenience function for creating cache keys
    /// in the format used by AutoCache.
    ///
    /// # Arguments
    /// * `module_name` - Name of the module (e.g., "std:io")
    /// * `interface_hash` - AIE interface hash (L3 hash)
    /// * `target` - Compilation target
    ///
    /// # Returns
    /// Cache key string for use with AutoCache::get() and AutoCache::put()
    pub fn cache_key_from_module(
        module_name: &str,
        interface_hash: [u8; 32],
        target: &CompilationTarget,
    ) -> String {
        let fp = Self::fingerprint_from_aie(interface_hash, target, &[]);
        format!("{}:{}", module_name, fp.target_hash())
    }

    /// Extract interface hash from AIE Database (stub)
    ///
    /// In Phase 3, this will connect to actual AIE Database.
    /// For now, it's a placeholder showing the integration point.
    ///
    /// # Future Implementation
    /// ```ignore
    /// pub fn get_interface_hash(
    ///     db: &Database,
    ///     frag_id: &FragId
    /// ) -> Option<[u8; 32]> {
    ///     // Get L3 hash from AIE Database
    ///     let hash = db.get_fragment_iface_hash(frag_id)?;
    ///
    ///     // Convert u64 to [u8; 32] by zero-padding
    ///     let mut bytes = [0u8; 32];
    ///     bytes[0..8].copy_from_slice(&hash.to_le_bytes());
    ///
    ///     Some(bytes)
    /// }
    /// ```
    pub fn get_interface_hash_stub(_frag_id: &str) -> Option<[u8; 32]> {
        // TODO: Phase 3 - Connect to AIE Database
        None
    }
}

/// Hash utilities for working with AIE's hash format
///
/// AIE uses u64 for hashes (truncated BLAKE3), while AutoCache
/// uses full [u8; 32] arrays. These utilities help convert between formats.
pub struct HashUtils;

impl HashUtils {
    /// Convert AIE's u64 hash to AutoCache's [u8; 32] format
    ///
    /// AIE stores interface hashes as u64 (first 8 bytes of BLAKE3).
    /// AutoCache uses full 32-byte hashes. This conversion
    /// zero-pads the u64 to create a compatible hash.
    pub fn u64_to_array(hash: u64) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes[0..8].copy_from_slice(&hash.to_le_bytes());
        bytes
    }

    /// Convert AutoCache's [u8; 32] hash to AIE's u64 format
    ///
    /// Extracts the first 8 bytes (little-endian) for AIE compatibility.
    pub fn array_to_u64(bytes: &[u8; 32]) -> u64 {
        u64::from_le_bytes(bytes[0..8].try_into().unwrap())
    }

    /// Convert hex string to hash array
    ///
    /// Useful for converting between string and binary representations.
    pub fn hex_to_array(hex: &str) -> Option<[u8; 32]> {
        if hex.len() != 64 {
            return None;
        }

        let mut bytes = [0u8; 32];
        for i in 0..32 {
            let byte_str = &hex[i * 2..i * 2 + 2];
            bytes[i] = u8::from_str_radix(byte_str, 16).ok()?;
        }

        Some(bytes)
    }

    /// Convert hash array to hex string
    pub fn array_to_hex(bytes: &[u8; 32]) -> String {
        bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_from_aie() {
        let interface_hash = [42u8; 32];
        let target = CompilationTarget::current();
        let dep_hashes = vec![[1u8; 32], [2u8; 32]];

        let fp = AieBridge::fingerprint_from_aie(
            interface_hash,
            &target,
            &dep_hashes,
        );

        assert_eq!(fp.content_hash, interface_hash);
        assert_ne!(fp.context_hash, [0u8; 32]);
        assert_ne!(fp.dependency_hash, [0u8; 32]);
    }

    #[test]
    fn test_cache_key_from_module() {
        let interface_hash = [123u8; 32];
        let target = CompilationTarget::current();

        let key = AieBridge::cache_key_from_module(
            "std:io",
            interface_hash,
            &target,
        );

        assert!(key.starts_with("std:io:"));
        assert!(key.contains("std:io:"));
    }

    #[test]
    fn test_hash_utils_u64_conversion() {
        let hash_u64 = 0x1234567890ABCDEFu64;
        let bytes = HashUtils::u64_to_array(hash_u64);

        // First 8 bytes should be the little-endian representation
        assert_eq!(bytes[0], 0xEF);
        assert_eq!(bytes[1], 0xCD);
        assert_eq!(bytes[2], 0xAB);
        assert_eq!(bytes[3], 0x90);
        assert_eq!(bytes[4], 0x78);
        assert_eq!(bytes[5], 0x56);
        assert_eq!(bytes[6], 0x34);
        assert_eq!(bytes[7], 0x12);

        // Round-trip conversion
        let back_to_u64 = HashUtils::array_to_u64(&bytes);
        assert_eq!(hash_u64, back_to_u64);
    }

    #[test]
    fn test_hash_utils_hex_conversion() {
        let mut bytes = [0u8; 32];
        bytes[0] = 0xAB;
        bytes[1] = 0xCD;
        bytes[2] = 0xEF;

        let hex = HashUtils::array_to_hex(&bytes);
        assert_eq!(hex.len(), 64);
        assert_eq!(&hex[..6], "abcdef");

        // Round-trip
        let back_to_bytes = HashUtils::hex_to_array(&hex).unwrap();
        assert_eq!(bytes, back_to_bytes);
    }

    #[test]
    fn test_dep_hash_empty() {
        let hash = AieBridge::compute_dep_hash_from_aie(&[]);
        assert_eq!(hash, [0u8; 32]);
    }

    #[test]
    fn test_dep_hash_single() {
        let deps = [[1u8; 32]];
        let hash = AieBridge::compute_dep_hash_from_aie(&deps);

        // Non-zero hash for non-empty deps
        assert_ne!(hash, [0u8; 32]);
    }

    #[test]
    fn test_dep_hash_ordering() {
        let dep1 = [1u8; 32];
        let dep2 = [2u8; 32];

        // Same deps in different order should produce same hash
        let hash1 = AieBridge::compute_dep_hash_from_aie(&[dep1, dep2]);
        let hash2 = AieBridge::compute_dep_hash_from_aie(&[dep2, dep1]);

        assert_eq!(hash1, hash2);
    }
}

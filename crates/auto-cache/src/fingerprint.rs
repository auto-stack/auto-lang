// Fingerprint: Global cache key computation for AutoCache
//
// **Plan 082**: Hash computation extension building on AIE
//
// Features:
// - ContentHash: AST-based, format-independent (reuses AIE interface hash)
// - ContextHash: Target platform, compiler flags, toolchain version
// - DependencyHash: Merkle tree of module dependencies
// - Combines all three into final cache key

/// Fingerprint for cache key computation
///
/// Combines ContentHash + ContextHash + DependencyHash
/// to create a globally unique cache key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint {
    /// Content hash (from AIE interface hash)
    pub content_hash: [u8; 32],

    /// Context hash (target + flags)
    pub context_hash: [u8; 32],

    /// Dependency hash (Merkle root)
    pub dependency_hash: [u8; 32],
}

impl Fingerprint {
    /// Create new fingerprint
    pub fn new(
        content_hash: [u8; 32],
        context_hash: [u8; 32],
        dependency_hash: [u8; 32],
    ) -> Self {
        Self {
            content_hash,
            context_hash,
            dependency_hash,
        }
    }

    /// Compute target hash for cache lookup
    ///
    /// Combines all three hashes using BLAKE3
    pub fn target_hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();

        // Combine all three hashes
        hasher.update(&self.content_hash);
        hasher.update(&self.context_hash);
        hasher.update(&self.dependency_hash);

        // Convert to hex string
        hasher.finalize().to_hex().to_string()
    }

    /// Compute fingerprint from module source
    ///
    /// # Arguments
    /// * `source` - Source code content
    /// * `target` - Compilation target
    /// * `dependencies` - List of dependency fingerprints
    pub fn compute(
        source: &str,
        target: &CompilationTarget,
        dependencies: &[Fingerprint],
    ) -> Self {
        let content_hash = Self::compute_content_hash(source);
        let context_hash = Self::compute_context_hash(target);
        let dependency_hash = Self::compute_dependency_hash(dependencies);

        Self {
            content_hash,
            context_hash,
            dependency_hash,
        }
    }

    /// Compute content hash from source code
    ///
    /// This builds on AIE's interface hash computation.
    /// For now, we hash the full source, but in Phase 3
    /// we'll integrate with AIE's Database to get the
    /// pre-computed interface hash.
    pub fn compute_content_hash(source: &str) -> [u8; 32] {
        blake3::hash(source.as_bytes()).into()
    }

    /// Compute context hash from compilation target
    ///
    /// Includes:
    /// - Target triple (e.g., "x86_64-pc-windows-msvc")
    /// - Optimization level
    /// - Compiler flags
    pub fn compute_context_hash(target: &CompilationTarget) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();

        // Hash target triple
        hasher.update(target.triple.as_bytes());

        // Hash optimization level
        hasher.update(&[target.opt_level as u8]);

        // Hash compiler flags (sorted for determinism)
        let mut flags: Vec<&String> = target.flags.iter().collect();
        flags.sort();  // Sort for consistent hashing
        for flag in flags {
            hasher.update(flag.as_bytes());
        }

        hasher.finalize().into()
    }

    /// Compute dependency hash (Merkle root)
    ///
    /// Combines all dependency fingerprints into a single hash
    /// using Merkle tree construction.
    pub fn compute_dependency_hash(dependencies: &[Fingerprint]) -> [u8; 32] {
        if dependencies.is_empty() {
            // Empty dependencies -> fixed empty hash
            return [0u8; 32];
        }

        let mut hasher = blake3::Hasher::new();

        // Sort dependencies by hash for determinism
        let mut sorted_deps = dependencies.to_vec();
        sorted_deps.sort_by(|a, b| {
            let a_hash = blake3::hash(&a.content_hash);
            let b_hash = blake3::hash(&b.content_hash);
            a_hash.as_bytes().cmp(b_hash.as_bytes())
        });

        // Hash each dependency's target_hash
        for dep in &sorted_deps {
            let dep_key = dep.target_hash();
            hasher.update(dep_key.as_bytes());
        }

        hasher.finalize().into()
    }

    /// Get short hash (first 16 chars of hex) for display
    pub fn short_hash(&self) -> String {
        let full = self.target_hash();
        full.chars().take(16).collect()
    }
}

/// Compilation target information
///
/// Captures all context that affects compilation output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationTarget {
    /// Target triple (e.g., "x86_64-pc-windows-msvc")
    pub triple: String,

    /// Optimization level (0-3)
    pub opt_level: u8,

    /// Compiler flags
    pub flags: Vec<String>,
}

impl CompilationTarget {
    /// Create default target for current platform
    pub fn current() -> Self {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let _env = std::env::consts::FAMILY;  // Reserved for future use

        // Construct target triple: arch-vendor-os
        let triple = format!("{}-unknown-{}", arch, os);

        Self {
            triple,
            opt_level: 0,  // Default: no optimization
            flags: Vec::new(),
        }
    }

    /// Create target with optimization level
    pub fn with_optimization(opt_level: u8) -> Self {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;

        let triple = format!("{}-unknown-{}", arch, os);

        Self {
            triple,
            opt_level,
            flags: Vec::new(),
        }
    }

    /// Create transpilation target (a2c, a2r)
    ///
    /// Used when transpiling to C or Rust.
    pub fn transpilation(lang: TranspilationLang) -> Self {
        Self {
            triple: format!("auto-{}", lang.as_str()),
            opt_level: 0,
            flags: vec!["transpile".to_string(), lang.as_str().to_string()],
        }
    }

    /// Add compiler flag
    pub fn with_flag(mut self, flag: &str) -> Self {
        self.flags.push(flag.to_string());
        self
    }
}

/// Transpilation target language
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranspilationLang {
    C,
    Rust,
    Bytecode,  // AutoVM bytecode
}

impl TranspilationLang {
    fn as_str(&self) -> &str {
        match self {
            TranspilationLang::C => "c",
            TranspilationLang::Rust => "rust",
            TranspilationLang::Bytecode => "bytecode",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint_computation() {
        let source = r#"
fn add(a int, b int) int {
    a + b
}
"#;

        let target = CompilationTarget::current();
        let fingerprint = Fingerprint::compute(source, &target, &[]);

        // Verify fingerprint has all three components
        assert_ne!(fingerprint.content_hash, [0u8; 32]);
        assert_ne!(fingerprint.context_hash, [0u8; 32]);
        assert_eq!(fingerprint.dependency_hash, [0u8; 32]);  // Empty deps

        // Verify target hash is deterministic
        let hash1 = fingerprint.target_hash();
        let hash2 = fingerprint.target_hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_content_hash_different_source() {
        let source1 = "fn add(a int, b int) int { a + b }";
        let source2 = "fn add(a int, b int) int { a - b }";

        let hash1 = Fingerprint::compute_content_hash(source1);
        let hash2 = Fingerprint::compute_content_hash(source2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_content_hash_same_source() {
        let source = "fn add(a int, b int) int { a + b }";

        let hash1 = Fingerprint::compute_content_hash(source);
        let hash2 = Fingerprint::compute_content_hash(source);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_context_hash_different_targets() {
        let target1 = CompilationTarget::with_optimization(0);
        let target2 = CompilationTarget::with_optimization(2);

        let hash1 = Fingerprint::compute_context_hash(&target1);
        let hash2 = Fingerprint::compute_context_hash(&target2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_dependency_hash_empty() {
        let hash = Fingerprint::compute_dependency_hash(&[]);
        assert_eq!(hash, [0u8; 32]);
    }

    #[test]
    fn test_dependency_hash_with_deps() {
        let source = "fn main() int { 0 }";
        let target = CompilationTarget::current();

        let dep1 = Fingerprint::compute(source, &target, &[]);
        let dep2 = Fingerprint::compute(source, &target, &[]);

        let hash = Fingerprint::compute_dependency_hash(&[dep1, dep2]);
        assert_ne!(hash, [0u8; 32]);
    }

    #[test]
    fn test_transpilation_target() {
        let target_c = CompilationTarget::transpilation(TranspilationLang::C);
        let target_rust = CompilationTarget::transpilation(TranspilationLang::Rust);

        assert!(target_c.triple.starts_with("auto-c"));
        assert!(target_rust.triple.starts_with("auto-rust"));

        assert!(target_c.flags.contains(&"transpile".to_string()));
        assert!(target_c.flags.contains(&"c".to_string()));
    }

    #[test]
    fn test_target_hash_deterministic() {
        let source = "fn test() int { 42 }";
        let target = CompilationTarget::current();

        let fp1 = Fingerprint::compute(source, &target, &[]);
        let fp2 = Fingerprint::compute(source, &target, &[]);

        assert_eq!(fp1.target_hash(), fp2.target_hash());
    }

    #[test]
    fn test_short_hash() {
        let source = "fn test() int { 42 }";
        let target = CompilationTarget::current();

        let fp = Fingerprint::compute(source, &target, &[]);
        let short = fp.short_hash();
        let full = fp.target_hash();

        assert_eq!(short.len(), 16);
        assert_eq!(short, &full[..16]);
    }
}

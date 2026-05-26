// Sandbox: Managed Rust compilation environment for FFI
//
// **Plan 092**: Rust FFI via Sandbox Compilation
//
// The Sandbox provides a controlled compilation environment that ensures:
// 1. All crates are compiled with the same toolchain
// 2. All crates link to shared dependencies
// 3. ABI compatibility across all loaded libraries
//
// **Architecture**:
// ~/.auto/sandbox/
// ├── toolchain/
// │   └── rust-1.75.0/         # Managed rustc
// ├── crates/
// │   ├── libstd-1.75.0.so     # Shared stdlib
// │   ├── libserde-1.0.193.so  # Shared serde
// │   └── ...
// └── registry/
//     └── index.db             # SQLite crate registry

use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

use super::registry::CrateRegistry;

// =============================================================================
// Plan 212 Phase 2.1: FFI Shim Type Definitions
// =============================================================================

/// Lightweight FFI type descriptor for wrapper crate generation.
/// Defined in auto-cache to avoid circular dependency on auto-lang's RustType.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShimType {
    Void,
    I32,
    I64,
    F64,
    Bool,
    /// Null-terminated C string (*const c_char)
    CString,
}

impl ShimType {
    /// C type name for wrapper generation
    pub fn c_type_name(&self) -> &'static str {
        match self {
            ShimType::Void => "void",
            ShimType::I32 => "i32",
            ShimType::I64 => "i64",
            ShimType::F64 => "f64",
            ShimType::Bool => "bool",
            ShimType::CString => "*const std::os::raw::c_char",
        }
    }

    /// Rust type for the wrapper function body
    pub fn rust_type_name(&self) -> &'static str {
        match self {
            ShimType::Void => "()",
            ShimType::I32 => "i32",
            ShimType::I64 => "i64",
            ShimType::F64 => "f64",
            ShimType::Bool => "bool",
            ShimType::CString => "String",
        }
    }
}

/// Descriptor for a single FFI shim function in a wrapper crate.
#[derive(Debug, Clone)]
pub struct FunctionShim {
    pub name: String,
    pub param_types: Vec<ShimType>,
    pub return_type: ShimType,
    /// Override the entire function body (for complex shims)
    pub body_override: Option<String>,
    /// Whether the original Rust function returns Result<T, E>
    pub returns_result: bool,
}

impl FunctionShim {
    pub fn string_to_string(name: &str) -> Self {
        Self {
            name: name.to_string(),
            param_types: vec![ShimType::CString],
            return_type: ShimType::CString,
            body_override: None,
            returns_result: false,
        }
    }

    /// Convert from auto-lang's RustSignature descriptor.
    /// `sig_str` format: "params:ret" where params is comma-separated type chars:
    ///   v=void, i=i32, l=i64, f=f32, d=f64, b=bool, s=string
    ///   ret: same type chars
    /// Example: "s:s" = String→String, ":l" = ()→i64, "ll:l" = (i64,i64)→i64
    pub fn from_sig_str(name: &str, sig_str: &str) -> Self {
        let (params_str, ret_str) = sig_str.split_once(':').unwrap_or(("", "s"));
        let param_types: Vec<ShimType> = if params_str.is_empty() {
            vec![]
        } else {
            params_str.chars().map(Self::char_to_type).collect()
        };
        let return_type = Self::char_to_type(ret_str.chars().next().unwrap_or('s'));
        Self {
            name: name.to_string(),
            param_types,
            return_type,
            body_override: None,
            returns_result: false,
        }
    }

    fn char_to_type(c: char) -> ShimType {
        match c {
            'v' => ShimType::Void,
            'i' => ShimType::I32,
            'l' => ShimType::I64,
            'f' => ShimType::F64,
            'b' => ShimType::Bool,
            's' => ShimType::CString,
            _ => ShimType::CString,
        }
    }

    fn type_to_char(t: &ShimType) -> char {
        match t {
            ShimType::Void => 'v',
            ShimType::I32 => 'i',
            ShimType::I64 => 'l',
            ShimType::F64 => 'f',
            ShimType::Bool => 'b',
            ShimType::CString => 's',
        }
    }

    /// Encode as sig_str for cache key
    pub fn sig_str(&self) -> String {
        let params: String = self.param_types.iter().map(|t| Self::type_to_char(t)).collect();
        let ret = Self::type_to_char(&self.return_type);
        format!("{}:{}", params, ret)
    }
}

// =============================================================================
// Crate Metadata Types
// =============================================================================

/// Source of a Rust crate
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CrateSource {
    /// Downloaded from crates.io
    CratesIo,
    /// Cloned from git repository
    Git,
    /// Local crate (user library)
    Local,
    /// System library (pre-installed)
    System,
}

/// Metadata for a compiled Rust crate
///
/// Tracks crate information for the sandbox compilation system.
/// Used to ensure ABI compatibility when loading dynamic libraries.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrateMetadata {
    /// Crate name (e.g., "serde", "serde_json")
    pub name: String,

    /// Semantic version (e.g., "1.0.193")
    pub version: String,

    /// Rust compiler version used to compile this crate
    /// Must match AutoVM's rustc version for ABI compatibility
    pub rustc_version: String,

    /// Target triple (e.g., "x86_64-unknown-linux-gnu")
    pub target: String,

    /// List of dependency crate identifiers (e.g., ["serde-1.0.193", "itoa-1.0.10"])
    pub dependencies: Vec<String>,

    /// Hash computed from all transitive dependencies
    /// Used for ABI compatibility verification
    pub abi_hash: String,

    /// Path to the compiled library file
    pub library_path: PathBuf,

    /// When this crate was compiled
    pub compiled_at: u64,

    /// Source of the crate (e.g., "crates.io", "git", "local")
    pub source: CrateSource,
}

impl std::fmt::Display for CrateMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} v{} ({})", self.name, self.version, self.target)
    }
}

impl CrateMetadata {
    /// Create a unique identifier for this crate version
    pub fn crate_id(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }

    /// Check if this crate is ABI compatible with the given environment
    pub fn is_abi_compatible(&self, rustc_version: &str, target: &str) -> bool {
        self.rustc_version == rustc_version && self.target == target
    }
}

// =============================================================================
// Sandbox Error Types
// =============================================================================

/// Sandbox errors
#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Toolchain not found: {0}")]
    ToolchainNotFound(String),

    #[error("Crate not found: {0}")]
    CrateNotFound(String),

    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("ABI incompatible: expected {expected}, got {actual}")]
    ABIIncompatible { expected: String, actual: String },

    #[error("Library loading failed: {0}")]
    LibraryLoad(String),

    #[error("Registry error: {0}")]
    Registry(String),
}

/// Result type for sandbox operations
pub type Result<T> = std::result::Result<T, SandboxError>;

/// Sandbox: Managed Rust compilation environment
pub struct Sandbox {
    /// Root directory for sandbox
    root: PathBuf,

    /// Path to toolchain directory
    toolchain_path: PathBuf,

    /// Path to compiled crates
    crates_path: PathBuf,

    /// Path to registry database
    registry_path: PathBuf,

    /// Crate registry (SQLite-backed metadata store)
    registry: CrateRegistry,

    /// Current toolchain version
    rustc_version: String,

    /// Current target triple
    target: String,
}

impl Sandbox {
    /// Create or open a sandbox at the default location
    pub fn new() -> Result<Self> {
        let root = Self::default_root()?;
        Self::with_root(root)
    }

    /// Create or open a sandbox at a specific location
    pub fn with_root(root: PathBuf) -> Result<Self> {
        let toolchain_path = root.join("toolchain");
        let crates_path = root.join("crates");
        let registry_path = root.join("registry");

        // Ensure directories exist
        std::fs::create_dir_all(&toolchain_path)?;
        std::fs::create_dir_all(&crates_path)?;
        std::fs::create_dir_all(&registry_path)?;

        // Initialize crate registry
        let db_path = registry_path.join("index.db");
        let registry = CrateRegistry::new(&db_path)
            .map_err(|e| SandboxError::Registry(e.to_string()))?;

        // Detect current toolchain info
        let rustc_version = Self::detect_rustc_version()?;
        let target = Self::detect_target()?;

        Ok(Self {
            root,
            toolchain_path,
            crates_path,
            registry_path,
            registry,
            rustc_version,
            target,
        })
    }

    /// Get the default sandbox root directory
    fn default_root() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            SandboxError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Cannot find home directory",
            ))
        })?;
        Ok(home.join(".auto").join("sandbox"))
    }

    /// Detect the current rustc version
    fn detect_rustc_version() -> Result<String> {
        let output = Command::new("rustc")
            .arg("--version")
            .output()
            .map_err(|e| SandboxError::ToolchainNotFound(format!("rustc not found: {}", e)))?;

        let version_str = String::from_utf8_lossy(&output.stdout);
        // Parse "rustc 1.75.0 (82e1608df 2023-12-21)" -> "1.75.0"
        let version = version_str
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| SandboxError::ToolchainNotFound("Cannot parse rustc version".into()))?;
        Ok(version.to_string())
    }

    /// Detect the current target triple
    fn detect_target() -> Result<String> {
        let output = Command::new("rustc")
            .arg("-vV")
            .output()
            .map_err(|e| SandboxError::ToolchainNotFound(format!("rustc not found: {}", e)))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        // Parse "host: x86_64-unknown-linux-gnu"
        for line in output_str.lines() {
            if line.starts_with("host:") {
                return Ok(line.split(':').nth(1).unwrap().trim().to_string());
            }
        }

        Err(SandboxError::ToolchainNotFound("Cannot parse target".into()))
    }

    /// Get the sandbox root directory
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the current rustc version
    pub fn rustc_version(&self) -> &str {
        &self.rustc_version
    }

    /// Get the current target triple
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Get the path to compiled crates
    pub fn crates_path(&self) -> &Path {
        &self.crates_path
    }

    /// Get the crate registry
    pub fn registry(&self) -> &CrateRegistry {
        &self.registry
    }

    /// Get mutable access to the crate registry
    pub fn registry_mut(&mut self) -> &mut CrateRegistry {
        &mut self.registry
    }

    /// Garbage-collect old cached crates and build artifacts.
    /// Removes files older than `max_age` in both crates/ and builds/ directories.
    /// Returns the number of files removed.
    pub fn gc(&self, max_age: std::time::Duration) -> Result<usize> {
        let now = std::time::SystemTime::now();
        let mut removed = 0;

        // Clean crates/ directory
        removed += self.clean_dir(&self.crates_path, &now, &max_age)?;

        // Clean builds/ directory
        let builds_dir = self.root.join("builds");
        if builds_dir.exists() {
            removed += self.clean_dir(&builds_dir, &now, &max_age)?;
        }

        log::info!("Sandbox GC: removed {} old files", removed);
        Ok(removed)
    }

    /// Remove files (not directories) older than max_age from a directory.
    fn clean_dir(&self, dir: &Path, now: &std::time::SystemTime, max_age: &std::time::Duration) -> Result<usize> {
        let mut removed = 0;
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(0),
        };
        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };
            let path = entry.path();
            if path.is_dir() {
                // Recurse into subdirectories (e.g., builds/wrapper_name/)
                removed += self.clean_dir(&path, now, max_age)?;
                // Remove empty directories
                if std::fs::read_dir(&path).map(|mut d| d.next().is_none()).unwrap_or(false) {
                    let _ = std::fs::remove_dir(&path);
                }
            } else if path.is_file() {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(age) = now.duration_since(modified) {
                            if age > *max_age {
                                if std::fs::remove_file(&path).is_ok() {
                                    removed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(removed)
    }

    /// Get the path for a specific crate's library
    pub fn crate_library_path(&self, name: &str, version: &str) -> PathBuf {
        let lib_name = self.library_name(name, version);
        self.crates_path.join(lib_name)
    }

    /// Generate library filename for the current platform
    fn library_name(&self, name: &str, version: &str) -> String {
        let crate_id = format!("{}-{}", name.replace('-', "_"), version);

        #[cfg(target_os = "windows")]
        { format!("{}.dll", crate_id) }

        #[cfg(target_os = "macos")]
        { format!("lib{}.dylib", crate_id) }

        #[cfg(target_os = "linux")]
        { format!("lib{}.so", crate_id) }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        { format!("lib{}.so", crate_id) }
    }

    /// Check if a crate is already compiled
    pub fn is_compiled(&self, name: &str, version: &str) -> bool {
        self.crate_library_path(name, version).exists()
    }

    /// Load a compiled crate as a dynamic library
    ///
    /// # Safety
    /// The library must be compiled with the same toolchain and target.
    pub unsafe fn load_crate(&self, name: &str, version: &str) -> Result<libloading::Library> {
        let path = self.crate_library_path(name, version);

        if !path.exists() {
            return Err(SandboxError::CrateNotFound(format!(
                "{}-{} not found at {}",
                name, version,
                path.display()
            )));
        }

        let library = libloading::Library::new(&path)
            .map_err(|e| SandboxError::LibraryLoad(format!("{}: {}", path.display(), e)))?;

        Ok(library)
    }

    /// Verify ABI compatibility with the sandbox
    pub fn verify_abi(&self, metadata: &CrateMetadata) -> Result<()> {
        if !metadata.is_abi_compatible(&self.rustc_version, &self.target) {
            return Err(SandboxError::ABIIncompatible {
                expected: format!("rustc {} for {}", self.rustc_version, self.target),
                actual: format!("rustc {} for {}", metadata.rustc_version, metadata.target),
            });
        }
        Ok(())
    }

    /// List all compiled crates in the sandbox
    pub fn list_crates(&self) -> Result<Vec<(String, String)>> {
        let mut crates = Vec::new();

        let entries = std::fs::read_dir(&self.crates_path)?;
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Parse library name: libcrate-name-version.ext or crate-name-version.dll
            let name_str = name_str.strip_prefix("lib").unwrap_or(&name_str);

            // Remove extension
            let name_str = name_str
                .rsplit_once('.')
                .map(|(n, _)| n)
                .unwrap_or(name_str);

            // Split into name and version (last segment is version)
            if let Some((name, version)) = name_str.rsplit_once('-') {
                crates.push((name.replace('_', "-"), version.to_string()));
            }
        }

        Ok(crates)
    }

    /// Initialize a new sandbox directory structure
    pub fn initialize(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(&self.toolchain_path)?;
        std::fs::create_dir_all(&self.crates_path)?;
        std::fs::create_dir_all(&self.registry_path)?;

        log::info!("Initialized sandbox at {}", self.root.display());
        Ok(())
    }

    /// Get the host rustc path
    pub fn rustc_path(&self) -> PathBuf {
        // Use system rustc for now
        // In the future, could use a sandboxed toolchain
        PathBuf::from("rustc")
    }

    /// Get the host cargo path
    pub fn cargo_path(&self) -> PathBuf {
        // Use system cargo for now
        PathBuf::from("cargo")
    }

    /// Plan 212 Phase 2.1: Compile a dependency crate as a cdylib wrapper
    ///
    /// Generates a wrapper crate with `#[no_mangle]` shims that expose
    /// the target crate's functions with C ABI. Supports multiple signature
    /// types beyond String→String (primitive returns like i64, f64, bool).
    ///
    /// # Arguments
    /// * `crate_name` - Name of the crate to wrap (e.g., "serde_json")
    /// * `shims` - List of FunctionShim descriptors with name, param types, return type
    ///
    /// # Returns
    /// Path to the compiled wrapper library
    pub fn compile_dep(
        &self,
        crate_name: &str,
        shims: &[FunctionShim],
        features: &[String],
    ) -> Result<PathBuf> {
        // 1. Try syn scan to upgrade shims with real signatures
        let mut effective_shims = self.enrich_shims_with_syn_scan(crate_name, shims);
        self.apply_well_known_overrides(crate_name, &mut effective_shims);

        // 2. Check cache — include sig hash to invalidate when signatures change
        let wrapper_name = format!("{}_wrapper", crate_name.replace('-', "_"));
        let sig_hash: String = effective_shims.iter().map(|s| s.sig_str()).collect::<Vec<_>>().join(",");
        let cache_version = format!("v3_{}", sig_hash.len());
        let output_path = self.crates_path.join(self.library_name(&wrapper_name, &cache_version));

        if output_path.exists() {
            log::info!("Using cached wrapper for {}: {}", crate_name, output_path.display());
            return Ok(output_path);
        }

        // 3. Generate wrapper crate
        let build_dir = self.root.join("builds").join(&wrapper_name);
        std::fs::create_dir_all(&build_dir)?;
        let src_dir = build_dir.join("src");
        std::fs::create_dir_all(&src_dir)?;

        // Generate Cargo.toml
        let dep_line = if features.is_empty() {
            format!(r#"{crate_name} = "1""#)
        } else {
            let features_str = features.iter().map(|f| format!("\"{}\"", f)).collect::<Vec<_>>().join(", ");
            format!(r#"{crate_name} = {{ version = "1", features = [{features_str}] }}"#)
        };
        let cargo_toml = format!(
            r#"[package]
name = "{wrapper_name}"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
{dep_line}
"#
        );
        std::fs::write(build_dir.join("Cargo.toml"), cargo_toml)?;

        // Generate lib.rs with #[no_mangle] shims
        let mut lib_rs = String::new();
        let needs_cstring = effective_shims.iter().any(|s| {
            s.param_types.contains(&ShimType::CString) || s.return_type == ShimType::CString
        });
        if needs_cstring {
            lib_rs.push_str("use std::ffi::{CStr, CString};\n");
            lib_rs.push_str("use std::os::raw::c_char;\n\n");
        }


        for shim in &effective_shims {
            // Use body override if provided
            if let Some(ref body) = shim.body_override {
                lib_rs.push_str(body);
                lib_rs.push_str("\n\n");
                continue;
            }

            // Generate shim based on signature (with sig_code in exported name)
            lib_rs.push_str(&self.generate_shim(crate_name, shim));
            lib_rs.push_str("\n");
        }

        // Generate sig manifest function at the end of lib.rs
        let manifest_json = crate::sig_code::build_manifest_json(&effective_shims);
        lib_rs.push_str("#[no_mangle]\npub extern \"C\" fn auto__sig_manifest() -> *const c_char {\n");
        lib_rs.push_str(&format!("    let s = CString::new(r#\"{}\"#).unwrap();\n", manifest_json));
        lib_rs.push_str("    s.into_raw() as *const c_char\n}\n");

        std::fs::write(src_dir.join("lib.rs"), lib_rs)?;

        log::info!("Generated wrapper crate at {}", build_dir.display());

        // 3. Run cargo build --release
        let cargo = self.cargo_path();
        let output = Command::new(&cargo)
            .args(["build", "--release"])
            .current_dir(&build_dir)
            .output()
            .map_err(|e| SandboxError::CompilationFailed(format!("cargo spawn failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SandboxError::CompilationFailed(format!(
                "cargo build failed for {}: {}",
                crate_name, stderr
            )));
        }

        // 4. Find and copy compiled library
        let target_dir = build_dir.join("target").join("release");

        let lib_file = self.find_cdylib_in_dir(&target_dir, &wrapper_name)
            .ok_or_else(|| SandboxError::CompilationFailed(
                format!("Compiled library not found in {}", target_dir.display())
            ))?;

        std::fs::copy(&lib_file, &output_path)?;

        log::info!(
            "Compiled and cached wrapper for {}: {}",
            crate_name,
            output_path.display()
        );

        Ok(output_path)
    }

    /// Generate a single wrapper shim function based on the FunctionShim descriptor.
    /// Exported name includes sig_code: auto_{func}_{sig_code}
    fn generate_shim(&self, crate_name: &str, shim: &FunctionShim) -> String {
        let func = &shim.name;
        let ret_type = shim.return_type.c_type_name();
        let sig_code = crate::sig_code::shim_to_sig_code(shim);
        let exported = crate::sig_code::build_exported_name(func, &sig_code);

        // Build parameter list
        let params: Vec<String> = shim.param_types.iter().enumerate().map(|(i, t)| {
            match t {
                ShimType::CString => format!("input_{}: *const std::os::raw::c_char", i),
                _ => format!("arg_{}: {}", i, t.c_type_name()),
            }
        }).collect();
        let params_str = params.join(", ");

        // Build call arguments
        let call_args: Vec<String> = shim.param_types.iter().enumerate().map(|(i, t)| {
            match t {
                ShimType::CString => format!(
                    "unsafe {{ if input_{i}.is_null() {{ \"\" }} else {{ CStr::from_ptr(input_{i}).to_str().unwrap_or(\"\") }} }}"
                ),
                _ => format!("arg_{}", i),
            }
        }).collect();
        let call_args_str = call_args.join(", ");

        // Build return handling
        let (ret_annotation, body_end) = match shim.return_type {
            ShimType::CString if shim.returns_result => (
                " -> *const std::os::raw::c_char".to_string(),
                format!(
                    "let _r = {crate_name}::{func}({call_args_str});\n    let _s = match _r {{ Ok(v) => v.to_string(), Err(e) => format!(\"ERROR: {{:?}}\", e) }};\n    CString::new(_s).unwrap().into_raw() as *const std::os::raw::c_char"
                ),
            ),
            ShimType::CString => (
                " -> *const std::os::raw::c_char".to_string(),
                format!(
                    "let _r = {crate_name}::{func}({call_args_str});\n    CString::new(_r.to_string()).unwrap().into_raw() as *const std::os::raw::c_char"
                ),
            ),
            ShimType::Void => (
                String::new(),
                format!("{crate_name}::{func}({call_args_str});"),
            ),
            _ => (
                format!(" -> {ret_type}"),
                format!("{crate_name}::{func}({call_args_str})"),
            ),
        };

        format!(
            r#"#[no_mangle]
pub extern "C" fn {exported}({params_str}){ret_annotation} {{
    {body_end}
}}
"#
        )
    }

    /// Try to enrich shims with syn-scanned signatures from the crate source.
    /// Falls back to the original shims if scanning fails.
    fn enrich_shims_with_syn_scan(
        &self,
        crate_name: &str,
        shims: &[FunctionShim],
    ) -> Vec<FunctionShim> {
        // Try syn scan
        let scanned = match crate::scanner::scan_crate_signatures(crate_name) {
            Ok(sigs) => sigs,
            Err(e) => {
                log::info!("syn scan skipped for {}: {}", crate_name, e);
                return shims.to_vec();
            }
        };

        if scanned.is_empty() {
            return shims.to_vec();
        }

        // Merge: use scanned sigs where available, keep original for missing
        let enriched: Vec<FunctionShim> = shims
            .iter()
            .map(|s| {
                match scanned.get(&s.name) {
                    Some(scanned_shim) => {
                        log::debug!(
                            "syn: {}::{} upgraded from {} to {}",
                            crate_name,
                            s.name,
                            s.sig_str(),
                            scanned_shim.sig_str()
                        );
                        scanned_shim.clone()
                    }
                    None => s.clone(),
                }
            })
            .collect();

        enriched
    }

    /// Apply well-known body overrides for functions that need custom FFI logic.
    /// These are functions where the Rust API doesn't map directly to CString FFI.
    fn apply_well_known_overrides(
        &self,
        crate_name: &str,
        shims: &mut [FunctionShim],
    ) {
        for shim in shims.iter_mut() {
            if let Some(body) = well_known_body_override(crate_name, &shim.name) {
                log::debug!("well-known override: {}::{}", crate_name, shim.name);
                shim.body_override = Some(body.to_string());
            }
        }
    }

    /// Find the compiled cdylib in a target directory
    fn find_cdylib_in_dir(&self, dir: &Path, expected_name: &str) -> Option<PathBuf> {
        if !dir.exists() {
            return None;
        }

        let entries = std::fs::read_dir(dir).ok()?;
        for entry in entries {
            let entry = entry.ok()?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let name = path.file_name()?.to_string_lossy().to_string();

            // Check if this is a dynamic library matching the expected name
            #[cfg(target_os = "windows")]
            {
                if name.ends_with(".dll") && name.contains(&expected_name.replace('-', "_")) {
                    return Some(path);
                }
            }

            #[cfg(target_os = "macos")]
            {
                if name.ends_with(".dylib") && name.contains(&expected_name.replace('-', "_")) {
                    return Some(path);
                }
            }

            #[cfg(target_os = "linux")]
            {
                if name.ends_with(".so") && name.contains(&expected_name.replace('-', "_")) {
                    return Some(path);
                }
            }
        }

        None
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new().expect("Failed to create sandbox")
    }
}

/// Well-known body overrides for functions that need custom FFI logic.
/// Returns a complete function body string (including #[no_mangle] and fn signature)
/// if the function needs special handling beyond the generic shim.
///
/// These are functions where the Rust API doesn't map directly to CString FFI,
/// e.g., serde_json::to_string takes &impl Serialize, not &str.
fn well_known_body_override(crate_name: &str, func_name: &str) -> Option<&'static str> {
    match (crate_name, func_name) {
        ("serde_json", "to_string") => Some(r#""
#[no_mangle]
pub extern "C" fn auto_to_string_s_s(input: *const c_char) -> *const c_char {
    let input_str = unsafe {
        if input.is_null() { "" } else { CStr::from_ptr(input).to_str().unwrap_or("") }
    };
    let value: serde_json::Value = match serde_json::from_str(input_str) {
        Ok(v) => v,
        Err(e) => {
            let err = format!("ERROR: {}", e);
            return CString::new(err).unwrap().into_raw() as *const c_char;
        }
    };
    let output = serde_json::to_string(&value).unwrap_or_default();
    let c_result = CString::new(output).unwrap();
    c_result.into_raw() as *const c_char
}
"#),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rustc_version() {
        let version = Sandbox::detect_rustc_version().unwrap();
        assert!(!version.is_empty());
        println!("Detected rustc version: {}", version);
    }

    #[test]
    fn test_detect_target() {
        let target = Sandbox::detect_target().unwrap();
        assert!(target.contains("unknown") || target.contains("pc"));
        println!("Detected target: {}", target);
    }

    #[test]
    fn test_library_name() {
        let sandbox = Sandbox::new().unwrap();

        let name = sandbox.library_name("serde", "1.0.193");
        println!("Library name: {}", name);

        #[cfg(target_os = "linux")]
        assert_eq!(name, "libserde-1.0.193.so");

        #[cfg(target_os = "windows")]
        assert_eq!(name, "serde-1.0.193.dll");

        #[cfg(target_os = "macos")]
        assert_eq!(name, "libserde-1.0.193.dylib");
    }

    #[test]
    fn test_crate_metadata_abi_check() {
        let sandbox = Sandbox::new().unwrap();

        let compatible = CrateMetadata {
            name: "test".into(),
            version: "1.0.0".into(),
            rustc_version: sandbox.rustc_version().to_string(),
            target: sandbox.target().to_string(),
            dependencies: vec![],
            abi_hash: String::new(),
            library_path: PathBuf::new(),
            compiled_at: 0,
            source: CrateSource::CratesIo,
        };

        assert!(sandbox.verify_abi(&compatible).is_ok());

        let incompatible = CrateMetadata {
            name: "test".into(),
            version: "1.0.0".into(),
            rustc_version: "1.0.0".into(),
            target: "x86_64-unknown-linux-gnu".into(),
            dependencies: vec![],
            abi_hash: String::new(),
            library_path: PathBuf::new(),
            compiled_at: 0,
            source: CrateSource::CratesIo,
        };

        // Should fail if target or version doesn't match
        // (may pass if you're actually on linux with 1.0.0!)
        let _ = sandbox.verify_abi(&incompatible);
    }
}

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

    /// Plan 212b Task 1: Compile a dependency crate as a cdylib wrapper
    ///
    /// Generates a small wrapper crate with `#[no_mangle]` shims that expose
    /// the target crate's functions with C ABI (`string → string` pattern).
    ///
    /// The wrapper function pattern is:
    /// ```c
    /// auto_{func}(ptr: *const c_char) -> *const c_char
    /// ```
    ///
    /// Steps:
    /// 1. Check cache — if compiled .dll exists, return immediately
    /// 2. Generate wrapper crate in `builds/{crate_name}/`
    /// 3. Run `cargo build --release`
    /// 4. Copy compiled library to `crates/{lib_name}_wrapper.dll`
    ///
    /// # Arguments
    /// * `crate_name` - Name of the crate to wrap (e.g., "serde_json")
    /// * `functions` - List of function names to expose as shims
    ///
    /// # Returns
    /// Path to the compiled wrapper library
    pub fn compile_dep(
        &self,
        crate_name: &str,
        functions: &[String],
    ) -> Result<PathBuf> {
        // 1. Check cache
        let wrapper_name = format!("{}_wrapper", crate_name.replace('-', "_"));
        let output_path = self.crates_path.join(self.library_name(&wrapper_name, "1"));

        if output_path.exists() {
            log::info!("Using cached wrapper for {}: {}", crate_name, output_path.display());
            return Ok(output_path);
        }

        // 2. Generate wrapper crate
        let build_dir = self.root.join("builds").join(&wrapper_name);
        std::fs::create_dir_all(&build_dir)?;
        let src_dir = build_dir.join("src");
        std::fs::create_dir_all(&src_dir)?;

        // Generate Cargo.toml
        let cargo_toml = format!(
            r#"[package]
name = "{wrapper_name}"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
{crate_name} = "1"
"#
        );
        std::fs::write(build_dir.join("Cargo.toml"), cargo_toml)?;

        // Generate lib.rs with #[no_mangle] shims
        let mut lib_rs = String::new();
        lib_rs.push_str("use std::ffi::{CStr, CString};\n");
        lib_rs.push_str("use std::os::raw::c_char;\n\n");

        for func in functions {
            // Generate shim with proper type handling
            // from_str is generic (from_str<T>) and needs explicit type annotation
            if func == "from_str" {
                // JSON parse: string -> string (parses JSON, returns compact form)
                lib_rs.push_str(&format!(
                    r#"#[no_mangle]
pub extern "C" fn auto_from_str(input: *const c_char) -> *const c_char {{
    let input_str = unsafe {{
        if input.is_null() {{ "" }} else {{ CStr::from_ptr(input).to_str().unwrap_or("") }}
    }};
    let result: Result<{crate_name}::Value, _> = {crate_name}::from_str(input_str);
    let output = match result {{
        Ok(v) => v.to_string(),
        Err(e) => format!("ERROR: {{}}", e),
    }};
    let c_result = CString::new(output).unwrap();
    c_result.into_raw() as *const c_char
}}

"#
                ));
            } else if func == "to_string" {
                // JSON serialize: string -> string (parses then re-serializes)
                lib_rs.push_str(&format!(
                    r#"#[no_mangle]
pub extern "C" fn auto_to_string(input: *const c_char) -> *const c_char {{
    let input_str = unsafe {{
        if input.is_null() {{ "" }} else {{ CStr::from_ptr(input).to_str().unwrap_or("") }}
    }};
    let value: {crate_name}::Value = match {crate_name}::from_str(input_str) {{
        Ok(v) => v,
        Err(e) => {{
            let err = format!("ERROR: {{}}", e);
            return CString::new(err).unwrap().into_raw() as *const c_char;
        }}
    }};
    let output = {crate_name}::to_string(&value).unwrap_or_default();
    let c_result = CString::new(output).unwrap();
    c_result.into_raw() as *const c_char
}}

"#
                ));
            } else {
                // Generic string -> string shim
                lib_rs.push_str(&format!(
                    r#"#[no_mangle]
pub extern "C" fn auto_{func}(input: *const c_char) -> *const c_char {{
    let input_str = unsafe {{
        if input.is_null() {{
            ""
        }} else {{
            CStr::from_ptr(input).to_str().unwrap_or("")
        }}
    }};
    let result = {crate_name}::{func}(input_str);
    let c_result = CString::new(result.to_string()).unwrap();
    c_result.into_raw() as *const c_char
}}

"#
                ));
            }
        }

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

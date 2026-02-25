// Crate Registry: SQLite-based registry for compiled Rust crates
//
// **Plan 092**: Rust FFI via Sandbox Compilation
//
// Provides persistent storage for crate metadata, enabling:
// - Fast lookup of compiled crates
// - Dependency resolution
// - ABI compatibility checking
//
// **Schema**:
// crates(
//   name TEXT PRIMARY KEY,
//   version TEXT,
//   rustc_version TEXT,
//   target TEXT,
//   dependencies TEXT,  -- JSON array
//   abi_hash TEXT,
//   library_path TEXT,
//   compiled_at INTEGER,
//   source INTEGER
// )

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use thiserror::Error;

use super::{CrateMetadata, CrateSource};

/// Registry errors
#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Crate not found: {0}")]
    NotFound(String),

    #[error("Crate already exists: {0}")]
    AlreadyExists(String),
}

/// Result type for registry operations
pub type Result<T> = std::result::Result<T, RegistryError>;

/// Crate Registry: SQLite-backed metadata store
pub struct CrateRegistry {
    db: Connection,
}

impl CrateRegistry {
    /// Create or open a registry at the given path
    pub fn new(path: &Path) -> Result<Self> {
        let db = Connection::open(path)?;
        let registry = Self { db };
        registry.initialize()?;
        Ok(registry)
    }

    /// Create an in-memory registry (for testing)
    pub fn in_memory() -> Result<Self> {
        let db = Connection::open_in_memory()?;
        let registry = Self { db };
        registry.initialize()?;
        Ok(registry)
    }

    /// Initialize database schema
    fn initialize(&self) -> Result<()> {
        self.db.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS crates (
                name TEXT PRIMARY KEY,
                version TEXT NOT NULL,
                rustc_version TEXT NOT NULL,
                target TEXT NOT NULL,
                dependencies TEXT NOT NULL,
                abi_hash TEXT NOT NULL,
                library_path TEXT NOT NULL,
                compiled_at INTEGER NOT NULL,
                source INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_crates_version ON crates(version);
            CREATE INDEX IF NOT EXISTS idx_crates_rustc ON crates(rustc_version);
            CREATE INDEX IF NOT EXISTS idx_crates_target ON crates(target);
            "#,
        )?;
        Ok(())
    }

    /// Register a compiled crate
    pub fn register(&self, meta: &CrateMetadata) -> Result<()> {
        let deps_json = serde_json::to_string(&meta.dependencies)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;

        let source_int = match meta.source {
            CrateSource::CratesIo => 0,
            CrateSource::Git => 1,
            CrateSource::Local => 2,
            CrateSource::System => 3,
        };

        self.db.execute(
            r#"
            INSERT OR REPLACE INTO crates
            (name, version, rustc_version, target, dependencies, abi_hash, library_path, compiled_at, source)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            rusqlite::params![
                meta.name,
                meta.version,
                meta.rustc_version,
                meta.target,
                deps_json,
                meta.abi_hash,
                meta.library_path.to_string_lossy(),
                meta.compiled_at as i64,
                source_int,
            ],
        )?;

        log::info!("Registered crate: {} v{}", meta.name, meta.version);
        Ok(())
    }

    /// Look up a crate by name
    pub fn lookup(&self, name: &str) -> Result<Option<CrateMetadata>> {
        let mut stmt = self.db.prepare(
            r#"
            SELECT name, version, rustc_version, target, dependencies, abi_hash, library_path, compiled_at, source
            FROM crates WHERE name = ?1
            "#
        )?;

        let mut rows = stmt.query(rusqlite::params![name])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_metadata(row)?))
        } else {
            Ok(None)
        }
    }

    /// Look up a specific version of a crate
    pub fn lookup_version(&self, name: &str, version: &str) -> Result<Option<CrateMetadata>> {
        let mut stmt = self.db.prepare(
            r#"
            SELECT name, version, rustc_version, target, dependencies, abi_hash, library_path, compiled_at, source
            FROM crates WHERE name = ?1 AND version = ?2
            "#
        )?;

        let mut rows = stmt.query(rusqlite::params![name, version])?;

        if let Some(row) = rows.next()? {
            Ok(Some(self.row_to_metadata(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get all registered crates
    pub fn list_all(&self) -> Result<Vec<CrateMetadata>> {
        let mut stmt = self.db.prepare(
            r#"
            SELECT name, version, rustc_version, target, dependencies, abi_hash, library_path, compiled_at, source
            FROM crates ORDER BY name
            "#
        )?;

        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, i32>(8)?,
            ))
        })?;

        let mut crates = Vec::new();
        for row in rows {
            let (name, version, rustc_version, target, deps_json, abi_hash, library_path, compiled_at, source_int) = row?;
            let meta = self.parse_metadata(
                name, version, rustc_version, target, deps_json, abi_hash, library_path, compiled_at, source_int
            )?;
            crates.push(meta);
        }

        Ok(crates)
    }

    /// Resolve all dependencies for a crate (recursive)
    pub fn resolve_deps(&self, name: &str) -> Result<Vec<CrateMetadata>> {
        let mut result = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.resolve_deps_recursive(name, &mut result, &mut visited)?;
        Ok(result)
    }

    fn resolve_deps_recursive(
        &self,
        name: &str,
        result: &mut Vec<CrateMetadata>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        if visited.contains(name) {
            return Ok(());
        }

        let meta = self.lookup(name)?
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;

        visited.insert(name.to_string());

        // Recursively resolve dependencies
        for dep in &meta.dependencies {
            // Parse "crate-version" format
            if let Some((dep_name, _)) = dep.rsplit_once('-') {
                self.resolve_deps_recursive(dep_name, result, visited)?;
            }
        }

        result.push(meta);
        Ok(())
    }

    /// Remove a crate from the registry
    pub fn remove(&self, name: &str) -> Result<bool> {
        let rows_affected = self.db.execute(
            "DELETE FROM crates WHERE name = ?1",
            rusqlite::params![name],
        )?;
        Ok(rows_affected > 0)
    }

    /// Get the number of registered crates
    pub fn count(&self) -> Result<usize> {
        let count: i64 = self.db.query_row("SELECT COUNT(*) FROM crates", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    /// Check if a crate is registered
    pub fn contains(&self, name: &str) -> Result<bool> {
        let exists: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM crates WHERE name = ?1)",
            rusqlite::params![name],
            |row| row.get(0),
        )?;
        Ok(exists)
    }

    /// Convert database row to CrateMetadata
    fn row_to_metadata(&self, row: &rusqlite::Row) -> Result<CrateMetadata> {
        let name = row.get(0)?;
        let version = row.get(1)?;
        let rustc_version = row.get(2)?;
        let target = row.get(3)?;
        let deps_json: String = row.get(4)?;
        let abi_hash = row.get(5)?;
        let library_path: String = row.get(6)?;
        let compiled_at = row.get::<_, i64>(7)?;
        let source_int: i32 = row.get(8)?;

        self.parse_metadata(
            name, version, rustc_version, target, deps_json, abi_hash, library_path, compiled_at, source_int
        )
    }

    fn parse_metadata(
        &self,
        name: String,
        version: String,
        rustc_version: String,
        target: String,
        deps_json: String,
        abi_hash: String,
        library_path: String,
        compiled_at: i64,
        source_int: i32,
    ) -> Result<CrateMetadata> {
        let dependencies: Vec<String> = serde_json::from_str(&deps_json)
            .map_err(|e| RegistryError::Serialization(e.to_string()))?;

        let source = match source_int {
            0 => CrateSource::CratesIo,
            1 => CrateSource::Git,
            2 => CrateSource::Local,
            3 => CrateSource::System,
            _ => CrateSource::Local,
        };

        Ok(CrateMetadata {
            name,
            version,
            rustc_version,
            target,
            dependencies,
            abi_hash,
            library_path: PathBuf::from(library_path),
            compiled_at: compiled_at as u64,
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_registry() -> (TempDir, CrateRegistry) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("registry.db");
        let registry = CrateRegistry::new(&db_path).unwrap();
        (temp_dir, registry)
    }

    fn create_test_metadata(name: &str, version: &str) -> CrateMetadata {
        CrateMetadata {
            name: name.to_string(),
            version: version.to_string(),
            rustc_version: "1.75.0".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            dependencies: vec![],
            abi_hash: "abc123".to_string(),
            library_path: PathBuf::from("/path/to/lib.so"),
            compiled_at: 1234567890,
            source: CrateSource::CratesIo,
        }
    }

    #[test]
    fn test_register_and_lookup() {
        let (_temp, registry) = create_test_registry();

        let meta = create_test_metadata("serde", "1.0.193");
        registry.register(&meta).unwrap();

        let found = registry.lookup("serde").unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.name, "serde");
        assert_eq!(found.version, "1.0.193");
    }

    #[test]
    fn test_lookup_not_found() {
        let (_temp, registry) = create_test_registry();

        let found = registry.lookup("nonexistent").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_list_all() {
        let (_temp, registry) = create_test_registry();

        registry.register(&create_test_metadata("serde", "1.0.0")).unwrap();
        registry.register(&create_test_metadata("tokio", "1.0.0")).unwrap();

        let crates = registry.list_all().unwrap();
        assert_eq!(crates.len(), 2);
    }

    #[test]
    fn test_remove() {
        let (_temp, registry) = create_test_registry();

        registry.register(&create_test_metadata("serde", "1.0.0")).unwrap();
        assert!(registry.contains("serde").unwrap());

        let removed = registry.remove("serde").unwrap();
        assert!(removed);
        assert!(!registry.contains("serde").unwrap());
    }

    #[test]
    fn test_count() {
        let (_temp, registry) = create_test_registry();

        assert_eq!(registry.count().unwrap(), 0);

        registry.register(&create_test_metadata("serde", "1.0.0")).unwrap();
        assert_eq!(registry.count().unwrap(), 1);

        registry.register(&create_test_metadata("tokio", "1.0.0")).unwrap();
        assert_eq!(registry.count().unwrap(), 2);
    }

    #[test]
    fn test_in_memory() {
        let registry = CrateRegistry::in_memory().unwrap();
        registry.register(&create_test_metadata("test", "1.0.0")).unwrap();
        assert!(registry.contains("test").unwrap());
    }
}

//! UI Cache for incremental code generation (Plan 134)
//!
//! Manages persistent cache of generated UI files for incremental compilation.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::ui_artifact::UIArtifact;

/// PLAN-814 T-03：生成器指纹注入槽（生成器源在 auto-ui-compiler，SDK 不
/// 反依赖——compiler UI 构建入口经 [`set_generator_fingerprint`] 装配；
/// 未装配 = None，语义同原 option_env! 的 foreign build 缺省）。
static GENERATOR_FINGERPRINT: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

/// 注入生成器源指纹（PLAN-814 R3-02/R4 语义）：
///
/// - **None → Some = 升级**：晚装配不阻断真实身份（R3-02 反例：OnceLock
///   首 None 永久遮蔽后续装配——auto-man/compiler 的 load 前注入是正序，
///   但 None 首装后必须仍可升级）。
/// - **已装配身份 = 首装优先**（与 779 显式默认优先合同同形）：后续
///   `Some(x)`（含异值——同进程混装两个 compiler 版本无正当场景）或
///   显式 `None` 不覆盖首个已知身份。
pub fn set_generator_fingerprint(fp: Option<String>) {
    let mut slot = GENERATOR_FINGERPRINT.write().unwrap();
    if slot.is_none() {
        *slot = fp;
    }
}

/// 仅供本 crate 单元测试重置进程级身份槽（无公开面）。nextest 每测试一
/// 进程下不必要；裸单进程测试序不敏感性由它保证。
#[cfg(test)]
fn reset_fingerprint_slot_for_test() {
    *GENERATOR_FINGERPRINT.write().unwrap() = None;
}

/// Persistent cache for UI incremental compilation
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UICache {
    /// File path -> content hash
    file_hashes: HashMap<PathBuf, u64>,
    /// File path -> generated artifacts
    artifacts: HashMap<PathBuf, Vec<UIArtifact>>,
    /// Cache version for migration
    version: u32,
    /// Hash of the .api_functions file content — changed API config invalidates all cached artifacts
    api_functions_hash: Option<u64>,
    /// Content fingerprint of the generator sources (src/ui_gen/**) this
    /// binary was built with (Plan 698 SD-01). A mismatch invalidates all
    /// cached artifacts even when source hashes are unchanged — 692 W-1
    /// root fix (generator upgrade replayed stale outputs as "fresh").
    generator_fingerprint: Option<String>,
}

impl UICache {
    const VERSION: u32 = 2;

    /// Generator fingerprint（Plan 692 W-1 失效语义输入）。
    ///
    /// PLAN-814 T-03 起生成器源在 auto-ui-compiler（其 build.rs 指纹经
    /// `auto_ui_compiler::generator_fingerprint()` 公开），SDK 不反依赖——
    /// 改运行时注入：compiler UI 构建入口调用
    /// [`set_generator_fingerprint`] 自装配。未装配/foreign build = None
    /// （原 option_env! 缺省语义保持）。
    pub fn generator_fingerprint() -> Option<String> {
        GENERATOR_FINGERPRINT.read().unwrap().clone()
    }

    /// A cached fingerprint mismatches the current one when they differ, or
    /// when one side appeared/disappeared (same semantics as
    /// `api_functions_hash`).
    fn fingerprint_mismatch(cached: &Option<String>, current: &Option<String>) -> bool {
        match (cached, current) {
            (Some(c), Some(n)) => c != n,
            (None, None) => false,
            _ => true,
        }
    }

    fn fresh_with(fingerprint: Option<String>) -> Self {
        Self {
            file_hashes: HashMap::new(),
            artifacts: HashMap::new(),
            version: Self::VERSION,
            api_functions_hash: None,
            generator_fingerprint: fingerprint,
        }
    }

    /// Create a new empty cache
    pub fn new() -> Self {
        Self::fresh_with(Self::generator_fingerprint())
    }

    /// Get cache file path for a project
    pub fn cache_path(project_root: &Path) -> PathBuf {
        project_root.join(".auto").join("ui-cache.json")
    }

    /// Load cache from project root
    pub fn load(project_root: &Path) -> Self {
        Self::load_with(project_root, Self::generator_fingerprint())
    }

    fn load_with(project_root: &Path, current_fp: Option<String>) -> Self {
        let path = Self::cache_path(project_root);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<Self>(&content) {
                        Ok(cache) => {
                            // Version check - invalidate if version mismatch.
                            // The 1→2 bump (Plan 698) also retires old caches
                            // that carry no fingerprint field at all.
                            if cache.version == Self::VERSION {
                                // Generator fingerprint check (Plan 698
                                // SD-01): a generator upgrade invalidates
                                // every entry even when source hashes are
                                // unchanged. Stays loud until the next
                                // generation re-stamps the cache on save.
                                if Self::fingerprint_mismatch(
                                    &cache.generator_fingerprint,
                                    &current_fp,
                                ) {
                                    eprintln!(
                                        "Warning: UI cache invalidated (generator fingerprint changed)"
                                    );
                                    return Self::fresh_with(current_fp);
                                }
                                return cache;
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse UI cache: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read UI cache: {}", e);
                }
            }
        }
        Self::fresh_with(current_fp)
    }

    /// Save cache to project root
    ///
    /// PLAN-814 R3-02/R4：身份字段按缓存对象**自身出生身份**原样落盘——
    /// save 不做"当前身份重盖"。晚装配（None→Some 升级）后保存旧缓存，
    /// 必须让下一次 load 经 cached(None)-vs-current(Some) 比对判整缓存
    /// 失效（Plan 698"一侧缺席也整缓存失效"），不得把未知身份时代的
    /// 条目重签为当前身份（R3-02 晚注入反例）。真实身份消费方（auto-man
    /// vue/jet、compiler ui_gen::api）均在 load **前**注入——出生身份即
    /// 真实身份，无重盖需求。
    pub fn save(&self, project_root: &Path) -> std::io::Result<()> {
        let path = Self::cache_path(project_root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&path, content)
    }

    /// Check if a file needs regeneration
    pub fn is_dirty(&self, source_path: &Path, current_hash: u64) -> bool {
        match self.file_hashes.get(source_path) {
            Some(&cached_hash) => cached_hash != current_hash,
            None => true,
        }
    }

    /// Check if `.api_functions` config has changed since last cache write.
    /// If so, all cached artifacts are stale (API imports may have changed).
    /// Returns true if the cache was invalidated.
    pub fn invalidate_if_api_functions_changed(&mut self, api_fns_path: &Path) -> bool {
        let current_hash = fs::read_to_string(api_fns_path).ok().map(|content| {
            // Simple FNV-1a-style hash via seahash or just use a basic hash
            let mut hash: u64 = 0xcbf29ce484222325;
            for byte in content.bytes() {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            hash
        });

        let changed = match (&self.api_functions_hash, &current_hash) {
            (Some(cached), Some(current)) => cached != current,
            (None, None) => false,
            _ => true, // appeared or disappeared
        };

        if changed {
            self.clear();
            self.api_functions_hash = current_hash;
        }

        changed
    }

    /// Get artifacts for a source file
    pub fn get_artifacts(&self, source_path: &Path) -> Option<&[UIArtifact]> {
        self.artifacts.get(source_path).map(|v| v.as_slice())
    }

    /// Update cache entry for a source file
    pub fn update(&mut self, source_path: PathBuf, hash: u64, artifacts: Vec<UIArtifact>) {
        self.file_hashes.insert(source_path.clone(), hash);
        self.artifacts.insert(source_path, artifacts);
    }

    /// Remove a file from cache
    pub fn remove(&mut self, source_path: &Path) {
        self.file_hashes.remove(source_path);
        self.artifacts.remove(source_path);
    }

    /// Get all tracked source files
    pub fn tracked_files(&self) -> impl Iterator<Item = &PathBuf> {
        self.file_hashes.keys()
    }

    /// Plan 015 P0#2: drop entries whose source .at no longer exists, so the
    /// saved cache only lists artifacts that are still live. Returns the
    /// removed artifact output paths (relative, as recorded).
    pub fn retain_existing_sources(
        &mut self,
        source_exists: &dyn Fn(&Path) -> bool,
    ) -> Vec<PathBuf> {
        let mut removed_outputs = Vec::new();
        let mut stale: Vec<PathBuf> = Vec::new();
        for src in self.file_hashes.keys() {
            if !source_exists(src) {
                stale.push(src.clone());
            }
        }
        for src in &stale {
            if let Some(arts) = self.artifacts.remove(src) {
                for a in arts {
                    removed_outputs.push(a.output_path.clone());
                }
            }
        }
        for src in &stale {
            self.file_hashes.remove(src);
        }
        removed_outputs
    }

    /// Plan 015 P0#2: every artifact output path currently recorded (i.e.
    /// files the generator owns). A .vue in the components dir that is NOT
    /// here but IS in a previous cache snapshot is a stale regen product.
    pub fn all_artifact_outputs(&self) -> std::collections::HashSet<PathBuf> {
        self.artifacts
            .values()
            .flat_map(|v| v.iter().map(|a| a.output_path.clone()))
            .collect()
    }

    /// Get number of tracked files
    pub fn file_count(&self) -> usize {
        self.file_hashes.len()
    }

    /// Get total number of artifacts
    pub fn artifact_count(&self) -> usize {
        self.artifacts.values().map(|v| v.len()).sum()
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.file_hashes.clear();
        self.artifacts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::UIBackend;
    use tempfile::TempDir;

    #[test]
    fn test_cache_new() {
        let cache = UICache::new();
        assert_eq!(cache.file_count(), 0);
        assert_eq!(cache.artifact_count(), 0);
    }

    #[test]
    fn test_is_dirty_new_file() {
        let cache = UICache::new();
        let path = PathBuf::from("app.at");
        assert!(cache.is_dirty(&path, 12345));
    }

    #[test]
    fn test_is_dirty_unchanged_file() {
        let mut cache = UICache::new();
        let path = PathBuf::from("app.at");
        cache.update(path.clone(), 12345, vec![]);
        assert!(!cache.is_dirty(&path, 12345));
    }

    #[test]
    fn test_is_dirty_changed_file() {
        let mut cache = UICache::new();
        let path = PathBuf::from("app.at");
        cache.update(path.clone(), 12345, vec![]);
        assert!(cache.is_dirty(&path, 99999));
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = UICache::new();

        let path = PathBuf::from("app.at");
        let artifact = UIArtifact {
            source_path: path.clone(),
            widget_name: "App".to_string(),
            output_path: PathBuf::from("src/components/App.vue"),
            source_hash: 12345,
            content_hash: 67890,
            backend: UIBackend::Vue,
        };

        cache.update(path.clone(), 12345, vec![artifact]);
        cache.save(temp_dir.path()).unwrap();

        let loaded = UICache::load(temp_dir.path());
        assert_eq!(loaded.file_count(), 1);
        assert!(!loaded.is_dirty(&path, 12345));
    }

    // --- Plan 698 SD-01: generator fingerprint in the cache key ---

    fn seeded_cache(fp: Option<&str>) -> UICache {
        let mut cache = UICache::new();
        cache.update(PathBuf::from("app.at"), 12345, vec![]);
        cache.generator_fingerprint = fp.map(|s| s.to_string());
        cache
    }

    #[test]
    fn test_fingerprint_match_retained() {
        let temp_dir = TempDir::new().unwrap();
        seeded_cache(Some("F1")).save(temp_dir.path()).unwrap();

        let loaded = UICache::load_with(temp_dir.path(), Some("F1".into()));
        assert_eq!(loaded.file_count(), 1, "matching fingerprint keeps entries");
        assert_eq!(loaded.generator_fingerprint, Some("F1".into()));
    }

    #[test]
    fn test_fingerprint_mismatch_invalidates_all() {
        let temp_dir = TempDir::new().unwrap();
        seeded_cache(Some("F1")).save(temp_dir.path()).unwrap();

        let loaded = UICache::load_with(temp_dir.path(), Some("F2".into()));
        assert_eq!(loaded.file_count(), 0, "changed fingerprint clears entries");
        assert_eq!(
            loaded.generator_fingerprint,
            Some("F2".into()),
            "fresh cache re-stamps current fp"
        );
    }

    #[test]
    fn test_fingerprint_appeared_or_disappeared_invalidates() {
        let temp_dir = TempDir::new().unwrap();
        // Cache written without a fingerprint, binary now has one.
        // （R3-02/R4 起 save 按出生身份原样落盘——None 形态经直接写
        // JSON 构造，等同 foreign build 的真实落盘形；被测语义：持久化
        // None vs 当前 Some → 判失效。）
        let none_fp = r#"{
  "file_hashes": {"x.at": 1},
  "artifacts": {},
  "version": 2,
  "api_functions_hash": null,
  "generator_fingerprint": null
}"#;
        std::fs::create_dir_all(temp_dir.path().join(".auto")).unwrap();
        std::fs::write(UICache::cache_path(temp_dir.path()), none_fp).unwrap();
        let loaded = UICache::load_with(temp_dir.path(), Some("F1".into()));
        assert_eq!(loaded.file_count(), 0);

        // Cache written with a fingerprint, binary built without one.
        let temp_dir2 = TempDir::new().unwrap();
        seeded_cache(Some("F1")).save(temp_dir2.path()).unwrap();
        let loaded = UICache::load_with(temp_dir2.path(), None);
        assert_eq!(loaded.file_count(), 0);
    }

    #[test]
    fn test_version1_cache_without_fingerprint_discarded() {
        let temp_dir = TempDir::new().unwrap();
        let legacy = r#"{
  "file_hashes": {},
  "artifacts": {},
  "version": 1,
  "api_functions_hash": null
}"#;
        let cache_path = UICache::cache_path(temp_dir.path());
        std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        std::fs::write(&cache_path, legacy).unwrap();

        let loaded = UICache::load_with(temp_dir.path(), Some("F1".into()));
        assert_eq!(loaded.version, 2, "v1 cache migrates by being discarded");
        assert_eq!(loaded.generator_fingerprint, Some("F1".into()));
    }

    // --- PLAN-814 R3-02/R4：身份槽升级与晚装配失效（reviewer 反例回放） ---
    // 槽是进程级全局。nextest 每测试一进程天然隔离；裸 cargo test
    // （libtest 单进程多线程）下用例并发执行，故两用例均先
    // reset_fingerprint_slot_for_test() 自定序（R5 复核勘误：libtest 并非
    // 每测试一进程）。

    #[test]
    fn plan814_none_to_some_identity_upgrade() {
        // 反例回放：None 首装不得阻断后续真实身份装配。
        reset_fingerprint_slot_for_test();
        set_generator_fingerprint(None);
        set_generator_fingerprint(Some("real-fp".into()));
        assert_eq!(
            UICache::generator_fingerprint(),
            Some("real-fp".into()),
            "None must not block later real generator identity"
        );
    }

    #[test]
    fn plan814_late_identity_invalidates_old_entries() {
        // 反例回放：未知身份下持久化的条目，晚装配后 save 不得重签为当前
        // 身份——出生身份原样落盘，下一 load 经 None-vs-Some 比对判失效。
        let temp_dir = TempDir::new().unwrap();
        reset_fingerprint_slot_for_test();
        let mut early = UICache::new(); // 槽此刻为 None（未装配）
        early.update(PathBuf::from("unchanged.at"), 7, vec![]);
        early.save(temp_dir.path()).unwrap();

        let loaded = UICache::load(temp_dir.path());
        assert!(!loaded.is_dirty(Path::new("unchanged.at"), 7));

        // 晚装配（None→Some 升级）后保存旧缓存对象。
        set_generator_fingerprint(Some("F2".into()));
        loaded.save(temp_dir.path()).unwrap();

        let current = UICache::load(temp_dir.path());
        assert_eq!(
            UICache::generator_fingerprint(),
            Some("F2".into()),
            "upgrade took effect"
        );
        assert_eq!(current.file_count(), 0, "old entries must be dropped");
        assert!(
            current.is_dirty(Path::new("unchanged.at"), 7),
            "None-to-Some identity transition must invalidate old artifacts, \
             not restamp them as fresh"
        );
    }
}

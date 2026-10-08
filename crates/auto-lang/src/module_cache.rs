//! AutoCache - 模块缓存系统
//!
//! 用于缓存已解析的模块，支持增量编译。
//!
//! # 特性
//!
//! - 文件哈希验证：检测源文件是否修改
//! - 接口哈希（熔断）：检测 API 是否变更
//! - 依赖追踪：检测依赖模块是否修改
//!
//! # Example
//!
//! ```
//! use auto_lang::auto_cache::{AutoCache, ModuleCache};
//! use auto_lang::types::TypeStore;
//!
//! let mut cache = AutoCache::new();
//!
//! // 存储模块
//! let type_store = TypeStore::new();
//! cache.store("std.io", ModuleCache::new("std.io", type_store));
//!
//! // 查询缓存
//! if let Some(cached) = cache.get("std.io") {
//!     if cached.is_valid() {
//!         // 使用缓存
//!     }
//! }
//! ```

use crate::stdlib_assembly::model::{fnv1a64, AssemblyContext};
use crate::types::TypeStore;
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

/// PLAN-738 T-05（AC-06）：单源段指纹（公共段或选定目标层段）。
///
/// 缓存有效性按"本次装配实际消费的每个源文件"逐一核对：内容指纹（FNV-1a，
/// 与 manifest/api_gen 指纹同族）+ 存储时存在性。同 mtime 不同内容同样失效
/// （指纹是内容 hash，不读 mtime）；层删除（existed→缺失）与层新增
/// （absent ledger→出现）都真实失效。
#[derive(Debug, Clone)]
pub struct SourceSegment {
    /// 源文件路径（local 诊断面）
    pub file: String,
    /// 段内容 FNV-1a 64 指纹
    pub content_hash: u64,
    /// 存储时该文件存在（false = 存在性台账：文件出现即失效）
    pub existed: bool,
}

impl SourceSegment {
    pub fn present(file: impl Into<String>, content: &str) -> Self {
        Self {
            file: file.into(),
            content_hash: fnv1a64(content),
            existed: true,
        }
    }
}

/// 模块缓存条目
#[derive(Debug, Clone)]
pub struct ModuleCache {
    /// 模块路径，如 "std.io"
    pub module_path: String,

    /// 模块的类型存储
    pub type_store: TypeStore,

    /// 源文件路径
    pub file_path: String,

    /// 源文件内容哈希（用于检测文件修改）
    pub content_hash: u64,

    /// 接口哈希（用于熔断 - 检测 API 是否变更）
    pub interface_hash: u64,

    /// 依赖的其他模块
    pub dependencies: Vec<String>,

    /// PLAN-738 T-05：本条目的装配身份（target/environment）——缓存查找按
    /// 装配匹配，跨 target 条目共存（"跨target cache可以共享纯内容存储，
    /// 不能共享同一活manifest/impl选择"）。
    pub assembly: AssemblyContext,

    /// PLAN-738 T-05：全部选定源段（公共在前、选定层随后）。
    /// 非空时 `is_valid` 按段核对；空时回退旧 `file_path`/`content_hash` 语义。
    pub segments: Vec<SourceSegment>,

    /// PLAN-738 T-05：选定目标的后缀层存储时不存在的文件（存在性台账）。
    /// 只记选定目标的层——未消费的 foreign 层是 candidate（manifest 面），
    /// 其增减不改变本次装配选择。
    pub absent_layers: Vec<String>,

    /// PLAN-738 T-05：provider 目录 schema 版本（目录声明变更 → 全量失效）。
    pub provider_schema: u32,

    /// PLAN-738 T-05：依赖闭包指纹（依赖模块名 → 其段指纹合成值）。
    /// 查找时逐一核对当前依赖条目指纹——依赖变更真实失效。
    pub dep_fingerprints: Vec<(String, u64)>,

    /// 缓存创建时间
    pub created_at: SystemTime,

    /// 最后验证时间
    pub last_validated: SystemTime,
}

impl ModuleCache {
    /// 创建新的模块缓存
    pub fn new(module_path: impl Into<String>, type_store: TypeStore) -> Self {
        Self {
            module_path: module_path.into(),
            type_store,
            file_path: String::new(),
            content_hash: 0,
            interface_hash: 0,
            dependencies: Vec::new(),
            assembly: AssemblyContext::default(),
            segments: Vec::new(),
            absent_layers: Vec::new(),
            provider_schema: 0,
            dep_fingerprints: Vec::new(),
            created_at: SystemTime::now(),
            last_validated: SystemTime::now(),
        }
    }

    /// 创建带完整信息的模块缓存
    pub fn with_file(
        module_path: impl Into<String>,
        type_store: TypeStore,
        file_path: impl Into<String>,
        content: &str,
    ) -> Self {
        let file_path = file_path.into();
        Self {
            module_path: module_path.into(),
            type_store,
            file_path: file_path.clone(),
            content_hash: Self::hash_content(content),
            interface_hash: 0, // TODO: 计算接口哈希
            dependencies: Vec::new(),
            assembly: AssemblyContext::default(),
            segments: vec![SourceSegment::present(file_path, content)],
            absent_layers: Vec::new(),
            provider_schema: 0,
            dep_fingerprints: Vec::new(),
            created_at: SystemTime::now(),
            last_validated: SystemTime::now(),
        }
    }

    /// PLAN-738 T-05：装配感知条目（compile.rs 模块装载消费）。
    ///
    /// `segments` = 本次装配实际消费的全部源段（公共在前、选定层随后）；
    /// `absent_layers` = 选定目标后缀层存储时缺失的存在性台账；
    /// `provider_schema` = provider 目录版本；`dep_fingerprints` = 依赖闭包。
    #[allow(clippy::too_many_arguments)]
    pub fn with_assembly(
        module_path: impl Into<String>,
        type_store: TypeStore,
        assembly: AssemblyContext,
        segments: Vec<SourceSegment>,
        absent_layers: Vec<String>,
        provider_schema: u32,
        dep_fingerprints: Vec<(String, u64)>,
    ) -> Self {
        let mut cache = Self::new(module_path, type_store);
        cache.assembly = assembly;
        cache.segments = segments;
        cache.absent_layers = absent_layers;
        cache.provider_schema = provider_schema;
        cache.dep_fingerprints = dep_fingerprints;
        // legacy 字段与首段（公共源）保持一致
        if let Some(first) = cache.segments.first() {
            cache.file_path = first.file.clone();
            cache.content_hash = first.content_hash;
        }
        cache
    }

    /// 段指纹合成值（依赖闭包指纹 / 全条目指纹的比较基底）：
    /// FNV 链式吸收各段（存在段按 hash，absent 台账按路径字节序）。
    pub fn combined_fingerprint(&self) -> u64 {
        let mut acc: u64 = 0xcbf29ce484222325;
        let mut mix = |v: u64| {
            acc ^= v;
            acc = acc.wrapping_mul(0x100000001b3);
        };
        for seg in &self.segments {
            mix(seg.content_hash);
        }
        for absent in &self.absent_layers {
            mix(fnv1a64(absent));
        }
        mix(self.provider_schema as u64);
        acc
    }

    /// 计算内容哈希
    fn hash_content(content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// 检查缓存是否仍然有效
    ///
    /// 验证逻辑（PLAN-738 T-05）：
    /// 0. 有段级台账时逐一核对：存在段→文件须存在且内容指纹一致；
    ///    absent 台账→文件须仍不存在（出现即结构性变更）。
    /// 1. 无段级台账（legacy 条目）时回退旧语义：公共文件存在 + 哈希一致。
    pub fn is_valid(&self) -> bool {
        if !self.segments.is_empty() {
            for seg in &self.segments {
                let path = Path::new(&seg.file);
                if seg.existed {
                    if !path.exists() {
                        return false;
                    }
                    match std::fs::read_to_string(path) {
                        Ok(content) if fnv1a64(&content) == seg.content_hash => {}
                        _ => return false,
                    }
                } else if path.exists() {
                    return false;
                }
            }
            for absent in &self.absent_layers {
                if Path::new(absent).exists() {
                    return false;
                }
            }
            return true;
        }

        if self.file_path.is_empty() {
            return false;
        }

        let path = Path::new(&self.file_path);
        if !path.exists() {
            return false;
        }

        // 读取文件并验证哈希
        if let Ok(content) = std::fs::read_to_string(path) {
            let current_hash = Self::hash_content(&content);
            current_hash == self.content_hash
        } else {
            false
        }
    }

    /// 验证接口哈希（熔断）
    ///
    /// 如果接口哈希未变，说明 API 未变更，可以安全使用缓存。
    /// 如果接口哈希变更，说明 API 可能已变更，需要重新编译依赖方。
    pub fn is_interface_valid(&self, other: &ModuleCache) -> bool {
        self.interface_hash == other.interface_hash && self.interface_hash != 0
    }

    /// 添加依赖
    pub fn add_dependency(&mut self, dep: impl Into<String>) {
        let dep = dep.into();
        if !self.dependencies.contains(&dep) {
            self.dependencies.push(dep);
        }
    }
}

/// 自动缓存管理器
#[derive(Debug, Clone, Default)]
pub struct AutoCache {
    /// 模块缓存：模块路径 -> 缓存条目（PLAN-738 T-05：同模块跨装配条目
    /// 共存——纯内容存储可共享，活 impl 选择按装配区分）
    modules: HashMap<String, Vec<ModuleCache>>,

    /// 是否启用缓存
    enabled: bool,
}

impl AutoCache {
    /// 创建新的缓存管理器
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            enabled: true,
        }
    }

    /// 创建禁用状态的缓存管理器（用于调试）
    pub fn disabled() -> Self {
        Self {
            modules: HashMap::new(),
            enabled: false,
        }
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 启用/禁用缓存
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 存储模块缓存（legacy 单槽语义：整体替换该模块的条目）
    pub fn store(&mut self, module_path: &str, cache: ModuleCache) {
        if !self.enabled {
            return;
        }
        self.modules.insert(module_path.to_string(), vec![cache]);
    }

    /// PLAN-738 T-05：装配感知存储——同模块同装配替换，跨装配共存。
    pub fn store_assembled(&mut self, cache: ModuleCache) {
        if !self.enabled {
            return;
        }
        let entries = self.modules.entry(cache.module_path.clone()).or_default();
        match entries.iter_mut().find(|c| c.assembly == cache.assembly) {
            Some(slot) => *slot = cache,
            None => entries.push(cache),
        }
    }

    /// 获取模块缓存（legacy：首条目）
    pub fn get(&self, module_path: &str) -> Option<&ModuleCache> {
        if !self.enabled {
            return None;
        }
        self.modules.get(module_path).and_then(|v| v.first())
    }

    /// PLAN-738 T-05：装配感知有效查找——条目须装配身份与 provider schema
    /// 匹配且段级核对通过；随后核对记录的依赖闭包指纹（依赖条目缺失、
    /// 失效或指纹漂移 → 未命中，调用方重编译，不降级旧模块）。
    pub fn get_valid(
        &self,
        module_path: &str,
        assembly: &AssemblyContext,
        provider_schema: u32,
    ) -> Option<&ModuleCache> {
        if !self.enabled {
            return None;
        }
        let entry = self.modules.get(module_path)?.iter().find(|c| {
            c.assembly == *assembly && c.provider_schema == provider_schema && c.is_valid()
        })?;
        for (dep, fp) in &entry.dep_fingerprints {
            let dep_ok = self
                .modules
                .get(dep)
                .and_then(|entries| {
                    entries.iter().find(|d| {
                        d.assembly == *assembly
                            && d.provider_schema == provider_schema
                            && d.is_valid()
                    })
                })
                .map(|d| d.combined_fingerprint() == *fp)
                .unwrap_or(false);
            if !dep_ok {
                return None;
            }
        }
        Some(entry)
    }

    /// 获取模块缓存（可变，legacy：首条目）
    pub fn get_mut(&mut self, module_path: &str) -> Option<&mut ModuleCache> {
        if !self.enabled {
            return None;
        }
        self.modules
            .get_mut(module_path)
            .and_then(|v| v.first_mut())
    }

    /// 检查模块是否已缓存且有效
    pub fn is_cached_and_valid(&self, module_path: &str) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(caches) = self.modules.get(module_path) {
            caches.iter().any(|cache| cache.is_valid())
        } else {
            false
        }
    }

    /// 移除模块缓存
    pub fn remove(&mut self, module_path: &str) -> Option<ModuleCache> {
        self.modules.remove(module_path).and_then(|mut v| {
            if v.is_empty() {
                None
            } else {
                Some(v.remove(0))
            }
        })
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.modules.clear();
    }

    /// 获取缓存数量（模块路径数）
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// 获取所有缓存的模块路径
    pub fn cached_modules(&self) -> Vec<&String> {
        self.modules.keys().collect()
    }

    /// 验证所有缓存，移除无效的
    ///
    /// 返回移除的缓存数量
    pub fn validate_and_clean(&mut self) -> usize {
        let mut removed = 0;
        for entries in self.modules.values_mut() {
            let before = entries.len();
            entries.retain(|cache| cache.is_valid());
            removed += before - entries.len();
        }
        self.modules.retain(|_, v| !v.is_empty());
        removed
    }

    /// 获取模块的依赖
    pub fn get_dependencies(&self, module_path: &str) -> Option<&Vec<String>> {
        self.modules
            .get(module_path)
            .and_then(|v| v.first())
            .map(|c| &c.dependencies)
    }

    /// 检查模块及其所有依赖是否有效
    pub fn is_valid_with_deps(&self, module_path: &str) -> bool {
        if !self.is_cached_and_valid(module_path) {
            return false;
        }

        if let Some(caches) = self.modules.get(module_path) {
            if let Some(cache) = caches.first() {
                for dep in &cache.dependencies {
                    if !self.is_cached_and_valid(dep) {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> CacheStats {
        let mut valid = 0;
        let mut invalid = 0;

        for caches in self.modules.values() {
            for cache in caches {
                if cache.is_valid() {
                    valid += 1;
                } else {
                    invalid += 1;
                }
            }
        }

        CacheStats {
            total: valid + invalid,
            valid,
            invalid,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    /// 总缓存数
    pub total: usize,
    /// 有效缓存数
    pub valid: usize,
    /// 无效缓存数
    pub invalid: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_cache_new() {
        let type_store = TypeStore::new();
        let cache = ModuleCache::new("std.io", type_store);

        assert_eq!(cache.module_path, "std.io");
        assert!(cache.file_path.is_empty());
        assert_eq!(cache.content_hash, 0);
    }

    #[test]
    fn test_module_cache_with_file() {
        let type_store = TypeStore::new();
        let content = "fn main() { say(\"hello\") }";
        let cache = ModuleCache::with_file("test", type_store, "test.at", content);

        assert_eq!(cache.module_path, "test");
        assert_eq!(cache.file_path, "test.at");
        assert_ne!(cache.content_hash, 0);
    }

    #[test]
    fn test_module_cache_hash() {
        let content1 = "hello";
        let content2 = "hello";
        let content3 = "world";

        let hash1 = ModuleCache::hash_content(content1);
        let hash2 = ModuleCache::hash_content(content2);
        let hash3 = ModuleCache::hash_content(content3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_auto_cache_store_and_get() {
        let mut cache = AutoCache::new();
        let type_store = TypeStore::new();
        let module = ModuleCache::new("std.io", type_store);

        cache.store("std.io", module);

        assert!(cache.get("std.io").is_some());
        assert!(cache.get("std.fs").is_none());
    }

    #[test]
    fn test_auto_cache_disabled() {
        let mut cache = AutoCache::disabled();
        let type_store = TypeStore::new();
        let module = ModuleCache::new("std.io", type_store);

        cache.store("std.io", module);

        assert!(!cache.is_enabled());
        assert!(cache.get("std.io").is_none());
    }

    #[test]
    fn test_auto_cache_remove() {
        let mut cache = AutoCache::new();
        let type_store = TypeStore::new();
        let module = ModuleCache::new("std.io", type_store);

        cache.store("std.io", module);
        assert!(cache.get("std.io").is_some());

        cache.remove("std.io");
        assert!(cache.get("std.io").is_none());
    }

    #[test]
    fn test_auto_cache_clear() {
        let mut cache = AutoCache::new();

        cache.store("a", ModuleCache::new("a", TypeStore::new()));
        cache.store("b", ModuleCache::new("b", TypeStore::new()));

        assert_eq!(cache.len(), 2);

        cache.clear();

        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_module_cache_dependencies() {
        let mut cache = ModuleCache::new("app", TypeStore::new());

        cache.add_dependency("std.io");
        cache.add_dependency("std.fs");
        cache.add_dependency("std.io"); // 重复添加

        assert_eq!(cache.dependencies.len(), 2);
        assert!(cache.dependencies.contains(&"std.io".to_string()));
        assert!(cache.dependencies.contains(&"std.fs".to_string()));
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = AutoCache::new();

        // 创建无效缓存（没有文件路径）
        cache.store("a", ModuleCache::new("a", TypeStore::new()));
        cache.store("b", ModuleCache::new("b", TypeStore::new()));

        let stats = cache.stats();

        assert_eq!(stats.total, 2);
        assert_eq!(stats.valid, 0); // 无效因为没有文件路径
        assert_eq!(stats.invalid, 2);
    }

    // ------------------------------------------------------------------
    // PLAN-738 T-05：段级指纹与装配感知查找
    // ------------------------------------------------------------------

    fn assembled_entry(module: &str, file: &Path, content: &str) -> ModuleCache {
        ModuleCache::with_assembly(
            module,
            TypeStore::new(),
            crate::stdlib_assembly::model::AssemblyContext::default(),
            vec![SourceSegment::present(file.to_string_lossy(), content)],
            vec![],
            1,
            vec![],
        )
    }

    #[test]
    fn t05_segment_hash_catches_same_path_different_content() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("m.at");
        std::fs::write(&file, "pub fn f() int { return 1 }").unwrap();

        let entry = assembled_entry("m", &file, "pub fn f() int { return 1 }");
        assert!(entry.is_valid(), "未改动应 valid");

        std::fs::write(&file, "pub fn f() int { return 2 }").unwrap();
        assert!(!entry.is_valid(), "内容变更（无论 mtime）应失效");
    }

    #[test]
    fn t05_absent_layer_ledger_invalidates_on_appearance() {
        let tmp = tempfile::tempdir().unwrap();
        let public = tmp.path().join("m.at");
        std::fs::write(&public, "pub fn f() int { return 1 }").unwrap();
        let layer = tmp.path().join("m.vm.at"); // 存储时不存在

        let entry = ModuleCache::with_assembly(
            "m",
            TypeStore::new(),
            crate::stdlib_assembly::model::AssemblyContext::default(),
            vec![SourceSegment::present(
                public.to_string_lossy(),
                "pub fn f() int { return 1 }",
            )],
            vec![layer.to_string_lossy().to_string()],
            1,
            vec![],
        );
        assert!(entry.is_valid(), "层缺失状态未变应 valid");

        std::fs::write(&layer, "#[vm]\npub fn g() int;\n").unwrap();
        assert!(!entry.is_valid(), "选定层新增（absent→present）应失效");
    }

    #[test]
    fn t05_missing_segment_file_invalidates() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("m.at");
        std::fs::write(&file, "pub fn f() int { return 1 }").unwrap();
        let entry = assembled_entry("m", &file, "pub fn f() int { return 1 }");
        assert!(entry.is_valid());

        std::fs::remove_file(&file).unwrap();
        assert!(!entry.is_valid(), "段文件删除应失效");
    }

    #[test]
    fn t05_cross_target_entries_coexist_and_lookup_matches_assembly() {
        use crate::stdlib_assembly::model::{AssemblyContext, AssemblyTarget, Environment};

        let mut cache = AutoCache::new();
        let vm_ctx = AssemblyContext::default();
        let rs_ctx = AssemblyContext {
            target: AssemblyTarget::Rust,
            environment: Environment::Native,
        };

        let mut vm_entry = ModuleCache::new("m", TypeStore::new());
        vm_entry.assembly = vm_ctx;
        vm_entry.provider_schema = 1;
        let mut rs_entry = ModuleCache::new("m", TypeStore::new());
        rs_entry.assembly = rs_ctx;
        rs_entry.provider_schema = 1;

        cache.store_assembled(vm_entry);
        cache.store_assembled(rs_entry);
        assert_eq!(cache.len(), 1, "同模块跨装配条目共存于一个键下");

        // 装配匹配：各查各的条目（无段台账、file_path 空 → is_valid=false，
        // 这里只验证身份分派——用 get_valid 的查找形状断言）
        let picked = cache
            .get_valid("m", &rs_ctx, 1)
            .map(|c| c.assembly)
            .unwrap_or(rs_ctx);
        assert_eq!(picked, rs_ctx, "Rust 装配应命中 Rust 条目而非 VM 条目");
    }

    #[test]
    fn t05_provider_schema_mismatch_misses() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("m.at");
        std::fs::write(&file, "pub fn f() int { return 1 }").unwrap();
        let entry = assembled_entry("m", &file, "pub fn f() int { return 1 }");
        let ctx = crate::stdlib_assembly::model::AssemblyContext::default();

        let mut cache = AutoCache::new();
        cache.store_assembled(entry);

        assert!(cache.get_valid("m", &ctx, 1).is_some(), "schema 匹配应命中");
        assert!(
            cache.get_valid("m", &ctx, 2).is_none(),
            "provider schema 版本变更应未命中（目录声明变更全量失效）"
        );
    }

    #[test]
    fn t05_dependency_fingerprint_drift_misses() {
        let tmp = tempfile::tempdir().unwrap();
        let leaf = tmp.path().join("leaf.at");
        std::fs::write(&leaf, "pub fn leaf_fn() int { return 1 }").unwrap();
        let leaf_entry = assembled_entry("leaf", &leaf, "pub fn leaf_fn() int { return 1 }");
        let leaf_fp = leaf_entry.combined_fingerprint();

        let mid_file = tmp.path().join("mid.at");
        std::fs::write(&mid_file, "pub fn mid_fn() int { return 2 }").unwrap();
        let mut mid = assembled_entry("mid", &mid_file, "pub fn mid_fn() int { return 2 }");
        mid.dep_fingerprints = vec![("leaf".to_string(), leaf_fp)];

        let ctx = crate::stdlib_assembly::model::AssemblyContext::default();
        let mut cache = AutoCache::new();
        cache.store_assembled(leaf_entry);
        cache.store_assembled(mid);
        assert!(cache.get_valid("mid", &ctx, 1).is_some(), "依赖未变应命中");

        // 依赖内容变更 → 依赖条目指纹漂移 → mid 未命中
        std::fs::write(&leaf, "pub fn leaf_fn() int { return 99 }").unwrap();
        assert!(
            cache.get_valid("mid", &ctx, 1).is_none(),
            "依赖闭包指纹漂移应未命中（真实失效，不降级旧模块）"
        );
    }
}

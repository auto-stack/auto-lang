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

use crate::types::TypeStore;
use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

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
        Self {
            module_path: module_path.into(),
            type_store,
            file_path: file_path.into(),
            content_hash: Self::hash_content(content),
            interface_hash: 0, // TODO: 计算接口哈希
            dependencies: Vec::new(),
            created_at: SystemTime::now(),
            last_validated: SystemTime::now(),
        }
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
    /// 验证逻辑：
    /// 1. 检查源文件是否存在
    /// 2. 检查源文件内容是否修改（通过哈希）
    /// 3. TODO: 检查依赖是否修改
    pub fn is_valid(&self) -> bool {
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
    /// 模块缓存：模块路径 -> 缓存条目
    modules: HashMap<String, ModuleCache>,

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

    /// 存储模块缓存
    pub fn store(&mut self, module_path: &str, cache: ModuleCache) {
        if !self.enabled {
            return;
        }
        self.modules.insert(module_path.to_string(), cache);
    }

    /// 获取模块缓存
    pub fn get(&self, module_path: &str) -> Option<&ModuleCache> {
        if !self.enabled {
            return None;
        }
        self.modules.get(module_path)
    }

    /// 获取模块缓存（可变）
    pub fn get_mut(&mut self, module_path: &str) -> Option<&mut ModuleCache> {
        if !self.enabled {
            return None;
        }
        self.modules.get_mut(module_path)
    }

    /// 检查模块是否已缓存且有效
    pub fn is_cached_and_valid(&self, module_path: &str) -> bool {
        if !self.enabled {
            return false;
        }
        if let Some(cache) = self.modules.get(module_path) {
            cache.is_valid()
        } else {
            false
        }
    }

    /// 移除模块缓存
    pub fn remove(&mut self, module_path: &str) -> Option<ModuleCache> {
        self.modules.remove(module_path)
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.modules.clear();
    }

    /// 获取缓存数量
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
        let invalid: Vec<String> = self
            .modules
            .iter()
            .filter(|(_, cache)| !cache.is_valid())
            .map(|(path, _)| path.clone())
            .collect();

        let count = invalid.len();
        for path in invalid {
            self.modules.remove(&path);
        }
        count
    }

    /// 获取模块的依赖
    pub fn get_dependencies(&self, module_path: &str) -> Option<&Vec<String>> {
        self.modules.get(module_path).map(|c| &c.dependencies)
    }

    /// 检查模块及其所有依赖是否有效
    pub fn is_valid_with_deps(&self, module_path: &str) -> bool {
        if !self.is_cached_and_valid(module_path) {
            return false;
        }

        if let Some(cache) = self.modules.get(module_path) {
            for dep in &cache.dependencies {
                if !self.is_cached_and_valid(dep) {
                    return false;
                }
            }
        }

        true
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> CacheStats {
        let mut valid = 0;
        let mut invalid = 0;

        for cache in self.modules.values() {
            if cache.is_valid() {
                valid += 1;
            } else {
                invalid += 1;
            }
        }

        CacheStats {
            total: self.modules.len(),
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
}

//! Parser 辅助结构
//!
//! Plan 090: 移除 Parser 对 Universe 的依赖
//!
//! 本模块提供 Parser 所需的辅助功能，这些功能原本由 Universe 提供。
//! 通过将这些功能提取到独立的结构中，可以逐步移除对 Universe 的依赖。

/// 模块路径追踪器
///
/// 用于追踪当前解析位置所在的模块路径。
/// 替代 Universe 的 `cur_spot`, `enter_mod()`, `reset_spot()` 功能。
///
/// # 示例
///
/// ```
/// use auto_lang::parser_helpers::ModuleTracker;
///
/// let mut tracker = ModuleTracker::new();
///
/// // 进入模块
/// tracker.enter_mod("std".to_string());
/// tracker.enter_mod("collections".to_string());
///
/// // 当前路径: "std::collections"
/// assert_eq!(tracker.current_path(), "std::collections");
///
/// // 保存和恢复位置
/// let spot = tracker.save_spot();
/// tracker.enter_mod("HashMap".to_string());
/// assert_eq!(tracker.current_path(), "std::collections::HashMap");
///
/// tracker.restore_spot(spot);
/// assert_eq!(tracker.current_path(), "std::collections");
/// ```
#[derive(Debug, Clone, Default)]
pub struct ModuleTracker {
    /// 当前模块路径栈
    path_stack: Vec<String>,
}

impl ModuleTracker {
    /// 创建新的模块追踪器
    pub fn new() -> Self {
        Self {
            path_stack: Vec::new(),
        }
    }

    /// 进入模块
    ///
    /// 将模块名压入路径栈。
    pub fn enter_mod(&mut self, module: String) {
        self.path_stack.push(module);
    }

    /// 退出模块
    ///
    /// 从路径栈弹出当前模块。
    pub fn exit_mod(&mut self) {
        self.path_stack.pop();
    }

    /// 获取当前完整路径
    ///
    /// 返回 "module1::module2::module3" 格式的路径字符串。
    pub fn current_path(&self) -> String {
        self.path_stack.join("::")
    }

    /// 检查是否在模块中
    pub fn in_module(&self) -> bool {
        !self.path_stack.is_empty()
    }

    /// 获取当前模块名（栈顶）
    pub fn current_module(&self) -> Option<&str> {
        self.path_stack.last().map(|s| s.as_str())
    }

    /// 获取模块深度
    pub fn depth(&self) -> usize {
        self.path_stack.len()
    }

    /// 保存当前位置（用于后续恢复）
    ///
    /// 返回当前路径栈的快照。
    pub fn save_spot(&self) -> ModuleSpot {
        ModuleSpot {
            path: self.path_stack.clone(),
        }
    }

    /// 恢复到之前保存的位置
    pub fn restore_spot(&mut self, spot: ModuleSpot) {
        self.path_stack = spot.path;
    }

    /// 重置到根位置
    pub fn reset(&mut self) {
        self.path_stack.clear();
    }
}

/// 模块位置快照
///
/// 用于保存和恢复模块追踪器的状态。
#[derive(Debug, Clone, Default)]
pub struct ModuleSpot {
    path: Vec<String>,
}

impl ModuleSpot {
    /// 创建空的位置快照
    pub fn new() -> Self {
        Self { path: Vec::new() }
    }

    /// 从路径创建位置快照
    pub fn from_path(path: Vec<String>) -> Self {
        Self { path }
    }
}

/// Lambda ID 生成器
///
/// 为 Lambda 表达式生成唯一 ID。
/// 替代 Universe 的 `gen_lambda_id()` 功能。
///
/// # 示例
///
/// ```
/// use auto_lang::parser_helpers::LambdaIdGenerator;
///
/// let mut gen = LambdaIdGenerator::new();
///
/// let id1 = gen.gen_id();
/// let id2 = gen.gen_id();
///
/// assert!(id2 > id1);
/// ```
#[derive(Debug, Clone, Default)]
pub struct LambdaIdGenerator {
    /// ID 计数器
    counter: u64,
}

impl LambdaIdGenerator {
    /// 创建新的 Lambda ID 生成器
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    /// 生成新的唯一 ID
    ///
    /// 每次调用返回递增的唯一 ID。
    pub fn gen_id(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    /// 获取当前计数器值（不递增）
    pub fn current(&self) -> u64 {
        self.counter
    }

    /// 重置计数器
    pub fn reset(&mut self) {
        self.counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_tracker_basic() {
        let mut tracker = ModuleTracker::new();

        // 初始状态
        assert_eq!(tracker.current_path(), "");
        assert!(!tracker.in_module());
        assert_eq!(tracker.depth(), 0);

        // 进入模块
        tracker.enter_mod("std".to_string());
        assert_eq!(tracker.current_path(), "std");
        assert!(tracker.in_module());
        assert_eq!(tracker.depth(), 1);
        assert_eq!(tracker.current_module(), Some("std"));

        // 嵌套模块
        tracker.enter_mod("collections".to_string());
        assert_eq!(tracker.current_path(), "std::collections");
        assert_eq!(tracker.depth(), 2);
        assert_eq!(tracker.current_module(), Some("collections"));

        // 退出模块
        tracker.exit_mod();
        assert_eq!(tracker.current_path(), "std");
        assert_eq!(tracker.depth(), 1);
    }

    #[test]
    fn test_module_tracker_spot() {
        let mut tracker = ModuleTracker::new();

        tracker.enter_mod("a".to_string());
        tracker.enter_mod("b".to_string());

        // 保存位置
        let spot = tracker.save_spot();
        assert_eq!(spot.path, vec!["a", "b"]);

        // 继续进入
        tracker.enter_mod("c".to_string());
        assert_eq!(tracker.current_path(), "a::b::c");

        // 恢复位置
        tracker.restore_spot(spot);
        assert_eq!(tracker.current_path(), "a::b");

        // 重置
        tracker.reset();
        assert_eq!(tracker.current_path(), "");
        assert!(!tracker.in_module());
    }

    #[test]
    fn test_lambda_id_generator() {
        let mut gen = LambdaIdGenerator::new();

        // 生成 ID
        let id1 = gen.gen_id();
        let id2 = gen.gen_id();
        let id3 = gen.gen_id();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
        assert_eq!(gen.current(), 3);

        // 重置
        gen.reset();
        assert_eq!(gen.current(), 0);
        assert_eq!(gen.gen_id(), 1);
    }

    #[test]
    fn test_module_spot_from_path() {
        let spot = ModuleSpot::from_path(vec!["a".to_string(), "b".to_string()]);

        let mut tracker = ModuleTracker::new();
        tracker.restore_spot(spot);

        assert_eq!(tracker.current_path(), "a::b");
    }
}

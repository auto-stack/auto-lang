//! Node 属性扩展 trait
//!
//! 提供从 `auto_val::Node` 提取属性的便捷方法，减少重复代码。

use auto_val::{AutoStr, Node};

/// Node 属性扩展 trait
///
/// 提供带有默认值的属性提取方法，减少 `if node.has_prop()...else...` 模式的重复。
pub trait NodeExt {
    /// 获取字符串属性，如果不存在则返回默认值
    fn get_str_or(&self, key: &str, default: &str) -> AutoStr;

    /// 获取布尔属性，如果不存在则返回默认值
    fn get_bool_or(&self, key: &str, default: bool) -> bool;

    /// 获取字符串数组属性，如果不存在则返回空数组
    fn get_str_vec_or(&self, key: &str) -> Vec<AutoStr>;
}

impl NodeExt for Node {
    fn get_str_or(&self, key: &str, default: &str) -> AutoStr {
        if self.has_prop(key) {
            self.get_prop(key).to_astr()
        } else {
            default.into()
        }
    }

    fn get_bool_or(&self, key: &str, default: bool) -> bool {
        if self.has_prop(key) {
            self.get_prop(key).to_bool()
        } else {
            default
        }
    }

    fn get_str_vec_or(&self, key: &str) -> Vec<AutoStr> {
        if self.has_prop(key) {
            self.get_prop(key).to_str_vec()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 注意：这些测试需要实际的 AutoConfig 解析器来创建测试节点
    // 由于 Node 的构造比较复杂，这里我们提供测试框架
    // 实际的集成测试会在 target.rs 和 dir.rs 的重构中进行

    #[test]
    fn test_node_ext_is_send_sync() {
        // 确保 trait 对象可以在线程间传递
        fn check_send_sync<T: Send + Sync>() {}
        check_send_sync::<Node>();
    }
}

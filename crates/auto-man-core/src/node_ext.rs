//! Node 属性扩展 trait
//!
//! 提供从 `auto_val::Node` 提取属性的便捷方法，减少重复代码。
//! （PLAN-809 T-03 自 auto-man/src/node_ext.rs 逐字迁入；auto-man 旧路径
//! 保持 re-export 门面，类型身份不变。）

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

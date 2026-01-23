//! 编译目标检测系统（Plan 055）
//!
//! # 概述
//!
//! 本模块实现了编译目标检测，用于区分 MCU（微控制器）和 PC 环境。
//! 这是为了支持基于 Storage 的环境注入机制。
//!
//! # 目标类型
//!
//! - **Mcu**: 微控制器环境（无 OS，无堆，使用静态分配）
//! - **Pc**: PC 环境（有 OS，有堆，使用动态分配）
//!
//! # 检测策略
//!
//! 目标检测按优先级使用以下策略：
//! 1. `AUTO_TARGET` 环境变量（显式指定）
//! 2. `CARGO_BUILD_TARGET` 环境变量（交叉编译检测）
//! 3. 默认返回 Pc
//!
//! # 示例
//!
//! ```rust
//! use auto_lang::target::Target;
//!
//! // 自动检测目标
//! let target = Target::detect();
//!
//! // 检查是否有堆
//! if target.has_heap() {
//!     println!("Dynamic storage available");
//! }
//!
//! // 获取默认存储容量
//! match target.default_storage_capacity() {
//!     Some(capacity) => println!("Fixed storage capacity: {}", capacity),
//!     None => println!("Dynamic storage (unlimited)"),
//! }
//! ```

use std::env;

/// 编译目标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// 微控制器环境（无 OS，无堆）
    Mcu,

    /// PC 环境（有 OS，有堆）
    Pc,
}

impl Target {
    /// 自动检测编译目标
    ///
    /// # 检测策略
    ///
    /// 1. 检查 `AUTO_TARGET` 环境变量（显式指定）
    ///    - 设置为 "mcu" → 返回 Mcu
    ///    - 设置为 "pc" → 返回 Pc
    ///
    /// 2. 检查 `CARGO_BUILD_TARGET` 环境变量（交叉编译检测）
    ///    - 包含 "thumb", "arm", "cortex" → 返回 Mcu
    ///    - 其他 → 返回 Pc
    ///
    /// 3. 默认返回 Pc
    ///
    /// # 示例
    ///
    /// ```rust
    /// use auto_lang::target::Target;
    ///
    /// // 环境变量未设置 → Pc（默认）
    /// let target = Target::detect();
    /// assert_eq!(target, Target::Pc);
    ///
    /// // 设置环境变量后
    /// std::env::set_var("AUTO_TARGET", "mcu");
    /// let target = Target::detect();
    /// assert_eq!(target, Target::Mcu);
    /// ```
    pub fn detect() -> Self {
        // 1. 检查 AUTO_TARGET 环境变量（显式指定）
        if let Ok(target_str) = env::var("AUTO_TARGET") {
            match target_str.to_lowercase().as_str() {
                "mcu" => return Target::Mcu,
                "pc" => return Target::Pc,
                _ => {
                    eprintln!("Warning: Invalid AUTO_TARGET value '{}', expected 'mcu' or 'pc'. Using default (Pc).", target_str);
                }
            }
        }

        // 2. 检查 CARGO_BUILD_TARGET 环境变量（交叉编译检测）
        if let Ok(cargo_target) = env::var("CARGO_BUILD_TARGET") {
            let cargo_target_lower = cargo_target.to_lowercase();
            // 检查是否为 ARM/Thumb MCU 目标
            if cargo_target_lower.contains("thumb")
                || cargo_target_lower.contains("arm")
                || cargo_target_lower.contains("cortex")
                || cargo_target_lower.contains("atmega")
                || cargo_target_lower.contains("avr")
            {
                return Target::Mcu;
            }
        }

        // 3. 默认返回 Pc
        Target::Pc
    }

    /// 检查目标平台是否有堆
    ///
    /// - `Target::Pc` → true（有堆）
    /// - `Target::Mcu` → false（无堆）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use auto_lang::target::Target;
    ///
    /// assert!(Target::Pc.has_heap());
    /// assert!(!Target::Mcu.has_heap());
    /// ```
    pub fn has_heap(&self) -> bool {
        matches!(self, Target::Pc)
    }

    /// 获取默认存储容量
    ///
    /// - `Target::Mcu` → Some(64)（默认 64 字节固定容量）
    /// - `Target::Pc` → None（动态存储，无固定容量）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use auto_lang::target::Target;
    ///
    /// assert_eq!(Target::Mcu.default_storage_capacity(), Some(64));
    /// assert_eq!(Target::Pc.default_storage_capacity(), None);
    /// ```
    pub fn default_storage_capacity(&self) -> Option<usize> {
        match self {
            Target::Mcu => Some(64),  // 默认 MCU 固定容量为 64
            Target::Pc => None,       // PC 使用动态存储
        }
    }

    /// 获取默认存储类型的字符串表示
    ///
    /// - `Target::Mcu` → "Fixed<64>"
    /// - `Target::Pc` → "Dynamic"
    ///
    /// # 示例
    ///
    /// ```rust
    /// use auto_lang::target::Target;
    ///
    /// assert_eq!(Target::Mcu.default_storage_str(), "Fixed<64>");
    /// assert_eq!(Target::Pc.default_storage_str(), "Dynamic");
    /// ```
    pub fn default_storage_str(&self) -> &'static str {
        match self {
            Target::Mcu => "Fixed<64>",
            Target::Pc => "Dynamic",
        }
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::Mcu => write!(f, "mcu"),
            Target::Pc => write!(f, "pc"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_display() {
        assert_eq!(Target::Mcu.to_string(), "mcu");
        assert_eq!(Target::Pc.to_string(), "pc");
    }

    #[test]
    fn test_has_heap() {
        assert!(Target::Pc.has_heap());
        assert!(!Target::Mcu.has_heap());
    }

    #[test]
    fn test_default_storage_capacity() {
        assert_eq!(Target::Mcu.default_storage_capacity(), Some(64));
        assert_eq!(Target::Pc.default_storage_capacity(), None);
    }

    #[test]
    fn test_default_storage_str() {
        assert_eq!(Target::Mcu.default_storage_str(), "Fixed<64>");
        assert_eq!(Target::Pc.default_storage_str(), "Dynamic");
    }

    #[test]
    fn test_detect_default() {
        // 清除环境变量
        env::remove_var("AUTO_TARGET");
        env::remove_var("CARGO_BUILD_TARGET");

        // 默认应为 Pc
        let target = Target::detect();
        assert_eq!(target, Target::Pc);

        // 清理
        env::remove_var("AUTO_TARGET");
        env::remove_var("CARGO_BUILD_TARGET");
    }

    #[test]
    fn test_detect_from_env_var() {
        // 清除环境变量
        env::remove_var("AUTO_TARGET");
        env::remove_var("CARGO_BUILD_TARGET");

        // 测试 AUTO_TARGET=mcu
        env::set_var("AUTO_TARGET", "mcu");
        assert_eq!(Target::detect(), Target::Mcu);

        // 测试 AUTO_TARGET=pc
        env::set_var("AUTO_TARGET", "pc");
        assert_eq!(Target::detect(), Target::Pc);

        // 测试大小写不敏感
        env::set_var("AUTO_TARGET", "MCU");
        assert_eq!(Target::detect(), Target::Mcu);

        env::set_var("AUTO_TARGET", "PC");
        assert_eq!(Target::detect(), Target::Pc);

        // 清理
        env::remove_var("AUTO_TARGET");
        env::remove_var("CARGO_BUILD_TARGET");
    }

    #[test]
    fn test_detect_from_cargo_target() {
        // 清除其他环境变量
        env::remove_var("AUTO_TARGET");
        env::remove_var("CARGO_BUILD_TARGET");

        // 测试 ARM/Thumb MCU 目标
        env::set_var("CARGO_BUILD_TARGET", "thumbv7em-none-eabihf");
        assert_eq!(Target::detect(), Target::Mcu);

        env::set_var("CARGO_BUILD_TARGET", "arm-none-eabi");
        assert_eq!(Target::detect(), Target::Mcu);

        // 测试 PC 目标
        env::set_var("CARGO_BUILD_TARGET", "x86_64-unknown-linux-gnu");
        assert_eq!(Target::detect(), Target::Pc);

        env::set_var("CARGO_BUILD_TARGET", "x86_64-pc-windows-msvc");
        assert_eq!(Target::detect(), Target::Pc);

        // 清理
        env::remove_var("AUTO_TARGET");
        env::remove_var("CARGO_BUILD_TARGET");
    }

    #[test]
    fn test_auto_target_takes_precedence() {
        // 清除环境变量
        env::remove_var("AUTO_TARGET");
        env::remove_var("CARGO_BUILD_TARGET");

        // AUTO_TARGET 优先于 CARGO_BUILD_TARGET
        env::set_var("AUTO_TARGET", "pc");
        env::set_var("CARGO_BUILD_TARGET", "thumbv7em-none-eabihf");

        assert_eq!(Target::detect(), Target::Pc);

        // 清理
        env::remove_var("AUTO_TARGET");
        env::remove_var("CARGO_BUILD_TARGET");
    }
}

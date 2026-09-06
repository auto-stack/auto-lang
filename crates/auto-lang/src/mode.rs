// Plan 081 Phase 2: Execution Mode Selection
//
// This module defines the ExecutionMode enum which specifies how AutoLang code
// should be executed or transpiled.

/// Execution or transpilation mode for AutoLang code
///
/// **Plan 081**: Each package or dependency can specify its execution mode.
/// This allows mixing AutoVM bytecode, C transpilation, Rust transpilation,
/// and Evaluator interpretation within a single project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionMode {
    /// AutoVM bytecode execution (default)
    /// Code is compiled to ABC bytecode and executed on the AutoVM virtual machine
    AutoVM,

    /// TreeWalker evaluator (legacy, slower)
    /// Code is interpreted directly using the TreeWalker interpreter
    Evaluator,

    /// C transpilation (a2c)
    /// Code is transpiled to C for embedded systems or native compilation
    C,

    /// Rust transpilation (a2r)
    /// Code is transpiled to Rust for native applications
    Rust,
}

impl ExecutionMode {
    /// Parse execution mode from string
    ///
    /// # Examples
    ///
    /// ```
    /// use auto_lang::mode::ExecutionMode;
    ///
    /// assert_eq!(ExecutionMode::from_str("autovm"), Some(ExecutionMode::AutoVM));
    /// assert_eq!(ExecutionMode::from_str("vm"), Some(ExecutionMode::AutoVM));
    /// assert_eq!(ExecutionMode::from_str("c"), Some(ExecutionMode::C));
    /// assert_eq!(ExecutionMode::from_str("rust"), Some(ExecutionMode::Rust));
    /// assert_eq!(ExecutionMode::from_str("evaluator"), Some(ExecutionMode::Evaluator));
    /// assert_eq!(ExecutionMode::from_str("invalid"), None);
    /// ```
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "autovm" | "vm" | "bytecode" => Some(ExecutionMode::AutoVM),
            "evaluator" | "eval" | "tree" | "treewalker" => Some(ExecutionMode::Evaluator),
            "c" | "a2c" | "transpile-c" => Some(ExecutionMode::C),
            "rust" | "a2r" | "transpile-rust" => Some(ExecutionMode::Rust),
            _ => None,
        }
    }

    /// Convert execution mode to string representation
    ///
    /// # Examples
    ///
    /// ```
    /// use auto_lang::mode::ExecutionMode;
    ///
    /// assert_eq!(ExecutionMode::AutoVM.as_str(), "autovm");
    /// assert_eq!(ExecutionMode::Evaluator.as_str(), "evaluator");
    /// assert_eq!(ExecutionMode::C.as_str(), "c");
    /// assert_eq!(ExecutionMode::Rust.as_str(), "rust");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionMode::AutoVM => "autovm",
            ExecutionMode::Evaluator => "evaluator",
            ExecutionMode::C => "c",
            ExecutionMode::Rust => "rust",
        }
    }

    /// Check if this mode requires compilation (as opposed to interpretation)
    pub fn requires_compilation(&self) -> bool {
        matches!(self, ExecutionMode::AutoVM | ExecutionMode::C | ExecutionMode::Rust)
    }

    /// Check if this mode is a transpilation mode (to C or Rust)
    pub fn is_transpilation(&self) -> bool {
        matches!(self, ExecutionMode::C | ExecutionMode::Rust)
    }

    /// Check if this mode uses bytecode VM
    pub fn is_bytecode(&self) -> bool {
        matches!(self, ExecutionMode::AutoVM)
    }

    /// Check if this mode uses interpreter
    pub fn is_interpreter(&self) -> bool {
        matches!(self, ExecutionMode::Evaluator)
    }
}

impl Default for ExecutionMode {
    fn default() -> Self {
        // Plan 081: AutoVM is the default execution mode
        ExecutionMode::AutoVM
    }
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ExecutionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ExecutionMode::from_str(s)
            .ok_or_else(|| format!("Invalid execution mode: '{}'. Expected: autovm, evaluator, c, or rust", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        // AutoVM variants
        assert_eq!(ExecutionMode::from_str("autovm"), Some(ExecutionMode::AutoVM));
        assert_eq!(ExecutionMode::from_str("vm"), Some(ExecutionMode::AutoVM));
        assert_eq!(ExecutionMode::from_str("bytecode"), Some(ExecutionMode::AutoVM));

        // Evaluator variants
        assert_eq!(ExecutionMode::from_str("evaluator"), Some(ExecutionMode::Evaluator));
        assert_eq!(ExecutionMode::from_str("eval"), Some(ExecutionMode::Evaluator));
        assert_eq!(ExecutionMode::from_str("tree"), Some(ExecutionMode::Evaluator));

        // C variants
        assert_eq!(ExecutionMode::from_str("c"), Some(ExecutionMode::C));
        assert_eq!(ExecutionMode::from_str("a2c"), Some(ExecutionMode::C));

        // Rust variants
        assert_eq!(ExecutionMode::from_str("rust"), Some(ExecutionMode::Rust));
        assert_eq!(ExecutionMode::from_str("a2r"), Some(ExecutionMode::Rust));

        // Invalid
        assert_eq!(ExecutionMode::from_str("invalid"), None);
        assert_eq!(ExecutionMode::from_str(""), None);
    }

    #[test]
    fn test_as_str() {
        assert_eq!(ExecutionMode::AutoVM.as_str(), "autovm");
        assert_eq!(ExecutionMode::Evaluator.as_str(), "evaluator");
        assert_eq!(ExecutionMode::C.as_str(), "c");
        assert_eq!(ExecutionMode::Rust.as_str(), "rust");
    }

    #[test]
    fn test_default() {
        assert_eq!(ExecutionMode::default(), ExecutionMode::AutoVM);
    }

    #[test]
    fn test_requires_compilation() {
        assert!(ExecutionMode::AutoVM.requires_compilation());
        assert!(ExecutionMode::C.requires_compilation());
        assert!(ExecutionMode::Rust.requires_compilation());
        assert!(!ExecutionMode::Evaluator.requires_compilation());
    }

    #[test]
    fn test_is_transpilation() {
        assert!(ExecutionMode::C.is_transpilation());
        assert!(ExecutionMode::Rust.is_transpilation());
        assert!(!ExecutionMode::AutoVM.is_transpilation());
        assert!(!ExecutionMode::Evaluator.is_transpilation());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", ExecutionMode::AutoVM), "autovm");
        assert_eq!(format!("{}", ExecutionMode::C), "c");
    }

    #[test]
    #[allow(unused_imports)]
    fn test_from_str_trait() {
        use std::str::FromStr;

        assert_eq!(ExecutionMode::from_str("autovm").unwrap(), ExecutionMode::AutoVM);
        assert_eq!(ExecutionMode::from_str("c").unwrap(), ExecutionMode::C);

        // Option doesn't have is_err(), use is_none() instead
        assert!(ExecutionMode::from_str("invalid").is_none());
    }
}

// Plan 555 T02: 文件级脚本模式信号（与 ExecutionMode 正交——那是执行
// 后端选择，这是语言方言：正常模式 .at / 脚本模式 .as AutoScript）。
// 设计源 script-mode-interop §2：扩展名为主 + pragma 覆盖。
// W1 语义 = passthrough（脚本语义激活在 W2 lowering 规则批）。

// Plan 555 T02: 文件级脚本模式信号（与 ExecutionMode 正交——那是执行
// 后端选择，这是语言方言：正常模式 .at / 脚本模式 .as AutoScript）。
// 设计源 script-mode-interop §2：扩展名为主 + pragma 覆盖。
// W1 语义 = passthrough（脚本语义激活在 W2 lowering 规则批）。

/// 文件的脚本方言模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScriptMode {
    /// 正常模式（.at 存量默认）：静态类型、null 禁令、显式 .?()
    #[default]
    Normal,
    /// 脚本模式（AutoScript，.as）：动态分派糖、隐式 Err 传播、null 是值
    /// ——W1 仅立信号通道，语义 passthrough。
    Script,
}

/// 解析文件级脚本模式。优先序：`#[rust]` 压回 > `#[script]` 提升 >
/// 扩展名（`.as` ≡ 隐式 `#[script]`）。extension 传小写不带点形式
/// （"at"/"as"），无扩展名信息（stdin/eval）传 None。
pub fn resolve_script_mode(
    extension: Option<&str>,
    script_pragma: bool,
    rust_pragma: bool,
) -> ScriptMode {
    if rust_pragma {
        return ScriptMode::Normal;
    }
    if script_pragma {
        return ScriptMode::Script;
    }
    match extension {
        Some("as") => ScriptMode::Script,
        _ => ScriptMode::Normal,
    }
}

#[cfg(test)]
mod tests_script_mode {
    use super::*;

    /// 八格矩阵（Plan 555 详细设计 T02 契约表）。
    /// 优先序：#[rust] 压回 > #[script] 提升 > 扩展名（.as ≡ 隐式 script）。
    #[test]
    fn test_script_mode_matrix_eight_cells() {
        use super::resolve_script_mode;
        use super::ScriptMode;
        // .at 无 pragma → Normal（550 门控 lint 照常）
        assert_eq!(resolve_script_mode(Some("at"), false, false), ScriptMode::Normal);
        // .at + #[script] → Script（550 已落通道）
        assert_eq!(resolve_script_mode(Some("at"), true, false), ScriptMode::Script);
        // .at + #[rust] → Normal（既有 rust pragma 语义不动）
        assert_eq!(resolve_script_mode(Some("at"), false, true), ScriptMode::Normal);
        // .as 无 pragma → Script（隐式）
        assert_eq!(resolve_script_mode(Some("as"), false, false), ScriptMode::Script);
        // .as + #[script] → Script（幂等）
        assert_eq!(resolve_script_mode(Some("as"), true, false), ScriptMode::Script);
        // .as + #[rust] → Normal（显式压回通道）
        assert_eq!(resolve_script_mode(Some("as"), false, true), ScriptMode::Normal);
        // 双 pragma → #[rust] 胜（压回优先，保守取向）
        assert_eq!(resolve_script_mode(Some("as"), true, true), ScriptMode::Normal);
        // 无扩展名信息（stdin/eval 场景）→ 仅 pragma 决定
        assert_eq!(resolve_script_mode(None, false, false), ScriptMode::Normal);
        assert_eq!(resolve_script_mode(None, true, false), ScriptMode::Script);
    }
}

//! REPL input mode system (Plan 322).
//!
//! Three input modes (Shell / AutoScript / AI) with auto-detection,
//! manual locking (F1/F2/F3), and syntax-based multiline continuation.

/// The three input modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Shell commands (ls, git, cargo, ...). Prompt: `>`
    Shell,
    /// Auto language code (let, fn, expressions). Prompt: `#`
    AutoScript,
    /// AI natural language input. Prompt: `?`
    AI,
}

impl InputMode {
    /// The prompt symbol for this mode.
    pub fn prompt_symbol(&self) -> &'static str {
        match self {
            Self::Shell => ">",
            Self::AutoScript => "#",
            Self::AI => "?",
        }
    }

    /// Continuation prompt (multiline).
    pub fn continuation_symbol(&self) -> &'static str {
        "·"
    }
}

/// Tracks the current mode state of the REPL.
#[derive(Debug, Clone)]
pub struct ModeState {
    /// Locked mode (None = auto-detect). Set by F1/F2.
    pub locked: Option<InputMode>,
    /// The last auto-detected mode (for restoring after AI).
    pub last_auto: InputMode,
    /// Whether we're in a multiline continuation.
    pub in_continuation: bool,
}

impl Default for ModeState {
    fn default() -> Self {
        Self {
            locked: None,
            last_auto: InputMode::Shell,
            in_continuation: false,
        }
    }
}

impl ModeState {
    /// The effective mode right now (locked takes priority).
    pub fn effective(&self) -> InputMode {
        self.locked.unwrap_or(self.last_auto)
    }

    /// Lock to a mode. If already locked to the same mode, unlock.
    pub fn toggle_lock(&mut self, mode: InputMode) {
        if self.locked == Some(mode) {
            self.locked = None; // Toggle off.
        } else {
            self.locked = Some(mode);
        }
    }

    /// Unlock (return to auto-detect).
    pub fn unlock(&mut self) {
        self.locked = None;
    }

    /// Enter AI mode temporarily (does not lock).
    pub fn enter_ai(&mut self) {
        self.last_auto = self.effective();
        // AI is transient — not stored in locked.
    }

    /// The prompt to display.
    pub fn prompt(&self) -> String {
        let mode = self.effective();
        if self.in_continuation {
            return mode.continuation_symbol().to_string();
        }
        let sym = mode.prompt_symbol();
        if self.locked.is_some() {
            format!("▌{}", sym) // Locked indicator.
        } else {
            sym.to_string()
        }
    }
}

/// Check if the input needs continuation (unclosed delimiters or trailing `\`).
pub fn needs_continuation(input: &str) -> bool {
    let trimmed = input.trim();

    // 1. Trailing backslash (shell line continuation).
    if trimmed.ends_with('\\') {
        return true;
    }

    // 2. Unclosed delimiters: { } ( ) [ ] and quotes.
    let mut depth_brace = 0i32;
    let mut depth_paren = 0i32;
    let mut depth_bracket = 0i32;
    let mut in_single = false;
    let mut in_double = false;

    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let prev = if i > 0 { Some(chars[i - 1]) } else { None };

        // Skip escaped chars inside double quotes.
        if in_double && c == '\\' && i + 1 < chars.len() {
            i += 2;
            continue;
        }

        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '{' if !in_single && !in_double => depth_brace += 1,
            '}' if !in_single && !in_double => depth_brace -= 1,
            '(' if !in_single && !in_double => depth_paren += 1,
            ')' if !in_single && !in_double => depth_paren -= 1,
            '[' if !in_single && !in_double => depth_bracket += 1,
            ']' if !in_single && !in_double => depth_bracket -= 1,
            _ => {}
        }
        i += 1;
        let _ = prev; // Currently unused but kept for future escape handling.
    }

    depth_brace > 0
        || depth_paren > 0
        || depth_bracket > 0
        || in_single
        || in_double
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_prompt_symbols() {
        assert_eq!(InputMode::Shell.prompt_symbol(), ">");
        assert_eq!(InputMode::AutoScript.prompt_symbol(), "#");
        assert_eq!(InputMode::AI.prompt_symbol(), "?");
    }

    #[test]
    fn mode_state_default_is_auto_shell() {
        let ms = ModeState::default();
        assert!(ms.locked.is_none());
        assert_eq!(ms.effective(), InputMode::Shell);
        assert_eq!(ms.prompt(), ">");
    }

    #[test]
    fn mode_state_lock_toggle() {
        let mut ms = ModeState::default();
        ms.toggle_lock(InputMode::AutoScript);
        assert_eq!(ms.locked, Some(InputMode::AutoScript));
        assert_eq!(ms.effective(), InputMode::AutoScript);
        assert_eq!(ms.prompt(), "▌#"); // Locked indicator.

        // Toggle again → unlock.
        ms.toggle_lock(InputMode::AutoScript);
        assert!(ms.locked.is_none());
        assert_eq!(ms.prompt(), ">");
    }

    #[test]
    fn mode_state_continuation_prompt() {
        let mut ms = ModeState::default();
        ms.in_continuation = true;
        assert_eq!(ms.prompt(), "·");
    }

    #[test]
    fn continuation_backslash() {
        assert!(needs_continuation("echo hello \\"));
        assert!(needs_continuation("ls \\"));
        assert!(!needs_continuation("echo hello"));
    }

    #[test]
    fn continuation_unclosed_brace() {
        assert!(needs_continuation("fn add(a, b) int {"));
        assert!(needs_continuation("fn add(a, b) int { a + b"));
        assert!(!needs_continuation("fn add(a, b) int { a + b }"));
    }

    #[test]
    fn continuation_unclosed_paren() {
        assert!(needs_continuation("print("));
        assert!(needs_continuation("foo(1, 2,"));
        assert!(!needs_continuation("foo(1, 2, 3)"));
    }

    #[test]
    fn continuation_unclosed_bracket() {
        assert!(needs_continuation("let x = [1, 2,"));
        assert!(!needs_continuation("let x = [1, 2, 3]"));
    }

    #[test]
    fn continuation_unclosed_string() {
        assert!(needs_continuation("let x = \"hello"));
        assert!(needs_continuation("let x = 'world"));
        assert!(!needs_continuation("let x = \"hello\""));
    }

    #[test]
    fn continuation_nested() {
        assert!(needs_continuation("fn f() { if true {"));
        assert!(!needs_continuation("fn f() { if true { } }"));
    }

    #[test]
    fn continuation_braces_in_string() {
        // Braces inside strings should NOT count as unclosed.
        assert!(!needs_continuation("echo \"hello {world}\""));
        assert!(!needs_continuation("let s = \"{{not a brace}}\""));
    }

    #[test]
    fn continuation_escaped_quote() {
        // Escaped quote inside double-quoted string does not close the string.
        // The last " closes it properly.
        assert!(!needs_continuation(r#"let s = "she said \"hi\"""#));
        // Genuinely unclosed: escaped quote at end, no real closing quote.
        assert!(needs_continuation("let s = \"unclosed \\\" end"));
    }
}

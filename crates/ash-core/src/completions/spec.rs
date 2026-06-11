//! Declarative completion specification for external commands
//!
//! Defines the Atom-format structures that describe how to complete arguments,
//! flags, and subcommands for external commands like `git`, `cargo`, etc.
//!
//! Example Atom format:
//! ```text
//! {
//!     command: "git",
//!     desc: "Git version control",
//!     subcommands: [
//!         { name: "checkout", desc: "Switch branches", ... }
//!     ],
//!     flags: [
//!         { short: "b", long: "branch", desc: "Create new branch", arg: "name" }
//!     ],
//!     args: [
//!         { position: 0, when: { flags_absent: ["b"] }, source: { command: "git branch --list" } }
//!     ]
//! }
//! ```

/// Top-level completion specification for an external command.
#[derive(Debug, Clone)]
pub struct CompletionSpec {
    pub command: String,
    pub desc: Option<String>,
    pub subcommands: Vec<SubcommandSpec>,
    pub flags: Vec<FlagSpec>,
    pub args: Vec<ArgSpec>,
}

/// A subcommand definition (e.g., `git checkout`).
#[derive(Debug, Clone)]
pub struct SubcommandSpec {
    pub name: String,
    pub desc: Option<String>,
    pub flags: Vec<FlagSpec>,
    pub args: Vec<ArgSpec>,
    /// Nested subcommands (e.g., `git remote add`).
    pub subcommands: Vec<SubcommandSpec>,
}

/// A flag definition (e.g., `-b`, `--branch`).
#[derive(Debug, Clone)]
pub struct FlagSpec {
    pub short: Option<String>,
    pub long: Option<String>,
    pub desc: Option<String>,
    /// If present, this flag takes a value (e.g., `-m "message"`).
    pub arg: Option<String>,
}

/// A positional argument definition with optional condition and source.
#[derive(Debug, Clone)]
pub struct ArgSpec {
    /// Position index (0-based). Special value `ARG_ANY_POSITION` means any position.
    pub position: usize,
    /// Whether this arg can repeat (e.g., `git add file1 file2 ...`).
    pub repeat: bool,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub when: Option<WhenCondition>,
    pub source: Option<CompletionSource>,
}

/// Sentinel: matches any position (useful for repeatable args).
pub const ARG_ANY_POSITION: usize = usize::MAX;

/// Condition for when an ArgSpec applies.
#[derive(Debug, Clone)]
pub enum WhenCondition {
    /// All listed flags must be present.
    FlagsPresent(Vec<String>),
    /// None of the listed flags may be present.
    FlagsAbsent(Vec<String>),
    /// Previous positional arg matches this value.
    PrevArg(String),
}

/// Source of completion candidates.
#[derive(Debug, Clone)]
pub enum CompletionSource {
    /// Fixed list of values.
    Static(Vec<String>),
    /// Execute a command and parse its output.
    Command { cmd: String, parse: ParseMode },
    /// Complete file paths, optionally filtered by glob.
    Files { filter: Option<String> },
    /// Complete directory paths only.
    Directories,
    /// Complete environment variable names.
    Variables,
}

/// How to parse command output into completion candidates.
#[derive(Debug, Clone)]
pub enum ParseMode {
    /// Each line is one candidate (strips leading whitespace and `*` markers).
    Line,
    /// Split by whitespace, take Nth field (0-indexed).
    Field(usize),
}

// ── Builder helpers ──────────────────────────────────────

impl CompletionSpec {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            desc: None,
            subcommands: Vec::new(),
            flags: Vec::new(),
            args: Vec::new(),
        }
    }

    pub fn desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }

    pub fn subcommand(mut self, sub: SubcommandSpec) -> Self {
        self.subcommands.push(sub);
        self
    }

    pub fn flag(mut self, flag: FlagSpec) -> Self {
        self.flags.push(flag);
        self
    }

    pub fn arg(mut self, arg: ArgSpec) -> Self {
        self.args.push(arg);
        self
    }
}

impl SubcommandSpec {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            desc: None,
            flags: Vec::new(),
            args: Vec::new(),
            subcommands: Vec::new(),
        }
    }

    pub fn desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }

    pub fn flag(mut self, flag: FlagSpec) -> Self {
        self.flags.push(flag);
        self
    }

    pub fn arg(mut self, arg: ArgSpec) -> Self {
        self.args.push(arg);
        self
    }

    pub fn subcommand(mut self, sub: SubcommandSpec) -> Self {
        self.subcommands.push(sub);
        self
    }
}

impl FlagSpec {
    pub fn short(short: &str) -> Self {
        Self {
            short: Some(short.to_string()),
            long: None,
            desc: None,
            arg: None,
        }
    }

    pub fn long(long: &str) -> Self {
        Self {
            short: None,
            long: Some(long.to_string()),
            desc: None,
            arg: None,
        }
    }

    pub fn both(short: &str, long: &str) -> Self {
        Self {
            short: Some(short.to_string()),
            long: Some(long.to_string()),
            desc: None,
            arg: None,
        }
    }

    pub fn desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }

    pub fn takes_arg(mut self, arg: &str) -> Self {
        self.arg = Some(arg.to_string());
        self
    }
}

impl ArgSpec {
    pub fn new(position: usize) -> Self {
        Self {
            position,
            repeat: false,
            name: None,
            desc: None,
            when: None,
            source: None,
        }
    }

    pub fn any() -> Self {
        Self::new(ARG_ANY_POSITION)
    }

    pub fn repeat(mut self) -> Self {
        self.repeat = true;
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn desc(mut self, desc: &str) -> Self {
        self.desc = Some(desc.to_string());
        self
    }

    pub fn when(mut self, cond: WhenCondition) -> Self {
        self.when = Some(cond);
        self
    }

    pub fn source(mut self, src: CompletionSource) -> Self {
        self.source = Some(src);
        self
    }
}

impl WhenCondition {
    pub fn flags_present(flags: &[&str]) -> Self {
        Self::FlagsPresent(flags.iter().map(|s| s.to_string()).collect())
    }

    pub fn flags_absent(flags: &[&str]) -> Self {
        Self::FlagsAbsent(flags.iter().map(|s| s.to_string()).collect())
    }

    pub fn prev_arg(value: &str) -> Self {
        Self::PrevArg(value.to_string())
    }

    /// Evaluate this condition against the current command-line state.
    pub fn evaluate(&self, present_flags: &[String], prev_args: &[String]) -> bool {
        match self {
            Self::FlagsPresent(required) => required
                .iter()
                .all(|f| present_flags.contains(f) || present_flags.contains(&format!("--{}", f))),
            Self::FlagsAbsent(forbidden) => forbidden
                .iter()
                .all(|f| !present_flags.contains(f) && !present_flags.contains(&format!("--{}", f))),
            Self::PrevArg(value) => prev_args.last().map(|s| s.as_str()) == Some(value.as_str()),
        }
    }
}

impl CompletionSource {
    pub fn static_list(items: &[&str]) -> Self {
        Self::Static(items.iter().map(|s| s.to_string()).collect())
    }

    pub fn command(cmd: &str) -> Self {
        Self::Command {
            cmd: cmd.to_string(),
            parse: ParseMode::Line,
        }
    }

    pub fn command_field(cmd: &str, field: usize) -> Self {
        Self::Command {
            cmd: cmd.to_string(),
            parse: ParseMode::Field(field),
        }
    }

    pub fn files() -> Self {
        Self::Files { filter: None }
    }

    pub fn files_with_filter(filter: &str) -> Self {
        Self::Files {
            filter: Some(filter.to_string()),
        }
    }

    pub fn directories() -> Self {
        Self::Directories
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_when_flags_present() {
        let cond = WhenCondition::flags_present(&["b"]);
        assert!(cond.evaluate(&["b".into()], &[]));
        assert!(cond.evaluate(&["--b".into()], &[]));
        assert!(!cond.evaluate(&[], &[]));
    }

    #[test]
    fn test_when_flags_absent() {
        let cond = WhenCondition::flags_absent(&["b", "B"]);
        assert!(cond.evaluate(&[], &[]));
        assert!(!cond.evaluate(&["b".into()], &[]));
        assert!(!cond.evaluate(&["--B".into()], &[]));
    }

    #[test]
    fn test_when_prev_arg() {
        let cond = WhenCondition::prev_arg("origin");
        assert!(cond.evaluate(&[], &["origin".to_string()]));
        assert!(!cond.evaluate(&[], &["upstream".to_string()]));
    }

    #[test]
    fn test_spec_builder() {
        let spec = CompletionSpec::new("git")
            .desc("Git version control")
            .subcommand(
                SubcommandSpec::new("checkout")
                    .desc("Switch branches")
                    .flag(FlagSpec::both("b", "branch").desc("Create new branch"))
                    .arg(
                        ArgSpec::new(0)
                            .when(WhenCondition::flags_absent(&["b"]))
                            .source(CompletionSource::command("git branch --list")),
                    ),
            );

        assert_eq!(spec.command, "git");
        assert_eq!(spec.subcommands.len(), 1);
        assert_eq!(spec.subcommands[0].name, "checkout");
        assert_eq!(spec.subcommands[0].flags.len(), 1);
        assert_eq!(spec.subcommands[0].args.len(), 1);
    }
}

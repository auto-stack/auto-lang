# 11 - Shell Tools (AutoShell)

## Status

**Implemented:**
- AutoShell (`crates/auto-shell/`) has a working REPL with command parsing, pipeline support, and structured data output.
- Built-in commands: `ls`, `cd`, `pwd`, `echo`, `help`, `cp`, `mv`, `rm`, `mkdir`, `grep`, `wc`, `ps`, `sys`, `build`, `run`, `select`, `where`, `get`.
- Structured output for information commands: `ls` returns structured file entries; `ps` returns process info; `sys disks`/`sys cpu`/`sys mem` return structured data using the `sysinfo` crate.
- Pipeline architecture: commands communicate via `PipelineData` carrying typed `Value` objects, not raw text.
- Command registry: extensible `Command` trait with `name()`, `signature()`, and `run()` methods.
- Tab completion framework exists in `completions/`.

**Partial / Planned:**
- File operation commands (`cp`, `mv`, `rm`, `mkdir`) exist but do not yet integrate `uutils` crates. Current implementations use direct Rust stdlib calls.
- SmartCmd (natural language interface) is designed but not implemented. No `SmartCmd` trait or NLP parsing exists.
- AI-assisted command understanding is planned for a future phase.

## Design

### Architecture: Three-Layer Model

AutoShell separates concerns into three layers:

1. **Engine layer** (platform primitives): File I/O, process enumeration, disk info -- delegated to battle-tested Rust crates rather than reimplemented.
2. **Adaptation layer** (type conversion): Raw data from engine crates is converted into Auto's typed value system (`Value`, `Obj`). This is where `ls` output becomes a `List<FileEntry>` rather than a raw text stream.
3. **Intelligence layer** (AI augmentation): Natural language understanding, smart defaults, and context-aware suggestions. This layer interprets user intent and selects the appropriate command pipeline.

### Coreutils Strategy: Reuse, Don't Rewrite

The core design decision is to avoid reimplementing standard Unix commands from scratch. The rationale:

- **Edge cases**: Commands like `ls` and `cp` contain decades of edge-case handling (symlink cycles, permission masks, non-UTF-8 filenames, atomic writes on crash). Reimplementing these introduces risk.
- **Performance**: By using Rust crates directly (in-process), AutoShell avoids the `fork/exec` overhead of shelling out to external binaries. Commands run an order of magnitude faster than traditional bash.
- **Cross-platform**: Using crates like `sysinfo` and `nu-system` provides Windows/Linux/macOS support out of the box.

**Recommended source crates per command:**

| Command | Source | Output |
|---|---|---|
| `ls` | Custom (nushell-inspired logic) | `List<FileEntry>` with name, type, size, modified, permissions |
| `ps` | `sysinfo` crate | `List<ProcessEntry>` with pid, ppid, name, status, cpu/mem usage |
| `sys disks` | `sysinfo` crate | `List<DiskEntry>` with device, filesystem, mount, total, free |
| `sys cpu` | `sysinfo` crate | `CpuInfo` with usage percentages |
| `sys mem` | `sysinfo` crate | `MemInfo` with total, used, free |
| `cp` | `uutils uu_cp` (planned) | Success/failure |
| `mv` | `uutils uu_mv` (planned) | Success/failure |
| `rm` | `uutils uu_rm` (planned) | Success/failure |
| `mkdir` | `uutils uu_mkdir` (planned) | Success/failure |
| `find` | `nu-glob` crate (planned) | `List<Path>` |

### Command Classification

AutoShell commands fall into two categories based on their output:

**Structured-output commands** return typed data that can be filtered, mapped, and piped:

```auto
let files = ls("src/")
for f in files {
    if f.size > 1024 { print(f.name) }
}
```

This "object-level pipeline" approach combines PowerShell's expressiveness with Rust's performance.

**Status-only commands** return success or failure. These are straightforward wrappers around `uutils` crates:

```
cp source.txt dest.txt    // returns: ok or error
rm -rf temp/              // returns: ok or error
```

### Internal Type System

AutoShell defines internal Rust types for structured command output:

```rust
pub struct AshFileEntry {
    pub name: String,
    pub file_type: String,       // "file" | "dir" | "symlink"
    pub size: i64,
    pub modified: Option<DateTime<Utc>>,
    pub permissions: Option<String>,
    pub owner: Option<String>,
}

pub struct AshProcessEntry {
    pub pid: i32,
    pub ppid: i32,
    pub name: String,
    pub status: String,
    pub cpu_usage: f64,
    pub mem_usage: i64,
}

pub struct AshDiskEntry {
    pub device: String,
    pub file_system: String,
    pub mount_point: String,
    pub total: i64,
    pub free: i64,
    pub removable: bool,
}
```

A conversion layer (`From` trait implementations) maps external crate types into these ASH types. This decouples the shell's public API from upstream crate internals.

### Pipeline Architecture

Commands communicate through `PipelineData`, which wraps Auto's `Value` type. This enables:

- **Structured piping**: `ls | grep ".at"` operates on typed records, not raw text.
- **Type-safe composition**: The compiler can verify pipeline compatibility.
- **Display abstraction**: The same structured data renders differently in interactive mode (table), script mode (text), or AI mode (JSON).

### SmartCmd Design

SmartCmd is a planned feature to make shell commands understand natural language input. The design envisions:

1. A `SmartCmd` trait that wraps standard commands with natural language parsing.
2. AI-assisted command interpretation: the user types intent in plain language, and the system selects and parameterizes the appropriate command.

This is deferred to a later phase, pending the natural language infrastructure.

## Open Questions

- Should `uutils` be integrated as crate dependencies or as git submodules?
- How should structured pipeline data be serialized for cross-process piping?
- What is the minimum viable set of commands before AutoShell can replace bash for daily development?
- Should SmartCmd use a local LLM, a cloud API, or rule-based NLP?

## Source Documents

- [raw/ash-coreutils.md](raw/ash-coreutils.md)
- [raw/ash-smartcmd-design.md](raw/ash-smartcmd-design.md)

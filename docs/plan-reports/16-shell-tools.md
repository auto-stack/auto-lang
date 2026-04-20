# 16 - Shell Tools and CLI

## Overview

AutoLang's shell and CLI layer encompasses AutoShell, a cross-platform shell environment with Nushell-inspired structured data pipelines, and AutoCode, an AI-powered coding agent. The shell provides a REPL with 20+ built-in commands, auto-completion, history, and deep AutoLang integration. Its value pipeline system passes structured Auto Value objects between commands, enabling zero-copy data flow without the serialization overhead of traditional text-based shells. The AI agent infrastructure extends the shell into an interactive coding assistant capable of streaming LLM conversations, tool execution, and multi-agent coordination. Three plans are fully implemented (155+176 tests), while the AI agent and coding agent remain in design phase.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 017 | AutoShell Design | Done | Cross-platform shell with REPL, pipelines, 20+ built-in commands, auto-completion, and history. Ten implementation phases, 155 tests. |
| 046 | AutoShell ls Flags | Done | Coreutils-style flags (-a, -l, -h, -t, -r, -R) for the ls command with cross-platform permission handling. |
| 047 | AutoShell Value Pipelines | Done | Zero-copy structured data pipelines using Auto Value system, with get, where, and select commands. 176 tests. |
| 153 | AutoShell AI Agent Design | Planned | Multi-granularity AI agent with LLM providers, tool system, MCP client, and multi-agent coordination. Eight phases designed. |
| 159 | AutoCode Coding Agent | Planned | AI-powered coding agent integrated into AutoShell. Rust prototype complete (Phase 1-5), AutoLang port complete (Phase 6A), a2r transpiler enhancement in progress (Phase 6B). |

## Status

**Implemented**: AutoShell REPL and command system (Plan 017), ls coreutils flags (Plan 046), structured value pipelines (Plan 047), AutoCode Rust prototype with all five phases (Plan 159 Phases 1-5), AutoCode AutoLang port via VM FFI (Plan 159 Phase 6A), a2r transpiler enhancements for Option/Result, HashMap, async fn, derive macros, pub visibility, impl Trait for Type, struct destructuring, multi-file modules, and Cargo.toml generation (Plan 159 Phase 6B).

**Partial**: a2r transpiler still lacks HashMap literal support, complex closure type inference, and String vs &str precise distinction (Plan 159 Phase 6B-3/4).

**Planned**: AI Agent for AutoShell (Plan 153, all eight phases), a2r full parity for pure .at to Rust transpilation (Plan 159 Phase 6B-3/4).

## Design

### AutoShell Core Architecture

AutoShell is a Nushell-inspired shell implemented as a standalone Rust crate (`auto-shell/`) that uses AutoLang as its scripting language. The architecture centers on a read-eval-print loop backed by `reedline` for terminal interaction, a custom command parser handling pipelines and quotes, and a command registry that dispatches built-in, external, and AutoLang-defined commands.

The shell progressed through ten implementation phases, starting with a basic REPL and external command execution, then building up pipeline parsing, built-in commands (cd, pwd, ls, sort, grep, head, tail, wc, set, export, echo), variable expansion with `$name` and `${name}` syntax, quote-aware argument parsing with escape sequences, table display with column alignment and color coding, AutoLang expression evaluation and function definition, auto-completion for commands/files/variables/flags, and a file-backed history system with up-arrow navigation and history expansion patterns (`!!`, `!n`, `!string`). The final codebase totals roughly 3,500 lines of Rust (excluding tests) across 25+ source files with 155 passing tests.

The command system defines a `Command` trait with `name()`, `signature()`, and `run()` methods. Commands declare their parameters through a `Signature` builder that supports required arguments, optional arguments, flags with short aliases, and rest arguments. The `ParsedArgs` struct provides `has_flag()` and positional access for extracting arguments at runtime. External commands execute via platform APIs (CreateProcess on Windows, execve on Unix), with stdout and stderr captured and returned as shell values.

Key design decisions include choosing `reedline` over `rustyline` for its modern features and active development, using a custom shell parser rather than leveraging the AutoLang parser directly (to handle shell-specific syntax like pipes and redirections), and maintaining a separate `ShellValue` type with bidirectional conversion to Auto's `Value` enum.

### Structured Data Pipelines

The most significant architectural evolution in AutoShell was the transition from string-based to structured data pipelines (Plan 047). In the original design, commands produced strings that the next command had to re-parse, losing type information at every pipe boundary. The solution introduces a `PipelineData` enum with two variants: `Value(Value)` for structured Auto values and `Text(String)` for legacy text output.

The `Command` trait was updated to accept `PipelineData` as input and return `PipelineData` as output, rather than `Option<String>`. The pipeline executor threads `PipelineData` through each command in sequence, converting to text only at the final display step. This means a command like `ls` returns a `Value::Array` of `Value::Obj` objects, and downstream commands like `where` or `select` operate directly on those structured objects without any serialization.

Three new data manipulation commands were added. The `get` command extracts a named field from each object in an array, producing a flat array of values. The `select` command projects specific fields from each object, producing an array of smaller objects with only the requested keys. The `where` command filters an array based on field comparisons using operators like `==`, `!=`, `>`, `<`, and `contains`. Together, these enable pipelines like `ls | where type == dir | get name` that work entirely with structured data.

The `ls` command was refactored to produce structured output via `ls_command_value()`, which returns a `Value::Array` of `Value::Obj` entries containing fields like name, type, size, modified, and permissions. The original string-based `ls_command()` function was retained as a wrapper that formats the structured value for display, maintaining backward compatibility. The display logic in `format_value_for_display()` automatically renders arrays of objects as aligned tables and single objects as key-value records.

All 176 tests pass, including 9 PipelineData-specific tests, 5 value helper tests, and the full existing shell test suite. The migration was incremental: commands were updated one at a time, with text-mode fallback available throughout.

### ls Coreutils Flags

The ls command was enhanced (Plan 046) to support six standard coreutils flags: `-a` (show hidden files), `-l` (long format with permissions/owner/size/time), `-h` (human-readable sizes like 1.2K, 3.4M), `-t` (sort by modification time, newest first), `-r` (reverse sort order), and `-R` (recursive directory listing). Both short and long forms are supported (`-a` equals `--all`), and flags can be combined in POSIX style (`-ltr`).

Cross-platform permission handling uses `#[cfg(unix)]` and `#[cfg(windows)]` conditional compilation. On Unix, permissions use `std::os::unix::fs::PermissionsExt` to produce `rwxrwxrwx` format strings, with file type indicators (`d` for directory, `l` for symlink, `-` for regular file). On Windows, permissions are simplified to a read-only attribute check, and the owner/group columns show `-` and `N/A` respectively. The `FileEntry` struct was extended to carry full `fs::Metadata` for long-format output.

### AI Agent Architecture (Plan 153)

The AutoShell AI Agent is designed as a multi-granularity system built on Auto's existing Task/Msg concurrency infrastructure (Plans 121, 124, 126). The agent model defines five agent types with different lifecycle characteristics: request agents (single LLM call, seconds), session agents (multi-turn conversation, minutes to hours), coordinator agents (task decomposition and result aggregation), worker agents (tool execution, Task or Process granularity), and sandbox agents (isolated process for untrusted code).

The core `AutoAgent` task uses Auto's `on` block pattern to handle message-driven events: `Ask` triggers an LLM call with streaming response, `ToolCall` dispatches to the tool registry, and `SubAgentComplete` aggregates results from child agents. The LLM provider abstraction (`spec LLMProvider`) defines async methods for chat completion and streaming, with implementations planned for Anthropic, OpenAI, and local models via Ollama.

The tool system follows the pattern established by Claude Code and claw-code: each tool implements a `Tool` spec with `name()`, `description()`, `input_schema()`, `is_read_only()`, `is_concurrency_safe()`, and `execute()`. The `ToolRegistry` manages registration and lookup. Built-in tools include ShellTool (command execution), FileReadTool, FileWriteTool, GrepTool, and GlobTool. External tools are accessed via MCP (Model Context Protocol) client tasks that connect over stdio, SSE, or HTTP transports.

Process-granularity agents are isolated in separate OS processes with their own AutoVM instances and memory spaces. Inter-process communication uses an `IPCTransport` enum supporting stdio, shared memory, Unix domain sockets, named pipes (Windows), and TCP. A `ProcessRegistry` task manages process lifecycle (spawn, heartbeat, kill). The granularity selection engine automatically decides between Task and Process based on sandbox requirements, resource limits, and task type.

The implementation plan spans eight phases: LLM API basics (dependent on streaming HTTP/SSE support), agent task foundation, tool system, MCP client, multi-agent coordination, AutoShell integration (/ask and /agent commands), process-granularity agents, and automatic granularity switching.

### AutoCode Coding Agent (Plan 159)

AutoCode is a Claude Code-like coding agent implemented as a Rust workspace (`auto-code-rs/`) with four crates: `ac-api` (LLM communication with Anthropic and OpenAI adapters), `ac-tools` (Tool trait, ToolRegistry, and five built-in tools: Bash, Read, Write, Edit, Grep), `ac-runtime` (Agent struct with agentic loop, context compression, permission policy, and JSONL session persistence), and `ac-cli` (CLI with REPL and single-prompt modes).

The agentic loop is the core execution pattern: the user sends a prompt, the agent appends it to the message history, calls the LLM via the streaming API, processes `StreamEvent` variants (TextDelta for real-time output, ToolUseBegin/ToolUseDelta for tool calls, Done for completion), executes any requested tools with permission checks, appends tool results to the message history, and loops back to the LLM until it returns `EndTurn`. The `StreamEvent` enum normalizes differences between Anthropic's content_block_start/delta/stop SSE events and OpenAI's delta/content/tool_calls chunk format.

The SSE parser handles chunked HTTP responses, splitting on `\n\n` boundaries and extracting `event:` and `data:` fields. Anthropic uses `x-api-key` + `anthropic-version` headers, while OpenAI uses `Authorization: Bearer` headers. Both adapters implement exponential backoff with jitter for retries (up to 5 attempts). The OpenAI adapter includes a `StreamState` machine to normalize its chunk format into the unified `StreamEvent` representation.

Context compression uses a `ContextManager` with a token budget (default 100K). When estimated tokens exceed the threshold, older messages are summarized by calling the LLM to generate a condensed version, which replaces the original messages. The `Session` struct persists conversations in JSONL format under `~/.autocode/sessions/<workspace-hash>/`, with one line per message and append-only writes for crash safety.

The AutoLang port (Phase 6A) uses VM FFI to bridge the gap. Seven FFI functions were added: `Process.spawn_with_output` (NATIVE_ID 1305), `http_post_stream_with_headers` (2255), `Regex.is_match`/`find_all` (2400-2401), `File.walk`/`append_text`/`read_lines` (1010-1012). The AutoLang version lives in `D:\autostack\auto-coder\` with modules for CLI/REPL, agent loop, LLM client, provider adapters, tools, context compression, and session persistence.

The a2r transpiler enhancement track (Phase 6B) systematically closes the feature gap between what AutoLang can express and what Rust requires. The work has been organized in four batches. Batch one (core structures, complete) added pub visibility, static fn (associated functions without self), `&mut self` methods, per-field serde attributes, and `#[tokio::main]` support. Batch two (trait system, complete) added `impl Trait for Type` for external traits and generic constraints via `#[with(T as Trait)]`. Batch three (advanced features, mostly complete) added struct destructuring in match, `impl From<A> for B`, const declarations, `Box::new()`/`Arc::new()` wrapping, `shared` keyword for static lazy initialization, and multi-file module system with automatic `mod X;` and `Cargo.toml` generation. Batch four (remaining gaps) includes `impl Into<String>` parameters, complex closure type inference, `String` vs `&str` distinction, lifetime annotations, and `Result<T>` error handling chains.

## Open Questions

1. **Reedline Tab integration**: The completion system is implemented but not yet bound to the Tab key in the reedline REPL. This requires implementing the `Completer` trait for reedline and activating history expansion.
2. **I/O redirection**: The `>`, `>>`, and `<` operators are not yet implemented. Pipeline integration with file descriptors remains a design decision.
3. **LLM API streaming dependency**: Plan 153 Phase 1 depends on Plan 152 (streaming HTTP and SSE parsing), which is a blocking dependency.
4. **a2r HashMap literal transpilation**: Map literals with type annotations still output struct syntax instead of `HashMap::from([(k, v), ...])`. This requires type context propagation through the transpiler.
5. **Agent granularity switching**: The automatic Task-to-Process escalation criteria and runtime mechanism need further specification before implementation.
6. **AutoShell configuration system**: Config file loading (`~/.config/auto-shell/config.at`), customizable prompts, and alias management were deferred from the initial implementation.

## Source Plans

- Plan 017: AutoShell Design (`docs/plans/017-auto-shell-design.md`)
- Plan 046: AutoShell ls Flags (`docs/plans/046-auto-shell-ls-flags.md`)
- Plan 047: AutoShell Value Pipelines (`docs/plans/047-auto-value-pipelines.md`)
- Plan 153: AutoShell AI Agent Design (`docs/plans/153-autoshell-ai-agent-design.md`)
- Plan 159: AutoCode Coding Agent (`docs/plans/159-autocode-coding-agent.md`)

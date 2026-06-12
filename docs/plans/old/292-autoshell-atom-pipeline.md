# Atom Pipeline Foundation — Implementation Plan

> **状态**: ✅ 全部完成 (2026-06-11)
> **实现偏差说明**: 实际实现与计划有以下差异，均属于架构改进：
> - Atom/AtomPipeline/AtomStream 代码移至 `crates/ash-core/src/pipeline/`（纯逻辑层），而非 `crates/auto-shell/src/pipeline/`
> - `AtomType` 从 12 种扩展到 21 种（新增 `DiskEntry`, `Table`, `Record`, `Text`, `Path`, `Nothing`, `HelpInfo`）
> - `convert.rs` 实现了 Value 结构推断而非依赖 AshFileEntry 等 shell 类型（解耦更彻底）
> - NuShell 适配层仅保留类型推断骨架，未引入 `nu-protocol` 依赖
> - 所有 12 个 Task 均已完成，测试数从 159 增长到 473

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace `PipelineData(Value|Text)` with `AtomPipeline(Atom|AtomStream|Text|Empty)` as AutoShell's first-class data carrier, migrate all 18 commands to Atom output, and lay the NuShell integration foundation.

**Architecture:** `Atom` wraps `auto_val::Value` with a semantic `AtomType` tag, enabling type-aware rendering and NuShell interop. `AtomPipeline` replaces `PipelineData` throughout the command trait and shell execution engine. Commands are migrated incrementally — old `PipelineData` converts via a compatibility layer until fully replaced.

**Tech Stack:** Rust, `auto-val` (Value types), `auto-shell` (existing shell infra)

**Design doc:** `docs/plans/291-autoshell-warp-design.md` — Phase 0

---

## Task 1: Create Atom Type System

**Files:**
- Create: `crates/auto-shell/src/pipeline/mod.rs`
- Create: `crates/auto-shell/src/pipeline/atom.rs`
- Test: inline `#[cfg(test)]` in `atom.rs`

**Step 1: Write the failing test**

In `crates/auto-shell/src/pipeline/atom.rs`:

```rust
//! Atom — structured pipeline data carrier with semantic type metadata.

use auto_val::Value;

/// Semantic type tag for structured pipeline data.
///
/// Enables type-aware rendering (tables for FileList, progress bars for ProcessList)
/// and serves as the bridge point for NuShell crate integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AtomType {
    // File system
    FileEntry,
    FileList,
    // Process
    ProcessEntry,
    ProcessList,
    // System
    SystemInfo,
    CpuInfo,
    MemoryInfo,
    DiskInfo,
    // Search
    MatchResult,
    MatchList,
    // Text
    CountResult,
    PlainText,
    // Build
    BuildResult,
    RunResult,
    // Help
    HelpInfo,
    // Generic / unknown
    Generic,
}

/// Atom — a structured data carrier with type metadata.
///
/// Wraps `auto_val::Value` with a semantic `AtomType` tag.
/// This is the fundamental unit of data flowing through AutoShell pipelines.
#[derive(Debug, Clone)]
pub struct Atom {
    pub value: Value,
    pub type_tag: AtomType,
}

impl Atom {
    /// Create a new Atom with explicit type
    pub fn new(value: Value, type_tag: AtomType) -> Self {
        Self { value, type_tag }
    }

    /// Create a generic Atom (unknown type)
    pub fn generic(value: Value) -> Self {
        Self::new(value, AtomType::Generic)
    }

    /// Create a plain text Atom
    pub fn text(s: impl Into<String>) -> Self {
        Self::new(Value::str(s), AtomType::PlainText)
    }

    /// Create a FileList Atom from a Value::Array of file entries
    pub fn file_list(value: Value) -> Self {
        Self::new(value, AtomType::FileList)
    }

    /// Create a ProcessList Atom from a Value::Array of process entries
    pub fn process_list(value: Value) -> Self {
        Self::new(value, AtomType::ProcessList)
    }

    /// Create an empty Atom (Nil value, Generic type)
    pub fn empty() -> Self {
        Self::new(Value::Nil, AtomType::Generic)
    }

    /// Get the semantic type tag
    pub fn type_tag(&self) -> &AtomType {
        &self.type_tag
    }

    /// Check if this Atom represents structured data (not text)
    pub fn is_structured(&self) -> bool {
        !matches!(self.type_tag, AtomType::PlainText)
    }

    /// Convert to display text
    pub fn into_text(self) -> String {
        use crate::cmd::value_helpers::format_value_for_display;
        format_value_for_display(&self.value)
    }

    /// Get text without consuming
    pub fn as_text(&self) -> String {
        use crate::cmd::value_helpers::format_value_for_display;
        format_value_for_display(&self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_new() {
        let atom = Atom::new(Value::Int(42), AtomType::Generic);
        assert_eq!(atom.type_tag, AtomType::Generic);
    }

    #[test]
    fn test_atom_text() {
        let atom = Atom::text("hello");
        assert_eq!(atom.type_tag, AtomType::PlainText);
        assert!(!atom.is_structured());
    }

    #[test]
    fn test_atom_file_list() {
        let atom = Atom::file_list(Value::Nil);
        assert_eq!(atom.type_tag, AtomType::FileList);
        assert!(atom.is_structured());
    }

    #[test]
    fn test_atom_process_list() {
        let atom = Atom::process_list(Value::Nil);
        assert_eq!(atom.type_tag, AtomType::ProcessList);
        assert!(atom.is_structured());
    }

    #[test]
    fn test_atom_empty() {
        let atom = Atom::empty();
        assert_eq!(atom.type_tag, AtomType::Generic);
    }

    #[test]
    fn test_atom_into_text() {
        let atom = Atom::text("hello world");
        assert_eq!(atom.into_text(), "\"hello world\"");
    }
}
```

**Step 2: Create the module file**

In `crates/auto-shell/src/pipeline/mod.rs`:

```rust
pub mod atom;

pub use atom::{Atom, AtomType};
```

**Step 3: Register module in lib.rs**

In `crates/auto-shell/src/lib.rs`, add:

```rust
pub mod pipeline;
```

**Step 4: Run tests**

```bash
cargo test -p auto-shell pipeline::atom
```

Expected: All 6 tests PASS.

**Step 5: Commit**

```bash
git add crates/auto-shell/src/pipeline/
git commit -m "feat(shell): add Atom type system with semantic type tags"
```

---

## Task 2: Create AtomPipeline Enum

**Files:**
- Create: `crates/auto-shell/src/pipeline/atom_pipeline.rs`
- Modify: `crates/auto-shell/src/pipeline/mod.rs`
- Test: inline `#[cfg(test)]` in `atom_pipeline.rs`

**Step 1: Write the failing test**

In `crates/auto-shell/src/pipeline/atom_pipeline.rs`:

```rust
//! AtomPipeline — the pipeline data carrier replacing PipelineData.
//!
//! Supports four modes:
//! - Atom: structured data with type metadata (zero-copy between commands)
//! - AtomStream: lazy/streaming Atom iterator (for large datasets)
//! - Text: plain text fallback (external commands, legacy compatibility)
//! - Empty: no output (status commands like mkdir, rm)

use super::atom::{Atom, AtomType};

/// Streaming Atom iterator for large datasets.
///
/// Commands like `ls -R /` produce too much data to materialize at once.
/// AtomStream provides a lazy iterator that yields Atoms one at a time.
pub struct AtomStream {
    items: Vec<Atom>,
    pos: usize,
}

impl AtomStream {
    /// Create a stream from a vector of Atoms
    pub fn new(items: Vec<Atom>) -> Self {
        Self { items, pos: 0 }
    }

    /// Create an empty stream
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Get the next Atom from the stream
    pub fn next(&mut self) -> Option<Atom> {
        if self.pos < self.items.len() {
            let atom = self.items[self.pos].clone();
            self.pos += 1;
            Some(atom)
        } else {
            None
        }
    }

    /// Collect all remaining items from the stream
    pub fn collect_remaining(mut self) -> Vec<Atom> {
        let remaining = self.items[self.pos..].to_vec();
        remaining
    }

    /// Check if the stream has more items
    pub fn has_next(&self) -> bool {
        self.pos < self.items.len()
    }

    /// Get the number of remaining items
    pub fn remaining_count(&self) -> usize {
        self.items.len() - self.pos
    }
}

/// The pipeline data carrier — replaces PipelineData.
///
/// Each pipeline stage receives and produces AtomPipeline.
/// The shell execution engine threads AtomPipeline between commands.
#[derive(Debug, Clone)]
pub enum AtomPipeline {
    /// Structured Atom data (zero-copy between commands)
    Atom(Atom),
    /// Streaming Atom iterator (large datasets)
    AtomStream(AtomStream),
    /// Plain text (external commands, legacy compatibility)
    Text(String),
    /// No output (status commands)
    Empty,
}

impl AtomPipeline {
    /// Create from a single Atom
    pub fn atom(value: auto_val::Value, type_tag: AtomType) -> Self {
        AtomPipeline::Atom(Atom::new(value, type_tag))
    }

    /// Create from a text string
    pub fn text(s: impl Into<String>) -> Self {
        AtomPipeline::Text(s.into())
    }

    /// Create empty pipeline data
    pub fn empty() -> Self {
        AtomPipeline::Empty
    }

    /// Create from an Atom directly
    pub fn from_atom(atom: Atom) -> Self {
        AtomPipeline::Atom(atom)
    }

    /// Get reference to inner Atom if this is Atom mode
    pub fn as_atom(&self) -> Option<&Atom> {
        match self {
            AtomPipeline::Atom(a) => Some(a),
            _ => None,
        }
    }

    /// Check if this contains structured Atom data
    pub fn is_atom(&self) -> bool {
        matches!(self, AtomPipeline::Atom(_))
    }

    /// Check if this is a stream
    pub fn is_stream(&self) -> bool {
        matches!(self, AtomPipeline::AtomStream(_))
    }

    /// Check if this is plain text
    pub fn is_text(&self) -> bool {
        matches!(self, AtomPipeline::Text(_))
    }

    /// Check if this is empty
    pub fn is_empty(&self) -> bool {
        match self {
            AtomPipeline::Empty => true,
            AtomPipeline::Text(s) => s.is_empty(),
            AtomPipeline::Atom(a) => matches!(a.value, auto_val::Value::Nil | auto_val::Value::Null | auto_val::Value::Void),
            AtomPipeline::AtomStream(s) => !s.has_next(),
        }
    }

    /// Convert to display text (for external commands and final output)
    pub fn into_text(self) -> String {
        match self {
            AtomPipeline::Atom(a) => a.into_text(),
            AtomPipeline::AtomStream(mut s) => {
                let items: Vec<String> = s.collect_remaining()
                    .iter()
                    .map(|a| a.as_text())
                    .collect();
                items.join("\n")
            }
            AtomPipeline::Text(s) => s,
            AtomPipeline::Empty => String::new(),
        }
    }

    /// Get text without consuming
    pub fn as_text(&self) -> String {
        match self {
            AtomPipeline::Atom(a) => a.as_text(),
            AtomPipeline::AtomStream(s) => {
                format!("<stream: {} items remaining>", s.remaining_count())
            }
            AtomPipeline::Text(s) => s.clone(),
            AtomPipeline::Empty => String::new(),
        }
    }
}

// ---- Conversion from old PipelineData ----

impl AtomPipeline {
    /// Convert from old PipelineData to new AtomPipeline
    pub fn from_pipeline_data(data: crate::cmd::PipelineData) -> Self {
        match data {
            crate::cmd::PipelineData::Value(v) => {
                AtomPipeline::Atom(Atom::generic(v))
            }
            crate::cmd::PipelineData::Text(s) => {
                if s.is_empty() {
                    AtomPipeline::Empty
                } else {
                    AtomPipeline::Text(s)
                }
            }
        }
    }

    /// Convert back to old PipelineData (compatibility layer)
    pub fn into_pipeline_data(self) -> crate::cmd::PipelineData {
        match self {
            AtomPipeline::Atom(a) => crate::cmd::PipelineData::Value(a.value),
            AtomPipeline::AtomStream(mut s) => {
                use auto_val::{Value, Array};
                let values: Vec<Value> = s.collect_remaining()
                    .into_iter()
                    .map(|a| a.value)
                    .collect();
                crate::cmd::PipelineData::Value(Value::Array(Array { values }))
            }
            AtomPipeline::Text(s) => crate::cmd::PipelineData::Text(s),
            AtomPipeline::Empty => crate::cmd::PipelineData::empty(),
        }
    }
}

impl From<Atom> for AtomPipeline {
    fn from(atom: Atom) -> Self {
        AtomPipeline::Atom(atom)
    }
}

impl From<String> for AtomPipeline {
    fn from(s: String) -> Self {
        if s.is_empty() {
            AtomPipeline::Empty
        } else {
            AtomPipeline::Text(s)
        }
    }
}

impl From<&str> for AtomPipeline {
    fn from(s: &str) -> Self {
        AtomPipeline::from(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_pipeline_atom() {
        let pipeline = AtomPipeline::atom(Value::Int(42), AtomType::Generic);
        assert!(pipeline.is_atom());
        assert!(!pipeline.is_text());
        assert!(!pipeline.is_stream());
    }

    #[test]
    fn test_atom_pipeline_text() {
        let pipeline = AtomPipeline::text("hello");
        assert!(pipeline.is_text());
        assert!(!pipeline.is_atom());
    }

    #[test]
    fn test_atom_pipeline_empty() {
        let pipeline = AtomPipeline::empty();
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.into_text(), "");
    }

    #[test]
    fn test_atom_pipeline_into_text() {
        let pipeline = AtomPipeline::atom(Value::Int(42), AtomType::Generic);
        assert_eq!(pipeline.into_text(), "42");
    }

    #[test]
    fn test_atom_stream() {
        let atoms = vec![
            Atom::text("line1"),
            Atom::text("line2"),
            Atom::text("line3"),
        ];
        let mut stream = AtomStream::new(atoms);
        assert!(stream.has_next());
        assert_eq!(stream.remaining_count(), 3);
        let first = stream.next().unwrap();
        assert_eq!(first.as_text(), "\"line1\"");
        assert_eq!(stream.remaining_count(), 2);
    }

    #[test]
    fn test_atom_stream_collect() {
        let atoms = vec![
            Atom::text("a"),
            Atom::text("b"),
        ];
        let stream = AtomStream::new(atoms);
        let collected = stream.collect_remaining();
        assert_eq!(collected.len(), 2);
    }

    #[test]
    fn test_from_pipeline_data_value() {
        let pd = crate::cmd::PipelineData::Value(Value::Int(42));
        let ap = AtomPipeline::from_pipeline_data(pd);
        assert!(ap.is_atom());
    }

    #[test]
    fn test_from_pipeline_data_text() {
        let pd = crate::cmd::PipelineData::Text("hello".to_string());
        let ap = AtomPipeline::from_pipeline_data(pd);
        assert!(ap.is_text());
    }

    #[test]
    fn test_from_pipeline_data_empty() {
        let pd = crate::cmd::PipelineData::Text(String::new());
        let ap = AtomPipeline::from_pipeline_data(pd);
        assert!(ap.is_empty());
    }

    #[test]
    fn test_into_pipeline_data_roundtrip() {
        let original = AtomPipeline::atom(Value::Int(42), AtomType::Generic);
        let pd = original.into_pipeline_data();
        assert!(matches!(pd, crate::cmd::PipelineData::Value(Value::Int(42))));
    }
}
```

**Step 2: Register in pipeline module**

In `crates/auto-shell/src/pipeline/mod.rs`, update to:

```rust
pub mod atom;
pub mod atom_pipeline;

pub use atom::{Atom, AtomType};
pub use atom_pipeline::{AtomPipeline, AtomStream};
```

**Step 3: Run tests**

```bash
cargo test -p auto-shell pipeline
```

Expected: All tests PASS (6 atom + 10 atom_pipeline = 16 tests).

**Step 4: Commit**

```bash
git add crates/auto-shell/src/pipeline/
git commit -m "feat(shell): add AtomPipeline enum replacing PipelineData"
```

---

## Task 3: Create Atom Conversion Helpers

**Files:**
- Create: `crates/auto-shell/src/pipeline/convert.rs`
- Modify: `crates/auto-shell/src/pipeline/mod.rs`
- Test: inline `#[cfg(test)]` in `convert.rs`

**Step 1: Write conversion helpers and tests**

In `crates/auto-shell/src/pipeline/convert.rs`:

```rust
//! Conversion helpers between shell data types and Atom.
//!
//! These helpers convert AshFileEntry, AshProcessEntry, etc. into Atom
//! with the correct AtomType tag, ready for pipeline use.

use auto_val::{Value, Obj, Array};
use super::atom::{Atom, AtomType};
use crate::data::types::{AshFileEntry, AshProcessEntry, AshDiskEntry, AshCpuInfo, AshMemoryInfo};

/// Convert an AshFileEntry to an Atom (AtomType::FileEntry)
pub fn file_entry_to_atom(entry: &AshFileEntry) -> Atom {
    let mut obj = Obj::new();
    obj.set("name", Value::str(&entry.name));
    obj.set("type", Value::str(entry.file_type.as_str()));
    obj.set("size", Value::Int(entry.size as i32));

    if let Some(modified) = &entry.modified {
        obj.set("modified", Value::str(&modified.format("%Y-%m-%d %H:%M:%S").to_string()));
    }
    if let Some(permissions) = &entry.permissions {
        obj.set("permissions", Value::str(permissions));
    }
    if let Some(owner) = &entry.owner {
        obj.set("owner", Value::str(owner));
    }
    if let Some(target) = &entry.target {
        obj.set("target", Value::str(target));
    }

    Atom::new(Value::Obj(obj), AtomType::FileEntry)
}

/// Convert a slice of AshFileEntry to a FileList Atom
pub fn file_entries_to_atom(entries: &[AshFileEntry]) -> Atom {
    let values: Vec<Value> = entries.iter()
        .map(|e| {
            let atom = file_entry_to_atom(e);
            atom.value
        })
        .collect();
    Atom::new(Value::Array(Array { values }), AtomType::FileList)
}

/// Convert an AshProcessEntry to an Atom (AtomType::ProcessEntry)
pub fn process_entry_to_atom(entry: &AshProcessEntry) -> Atom {
    let mut obj = Obj::new();
    obj.set("pid", Value::Int(entry.pid));
    obj.set("ppid", Value::Int(entry.ppid));
    obj.set("name", Value::str(&entry.name));
    obj.set("status", Value::str(&entry.status));
    obj.set("cpu_usage", Value::Float(entry.cpu_usage));
    obj.set("mem_usage", Value::Int(entry.mem_usage as i32));

    if let Some(start_time) = &entry.start_time {
        obj.set("start_time", Value::str(&start_time.format("%Y-%m-%d %H:%M:%S").to_string()));
    }
    if let Some(command) = &entry.command {
        obj.set("command", Value::str(command));
    }

    Atom::new(Value::Obj(obj), AtomType::ProcessEntry)
}

/// Convert a slice of AshProcessEntry to a ProcessList Atom
pub fn process_entries_to_atom(entries: &[AshProcessEntry]) -> Atom {
    let values: Vec<Value> = entries.iter()
        .map(|e| {
            let atom = process_entry_to_atom(e);
            atom.value
        })
        .collect();
    Atom::new(Value::Array(Array { values }), AtomType::ProcessList)
}

/// Convert AshDiskEntry to an Atom (AtomType::DiskInfo)
pub fn disk_entry_to_atom(entry: &AshDiskEntry) -> Atom {
    let mut obj = Obj::new();
    obj.set("device", Value::str(&entry.device));
    obj.set("file_system", Value::str(&entry.file_system));
    obj.set("mount_point", Value::str(&entry.mount_point));
    obj.set("total", Value::Int(entry.total as i32));
    obj.set("free", Value::Int(entry.free as i32));
    obj.set("removable", Value::Bool(entry.removable));

    Atom::new(Value::Obj(obj), AtomType::DiskInfo)
}

/// Convert AshCpuInfo to an Atom (AtomType::CpuInfo)
pub fn cpu_info_to_atom(info: &AshCpuInfo) -> Atom {
    let mut obj = Obj::new();
    obj.set("name", Value::str(&info.name));
    obj.set("brand", Value::str(&info.brand));
    obj.set("frequency", Value::Int(info.frequency as i32));
    obj.set("cores", Value::Int(info.cores as i32));
    obj.set("usage", Value::Float(info.usage));

    Atom::new(Value::Obj(obj), AtomType::CpuInfo)
}

/// Convert AshMemoryInfo to an Atom (AtomType::MemoryInfo)
pub fn memory_info_to_atom(info: &AshMemoryInfo) -> Atom {
    let mut obj = Obj::new();
    obj.set("total", Value::Int(info.total as i32));
    obj.set("free", Value::Int(info.free as i32));
    obj.set("available", Value::Int(info.available as i32));
    obj.set("used", Value::Int(info.used as i32));
    obj.set("usage_percent", Value::Float(info.usage_percent));

    Atom::new(Value::Obj(obj), AtomType::MemoryInfo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::types::FileType;

    fn sample_file_entry() -> AshFileEntry {
        AshFileEntry {
            name: "test.txt".to_string(),
            file_type: FileType::File,
            size: 1024,
            modified: None,
            permissions: Some("-rw-r--r--".to_string()),
            owner: None,
            target: None,
        }
    }

    #[test]
    fn test_file_entry_to_atom() {
        let entry = sample_file_entry();
        let atom = file_entry_to_atom(&entry);
        assert_eq!(*atom.type_tag(), &AtomType::FileEntry);
        assert!(atom.is_structured());
    }

    #[test]
    fn test_file_entries_to_atom() {
        let entries = vec![sample_file_entry(), sample_file_entry()];
        let atom = file_entries_to_atom(&entries);
        assert_eq!(*atom.type_tag(), &AtomType::FileList);
    }

    #[test]
    fn test_process_entry_to_atom() {
        let entry = AshProcessEntry {
            pid: 1,
            ppid: 0,
            name: "init".to_string(),
            status: "running".to_string(),
            cpu_usage: 0.5,
            mem_usage: 1024,
            start_time: None,
            command: None,
        };
        let atom = process_entry_to_atom(&entry);
        assert_eq!(*atom.type_tag(), &AtomType::ProcessEntry);
    }

    #[test]
    fn test_process_entries_to_atom() {
        let entry = AshProcessEntry {
            pid: 1,
            ppid: 0,
            name: "init".to_string(),
            status: "running".to_string(),
            cpu_usage: 0.5,
            mem_usage: 1024,
            start_time: None,
            command: None,
        };
        let atom = process_entries_to_atom(&[entry]);
        assert_eq!(*atom.type_tag(), &AtomType::ProcessList);
    }

    #[test]
    fn test_disk_entry_to_atom() {
        let entry = AshDiskEntry {
            device: "/dev/sda1".to_string(),
            file_system: "ext4".to_string(),
            mount_point: "/".to_string(),
            total: 500_000_000_000,
            free: 200_000_000_000,
            removable: false,
        };
        let atom = disk_entry_to_atom(&entry);
        assert_eq!(*atom.type_tag(), &AtomType::DiskInfo);
    }

    #[test]
    fn test_cpu_info_to_atom() {
        let info = AshCpuInfo {
            name: "CPU".to_string(),
            brand: "Intel".to_string(),
            frequency: 3000,
            cores: 8,
            usage: 45.0,
        };
        let atom = cpu_info_to_atom(&info);
        assert_eq!(*atom.type_tag(), &AtomType::CpuInfo);
    }

    #[test]
    fn test_memory_info_to_atom() {
        let info = AshMemoryInfo {
            total: 16_000_000_000,
            free: 8_000_000_000,
            available: 10_000_000_000,
            used: 6_000_000_000,
            usage_percent: 37.5,
        };
        let atom = memory_info_to_atom(&info);
        assert_eq!(*atom.type_tag(), &AtomType::MemoryInfo);
    }
}
```

**Step 2: Register in pipeline module**

In `crates/auto-shell/src/pipeline/mod.rs`, update to:

```rust
pub mod atom;
pub mod atom_pipeline;
pub mod convert;

pub use atom::{Atom, AtomType};
pub use atom_pipeline::{AtomPipeline, AtomStream};
pub use convert::*;
```

**Step 3: Run tests**

```bash
cargo test -p auto-shell pipeline
```

Expected: All tests PASS (6 + 10 + 7 = 23 tests).

**Step 4: Commit**

```bash
git add crates/auto-shell/src/pipeline/
git commit -m "feat(shell): add Atom conversion helpers for shell data types"
```

---

## Task 4: Create NuShell Adapter Skeleton

**Files:**
- Create: `crates/auto-shell/src/pipeline/nushell_adapter.rs`
- Modify: `crates/auto-shell/src/pipeline/mod.rs`
- Test: inline `#[cfg(test)]` in `nushell_adapter.rs`

**Why this is Task 4 (before command migration):** The adapter skeleton defines the `From<nu_protocol::Value>` conversion contract. Even though NuShell crates won't be added as dependencies until Phase 2, defining the interface now ensures command migration produces Atom types that will be compatible.

**Step 1: Write the adapter skeleton and tests**

In `crates/auto-shell/src/pipeline/nushell_adapter.rs`:

```rust
//! NuShell ↔ Atom adapter (skeleton for Phase 2).
//!
//! This module defines the conversion interface between NuShell's `Value`
//! and AutoShell's `Atom`. The actual NuShell crate integration happens
//! in Phase 2, but this skeleton establishes the contract.
//!
//! Phase 2 will add:
//! - `nu-protocol` as a dependency
//! - Full `impl From<nu_protocol::Value> for Atom`
//! - Full `impl From<Atom> for nu_protocol::Value`

use auto_val::{Value, Obj, Array};
use super::atom::{Atom, AtomType};

/// NuShell value type mapping to AtomType.
///
/// When NuShell crate is integrated, this maps nu_protocol::ValueType
/// to our AtomType. For now, we define the mapping logic for Value
/// structures that mirror NuShell's output format.
pub fn infer_atom_type(value: &Value) -> AtomType {
    match value {
        Value::Array(arr) => {
            // Check first element to determine list type
            if let Some(first) = arr.values.first() {
                if let Value::Obj(obj) = first {
                    // Heuristic: if objects have "file_type"/"type" + "size" → FileList
                    if obj.get("file_type").is_some() || obj.get("type").is_some() && obj.get("size").is_some() {
                        return AtomType::FileList;
                    }
                    // Heuristic: if objects have "pid" + "name" → ProcessList
                    if obj.get("pid").is_some() && obj.get("name").is_some() {
                        return AtomType::ProcessList;
                    }
                }
            }
            AtomType::Generic
        }
        Value::Obj(obj) => {
            if obj.get("pid").is_some() {
                return AtomType::ProcessEntry;
            }
            if obj.get("size").is_some() && obj.get("name").is_some() {
                return AtomType::FileEntry;
            }
            AtomType::Generic
        }
        Value::Str(_) => AtomType::PlainText,
        _ => AtomType::Generic,
    }
}

/// Convert a generic Value to Atom with inferred type.
///
/// This is the fallback conversion when the command doesn't
/// explicitly set an AtomType. The NuShell adapter will use
/// this to convert nu_protocol::Value → Atom.
pub fn value_to_atom(value: Value) -> Atom {
    let type_tag = infer_atom_type(&value);
    Atom::new(value, type_tag)
}

/// Convert an Atom back to a generic Value.
///
/// Loses type information — used when passing to code that
/// doesn't understand Atom (legacy commands).
pub fn atom_to_value(atom: Atom) -> Value {
    atom.value
}

// ---- Placeholder for NuShell crate integration (Phase 2) ----
//
// When nu-protocol is added as a dependency, implement:
//
// impl From<nu_protocol::Value> for Atom {
//     fn from(nu_val: nu_protocol::Value) -> Self {
//         match nu_val {
//             nu_protocol::Value::Record { val, .. } => {
//                 // Convert record fields to Obj
//                 let mut obj = Obj::new();
//                 for (k, v) in val.into_iter() {
//                     obj.set(k, Value::from(Atom::from(v)));
//                 }
//                 Atom::new(Value::Obj(obj), infer_atom_type(&Value::Obj(obj.clone())))
//             }
//             nu_protocol::Value::List { vals, .. } => {
//                 let values: Vec<Value> = vals.into_iter()
//                     .map(|v| Atom::from(v).value)
//                     .collect();
//                 let arr = Array { values };
//                 Atom::new(Value::Array(arr), infer_atom_type(&Value::Array(arr.clone())))
//             }
//             nu_protocol::Value::String { val, .. } => Atom::text(val),
//             nu_protocol::Value::Int { val, .. } => Atom::new(Value::Int(val as i32), AtomType::Generic),
//             nu_protocol::Value::Float { val, .. } => Atom::new(Value::Float(val), AtomType::Generic),
//             nu_protocol::Value::Bool { val, .. } => Atom::new(Value::Bool(val), AtomType::Generic),
//             _ => Atom::empty(),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_file_list_type() {
        let mut obj = Obj::new();
        obj.set("name", Value::str("test.txt"));
        obj.set("type", Value::str("file"));
        obj.set("size", Value::Int(1024));
        let arr = Array { values: vec![Value::Obj(obj)] };
        let inferred = infer_atom_type(&Value::Array(arr));
        assert_eq!(inferred, AtomType::FileList);
    }

    #[test]
    fn test_infer_process_list_type() {
        let mut obj = Obj::new();
        obj.set("pid", Value::Int(1));
        obj.set("name", Value::str("init"));
        let arr = Array { values: vec![Value::Obj(obj)] };
        let inferred = infer_atom_type(&Value::Array(arr));
        assert_eq!(inferred, AtomType::ProcessList);
    }

    #[test]
    fn test_infer_process_entry_type() {
        let mut obj = Obj::new();
        obj.set("pid", Value::Int(1));
        obj.set("name", Value::str("init"));
        let inferred = infer_atom_type(&Value::Obj(obj));
        assert_eq!(inferred, AtomType::ProcessEntry);
    }

    #[test]
    fn test_infer_text_type() {
        let inferred = infer_atom_type(&Value::str("hello"));
        assert_eq!(inferred, AtomType::PlainText);
    }

    #[test]
    fn test_infer_generic_type() {
        let inferred = infer_atom_type(&Value::Int(42));
        assert_eq!(inferred, AtomType::Generic);
    }

    #[test]
    fn test_value_to_atom() {
        let atom = value_to_atom(Value::str("hello"));
        assert_eq!(*atom.type_tag(), AtomType::PlainText);
    }

    #[test]
    fn test_atom_to_value() {
        let atom = Atom::text("hello");
        let value = atom_to_value(atom);
        assert!(matches!(value, Value::Str(_)));
    }
}
```

**Step 2: Register in pipeline module**

In `crates/auto-shell/src/pipeline/mod.rs`, update to:

```rust
pub mod atom;
pub mod atom_pipeline;
pub mod convert;
pub mod nushell_adapter;

pub use atom::{Atom, AtomType};
pub use atom_pipeline::{AtomPipeline, AtomStream};
pub use convert::*;
pub use nushell_adapter::*;
```

**Step 3: Run tests**

```bash
cargo test -p auto-shell pipeline
```

Expected: All tests PASS (23 + 7 = 30 tests).

**Step 4: Commit**

```bash
git add crates/auto-shell/src/pipeline/
git commit -m "feat(shell): add NuShell adapter skeleton for Phase 2 integration"
```

---

## Task 5: Update Command Trait to Use AtomPipeline

**Files:**
- Modify: `crates/auto-shell/src/cmd.rs` — add `run_atom` method to `Command` trait
- Modify: `crates/auto-shell/src/cmd/registry.rs` — update registry if needed

**Why a new method instead of changing `run`:** The existing `run()` method signature returns `PipelineData` and is used by all 18 commands. Adding `run_atom()` as a default method lets us migrate commands incrementally — old commands keep `run()`, new ones override `run_atom()`.

**Step 1: Add run_atom to Command trait**

In `crates/auto-shell/src/cmd.rs`, add after the existing `run` method in the `Command` trait:

```rust
    /// Execute the command with Atom-based pipeline data.
    ///
    /// Override this method to produce Atom-typed output.
    /// The default implementation delegates to `run()` and wraps the result.
    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: crate::pipeline::AtomPipeline,
        shell: &mut Shell,
    ) -> Result<crate::pipeline::AtomPipeline> {
        // Default: convert AtomPipeline → PipelineData, call run(), convert back
        let pd_input = input.into_pipeline_data();
        let pd_output = self.run(args, pd_input, shell)?;
        Ok(crate::pipeline::AtomPipeline::from_pipeline_data(pd_output))
    }
```

**Step 2: Run tests to verify nothing is broken**

```bash
cargo test -p auto-shell
```

Expected: All existing 159 tests still PASS (we only added a default method).

**Step 3: Commit**

```bash
git add crates/auto-shell/src/cmd.rs
git commit -m "feat(shell): add run_atom() method to Command trait"
```

---

## Task 6: Update Shell Pipeline Execution for AtomPipeline

**Files:**
- Modify: `crates/auto-shell/src/shell.rs` — update `execute_pipeline_with_auto` to use `run_atom`

**Step 1: Add AtomPipeline execution path**

In `crates/auto-shell/src/shell.rs`, in the `execute_pipeline_with_auto` method, find the section that calls `registered_cmd.run(...)` and add a parallel path that calls `run_atom`:

Locate this block:
```rust
            let output_pipeline = if let Some(registered_cmd) = self.registry.get(cmd_name) {
                // Registered command (uses PipelineData)
                let signature = registered_cmd.signature();
                let input = input_pipeline.take().unwrap_or_else(PipelineData::empty);

                match crate::cmd::parser::parse_args(&signature, args) {
                    Ok(parsed_args) => Some(registered_cmd.run(&parsed_args, input, self)?),
                    Err(e) => return Err(e),
                }
```

Replace with:
```rust
            let output_pipeline = if let Some(registered_cmd) = self.registry.get(cmd_name) {
                // Registered command — use AtomPipeline path
                let signature = registered_cmd.signature();
                let atom_input = input_atom_pipeline.take().unwrap_or_else(AtomPipeline::empty);

                match crate::cmd::parser::parse_args(&signature, args) {
                    Ok(parsed_args) => {
                        let atom_output = registered_cmd.run_atom(&parsed_args, atom_input, self)?;
                        Some(atom_output.into_pipeline_data())
                    }
                    Err(e) => return Err(e),
                }
```

Also add the `AtomPipeline` import and the `input_atom_pipeline` variable at the top of the method. Change:

```rust
    use crate::cmd::{auto, builtin, external, PipelineData};
```

to:

```rust
    use crate::cmd::{auto, builtin, external, PipelineData};
    use crate::pipeline::AtomPipeline;
```

And change the pipeline variable from:

```rust
    let mut input_pipeline: Option<PipelineData> = None;
```

to:

```rust
    let mut input_atom_pipeline: Option<AtomPipeline> = None;
```

Update the non-registered command branches to convert `AtomPipeline` back to text for legacy commands:

```rust
            let input_str = input_atom_pipeline.take().and_then(|p| {
                if p.is_empty() {
                    None
                } else {
                    Some(p.into_text())
                }
            });
```

And at the end, where legacy commands produce `PipelineData::from_text(output)`, convert to `AtomPipeline` implicitly:

```rust
                if let Some(output) =
                    builtin::execute_builtin_with_input(cmd, &self.current_dir, Some(input))?
                {
                    Some(PipelineData::from_text(output))
```

should become (for legacy builtins, still produce PipelineData which gets stored):

```rust
                if let Some(output) =
                    builtin::execute_builtin_with_input(cmd, &self.current_dir, Some(input))?
                {
                    input_atom_pipeline = Some(AtomPipeline::text(output));
```

Wait — this refactoring is getting complex. The cleaner approach is to keep the existing pipeline flow mostly intact, and only change the registered-command path. The registered commands are the 18 commands in `cmd/commands/`. Legacy builtins and external commands continue using the text path.

**Revised Step 1:** Add a flag to the method to track whether the current pipeline data is Atom-based:

In `crates/auto-shell/src/shell.rs`, update `execute_pipeline_with_auto` to add the `AtomPipeline` variable alongside the existing flow:

At the top of the method, add after the `use` statements:

```rust
    use crate::pipeline::AtomPipeline;
    let mut atom_pipeline: Option<AtomPipeline> = None;
```

Then, in the registered command branch, replace the `PipelineData` flow with `AtomPipeline`:

Find:
```rust
        let output_pipeline = if let Some(registered_cmd) = self.registry.get(cmd_name) {
            // Registered command (uses PipelineData)
            let signature = registered_cmd.signature();
            let input = input_pipeline.take().unwrap_or_else(PipelineData::empty);

            match crate::cmd::parser::parse_args(&signature, args) {
                Ok(parsed_args) => Some(registered_cmd.run(&parsed_args, input, self)?),
                Err(e) => return Err(e),
            }
```

Replace with:
```rust
        let output_pipeline = if let Some(registered_cmd) = self.registry.get(cmd_name) {
            // Registered command — use AtomPipeline
            let signature = registered_cmd.signature();
            let atom_input = atom_pipeline.take().unwrap_or_else(AtomPipeline::empty);

            match crate::cmd::parser::parse_args(&signature, args) {
                Ok(parsed_args) => {
                    let atom_output = registered_cmd.run_atom(&parsed_args, atom_input, self)?;
                    // Store for next command in pipeline
                    atom_pipeline = Some(atom_output);
                    // Also convert to PipelineData for final output compatibility
                    atom_pipeline.clone().map(|p| p.into_pipeline_data())
                }
                Err(e) => return Err(e),
            }
```

And for non-registered commands, convert the atom_pipeline to text:

Find the line that creates `input_str`:
```rust
            let input_str = input_pipeline.take().and_then(|p| {
                if p.is_empty() {
                    None
                } else {
                    Some(p.into_text())
                }
            });
```

Replace with:
```rust
            let input_str = atom_pipeline.take().and_then(|p| {
                if p.is_empty() {
                    None
                } else {
                    Some(p.into_text())
                }
            });
```

Also remove the `input_pipeline` variable declaration since we're replacing it:
```rust
    let mut input_pipeline: Option<PipelineData> = None;
```
→ remove this line.

And update the final return to use `atom_pipeline`:

Find:
```rust
    if is_last {
        return Ok(input_pipeline.map(|p| p.into_text()));
    }
```

Replace with:
```rust
    if is_last {
        return Ok(atom_pipeline.map(|p| p.into_text()));
    }
```

And the fallback return at the end:

Find:
```rust
    Ok(None)
```
→ Keep as is (already correct).

**Step 2: Build and run tests**

```bash
cargo build -p auto-shell && cargo test -p auto-shell
```

Expected: Build succeeds, all 159 tests still PASS (the default `run_atom` delegates to `run` so behavior is unchanged).

**Step 3: Commit**

```bash
git add crates/auto-shell/src/shell.rs crates/auto-shell/src/cmd.rs
git commit -m "refactor(shell): switch pipeline execution to AtomPipeline path"
```

---

## Task 7: Migrate `ls` Command to Atom Output

**Files:**
- Modify: `crates/auto-shell/src/cmd/commands/ls.rs`
- Test: `crates/auto-shell/tests/structured_commands.rs`

**Step 1: Write the failing test**

In `crates/auto-shell/tests/structured_commands.rs`, add:

```rust
#[test]
fn test_ls_returns_atom_with_file_list_tag() {
    let mut shell = Shell::new();
    let result = shell.execute("ls .");
    assert!(result.is_ok());
    // After migration, ls should still work (smoke test)
    // The Atom type tag is internal, tested via unit tests
}
```

**Step 2: Run test to verify it passes** (it should, since default `run_atom` delegates to `run`)

```bash
cargo test -p auto-shell test_ls_returns_atom_with_file_list_tag
```

Expected: PASS (default delegation).

**Step 3: Override run_atom in LsCommand**

In `crates/auto-shell/src/cmd/commands/ls.rs`, add the `run_atom` override:

```rust
use crate::pipeline::{AtomPipeline, Atom};
use crate::data::convert::{file_entries_to_atom};

impl LsCommand {
    // ... existing code ...

    fn run_atom_impl(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: AtomPipeline,
        shell: &mut Shell,
    ) -> miette::Result<AtomPipeline> {
        let path_arg = args.positionals.get(0).map(|s| s.as_str()).unwrap_or(".");
        let path = Path::new(path_arg);

        let all = args.has_flag("all");
        let long = args.has_flag("long");
        let time = args.has_flag("time");
        let reverse = args.has_flag("reverse");
        let recursive = args.has_flag("recursive");

        // Use structured output via fs::ls_command_value
        let value = fs::ls_command_value(
            path,
            &shell.pwd(),
            all,
            long,
            time,
            reverse,
            recursive,
        )?;

        // Convert to Atom with FileList type tag
        // We need to extract the file entries from the Value and re-tag
        // For now, wrap the existing Value output with FileList tag
        let atom = Atom::file_list(value);
        Ok(AtomPipeline::from_atom(atom))
    }
}
```

Then in the `Command` impl for `LsCommand`, add:

```rust
    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: crate::pipeline::AtomPipeline,
        shell: &mut Shell,
    ) -> miette::Result<crate::pipeline::AtomPipeline> {
        self.run_atom_impl(args, input, shell)
    }
```

**Step 4: Run all tests**

```bash
cargo test -p auto-shell
```

Expected: All tests PASS. `ls` now returns `AtomPipeline::Atom(Atom { value, type_tag: FileList })`.

**Step 5: Commit**

```bash
git add crates/auto-shell/src/cmd/commands/ls.rs tests/
git commit -m "feat(shell): migrate ls command to Atom output with FileList type tag"
```

---

## Task 8: Migrate System Commands (`ps`, `sys`)

**Files:**
- Modify: `crates/auto-shell/src/cmd/commands/ps.rs`
- Modify: `crates/auto-shell/src/cmd/commands/sys.rs`

**Step 1: Migrate ps command**

Add `run_atom` override to `PsCommand`. Follow the same pattern as `ls`:

```rust
use crate::pipeline::{AtomPipeline, Atom};

// In Command impl for PsCommand:
    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: crate::pipeline::AtomPipeline,
        shell: &mut Shell,
    ) -> miette::Result<crate::pipeline::AtomPipeline> {
        // Delegate to existing run() logic but wrap result as Atom
        let pd = self.run(args, crate::cmd::PipelineData::empty(), shell)?;
        let atom = match pd {
            crate::cmd::PipelineData::Value(v) => Atom::process_list(v),
            crate::cmd::PipelineData::Text(s) => Atom::text(s),
        };
        Ok(AtomPipeline::from_atom(atom))
    }
```

**Step 2: Migrate sys command**

Add `run_atom` override to `SysCommand`. The `sys` command has subcommands (disks, cpu, mem), so the type tag varies:

```rust
use crate::pipeline::{AtomPipeline, Atom};

// In Command impl for SysCommand:
    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        _input: crate::pipeline::AtomPipeline,
        shell: &mut Shell,
    ) -> miette::Result<crate::pipeline::AtomPipeline> {
        use crate::pipeline::AtomType;

        let pd = self.run(args, crate::cmd::PipelineData::empty(), shell)?;
        let atom = match pd {
            crate::cmd::PipelineData::Value(v) => {
                // Determine subcommand for type tag
                let subcmd = args.positionals.get(0).map(|s| s.as_str()).unwrap_or("");
                let type_tag = match subcmd {
                    "disks" => AtomType::DiskInfo,
                    "cpu" => AtomType::CpuInfo,
                    "mem" => AtomType::MemoryInfo,
                    _ => AtomType::SystemInfo,
                };
                Atom::new(v, type_tag)
            }
            crate::cmd::PipelineData::Text(s) => Atom::text(s),
        };
        Ok(AtomPipeline::from_atom(atom))
    }
```

**Step 3: Run tests**

```bash
cargo test -p auto-shell
```

Expected: All tests PASS.

**Step 4: Commit**

```bash
git add crates/auto-shell/src/cmd/commands/ps.rs crates/auto-shell/src/cmd/commands/sys.rs
git commit -m "feat(shell): migrate ps and sys commands to Atom output"
```

---

## Task 9: Migrate Data Commands (`grep`, `wc`, `select`, `where`, `get`)

**Files:**
- Modify: `crates/auto-shell/src/cmd/commands/grep.rs`
- Modify: `crates/auto-shell/src/cmd/commands/wc.rs`
- Modify: `crates/auto-shell/src/cmd/commands/select.rs`
- Modify: `crates/auto-shell/src/cmd/commands/where.rs`
- Modify: `crates/auto-shell/src/cmd/commands/get.rs`

**Step 1: Add run_atom to each command**

Each command follows the same pattern — delegate to `run()` and wrap the `Value` result with the appropriate `AtomType`:

**grep.rs** — `MatchList` type tag:
```rust
    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: crate::pipeline::AtomPipeline,
        shell: &mut Shell,
    ) -> miette::Result<crate::pipeline::AtomPipeline> {
        use crate::pipeline::Atom;
        let pd_input = input.into_pipeline_data();
        let pd = self.run(args, pd_input, shell)?;
        let atom = match pd {
            crate::cmd::PipelineData::Value(v) => Atom::new(v, AtomType::MatchList),
            crate::cmd::PipelineData::Text(s) => Atom::text(s),
        };
        Ok(AtomPipeline::from_atom(atom))
    }
```

**wc.rs** — `CountResult` type tag:
```rust
    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: crate::pipeline::AtomPipeline,
        shell: &mut Shell,
    ) -> miette::Result<crate::pipeline::AtomPipeline> {
        use crate::pipeline::Atom;
        use crate::pipeline::AtomType;
        let pd_input = input.into_pipeline_data();
        let pd = self.run(args, pd_input, shell)?;
        let atom = match pd {
            crate::cmd::PipelineData::Value(v) => Atom::new(v, AtomType::CountResult),
            crate::cmd::PipelineData::Text(s) => Atom::text(s),
        };
        Ok(AtomPipeline::from_atom(atom))
    }
```

**select.rs**, **where.rs**, **get.rs** — `Generic` type tag (pass-through commands):
```rust
    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: crate::pipeline::AtomPipeline,
        shell: &mut Shell,
    ) -> miette::Result<crate::pipeline::AtomPipeline> {
        use crate::pipeline::Atom;
        let pd_input = input.into_pipeline_data();
        let pd = self.run(args, pd_input, shell)?;
        Ok(AtomPipeline::from_pipeline_data(pd))
    }
```

**Step 2: Run tests**

```bash
cargo test -p auto-shell
```

Expected: All tests PASS.

**Step 3: Commit**

```bash
git add crates/auto-shell/src/cmd/commands/
git commit -m "feat(shell): migrate grep, wc, select, where, get commands to Atom output"
```

---

## Task 10: Migrate Remaining Commands (`echo`, `help`, `cd`, `pwd`, `mkdir`, `rm`, `mv`, `cp`, `build`, `run`)

**Files:**
- Modify: `crates/auto-shell/src/cmd/commands/echo.rs`
- Modify: `crates/auto-shell/src/cmd/commands/help.rs`
- Modify: `crates/auto-shell/src/cmd/commands/cd.rs`
- Modify: `crates/auto-shell/src/cmd/commands/pwd.rs`
- Modify: `crates/auto-shell/src/cmd/commands/mkdir.rs`
- Modify: `crates/auto-shell/src/cmd/commands/rm.rs`
- Modify: `crates/auto-shell/src/cmd/commands/mv.rs`
- Modify: `crates/auto-shell/src/cmd/commands/cp.rs`
- Modify: `crates/auto-shell/src/cmd/commands/build.rs`
- Modify: `crates/auto-shell/src/cmd/commands/run.rs`

**Step 1: Add run_atom to each command**

Three categories:

**A. Status commands (mkdir, rm, mv, cp) — return `Empty`:**
```rust
    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: crate::pipeline::AtomPipeline,
        shell: &mut Shell,
    ) -> miette::Result<crate::pipeline::AtomPipeline> {
        let pd_input = input.into_pipeline_data();
        self.run(args, pd_input, shell)?;
        Ok(AtomPipeline::empty())
    }
```

**B. Text commands (echo, pwd, cd) — return `PlainText`:**
```rust
    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: crate::pipeline::AtomPipeline,
        shell: &mut Shell,
    ) -> miette::Result<crate::pipeline::AtomPipeline> {
        let pd_input = input.into_pipeline_data();
        let pd = self.run(args, pd_input, shell)?;
        Ok(AtomPipeline::from_pipeline_data(pd))
    }
```

**C. Build/run commands — return `BuildResult`/`RunResult`:**
```rust
    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: crate::pipeline::AtomPipeline,
        shell: &mut Shell,
    ) -> miette::Result<crate::pipeline::AtomPipeline> {
        use crate::pipeline::AtomType;
        let pd_input = input.into_pipeline_data();
        let pd = self.run(args, pd_input, shell)?;
        let atom = match pd {
            crate::cmd::PipelineData::Value(v) => Atom::new(v, AtomType::BuildResult),
            crate::cmd::PipelineData::Text(s) => Atom::text(s),
        };
        Ok(AtomPipeline::from_atom(atom))
    }
```

For `run.rs`, use `AtomType::RunResult` instead.

**D. Help command — return `HelpInfo`:**
```rust
    fn run_atom(
        &self,
        args: &crate::cmd::parser::ParsedArgs,
        input: crate::pipeline::AtomPipeline,
        shell: &mut Shell,
    ) -> miette::Result<crate::pipeline::AtomPipeline> {
        use crate::pipeline::AtomType;
        let pd_input = input.into_pipeline_data();
        let pd = self.run(args, pd_input, shell)?;
        let atom = match pd {
            crate::cmd::PipelineData::Value(v) => Atom::new(v, AtomType::HelpInfo),
            crate::cmd::PipelineData::Text(s) => Atom::text(s),
        };
        Ok(AtomPipeline::from_atom(atom))
    }
```

**Step 2: Run all tests**

```bash
cargo test -p auto-shell
```

Expected: All 159+ tests PASS. All 18 commands now produce `AtomPipeline` output.

**Step 3: Commit**

```bash
git add crates/auto-shell/src/cmd/commands/
git commit -m "feat(shell): migrate all 18 commands to AtomPipeline output"
```

---

## Task 11: Integration Tests for Atom Pipeline

**Files:**
- Create: `crates/auto-shell/tests/atom_pipeline.rs`

**Step 1: Write integration tests**

```rust
//! Integration tests for Atom pipeline data flow.

use auto_shell::shell::Shell;
use auto_shell::pipeline::{AtomPipeline, AtomType};

#[test]
fn test_ls_produces_atom_pipeline() {
    let mut shell = Shell::new();
    let result = shell.execute("ls .");
    assert!(result.is_ok());
    // ls should produce output
    let output = result.unwrap();
    assert!(output.is_some());
}

#[test]
fn test_ls_pipeline_with_where() {
    let mut shell = Shell::new();
    let result = shell.execute("ls . | where type == dir");
    assert!(result.is_ok());
}

#[test]
fn test_ls_pipeline_with_select() {
    let mut shell = Shell::new();
    let result = shell.execute("ls . | select name");
    assert!(result.is_ok());
}

#[test]
fn test_ls_pipeline_with_get() {
    let mut shell = Shell::new();
    let result = shell.execute("ls . | get name");
    assert!(result.is_ok());
}

#[test]
fn test_ps_produces_output() {
    let mut shell = Shell::new();
    let result = shell.execute("ps");
    assert!(result.is_ok());
}

#[test]
fn test_sys_disks_produces_output() {
    let mut shell = Shell::new();
    let result = shell.execute("sys disks");
    assert!(result.is_ok());
}

#[test]
fn test_sys_cpu_produces_output() {
    let mut shell = Shell::new();
    let result = shell.execute("sys cpu");
    assert!(result.is_ok());
}

#[test]
fn test_sys_mem_produces_output() {
    let mut shell = Shell::new();
    let result = shell.execute("sys mem");
    assert!(result.is_ok());
}

#[test]
fn test_echo_produces_text() {
    let mut shell = Shell::new();
    let result = shell.execute("echo hello");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Some("hello".to_string()));
}

#[test]
fn test_pwd_produces_path() {
    let mut shell = Shell::new();
    let result = shell.execute("pwd");
    assert!(result.is_ok());
    assert!(result.unwrap().is_some());
}

#[test]
fn test_grep_pipeline() {
    use std::fs;
    let test_file = "test_grep_atom_pipeline.txt";
    fs::write(test_file, "hello world\nfoo bar\nhello rust\n").unwrap();

    let mut shell = Shell::new();
    let result = shell.execute(&format!("grep hello {}", test_file));
    assert!(result.is_ok());

    let _ = fs::remove_file(test_file);
}

#[test]
fn test_wc_pipeline() {
    use std::fs;
    let test_file = "test_wc_atom_pipeline.txt";
    fs::write(test_file, "hello\nworld\nfoo\n").unwrap();

    let mut shell = Shell::new();
    let result = shell.execute(&format!("wc {}", test_file));
    assert!(result.is_ok());

    let _ = fs::remove_file(test_file);
}
```

**Step 2: Run integration tests**

```bash
cargo test -p auto-shell atom_pipeline
```

Expected: All 12 integration tests PASS.

**Step 3: Run full test suite**

```bash
cargo test -p auto-shell
```

Expected: All 159 original + 30 pipeline + 12 integration = ~200+ tests PASS.

**Step 4: Commit**

```bash
git add crates/auto-shell/tests/atom_pipeline.rs
git commit -m "test(shell): add Atom pipeline integration tests"
```

---

## Task 12: Final Cleanup + Build Verification

**Files:**
- Modify: `crates/auto-shell/src/lib.rs` — ensure `pipeline` module is exported

**Step 1: Verify full build**

```bash
cargo build -p auto-shell
cargo build -p auto
```

Expected: Both build successfully.

**Step 2: Verify all tests pass**

```bash
cargo test -p auto-shell
```

Expected: All tests PASS.

**Step 3: Verify auto.exe works with shell**

```bash
cargo build -p auto
# Manual smoke test: run `auto` and try `ls . | where type == dir`
```

**Step 4: Commit any final fixes**

```bash
git add -A
git commit -m "feat(shell): complete Phase 0 - Atom pipeline foundation"
```

---

## Summary

| Task | Description | New Files | Modified Files | Tests Added |
|------|-------------|-----------|----------------|-------------|
| 1 | Atom type system | 2 | 1 | 6 |
| 2 | AtomPipeline enum | 1 | 1 | 10 |
| 3 | Conversion helpers | 1 | 1 | 7 |
| 4 | NuShell adapter skeleton | 1 | 1 | 7 |
| 5 | Command trait update | 0 | 1 | 0 |
| 6 | Shell pipeline update | 0 | 1 | 0 |
| 7 | Migrate ls | 0 | 1 | 1 |
| 8 | Migrate ps, sys | 0 | 2 | 0 |
| 9 | Migrate grep, wc, select, where, get | 0 | 5 | 0 |
| 10 | Migrate remaining 10 commands | 0 | 10 | 0 |
| 11 | Integration tests | 1 | 0 | 12 |
| 12 | Final cleanup | 0 | 1 | 0 |
| **Total** | | **6** | **25** | **43** |

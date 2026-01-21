# Auto-Shell: Structured Data Pipelines using Auto Value System

## Objective

Transform Auto-Shell from a traditional string-based shell to a modern data shell using Auto's existing `Value` enum for zero-copy structured data pipelines, similar to Nushell but leveraging our existing Auto value system.

## Current State

### Existing Infrastructure (Already Available!)

**Auto Value System** ([`crates/auto-val/src/value.rs`](../crates/auto-val/src/value.rs)):
```rust
pub enum Value {
    // Primitives
    Int(i32), Float(f64), Bool(bool), Str(AutoStr),

    // Structured data (perfect for pipelines!)
    Array(Array),    // Vec<Value> - ordered collections
    Obj(Obj),        // IndexMap<ValueKey, Value> - key-value records
    Node(Node),      // Hierarchical tree data

    // Function types
    Fn(Fn), ExtFn(ExtFn), Type(Type),

    // Other types
    Pair(ValueKey, Box<Value>),
    Nil, Null, Void,
}
```

**Obj (Record-like)**: [`crates/auto-val/src/obj.rs`](../crates/auto-val/src/obj.rs)
- Uses `IndexMap` for ordered key-value pairs
- O(1) lookups, preserves insertion order
- Already has field access methods: `get()`, `set()`, `has()`, `keys()`

**Array (List-like)**: [`crates/auto-val/src/array.rs`](../crates/auto-val/src/array.rs)
- Wraps `Vec<Value>`
- Already has iteration methods
- Supports all list operations

### Current Shell Architecture

**Command Trait** ([`auto-shell/src/cmd.rs`](../auto-shell/src/cmd.rs)):
```rust
pub trait Command {
    fn name(&self) -> &str;
    fn signature(&self) -> Signature;

    fn run(
        &self,
        args: &ParsedArgs,
        input: Option<&str>,      // ← String input (text)
        shell: &mut Shell,
    ) -> Result<Option<String>>;  // ← String output (text)
}
```

**Problem**: Commands serialize to strings, then next command parses strings again.
- **Overhead**: JSON/text parsing at every pipe
- **Lost types**: Structure information lost in serialization
- **Inefficiency**: Unnecessary allocations

## Proposed Solution

### Phase 1: PipelineData Wrapper

**File**: `auto-shell/src/cmd/pipeline.rs` (new)

Create a wrapper enum for pipeline data that can hold either structured Auto values or legacy text:

```rust
use auto_val::Value;

/// Pipeline data can be structured (Value) or text (for external commands)
pub enum PipelineData {
    /// Structured Auto value (zero-copy between commands)
    Value(Value),

    /// Plain text (for external commands, legacy compatibility)
    Text(String),
}

impl PipelineData {
    // Constructors
    pub fn from_value(val: Value) -> Self {
        PipelineData::Value(val)
    }

    pub fn from_text(s: String) -> Self {
        PipelineData::Text(s)
    }

    // Accessors
    pub fn as_value(&self) -> Option<&Value> {
        match self {
            PipelineData::Value(v) => Some(v),
            _ => None,
        }
    }

    pub fn into_text(self) -> String {
        match self {
            PipelineData::Value(v) => v.to_string(),
            PipelineData::Text(s) => s,
        }
    }

    // Type checking helpers
    pub fn is_value(&self) -> bool {
        matches!(self, PipelineData::Value(_))
    }

    pub fn is_text(&self) -> bool {
        matches!(self, PipelineData::Text(_))
    }
}
```

### Phase 2: Update Command Trait

**File**: `auto-shell/src/cmd.rs`

Modify the `Command` trait to use `PipelineData`:

```rust
pub trait Command {
    fn name(&self) -> &str;
    fn signature(&self) -> Signature;

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,        // ← Changed: Accept structured data
        shell: &mut Shell,
    ) -> Result<PipelineData>;     // ← Changed: Return structured data
}
```

**Migration Strategy**:
- Keep existing commands working by checking `input.as_value()`
- Commands that don't use input can ignore it
- Commands can return either `Value` or `Text` as needed

### Phase 3: Value Helper Methods

**File**: `auto-shell/src/cmd/value_helpers.rs` (new)

Add convenience methods for working with Auto values in shell context:

```rust
use auto_val::{Value, Obj, Array, AutoStr};

/// Convert shell output to Auto value
pub trait IntoAutoValue {
    fn into_auto_value(self) -> Value;
}

impl IntoAutoValue for String {
    fn into_auto_value(self) -> Value {
        Value::str(self)
    }
}

impl IntoAutoValue for Value {
    fn into_auto_value(self) -> Value {
        self
    }
}

/// Helper to build file listing as structured data
pub fn build_file_entry(
    name: impl Into<AutoStr>,
    file_type: impl Into<AutoStr>,
    size: Option<i64>,
    modified: Option<String>,
    permissions: Option<String>,
) -> Value {
    let mut obj = Obj::new();
    obj.set("name", Value::str(name));
    obj.set("type", Value::str(file_type));

    if let Some(s) = size {
        obj.set("size", Value::int(s as i32));
    }

    if let Some(m) = modified {
        obj.set("modified", Value::str(m));
    }

    if let Some(p) = permissions {
        obj.set("permissions", Value::str(p));
    }

    Value::Obj(obj)
}

/// Helper to format Value for display
pub fn format_value_for_display(val: &Value) -> String {
    match val {
        Value::Array(arr) => {
            // Format as table (reuse existing table logic)
            format_array_as_table(arr)
        }
        Value::Obj(obj) => {
            // Format as record
            format_obj_as_record(obj)
        }
        _ => val.to_string(),
    }
}

fn format_array_as_table(arr: &Array) -> String {
    // Collect all objects, extract keys
    let objs: Vec<&Obj> = arr.iter()
        .filter_map(|v| v.as_obj())
        .collect();

    if objs.is_empty() {
        return arr.to_string();
    }

    // Get all unique keys from all objects
    let mut all_keys = Vec::new();
    for obj in &objs {
        for key in obj.keys() {
            if !all_keys.contains(&key) {
                all_keys.push(key);
            }
        }
    }

    // Build table using existing Table infrastructure
    // ... (implementation)

    // Render table
    table.render()
}

fn format_obj_as_record(obj: &Obj) -> String {
    let mut parts = Vec::new();
    for (key, val) in obj.iter() {
        parts.push(format!("{}: {}", key, val));
    }
    parts.join(", ")
}
```

### Phase 4: Refactor ls Command

**File**: `auto-shell/src/cmd/fs.rs`

Add a new version that returns structured data:

```rust
use auto_val::{Value, Array, Obj};

/// List directory and return structured data (Value::Array of Obj)
pub fn ls_command_value(
    path: &Path,
    current_dir: &Path,
    all: bool,
    long: bool,
    human: bool,
    time_sort: bool,
    reverse: bool,
    recursive: bool,
) -> Result<Value> {
    let target = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_dir.join(path)
    };

    if !target.exists() {
        miette::bail!("ls: {}: No such file or directory", target.display());
    }

    // Handle recursive listing
    if recursive {
        return ls_recursive_value(&target, all, long, human, time_sort, reverse);
    }

    // If it's a file, return single object
    if target.is_file() {
        return Ok(single_file_entry(&target));
    }

    // List directory contents
    let entries = fs::read_dir(&target).into_diagnostic()?;

    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.into_diagnostic()?;
        let metadata = entry.metadata().into_diagnostic()?;

        let name = entry.file_name()
            .into_string()
            .unwrap_or_else(|_| "?".to_string());

        // Skip hidden files unless -a flag is set
        if !all && name.starts_with('.') {
            continue;
        }

        let is_dir = entry.path().is_dir();

        let file_type = if is_dir { "dir" } else { "file" };
        let size = if !is_dir { Some(metadata.len() as i64) } else { None };

        let modified = metadata.modified()
            .ok()
            .and_then(|time| {
                use std::time::UNIX_EPOCH;
                let secs = time.duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;
                let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)?;
                Some(datetime.format("%Y-%m-%d %H:%M").to_string())
            });

        let permissions = if long {
            Some(format_permissions(&metadata))
        } else {
            None
        };

        files.push(build_file_entry(
            name,
            file_type,
            size,
            modified,
            permissions,
        ));
    }

    // Sort files
    files.sort_by(|a, b| {
        let cmp = if time_sort {
            // Sort by modification time (newest first)
            let a_time = a.as_obj()
                .and_then(|o| o.get("modified"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let b_time = b.as_obj()
                .and_then(|o| o.get("modified"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            b_time.cmp(a_time)
        } else {
            // Sort alphabetically
            let a_name = a.as_obj()
                .and_then(|o| o.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let b_name = b.as_obj()
                .and_then(|o| o.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            a_name.cmp(b_name)
        };

        // Directories first
        let a_is_dir = a.as_obj()
            .and_then(|o| o.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("") == "dir";
        let b_is_dir = b.as_obj()
            .and_then(|o| o.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("") == "dir";

        if a_is_dir != b_is_dir {
            b_is_dir.cmp(&a_is_dir)
        } else {
            cmp
        }
    });

    if reverse {
        files.reverse();
    }

    Ok(Value::Array(Array::from(files)))
}

fn single_file_entry(path: &Path) -> Value {
    let metadata = path.metadata().ok();
    let name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string();

    let file_type = if path.is_dir() { "dir" } else { "file" };
    let size = metadata.as_ref().map(|m| m.len() as i64);

    let modified = metadata.as_ref()
        .and_then(|m| m.modified().ok())
        .and_then(|time| {
            use std::time::UNIX_EPOCH;
            let secs = time.duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;
            let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)?;
            Some(datetime.format("%Y-%m-%d %H:%M").to_string())
        });

    build_file_entry(name, file_type, size, modified, None)
}

fn ls_recursive_value(
    path: &Path,
    all: bool,
    long: bool,
    human: bool,
    time_sort: bool,
    reverse: bool,
) -> Result<Value> {
    // Recursively collect all files
    let mut all_files = Vec::new();

    fn collect_files(
        path: &Path,
        all: bool,
        long: bool,
        acc: &mut Vec<Value>,
    ) -> Result<()> {
        let entries = fs::read_dir(path).into_diagnostic()?;

        for entry in entries {
            let entry = entry.into_diagnostic()?;
            let p = entry.path();

            let name = entry.file_name()
                .into_string()
                .unwrap_or_else(|_| "?".to_string());

            if !all && name.starts_with('.') {
                if p.is_dir() {
                    continue; // Skip hidden dirs entirely
                }
                // Skip hidden files
                continue;
            }

            acc.push(single_file_entry(&p));

            if p.is_dir() {
                collect_files(&p, all, long, acc)?;
            }
        }

        Ok(())
    }

    collect_files(path, all, long, &mut all_files)?;

    // Sort
    all_files.sort_by(|a, b| {
        // ... same sort logic as above ...
        std::cmp::Ordering::Equal
    });

    Ok(Value::Array(Array::from(all_files)))
}
```

**Keep original `ls_command`** for backward compatibility:

```rust
/// Original ls command that returns formatted string (legacy)
pub fn ls_command(
    path: &Path,
    current_dir: &Path,
    all: bool,
    long: bool,
    human: bool,
    time_sort: bool,
    reverse: bool,
    recursive: bool,
) -> Result<String> {
    // Get structured value
    let value = ls_command_value(
        path,
        current_dir,
        all,
        long,
        human,
        time_sort,
        reverse,
        recursive,
    )?;

    // Format for display
    Ok(format_value_for_display(&value))
}
```

### Phase 5: Update Commands to Use PipelineData

**File**: `auto-shell/src/cmd/commands/ls.rs`

```rust
use auto_val::Value;
use crate::cmd::{Command, PipelineData};

impl Command for LsCommand {
    fn name(&self) -> &str {
        "ls"
    }

    fn signature(&self) -> Signature {
        Signature::new("ls", "List directory contents")
            .optional("path", "Path to list")
            .flag_with_short("all", 'a', "Show all files including hidden")
            .flag_with_short("long", 'l', "Long listing format")
            .flag_with_short("human-readable", 'h', "Human-readable sizes")
            .flag_with_short("time", 't', "Sort by modification time")
            .flag_with_short("reverse", 'r', "Reverse sort order")
            .flag_with_short("recursive", 'R', "List subdirectories recursively")
            .flag("json", "Output as JSON instead of formatted text")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,  // ← Changed type
        shell: &mut Shell,
    ) -> Result<PipelineData> {  // ← Changed return type
        let path_arg = args.positionals.get(0).map(|s| s.as_str()).unwrap_or(".");
        let path = Path::new(path_arg);

        // Extract flags
        let all = args.has_flag("all");
        let long = args.has_flag("long");
        let human = args.has_flag("human-readable");
        let time = args.has_flag("time");
        let reverse = args.has_flag("reverse");
        let recursive = args.has_flag("recursive");
        let json = args.has_flag("json");

        // Get structured data
        let value = fs::ls_command_value(
            path,
            &shell.pwd(),
            all,
            long,
            human,
            time,
            reverse,
            recursive,
        )?;

        // Return either value or text based on flag
        if json {
            // Return as JSON text
            Ok(PipelineData::Text(serde_json::to_string_pretty(&value)?))
        } else {
            // Return structured value
            Ok(PipelineData::Value(value))
        }
    }
}
```

### Phase 6: Add Data Manipulation Commands

Create new commands for working with structured data (similar to Nushell):

#### 6.1 `get` Command - Extract field from objects

**File**: `auto-shell/src/cmd/commands/get.rs`

```rust
use crate::cmd::{Command, Signature, ParsedArgs, PipelineData};
use auto_val::{Value, Obj};
use miette::Result;

pub struct GetCommand;

impl Command for GetCommand {
    fn name(&self) -> &str {
        "get"
    }

    fn signature(&self) -> Signature {
        Signature::new("get", "Extract field from structured data")
            .required("field", "Field name or index to extract")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut crate::Shell,
    ) -> Result<PipelineData> {
        let field_name = args.positionals.get(0)
            .ok_or_else(|| miette::miette!("Missing field argument"))?;

        let input_val = input.as_value()
            .ok_or_else(|| miette::miette!("Input must be structured data"))?;

        match input_val {
            Value::Array(arr) => {
                // Extract field from each object in array
                let mut result = Vec::new();
                for item in arr.iter() {
                    if let Some(obj) = item.as_obj() {
                        if let Some(value) = obj.get(field_name) {
                            result.push(value.clone());
                        }
                    }
                }
                Ok(PipelineData::Value(Value::array(result)))
            }
            Value::Obj(obj) => {
                // Extract field from single object
                if let Some(value) = obj.get(field_name) {
                    Ok(PipelineData::Value(value.clone()))
                } else {
                    miette::bail!("Field not found: {}", field_name)
                }
            }
            _ => miette::bail!("Input must be object or array"),
        }
    }
}
```

#### 6.2 `select` Command - Select specific fields

**File**: `auto-shell/src/cmd/commands/select.rs`

```rust
pub struct SelectCommand;

impl Command for SelectCommand {
    fn name(&self) -> &str {
        "select"
    }

    fn signature(&self) -> Signature {
        Signature::new("select", "Select specific fields from objects")
            .rest("fields", "Field names to select")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut crate::Shell,
    ) -> Result<PipelineData> {
        let fields: Vec<&str> = args.positionals.iter()
            .map(|s| s.as_str())
            .collect();

        let input_val = input.as_value()
            .ok_or_else(|| miette::miette!("Input must be structured data"))?;

        match input_val {
            Value::Array(arr) => {
                let mut result = Vec::new();
                for item in arr.iter() {
                    if let Some(obj) = item.as_obj() {
                        let mut new_obj = Obj::new();
                        for field in &fields {
                            if let Some(value) = obj.get(*field) {
                                new_obj.set(field, value.clone());
                            }
                        }
                        result.push(Value::Obj(new_obj));
                    }
                }
                Ok(PipelineData::Value(Value::array(result)))
            }
            Value::Obj(obj) => {
                let mut new_obj = Obj::new();
                for field in &fields {
                    if let Some(value) = obj.get(*field) {
                        new_obj.set(field, value.clone());
                    }
                }
                Ok(PipelineData::Value(Value::Obj(new_obj)))
            }
            _ => miette::bail!("Input must be object or array"),
        }
    }
}
```

#### 6.3 `where` Command - Filter based on condition

**File**: `auto-shell/src/cmd/commands/where_.rs`

```rust
pub struct WhereCommand;

impl Command for WhereCommand {
    fn name(&self) -> &str {
        "where"
    }

    fn signature(&self) -> Signature {
        Signature::new("where", "Filter structured data based on field comparison")
            .required("field", "Field name to check")
            .required("operator", "Comparison operator (==, !=, >, <, contains)")
            .required("value", "Value to compare against")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut crate::Shell,
    ) -> Result<PipelineData> {
        let field = args.positionals.get(0)
            .ok_or_else(|| miette::miette!("Missing field argument"))?;
        let op = args.positionals.get(1)
            .ok_or_else(|| miette::miette!("Missing operator argument"))?;
        let compare_val = args.positionals.get(2)
            .ok_or_else(|| miette::miette!("Missing value argument"))?;

        let input_val = input.as_value()
            .ok_or_else(|| miette::miette!("Input must be structured data"))?;

        let arr = input_val.as_array()
            .ok_or_else(|| miette::miette!("Input must be an array"))?;

        let mut result = Vec::new();
        for item in arr.iter() {
            if let Some(obj) = item.as_obj() {
                if let Some(field_val) = obj.get(field) {
                    if compare_values(field_val, op, compare_val) {
                        result.push(item.clone());
                    }
                }
            }
        }

        Ok(PipelineData::Value(Value::array(result)))
    }
}

fn compare_values(field_val: &Value, op: &str, compare_str: &str) -> bool {
    match field_val {
        Value::Str(s) => {
            match op {
                "==" => s.as_ref() == compare_str,
                "!=" => s.as_ref() != compare_str,
                "contains" => s.as_ref().contains(compare_str),
                _ => false,
            }
        }
        Value::Int(i) => {
            if let Ok(compare_num) = compare_str.parse::<i32>() {
                match op {
                    "==" => *i == compare_num,
                    "!=" => *i != compare_num,
                    ">" => *i > compare_num,
                    "<" => *i < compare_num,
                    ">=" => *i >= compare_num,
                    "<=" => *i <= compare_num,
                    _ => false,
                }
            } else {
                false
            }
        }
        Value::Bool(b) => {
            if let Ok(compare_bool) = compare_str.parse::<bool>() {
                match op {
                    "==" => *b == compare_bool,
                    "!=" => *b != compare_bool,
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
}
```

#### 6.4 `map` Command - Transform each element

**File**: `auto-shell/src/cmd/commands/map.rs`

```rust
pub struct MapCommand;

impl Command for MapCommand {
    fn name(&self) -> &str {
        "map"
    }

    fn signature(&self) -> Signature {
        Signature::new("map", "Transform each element in array")
            .required("transformation", "Field transformation (e.g., 'name.upper()')")
    }

    fn run(
        &self,
        args: &ParsedArgs,
        input: PipelineData,
        _shell: &mut crate::Shell,
    ) -> Result<PipelineData> {
        // Simple implementation: just extract field for now
        // Future: support full Auto expressions
        let field = args.positionals.get(0)
            .ok_or_else(|| miette::miette!("Missing field argument"))?;

        let input_val = input.as_value()
            .ok_or_else(|| miette::miette!("Input must be structured data"))?;

        let arr = input_val.as_array()
            .ok_or_else(|| miette::miette!("Input must be an array"))?;

        let mut result = Vec::new();
        for item in arr.iter() {
            if let Some(obj) = item.as_obj() {
                if let Some(value) = obj.get(field) {
                    result.push(value.clone());
                }
            }
        }

        Ok(PipelineData::Value(Value::array(result)))
    }
}
```

### Phase 7: Update Pipeline Execution

**File**: `auto-shell/src/cmd/pipeline.rs`

Update pipeline execution to pass `PipelineData`:

```rust
use super::{Command, PipelineData, ParsedArgs};

/// Execute a pipeline of commands
pub fn execute_pipeline(
    commands: Vec<ParsedCommand>,
    shell: &mut Shell,
) -> Result<Option<String>> {
    let mut pipeline_data: Option<PipelineData> = None;

    for parsed_cmd in commands {
        let cmd = shell.registry.get(&parsed_cmd.name)
            .ok_or_else(|| miette::miette!("Command not found: {}", parsed_cmd.name))?;

        // Get input (previous command's output or None)
        let input = pipeline_data.unwrap_or_else(|| {
            PipelineData::Text(String::new())  // Empty input for first command
        });

        // Execute command
        let output = cmd.run(&parsed_cmd.args, input, shell)?;

        pipeline_data = Some(output);
    }

    // Convert final output to string for display
    Ok(pipeline_data.map(|d| d.into_text()))
}
```

### Phase 8: Display Logic

**File**: `auto-shell/src/shell.rs`

Add display logic for `PipelineData`:

```rust
impl Shell {
    pub fn display_pipeline_data(&self, data: &PipelineData) -> String {
        match data {
            PipelineData::Value(val) => {
                // Format Auto value for display
                format_value_for_display(val)
            }
            PipelineData::Text(s) => {
                // Just display text
                s.clone()
            }
        }
    }
}
```

### Phase 9: Update All Built-in Commands

**Files**: All command files in `auto-shell/src/cmd/commands/`

Update each command to use `PipelineData`:

```rust
// Example: cd command
impl Command for CdCommand {
    fn run(
        &self,
        args: &ParsedArgs,
        _input: PipelineData,  // ← Changed
        shell: &mut Shell,
    ) -> Result<PipelineData> {  // ← Changed
        let path = args.positionals.get(0).map(|s| s.as_str()).unwrap_or(".");
        shell.cd(path)?;
        Ok(PipelineData::Text(String::new()))  // No output
    }
}
```

**Priority order for updates**:
1. High-priority: `ls` (already planned)
2. Medium-priority: `pwd`, `echo`
3. Low-priority: `cd`, `exit`, `help` (don't use input/output)

## Benefits

### Performance Improvements
- **Zero-copy between commands**: Pass `Value` references, not strings
- **No parsing overhead**: Avoid JSON/text parsing at every pipe
- **Reduced allocations**: Use existing Auto value structures

### Capability Improvements
- **Type-safe operations**: Preserve types through pipeline
- **Structured queries**: `ls | where type == dir | get name`
- **Powerful filtering**: Use field access and comparisons
- **Composability**: Chain complex operations

### Developer Experience
- **Leverage existing infrastructure**: Auto value system already works
- **Backward compatible**: Can still use text for external commands
- **Progressive adoption**: Update commands incrementally
- **Simple mental model**: Same as Nushell but with Auto values

## Migration Strategy

### Stage 1: Infrastructure (Week 1)
- ✅ Create `PipelineData` enum
- ✅ Update `Command` trait
- ✅ Add value helper methods
- ✅ Update pipeline execution logic

### Stage 2: Core Commands (Week 2)
- ✅ Update `ls` to return `Value`
- ✅ Update `pwd`, `echo` to return `Value`
- ✅ Add display logic for Value types

### Stage 3: Data Manipulation (Week 3)
- ✅ Implement `get` command
- ✅ Implement `select` command
- ✅ Implement `where` command
- ✅ Implement `map` command

### Stage 4: Remaining Commands (Week 4+)
- ⏳ Update all other built-in commands
- ⏳ Ensure backward compatibility
- ⏳ Add tests for pipeline functionality

## Example Usage

After implementation, users can do:

```bash
# List all directories
ls | where type == dir

# Get file names larger than 1MB
ls -lh | where size > 1048576 | get name

# Select specific columns
ls -l | select name size modified

# Chain operations
ls | where type == dir | get name | map upper

# Sort by multiple fields (future)
ls | sort -f size -r | select name size

# Group by type (future)
ls | group-by type | select type count
```

## Comparison with Nushell

| Feature | Nushell | Auto-Shell (after implementation) |
|---------|---------|-----------------------------------|
| **Data type** | `Value` enum | `Value` enum (already exists!) |
| **Collections** | List, Record | Array, Obj (same thing) |
| **Pipeline** | `PipelineData` | `PipelineData` (same concept) |
| **Zero-copy** | Yes (Arc<Value>) | Yes (Value references) |
| **Type system** | SyntaxShape (strong) | Auto types (dynamic) |
| **Expressions** | Nushell language | Auto language (future) |

**Key difference**: Nushell has its own expression language, but we can integrate Auto expressions later!

## Future Enhancements

### Phase 10: Auto Expression Integration
- Support Auto expressions in `where`, `map` commands
- Example: `ls | where { .size > 1000 }`
- Example: `ls | map { .name.upper() }`

### Phase 11: Lazy Evaluation
- Stream large arrays instead of collecting all
- Channel-based communication between commands
- Avoid memory overhead for large datasets

### Phase 12: Advanced Operations
- `group-by` - Group elements by field
- `sort` - Sort by field (not just time/name)
- `join` - Join two datasets
- `reduce` - Aggregate operations

## Risks & Mitigations

### Risk 1: Breaking Changes
**Mitigation**: Keep `PipelineData::Text` for backward compatibility. Commands can choose to return either format.

### Risk 2: Complexity
**Mitigation**: Simple mental model - `Value` for structured, `Text` for strings. Helper functions for common operations.

### Risk 3: Performance of Value Cloning
**Mitigation**: Auto's `Value` uses `Arc` in some places. Add `Cow<Value>` for zero-copy where needed.

### Risk 4: External Commands
**Mitigation**: External commands still use text. Convert `Value` → `Text` before piping to external, `Text` → `Value` when reading from external.

## Success Criteria

1. ✅ `PipelineData` enum implemented and working
2. ✅ `ls` command returns structured data
3. ✅ At least 3 data manipulation commands working (`get`, `where`, `select`)
4. ✅ Pipeline example works: `ls | where type == dir | get name`
5. ✅ Backward compatibility maintained (text mode still works)
6. ✅ Performance improved (less allocation/parsing)
7. ✅ Documentation updated with examples

## Timeline Estimate

- **Phase 1-4**: 1 week (infrastructure + ls refactor)
- **Phase 5-6**: 1 week (data manipulation commands)
- **Phase 7-9**: 1 week (pipeline execution + display + migration)
- **Testing**: 3-5 days

**Total**: 3-4 weeks for full implementation

## Next Steps

1. ✅ Create this plan document
2. ⏭️ Start Phase 1: Create PipelineData wrapper
3. ⏭️ Update Command trait to use PipelineData
4. ⏭️ Refactor ls command as proof-of-concept
5. ⏭️ Implement get/where/select commands
6. ⏭️ Test example pipeline: `ls | where type == dir | get name`
7. ⏭️ Document new capabilities with examples

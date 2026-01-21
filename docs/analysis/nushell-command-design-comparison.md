# Nushell vs Auto-Shell Command Design Architecture

## Executive Summary

This document compares the command design philosophy and architecture between Nushell (a production-grade modern shell) and Auto-Shell (our traditional shell implementation).

**Key Difference**: Nushell is designed as a **structured data shell** where commands pass typed data, while Auto-Shell is a **traditional text shell** where commands pass strings.

---

## Command Trait Comparison

### Nushell's Command Trait

```rust
pub trait Command: Send + Sync + CommandClone {
    fn name(&self) -> &str;
    fn signature(&self) -> Signature;

    fn description(&self) -> &str;
    fn extra_description(&self) -> &str { "" }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,  // ← Structured data!
    ) -> Result<PipelineData, ShellError>;

    fn examples(&self) -> Vec<Example<'_>> { vec![] }
    fn search_terms(&self) -> Vec<&str> { vec![] }

    fn is_const(&self) -> bool { false }
    fn command_type(&self) -> CommandType { CommandType::Builtin }

    fn get_dynamic_completion(...) -> Result<Option<Vec<DynamicSuggestion>>, ShellError> {
        Ok(None)
    }
}
```

### Auto-Shell's Command Trait

```rust
pub trait Command {
    fn name(&self) -> &str;
    fn signature(&self) -> Signature;

    fn run(
        &self,
        args: &ParsedArgs,
        input: Option<&str>,  // ← String input
        shell: &mut Shell,
    ) -> Result<Option<String>>;  // ← String output
}
```

**Key Differences:**
1. **Nushell**: `PipelineData` in/out (structured tables, lists, records)
2. **Auto-Shell**: `Option<&str>` in, `Option<String>` out (text)
3. **Nushell**: Has `EngineState` and `Stack` for variable scope
4. **Auto-Shell**: Has mutable `Shell` reference for state
5. **Nushell**: Richer metadata (examples, search terms, command types)
6. **Auto-Shell**: Simpler, more focused API

---

## Signature System Comparison

### Nushell's Signature

```rust
pub struct Signature {
    pub name: String,
    pub usage: String,
    pub extra_usage: String,
    pub required_positional: Vec<PositionalArg>,
    pub optional_positional: Vec<PositionalArg>,
    pub rest_positional: Option<RestPositionalArg>,
    pub named: Vec<Flag>,
    pub input_output_types: Vec<(Type, Type)>,  // Type system!
    pub allow_variants: bool,
    pub category: Category,
}

pub struct Flag {
    pub long: String,
    pub short: Option<char>,
    pub arg: Option<SyntaxShape>,  // Typed flag arguments!
    pub required: bool,
    pub desc: String,
    pub completion: Option<Completion>,
    pub var_id: Option<VarId>,
    pub default_value: Option<Value>,
}

pub struct PositionalArg {
    pub name: String,
    pub desc: String,
    pub shape: SyntaxShape,  // Type validation!
    pub completion: Option<Completion>,
    pub var_id: Option<VarId>,
    pub default_value: Option<Value>,
}
```

**Signature Builder Example:**
```rust
Signature::build("ls")
    .input_output_types(vec![(Type::Nothing, Type::table())])
    .rest("pattern", SyntaxShape::GlobPattern, "The glob pattern to use.")
    .switch("all", "Show hidden files", Some('a'))
    .switch("long", "Get all available columns", Some('l'))
    .switch("du", "Display directory size", Some('d'))
    .category(Category::FileSystem)
```

### Auto-Shell's Signature

```rust
pub struct Signature {
    pub name: String,
    pub description: String,
    pub arguments: Vec<Argument>,
}

pub struct Argument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub is_flag: bool,
    pub short: Option<char>,
}
```

**Signature Builder Example:**
```rust
Signature::new("ls", "List directory contents")
    .optional("path", "Path to list")
    .flag_with_short("all", 'a', "Show all files")
    .flag_with_short("long", 'l', "Long format")
    .flag_with_short("human-readable", 'h', "Human-readable sizes")
```

**Key Differences:**
1. **Nushell**: Strongly typed with `SyntaxShape` (int, string, glob, etc.)
2. **Auto-Shell**: Untyped strings only
3. **Nushell**: Type checking at runtime, type coercion
4. **Auto-Shell**: No validation beyond existence/presence
5. **Nushell**: Input/output type declarations for pipeline compatibility
6. **Auto-Shell**: No type information
7. **Nushell**: Categories for organization (FileSystem, Strings, Math, etc.)
8. **Auto-Shell**: No categorization

---

## Data Flow Architecture

### Nushell: Structured Data Pipeline

```
Input PipelineData (Value enum)
    ↓
Command (processes structured data)
    ↓
Output PipelineData (Value enum)
    ↓
Next Command (receives structured data)

Value enum variants:
- String
- Int
- Float
- Bool
- List(Vec<Value>)
- Record(HashMap<String, Value>)
- Table(Vec<Record>)
- Error
- Nothing
```

**Example pipeline:**
```bash
ls | where type == dir | get name | str upper
```

Data flow:
1. `ls` → Table[{name, type, size, modified}]
2. `where type == dir` → Filtered table (only directories)
3. `get name` → List["src", "tests", "target"]
4. `str upper` → List["SRC", "TESTS", "TARGET"]

### Auto-Shell: String Text Pipeline

```
Input Option<&str> (raw string)
    ↓
Command (parses/processes text)
    ↓
Output Option<String> (formatted string)
    ↓
Next Command (receives raw string)

String format:
- Plain text
- Tables rendered as strings
- No structured typing
```

**Example pipeline:**
```bash
ls | grep -E "^d" | awk '{print $NF}' | tr a-z A-Z
```

Data flow:
1. `ls` → "drwxr-xr-x src/\ndrwxr-xr-x tests/\n-rw-r--r-- Cargo.toml"
2. `grep -E "^d"` → "drwxr-xr-x src/\ndrwxr-xr-x tests/"
3. `awk '{print $NF}'` → "src/\ntests/"
4. `tr a-z A-Z` → "SRC/\nTESTS/"

---

## Type System Comparison

### Nushell's SyntaxShape Types

```rust
pub enum SyntaxShape {
    Any,                    // Any type
    AnyClosing,             // Any type (except closure)
    Binary,                 // Binary data
    Block,                  // Code block
    Bool,                   // Boolean
    CellPath,               // Cell path (e.g., foo.0.bar)
    Closure,                // Closure/lambda
    Compare,                // Comparison operator (< > == !=)
    DateTime,               // Date/time
    Duration,               // Duration (e.g., 10sec, 5min)
    Error,                  // Error value
    Expression,             // Expression
    Filepath,               // File path
    Filesize,               // File size (1kb, 10mb)
    Filter,                 // Filter expression
    Flag,                   // Flag (true/false)
    Float,                  // Floating point number
    Format,                 // Format pattern
    GlobPattern,            // Glob pattern (*.rs, **/*.txt)
    Int,                    // Integer
    List(Box<SyntaxShape>),  // List of type
    Math,                   // Math expression
    Nothing,                // Nothing value
    Number,                 // Any number
    Operator,               // Operator
    Range,                  // Range (0..10, 0..=10)
    Regex,                  // Regular expression
    Signature,              // Command signature
    String,                 // String
    Variable,               // Variable name
}
```

**Type Validation Example:**
```rust
fn run(&self, call: &Call, input: PipelineData) -> Result<PipelineData> {
    // Engine validates arguments match SyntaxShape
    let count: i64 = call.req(engine_state, stack, 0)?;  // Must be Int
    let pattern: String = call.req(engine_state, stack, 1)?;  // Must be String
    let all: bool = call.has_flag(engine_state, stack, "all")?;  // Boolean flag

    // If user passes wrong type, gets compile-time error:
    // Error: Type mismatch during runtime
    //   × expected int, found string
}
```

### Auto-Shell: No Types

```rust
pub struct ParsedArgs {
    pub positionals: Vec<String>,  // Everything is a string!
    pub flags: HashMap<String, bool>,
    pub named: HashMap<String, String>,
}

// Manual parsing required:
fn run(&self, args: &ParsedArgs, ...) -> Result<Option<String>> {
    let count_str = args.positionals.get(0).ok_or("missing count")?;
    let count: i64 = count_str.parse().map_err(|e| "invalid integer")?;
    let pattern = args.positionals.get(1).ok_or("missing pattern")?;
    let all = args.has_flag("all");

    // Runtime parsing errors
}
```

---

## Advanced Features Comparison

### 1. Completion System

**Nushell:**
```rust
pub enum Completion {
    Command(DeclId),
    List(NuCow<&'static [&'static str], Vec<String>>),
}

fn get_dynamic_completion(
    &self,
    engine_state: &EngineState,
    stack: &mut Stack,
    call: DynamicCompletionCallRef,
    arg_type: &ArgType,
) -> Result<Option<Vec<DynamicSuggestion>>, ShellError> {
    // Return contextual suggestions based on user input
}
```

**Auto-Shell:** No completion system

### 2. Examples System

**Nushell:**
```rust
pub struct Example<'a> {
    pub example: &'a str,
    pub description: &'a str,
    pub result: Option<Value>,  // Actual output value!
}

impl Command for Ls {
    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                description: "List visible files in the current directory",
                example: "ls",
                result: None,  // Side effect, no result
            },
            Example {
                description: "List Rust files",
                example: "ls *.rs",
                result: None,
            },
        ]
    }
}
```

Users can run `help ls` and see executable examples with actual output!

**Auto-Shell:** No example system

### 3. Command Categories

**Nushell:**
```rust
pub enum Category {
    Async,
    Bytes,
    Chart,
    Conversion,
    Core,
    Custom,
    Date,
    Debugger,
    Default,
    Deprecated,
    Dev,
    Env,
    Experimental,
    Experimental2,
    External,
    FileSystem,
    Filters,
    Format,
    Generator,
    Hash,
    Into,
    Math,
    Network,
    Path,
    Platform,
    Random,
    Shell,
    Strings,
    System,
    Viewers,
}
```

Commands are organized by category for `help category` discovery.

**Auto-Shell:** No categorization

### 4. Command Types

**Nushell:**
```rust
pub enum CommandType {
    Builtin,   // Compiled into shell
    Custom,    // User-defined (def command)
    Keyword,   // Language keywords (if, for, etc.)
    External,  // External executables
    Alias,     // Command aliases
    Plugin,    // Dynamically loaded plugins
}
```

**Auto-Shell:** Only built-in and external

### 5. Const Evaluation

**Nushell:**
```rust
fn is_const(&self) -> bool {
    false  // Can this command run at parse time?
}

fn run_const(
    &self,
    working_set: &StateWorkingSet,
    call: &Call,
    input: PipelineData,
) -> Result<PipelineData, ShellError> {
    // Run during parsing (e.g., for compile-time constants)
}
```

Allows certain commands to run at compile-time for optimization.

**Auto-Shell:** No const evaluation

---

## Error Handling Comparison

### Nushell's Rich Error Types

```rust
pub enum ShellError {
    // Type errors
    TypeMismatch { expected: String, found: String, span: Span },
    CantConvert { to_type: String, from_type: String, span: Span },

    // Runtime errors
    IOError { msg: String },
    NotFound { span: Span },
    PermissionDenied { msg: String },

    // Argument errors
    MissingParameter { param_name: String, span: Span },
    InvalidGlobPattern { pattern: String, msg: String },

    // Command errors
    CommandNotFound { name: String },
    Deprecated { msg: String },

    // ... 50+ more error variants
}

impl Diagnostic for ShellError {
    fn code(&self) -> Option<ErrorKind> { ... }
    fn labels(&self) -> Option<Vec<Label>> { ... }
    fn help(&self) -> Option<String> { ... }
}
```

**Error Output:**
```
Error: Type mismatch

  × expected int, found string
   ╭─[script.nu:5:10]
 5 │ let x: int = "hello"
   ·                 ────┬───
   ·                     ╰── this is a string
   ╰────
  help: try removing the type annotation
```

### Auto-Shell's Simple Errors

```rust
pub type Result<T> = std::result::Result<T, miette::ErrReport>;

// Usage:
miette::bail!("ls: {}: No such file or directory", path.display());
```

**Error Output:**
```
Error:
  × ls: /nonexistent: No such file or directory
```

Both use miette, but Nushell has structured error types with context.

---

## Variable Scope and State

### Nushell's EngineState and Stack

```rust
pub struct EngineState {
    pub vars: Vec<VarId>,                  // All variables
    pub decls: Vec<Declaration>,           // All commands
    pub blocks: Vec<Block>,                // Code blocks
    pub modules: Vec<Module>,              // Modules
    pub scope: Vec<ScopeFrame>,            // Scope stack
    pub env_vars: HashMap<String, Value>,  // Environment variables
    // ... lots more state
}

pub struct Stack {
    pub env_vars: HashMap<String, Value>,     // Stack-local env
    pub vars: Vec<VarId>,                     // Stack-local vars
    pub parent: Option<Box<Stack>>,           // Parent stack
}

// Commands can access and modify scope:
fn run(&self, engine_state: &EngineState, stack: &mut Stack, ...) {
    // Get variable
    let x = stack.get_var(var_id, span)?;

    // Set variable
    stack.add_var(var_id, value);

    // Get environment variable
    let path = stack.get_env_var(engine_state, "PATH")?;
}
```

**Benefits:**
- Nested scopes (closures, blocks)
- Variable shadowing
- Environment isolation
- Thread-safe variable access
- Type-checked variable assignments

### Auto-Shell's Shell State

```rust
pub struct Shell {
    /// Current working directory
    cwd: PathBuf,
    /// Environment variables
    env: HashMap<String, String>,
    /// Command history
    history: Vec<String>,
    /// Exit code
    exit_code: Option<i32>,
    /// Command registry
    registry: CommandRegistry,
}

// Commands access shell state:
fn run(&self, shell: &mut Shell, ...) {
    let cwd = shell.pwd();
    shell.set_env("KEY", "value");
}
```

**Benefits:**
- Simpler, easier to understand
- Mutable state via &mut Shell
- No variable scoping complexity
- Direct environment manipulation

---

## Plugin System

### Nushell Plugins

```rust
// Plugins are dynamically loaded shared libraries
pub trait Plugin {
    fn signature(&self) -> Signature;
    fn run(&self, name: &str, call: &Call, input: &Value) -> Result<Value, ShellError>;
}

// Commands can delegate to plugins:
fn plugin_identity(&self) -> Option<&PluginIdentity> {
    Some(&self.identity)
}
```

Plugins extend Nushell without recompiling the shell.

**Auto-Shell:** No plugin system (all commands built-in)

---

## Performance Considerations

### Nushell Optimizations

1. **Lazy Evaluation**: `PipelineData` streams values, doesn't collect all
2. **Parallel Processing**: Rayon for parallel iteration
3. **Type Coercion Cache**: Avoids repeated type checks
4. **Span Tracking**: Zero-cost in release builds
5. **Copy-on-Write**: Values use Arc for cheap cloning

### Auto-Shell Characteristics

1. **Eager Evaluation**: Collects all output before returning
2. **Single-Threaded**: No parallelism
3. **String Allocations**: Many intermediate strings
4. **Simple Parsing**: Fast but untyped

**Trade-off:** Nushell has higher overhead per operation but enables powerful pipelines. Auto-Shell has lower overhead but limited composability.

---

## Code Organization

### Nushell Structure

```
crates/
├── nu-protocol/        # Core traits, types, engine
│   ├── engine/         # EngineState, Stack, Call
│   ├── signature.rs    # Signature, Flag, PositionalArg
│   └── value.rs        # Value enum, type system
├── nu-engine/          # Command execution, eval loop
├── nu-command/         # Built-in commands
│   ├── filesystem/     # ls, cd, mv, rm, etc.
│   ├── filters/        # where, select, sort, etc.
│   └── strings/        # str split, str replace, etc.
├── nu-cmd-lang/        # Language keywords (if, for, etc.)
└── nu-parser/          # Parser, lexer, AST
```

### Auto-Shell Structure

```
auto-shell/src/
├── cmd.rs              # Command trait, Signature
├── cmd/
│   ├── commands/       # Built-in commands (ls, cd, etc.)
│   ├── parser.rs       # Argument parsing
│   ├── fs.rs           # File system utilities
│   ├── builtin.rs      # Built-in command routing
│   ├── external.rs     # External command execution
│   └── registry.rs     # Command registry
└── shell.rs            # Shell state, REPL
```

Both follow similar separation of concerns, but Nushell has more granular module boundaries.

---

## Philosophy Summary

### Nushell Philosophy

**"Treat everything as structured data"**

- Commands output typed data (tables, lists, records)
- Pipeline preserves structure
- Type safety and validation
- Rich metadata for discovery
- Extensible via plugins
- SQL-like data manipulation

**Best for:**
- Data analysis and transformation
- Complex pipelines
- Type safety requirements
- Interactive data exploration
- Scripting with structured data

### Auto-Shell Philosophy

**"Traditional Unix shell, modernized"**

- Commands output strings
- Text-based pipelines
- Simplicity over structure
- Familiar Unix patterns
- Easy to understand
- Low overhead

**Best for:**
- Traditional system administration
- Simple automation scripts
- Users familiar with bash/zsh
- Quick one-liners
- Resource-constrained environments

---

## Recommendations for Auto-Shell

### Keep What We Have
✅ **String-based design** - Simpler, more familiar to Unix users
✅ **Simple signature API** - Easy to learn and implement
✅ **Shell state object** - Clear state management
✅ **Table rendering** - Good compromise between text and structure

### Consider Adding
🤔 **Command examples** - Help text with executable examples
🤔 **Command categories** - Better help organization (FileSystem, Network, etc.)
🤔 **Completion system** - Basic flag/path completion
🤔 **Better error types** - Structured errors with context (like ShellError)

### Don't Need
❌ **Full type system** - Adds complexity without proportional benefit
❌ **Value enum** - Our string approach is simpler
❌ **Plugin system** - Nice to have, but not essential initially
❌ **Complex scoping** - Flat shell state is easier to understand

---

## Conclusion

Nushell and Auto-Shell represent two different design philosophies:

- **Nushell**: A **data shell** for structured data manipulation
  - Rich type system
  - Structured pipelines
  - Advanced features (plugins, completions, examples)
  - Steeper learning curve
  - Powerful for data analysis

- **Auto-Shell**: A **modern traditional shell** with simplicity focus
  - Text-based pipelines
  - Simple, familiar API
  - Easy to learn and extend
  - Lower overhead
  - Better for system administration

Both approaches are valid for different use cases. Auto-Shell's simplicity is a strength, not a weakness - it makes the code accessible and maintainable while still providing useful features like table formatting, flag aliases, and recursive listing.

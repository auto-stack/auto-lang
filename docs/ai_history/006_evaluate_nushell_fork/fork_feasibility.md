# Feasibility Study: Forking Nushell for Auto-Shell

## Objective
Evaluate the viability of forking the `nushell` project and replacing its native scripting language with `AutoLang`, effectively using Nushell's shell infrastructure (CLI, table rendering, environment management) while executing AutoLang code.

## Executive Summary
**Verdict: High Risk / High Effort**
While Nushell provides excellent shell primitives, its architecture is **tightly coupled** to its own language semantics. Replacing the language engine is not a "plugin-swap" operation but a fundamental re-architecture. It would require rewriting approximately 60-70% of the core logic, negating many benefits of forking.

## Architectural Mismatch

### 1. Tight Coupling of Engine and Language
Nushell does not have a generic "Shell Runtime" separate from its "Language Runtime".
- **`nu-parser`**: Parses Nu source code directly into Nu-specific IR.
- **`nu-engine`**: The "VM" of Nushell. It executes Nu-specific AST/IR. It *is* the language interpreter.
- **`nu-protocol`**: Defines the data types (`Value`) and communication.

**Impact**: To use AutoLang, we would have to:
1.  Discard `nu-parser`.
2.  Discard `nu-engine` (the core execution loop).
3.  Write a bridge between `AutoLang` interpreter and `nu-protocol`.

### 2. Data Model Incompatibility
- **Nushell**: Pipeline-centric, structured data (Tables, Records). Every command returns a `Value` that flows to the next.
- **AutoLang**: General-purpose object model.
- **The Bridge**: We would need to marshal every AutoLang object into a `nu-protocol::Value` to use Nushell's table rendering and pipeline logic. This serialization/deserialization overhead is significant.

### 3. Command System
Nushell commands (like `ls`, `cd`) are written as `Plugin` or `InternalCommand` implementations that accept `nu-protocol::Signatures`.
- **Impact**: All of Nushell's built-in commands (`ls`, `where`, `sort-by`) expect Nu-style arguments and return Nu values. AutoLang would need to "fake" being Nu to invoke them, or we'd have to rewrite them.

## Effort Estimate

| Component | Status if Forking | Effort |
| :--- | :--- | :--- |
| **CLI / REPL** | **Keep** (`reedline` logic) | 🟢 Low |
| **Table Rendering** | **Keep** (`nu-table`) | 🟢 Low |
| **Language Engine** | **Replace** (Delete `nu-engine` core) | 🔴 High |
| **Parser** | **Replace** (Delete `nu-parser`) | 🔴 High |
| **Standard Library** | **Rewrite** (Port `nu-std` to AutoLang) | 🔴 High |
| **Command Interop** | **New** (Bridge Nu commands to AutoLang) | 🔴 High |

## Alternative Recommendation: "Nushell-Inspired" (Current Path)

Instead of forking the entire Nushell codebase, we should:
1.  **Adopt Components**: Continue using `reedline` (CLI) and `nu-ansi-term`.
2.  **Adopt `nu-table`**: Integrate Nushell's table rendering library into `auto-shell` to get beautiful output without the engine baggage.
3.  **Mimic Architecture**: Structure `auto-shell` similarly (Parser -> Engine -> Pipeline), but built *for* AutoLang from day one.

 This is what we are currently doing. It allows us to:
- Have 100% control over the language.
- Avoid maintaining a fork of a complex, fast-moving codebase.
- Pick and choose the "best parts" (Reedline, Table) without the coupling.

## Conclusion
Forking Nushell is **technically possible but strategically unwise**. We would spend more time fighting Nushell's language assumptions than building AutoLang features. Developing `auto-shell` as a native host for AutoLang, while leveraging Nushell's libraries (`reedline`, `nu-table`), is the correct approach.

# ArkTS Generator Bug Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix 6 bugs in the ArkTS generator that cause invalid TypeScript/ArkTS code generation (Kotlin syntax instead of TypeScript syntax).

**Architecture:** Fix bugs in `state.rs` (enum, switch) and `generator.rs` (member access, button order, imports). Use TDD approach - write failing tests first, then fix implementation.

**Tech Stack:** Rust, ArkTS, TypeScript

---

## Bug Summary

| Bug # | File | Issue | Complexity |
|-------|------|-------|------------|
| 1 | state.rs:59 | Kotlin `sealed class` → TS `enum` | Medium |
| 2 | state.rs:38 | Missing `case` keyword | Low |
| 3 | state.rs:44 | Missing `break` statements | Low |
| 4 | generator.rs | Double dot `..` syntax | Medium |
| 5 | generator.rs | Wrong Button order | Low |
| 6 | generator.rs | Missing imports | Low |

---

## Task 1: Fix Bug 1 - Enum Syntax (state.rs)

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/state.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_generate_msg_enum_produces_typescript_enum() {
    use crate::aura::{AuraMessage, AuraMsgVariant, AuraWidget};

    let mut widget = AuraWidget::default();
    widget.messages.push(AuraMessage {
        name: "Msg".to_string(),
        variants: vec![
            AuraMsgVariant { name: "Inc".to_string(), payload: None },
            AuraMsgVariant { name: "Dec".to_string(), payload: None },
        ],
    });

    let result = generate_msg_enum(&widget);

    // Should produce TypeScript enum, not Kotlin sealed class
    assert!(result.contains("enum Msg {"), "Should use 'enum' keyword");
    assert!(!result.contains("sealed class"), "Should not contain 'sealed class'");
    assert!(!result.contains("object"), "Should not contain 'object' keyword");
    assert!(result.contains("Inc,"), "Should contain 'Inc,' variant");
    assert!(result.contains("Dec,"), "Should contain 'Dec,' variant");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang ark::state::test_generate_msg_enum -v`
Expected: FAIL - test asserts "Should use 'enum' keyword"

**Step 3: Implement fix**

In `state.rs`, replace `generate_msg_sealed` with `generate_msg_enum`:

```rust
/// Generate Msg enum from widget messages (TypeScript syntax)
pub fn generate_msg_enum(widget: &AuraWidget) -> String {
    if widget.messages.is_empty() {
        return String::new();
    }

    let mut lines = vec!["enum Msg {".to_string()];

    for msg in &widget.messages {
        for variant in &msg.variants {
            // TypeScript enum - simple variant
            lines.push(format!("  {},", variant.name));
        }
    }

    lines.push("}".to_string());
    lines.join("\n")
}
```

**Step 4: Update callers to use new function name**

In `generator.rs` line 27, change:
```rust
use super::state::{generate_dispatch_function, generate_handler_body, generate_msg_sealed, generate_state_declarations};
```
To:
```rust
use super::state::{generate_dispatch_function, generate_handler_body, generate_msg_enum, generate_state_declarations};
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p auto-lang ark::state::test_generate_msg_enum -v`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/state.rs crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "fix(ark): use TypeScript enum instead of Kotlin sealed class"
```

---

## Task 2: Fix Bug 2 - Missing `case` Keyword (state.rs)

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/state.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_dispatch_function_uses_case_keyword() {
    use crate::aura::{AuraHandler, AuraWidget, LogicPayload, AuraStmt, AuraExpr};

    let mut widget = AuraWidget::default();
    widget.handlers.insert(".Inc".to_string(), LogicPayload::AstBlock(vec![
        AuraStmt::Assign { target: "count".to_string(), value: AuraExpr::Int(1) },
    ]));

    let result = generate_dispatch_function(&widget);

    // Should use 'case' keyword
    assert!(result.contains("case Msg."), "Should use 'case Msg.' pattern");
    assert!(!result.contains("Msg.Inc: {"), "Should not use 'Msg.Inc: {' without 'case'");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang ark::state::test_dispatch_function_uses_case -v`
Expected: FAIL

**Step 3: Implement fix**

In `state.rs` line 38, change:
```rust
lines.push(format!("      Msg.{}: {{", msg_name));
```
To:
```rust
lines.push(format!("      case Msg.{}: {{", msg_name));
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p auto-lang ark::state::test_dispatch_function_uses_case -v`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/state.rs
git commit -m "fix(ark): add missing 'case' keyword in switch statement"
```

---

## Task 3: Fix Bug 3 - Missing `break` Statements (state.rs)

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/state.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_dispatch_function_has_break_statements() {
    use crate::aura::{AuraHandler, AuraWidget, LogicPayload, AuraStmt, AuraExpr};

    let mut widget = AuraWidget::default();
    widget.handlers.insert(".Inc".to_string(), LogicPayload::AstBlock(vec![
        AuraStmt::Assign { target: "count".to_string(), value: AuraExpr::Int(1) },
    ]));

    let result = generate_dispatch_function(&widget);

    // Should have break statement
    assert!(result.contains("break;"), "Each case should end with 'break;'");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang ark::state::test_dispatch_function_has_break -v`
Expected: FAIL

**Step 3: Implement fix**

In `state.rs` after line 43 (after handler body). add:
```rust
lines.push("        break;".to_string());
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p auto-lang ark::state::test_dispatch_function_has_break -v`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/state.rs
git commit -m "fix(ark): add missing 'break' statements in switch cases"
```

---

## Task 4: Fix Bug 4 - Double Dot Syntax (generator.rs)

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Analyze the bug**

The generated code has:
- `${..count}` instead of `${this.count}`
- `Msg..Dec` instead of `Msg.Dec`

This is in the text interpolation and component generation.

**Step 2: Write failing test**

```rust
#[test]
fn test_text_interpolation_uses_this_not_double_dot() {
    use crate::aura::{AuraTextContent, AuraWidget};

    let mut gen = ArkGenerator::new();
    let text = AuraTextContent::Interpolated {
        template: "Count: ${.count}".to_string(),
        bindings: vec!["count".to_string()],
    };

    let result = gen.generate_text(&text).unwrap();

    // Should use this.count, not ..count
    assert!(result.contains("this.count"), "Should use 'this.count' for member access");
    assert!(!result.contains("..count"), "Should not use '..count' double dot");
}
```

**Step 3: Run test to verify it fails**

Run: `cargo test -p auto-lang ark::generator::test_text_interpolation -v`
Expected: FAIL

**Step 4: Implement fix**

Review `generate_text` method (line 347-360). The interpolation is done correctly there.
The issue is likely in other places - check `generate_component` (line 420):
```rust
".onClick(() => {{ this.dispatch(Msg.{}) }})", event.handler
```

The `event.handler` is like `.Inc` and we're emitting `Msg..Inc`. Fix:
```rust
// In generate_component, line 420
".onClick(() => {{ this.dispatch(Msg.{}) }})", event.handler.trim_start_matches('.')
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p auto-lang ark::generator::test_text_interpolation -v`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "fix(ark): use single dot for member access, not double dot"
```

---

## Task 5: Fix Bug 5 - Button Construction Order (generator.rs)

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_button_label_in_constructor() {
    use crate::aura::{AuraNode, AuraWidget, AuraPropValue, AuraExpr};

    let mut gen = ArkGenerator::new();
    let element = AuraNode::Element {
        tag: "button".to_string(),
        props: {
            let mut props = std::collections::HashMap::new();
            props.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("-".to_string())));
            props
        },
        events: std::collections::HashMap::new(),
        children: vec![],
    };

    let result = gen.generate_element("button", &props, &events, &children).unwrap();

    // Button label should be in constructor: Button('-')
    assert!(result.contains("Button('-')"), "Button label should be in constructor");
    assert!(!result.contains("Button()"), "Should not have empty Button() constructor");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang ark::generator::test_button_label -v`
Expected: FAIL (or inspect generated code)

**Step 3: Analyze current code**

In `generate_element` (line 188-242):
- Line 212-216: Creates component call with content arg
- Line 218: Adds modifiers

The current code looks correct - it puts the label in the constructor. The issue might be elsewhere.

Check the generated Counter.ets - the Button has `.onClick()` before `('-')`. This suggests the bug is in how modifiers are appended.

**Step 4: Implement fix**

If the bug is confirmed, ensure modifiers are appended AFTER the constructor, not before:

```rust
// Correct order:
Button('-')
  .onClick(...)
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p auto-lang ark::generator::test_button_label -v`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "fix(ark): place Button label in constructor, not as trailing modifier"
```

---

## Task 6: Fix Bug 6 - Missing Import Statements (generator.rs)

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_generated_file_has_imports() {
    use crate::aura::AuraWidget;

    let mut gen = ArkGenerator::new();
    let widget = AuraWidget {
        name: "TestWidget".to_string(),
        ..Default::default()
    };

    let result = gen.generate_entry_component(&widget).unwrap();

    // Should have import statement at top
    assert!(result.starts_with("import"), "File should start with import statement");
    assert!(result.contains("@kit.ArkUI"), "Should import from @kit.ArkUI");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang ark::generator::test_generated_file_has_imports -v`
Expected: FAIL

**Step 3: Implement fix**

In `generate_entry_component` (line 107-152), add imports at the beginning:

```rust
pub fn generate_entry_component(&mut self, widget: &AuraWidget) -> GenResult<String> {
    // ... existing setup ...

    let mut lines = Vec::new();

    // Add import statements
    lines.push("import { Button, Column, Row, Text } from '@kit.ArkUI';".to_string());
    lines.push(String::new());

    // ... rest of the function
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p auto-lang ark::generator::test_generated_file_has_imports -v`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "fix(ark): add missing import statements for ArkUI components"
```

---

## Task 7: End-to-End Verification

**Step 1: Build the project**

```bash
cargo build -p auto-lang
```

Expected: PASS (no compilation errors)

**Step 2: Run all ark tests**

```bash
cargo test -p auto-lang ark
```

Expected: All tests PASS

**Step 3: Generate Counter.ets and compare with reference**

```bash
cd examples/unified-demo
cargo run --release -- gen
cat arkts/entry/src/main/ets/pages/Counter.ets
```

Compare with `examples/counter.ets` reference file.

**Step 4: Verify key patterns in generated code**

Generated Counter.ets should have:
- [ ] `import { Button } from '@kit.ArkUI';` at top
- [ ] `enum Msg { Inc, Dec }` (not `sealed class`)
- [ ] `case Msg.Inc:` (with `case` keyword)
- [ ] `break;` at end of each case
- [ ] `this.count` (not `..count`)
- [ ] `Button('-')` (label in constructor)

---

## Files Changed Summary

```
crates/auto-lang/src/ui_gen/ark/
├── state.rs           # Fix enum, switch case/break (Tasks 1-3)
└── generator.rs       # Fix imports, member access, button order (Tasks 4-6)
```

---

## Success Criteria

- [ ] All 6 bugs fixed with passing tests
- [ ] `cargo test -p auto-lang ark` passes
- [ ] Generated Counter.ets matches reference patterns
- [ ] Generated code is valid TypeScript/ArkTS (no Kotlin syntax)

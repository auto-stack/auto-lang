# AURA → ArkTS Transpilation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Verify and test AURA → ArkTS transpilation for all 54 widgets, ensuring generated code compiles and runs correctly.

**Architecture:** Widget files already have `#[backend(ark, ...)]` annotations. ArkGenerator uses WidgetRegistry to look up mappings. Modifier DSL converts Tailwind to ArkTS modifiers. Focus is on testing and fixing any gaps.

**Tech Stack:** Rust (ArkGenerator, WidgetRegistry), ArkTS (HarmonyOS SDK), TypeScript definitions (.d.ts)

---

## Phase 1: Test Infrastructure Setup

### Task 1: Create a2ark Test Directory

**Files:**
- Create: `crates/auto-lang/test/a2ark/`
- Create: `crates/auto-lang/test/a2ark/README.md`

**Step 1: Create test directory structure**

```bash
mkdir -p crates/auto-lang/test/a2ark
```

**Step 2: Create README documenting test structure**

Create `crates/auto-lang/test/a2ark/README.md`:

```markdown
# A2ARK Tests (Auto → ArkTS)

Test cases for AURA → ArkTS transpilation.

## Structure

Each test case has:
- `input.at` - AURA source file
- `input.expected.ets` - Expected ArkTS output

## Running Tests

```bash
cargo test -p auto-lang -- ark
```

## Adding Tests

1. Create directory: `XXX_widget_name/`
2. Add `input.at` with widget test
3. Run test (will fail, create `.wrong.ets`)
4. Review output, rename to `.expected.ets` if correct
```

**Step 3: Commit**

```bash
git add crates/auto-lang/test/a2ark/
git commit -m "test(a2ark): create test directory structure"
```

---

### Task 2: Add a2ark Test Framework

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ark/generator.rs` (add test module)
- Create: `crates/auto-lang/test/a2ark/001_column/`
- Create: `crates/auto-lang/test/a2ark/001_column/input.at`
- Create: `crates/auto-lang/test/a2ark/001_column/input.expected.ets`

**Step 1: Create first test case directory**

```bash
mkdir -p crates/auto-lang/test/a2ark/001_column
```

**Step 2: Create test input file**

Create `crates/auto-lang/test/a2ark/001_column/input.at`:

```auto
widget TestColumn {
    view {
        col (class: "gap-4 p-2") {
            text "Hello"
            text "World"
        }
    }
}
```

**Step 3: Create expected output**

Create `crates/auto-lang/test/a2ark/001_column/input.expected.ets`:

```typescript
import { Button } from '@kit.ArkUI';

@Entry
@Component
struct TestColumnWidget {
  build() {
    Column() {
      Text('Hello')
      Text('World')
    }
    .padding(8)
    .justifyContent(FlexAlign.SpaceBetween)
  }
}
```

**Step 4: Add test helper function to generator.rs**

Add at end of `generator.rs` in the `#[cfg(test)]` module:

```rust
// A2ARK test helper
fn test_a2ark(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::path::PathBuf;

    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test/a2ark");

    let test_dir = base_dir.join(name);
    let input_path = test_dir.join("input.at");
    let expected_path = test_dir.join("input.expected.ets");
    let wrong_path = test_dir.join("input.wrong.ets");

    // Read input
    let input = fs::read_to_string(&input_path)?;

    // Parse and generate
    let widget = crate::aura::parse_aura(&input)?;
    let mut gen = super::ArkGenerator::new();
    let output = gen.generate_entry_component(&widget)?;

    // Compare with expected
    let expected = fs::read_to_string(&expected_path)?;

    // Normalize whitespace for comparison
    let output_normalized: String = output.lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    let expected_normalized: String = expected.lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    if output_normalized != expected_normalized {
        fs::write(&wrong_path, &output)?;
        return Err(format!(
            "Output mismatch for {}. See {}",
            name,
            wrong_path.display()
        ).into());
    }

    // Remove .wrong file if exists
    let _ = fs::remove_file(&wrong_path);

    Ok(())
}

#[test]
fn test_001_column() {
    test_a2ark("001_column").unwrap();
}
```

**Step 5: Run test to verify it fails**

```bash
cargo test -p auto-lang test_001_column -- --nocapture
```

Expected: Test fails (output doesn't match yet)

**Step 6: Review and fix expected output**

Run test, review `.wrong.ets`, update `.expected.ets` to match correct output.

**Step 7: Commit**

```bash
git add crates/auto-lang/test/a2ark/
git add crates/auto-lang/src/ui_gen/ark/generator.rs
git commit -m "test(a2ark): add column test case and framework"
```

---

### Task 3: Add Row and Box Tests

**Files:**
- Create: `crates/auto-lang/test/a2ark/002_row/`
- Create: `crates/auto-lang/test/a2ark/003_box/`

**Step 1: Create row test**

```bash
mkdir -p crates/auto-lang/test/a2ark/002_row
```

Create `002_row/input.at`:

```auto
widget TestRow {
    view {
        row (class: "gap-2 items-center") {
            text "Left"
            text "Right"
        }
    }
}
```

**Step 2: Add test function**

```rust
#[test]
fn test_002_row() {
    test_a2ark("002_row").unwrap();
}
```

**Step 3: Run test, review output**

```bash
cargo test -p auto-lang test_002_row -- --nocapture
```

**Step 4: Create box test**

```bash
mkdir -p crates/auto-lang/test/a2ark/003_box
```

Create `003_box/input.at`:

```auto
widget TestBox {
    view {
        box {
            text "Overlay"
        }
    }
}
```

**Step 5: Commit**

```bash
git add crates/auto-lang/test/a2ark/
git commit -m "test(a2ark): add row and box test cases"
```

---

## Phase 2: Core Widget Tests

### Task 4: Add Text Widget Test

**Files:**
- Create: `crates/auto-lang/test/a2ark/004_text/`

**Step 1: Create test**

```bash
mkdir -p crates/auto-lang/test/a2ark/004_text
```

Create `004_text/input.at`:

```auto
widget TestText {
    view {
        text (text: "Hello, World!", class: "text-lg font-bold text-blue-500") {}
    }
}
```

**Step 2: Add test function and run**

```rust
#[test]
fn test_004_text() {
    test_a2ark("004_text").unwrap();
}
```

**Step 3: Run test, review, commit**

```bash
cargo test -p auto-lang test_004_text -- --nocapture
git add crates/auto-lang/test/a2ark/
git commit -m "test(a2ark): add text widget test"
```

---

### Task 5: Add Button Widget Test

**Files:**
- Create: `crates/auto-lang/test/a2ark/005_button/`

**Step 1: Create test**

```bash
mkdir -p crates/auto-lang/test/a2ark/005_button
```

Create `005_button/input.at`:

```auto
widget TestButton {
    msg Msg { Click }

    view {
        button (text: "Submit", onclick: .Click) {}
    }
}
```

**Step 2: Add test function, run, commit**

```rust
#[test]
fn test_005_button() {
    test_a2ark("005_button").unwrap();
}
```

---

### Task 6: Add Input Widget Test

**Files:**
- Create: `crates/auto-lang/test/a2ark/006_input/`

**Step 1: Create test**

```bash
mkdir -p crates/auto-lang/test/a2ark/006_input
```

Create `006_input/input.at`:

```auto
widget TestInput {
    model {
        name str = ""
    }

    msg Msg { UpdateName(value: str) }

    view {
        input (value: .name, placeholder: "Enter name", onchange: .UpdateName) {}
    }

    on {
        UpdateName(value) => {
            name = value
        }
    }
}
```

**Step 2: Add test function, run, commit**

```rust
#[test]
fn test_006_input() {
    test_a2ark("006_input").unwrap();
}
```

---

### Task 7: Add Image Widget Test

**Files:**
- Create: `crates/auto-lang/test/a2ark/007_image/`

**Step 1: Create test with URL source**

```bash
mkdir -p crates/auto-lang/test/a2ark/007_image
```

Create `007_image/input.at`:

```auto
widget TestImage {
    view {
        image (src: "https://example.com/logo.png", class: "w-32 h-32 rounded-full") {}
    }
}
```

**Step 2: Add test function, run, commit**

---

### Task 8: Add Form Widgets Batch Test

**Files:**
- Create: `crates/auto-lang/test/a2ark/008_form_widgets/`

**Step 1: Create comprehensive form test**

```bash
mkdir -p crates/auto-lang/test/a2ark/008_form_widgets
```

Create `008_form_widgets/input.at`:

```auto
widget TestFormWidgets {
    model {
        checked bool = false
        toggled bool = false
        value float = 50.0
    }

    msg Msg {
        ToggleCheck
        ToggleSwitch
        UpdateSlider(v: float)
    }

    view {
        col (class: "gap-4") {
            checkbox (checked: .checked, onchange: .ToggleCheck) {}
            switch (value: .toggled, onchange: .ToggleSwitch) {}
            slider (value: .value, onchange: .UpdateSlider) {}
        }
    }

    on {
        ToggleCheck => { checked = !checked }
        ToggleSwitch => { toggled = !toggled }
        UpdateSlider(v) => { value = v }
    }
}
```

**Step 2: Add test function, run, review output**

---

## Phase 3: Verify Widget Annotations

### Task 9: Audit All Widget Annotations

**Files:**
- Read all: `stdlib/aura/widgets/**/*.at`

**Step 1: Run grep to list all ark backend annotations**

```bash
grep -r "#\[backend(ark" stdlib/aura/widgets/ | sort
```

**Step 2: Create checklist of mappings**

| Widget | Ark Component | Status |
|--------|---------------|--------|
| col | Column | ✓ |
| row | Row | ✓ |
| text | Text | ✓ |
| button | Button | ✓ |
| input | TextInput | ✓ |
| ... | ... | ... |

**Step 3: Verify each mapping against ArkTS SDK**

Check each component exists in `D:\Huawei\DevEco Studio\sdk\default\openharmony\ets\component\`

**Step 4: Document any mismatches**

Create issue list for incorrect mappings.

**Step 5: Commit checklist**

```bash
git add docs/plans/2026-03-22-aura-arkts-transpilation-implementation.md
git commit -m "docs: add widget annotation audit checklist"
```

---

### Task 10: Fix Incorrect Mappings

**Files:**
- Modify: Various `stdlib/aura/widgets/**/*.at` files

**Step 1: For each incorrect mapping found in Task 9, fix the annotation**

Example fix pattern:

```auto
// Before (incorrect)
#[backend(ark, component = "ListView")]

// After (correct)
#[backend(ark, component = "List")]
```

**Step 2: Run tests to verify fixes**

```bash
cargo test -p auto-lang -- ark
```

**Step 3: Commit fixes**

```bash
git add stdlib/aura/widgets/
git commit -m "fix(widgets): correct ark backend component mappings"
```

---

## Phase 4: Complex Widget Tests

### Task 11: Add List Widget Test

**Files:**
- Create: `crates/auto-lang/test/a2ark/010_list/`

**Step 1: Create test with ForEach**

```bash
mkdir -p crates/auto-lang/test/a2ark/010_list
```

Create `010_list/input.at`:

```auto
widget TestList {
    model {
        items List(str) = List.new()
    }

    view {
        list {
            for item in .items {
                list_item {
                    text item
                }
            }
        }
    }
}
```

**Step 2: Add test function, run, review**

---

### Task 12: Add Tabs Widget Test

**Files:**
- Create: `crates/auto-lang/test/a2ark/011_tabs/`

**Step 1: Create tabs test**

```bash
mkdir -p crates/auto-lang/test/a2ark/011_tabs
```

Create `011_tabs/input.at`:

```auto
widget TestTabs {
    model {
        activeTab int = 0
    }

    msg Msg { SwitchTab(index: int) }

    view {
        tabs (active: .activeTab, onchange: .SwitchTab) {
            tab (label: "Home") {
                text "Home content"
            }
            tab (label: "Settings") {
                text "Settings content"
            }
        }
    }

    on {
        SwitchTab(index) => { activeTab = index }
    }
}
```

**Step 2: Add test function, run, review**

---

### Task 13: Add Dialog Widget Test

**Files:**
- Create: `crates/auto-lang/test/a2ark/012_dialog/`

**Step 1: Create dialog test**

```bash
mkdir -p crates/auto-lang/test/a2ark/012_dialog
```

Create `012_dialog/input.at`:

```auto
widget TestDialog {
    model {
        showDialog bool = false
    }

    msg Msg {
        OpenDialog
        CloseDialog
    }

    view {
        col {
            button (text: "Show Dialog", onclick: .OpenDialog) {}
        }

        dialog (visible: .showDialog, onclose: .CloseDialog) {
            text "Dialog content"
        }
    }

    on {
        OpenDialog => { showDialog = true }
        CloseDialog => { showDialog = false }
    }
}
```

**Step 2: Add test function, run, review**

---

## Phase 5: Integration Testing

### Task 14: Add Full App Test

**Files:**
- Create: `crates/auto-lang/test/a2ark/100_full_app/`

**Step 1: Create comprehensive app test**

```bash
mkdir -p crates/auto-lang/test/a2ark/100_full_app
```

Create `100_full_app/input.at`:

```auto
widget App {
    routes {
        "/" -> index
        "/settings" -> settings
    }

    model {
        count int = 0
    }

    msg Msg {
        Increment
        Decrement
    }

    view {
        col (class: "w-full h-full justify-center items-center bg-white") {
            text (text: f"Count: ${.count}", class: "text-2xl font-bold") {}

            row (class: "gap-4") {
                button (text: "-", onclick: .Decrement) {}
                button (text: "+", onclick: .Increment) {}
            }

            navLink (to: "/settings") {
                text "Settings"
            }

            outlet
        }
    }

    on {
        Increment => { count += 1 }
        Decrement => { count -= 1 }
    }
}
```

**Step 2: Add test function, run, review**

---

### Task 15: Run All Tests and Fix Issues

**Step 1: Run all ark tests**

```bash
cargo test -p auto-lang -- ark
```

**Step 2: For each failing test:**
1. Review `.wrong.ets` output
2. Either fix expected output OR fix generator code
3. Re-run test

**Step 3: Ensure all tests pass**

```bash
cargo test -p auto-lang -- ark
```

Expected: All tests pass

**Step 4: Commit final state**

```bash
git add crates/auto-lang/test/a2ark/
git commit -m "test(a2ark): complete widget transpilation tests"
```

---

## Phase 6: Documentation

### Task 16: Update ArkTS Documentation

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Add ArkTS generation section to CLAUDE.md**

Add section:

```markdown
## a2ark - ArkTS (HarmonyOS) Code Generator

### Overview

a2ark generates ArkTS code from AURA widgets for HarmonyOS applications.

**Location**: `crates/auto-lang/src/ui_gen/ark/`

### Component Mappings

| AURA Tag | ArkTS Component |
|----------|-----------------|
| `col` | `Column` |
| `row` | `Row` |
| `text` | `Text` |
| `button` | `Button` |
| `input` | `TextInput` |
| ... | ... |

### Testing

```bash
# Run all a2ark tests
cargo test -p auto-lang -- ark

# Run specific test
cargo test -p auto-lang test_001_column
```
```

**Step 2: Commit documentation**

```bash
git add CLAUDE.md
git commit -m "docs: add a2ark generator documentation"
```

---

## Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| 1 | 1-3 | Test infrastructure setup |
| 2 | 4-8 | Core widget tests |
| 3 | 9-10 | Verify and fix widget annotations |
| 4 | 11-13 | Complex widget tests |
| 5 | 14-15 | Integration testing |
| 6 | 16 | Documentation |

**Total: 16 tasks**

**Success Criteria:**
- All 16 test cases pass
- All widget annotations verified
- Documentation updated

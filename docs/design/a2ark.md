# a2ark - ArkTS (HarmonyOS) Code Generator

> Extracted from CLAUDE.md for reference. See CLAUDE.md for rules and quick reference.

### Overview

a2ark generates ArkTS code from AURA widgets for HarmonyOS applications.

**Location**: `crates/auto-lang/src/ui_gen/ark/`

### Architecture

```text
AuraWidget → ArkGenerator → ArkTS Code
                │
                ├── WidgetRegistry (component mappings)
                ├── ArkModifierDsl (Tailwind → ArkTS modifiers)
                └── Test Framework (a2ark tests)
```

### ArkTS Code Generation Rules

- Generated ArkTS must use TypeScript syntax, not Kotlin (no sealed classes, use proper TypeScript types)
- Array literals need explicit type annotations in ArkTS (use `Object[]` or specific interface types)
- Component syntax follows `@Component` decorator patterns
- Use `Object` instead of `any` for dynamic types (ArkTS forbids `any`/`unknown`)
- Object literals must correspond to explicitly declared interfaces

### Component Mappings

| AURA Tag | ArkTS Component | Notes |
|----------|-----------------|-------|
| `col` | `Column` | Built-in |
| `row` | `Row` | Built-in |
| `box` | `Stack` | Built-in |
| `text` | `Text` | Built-in |
| `button` | `Button` | Built-in |
| `input` | `TextInput` | Built-in |
| `image` | `Image` | Built-in |
| `checkbox` | `Checkbox` | Built-in |
| `switch` | `Toggle` | Built-in |
| `slider` | `Slider` | Built-in |
| `tabs` | `Tabs` | Built-in |
| `dialog` | `AlertDialog` | Built-in |
| `Table` | `Column` | Composite |
| `TabsList` | `Row` | Composite |
| `TabsContent` | `TabContent` | Built-in |
| `DialogContent` | `Column` | Composite |

### Testing

```bash
# Run all a2ark tests
cargo test -p auto-lang --lib -- generator::tests::test_0

# Run specific test
cargo test -p auto-lang --lib -- generator::tests::test_001_column
```

### Test Structure

Located in `crates/auto-lang/test/a2ark/`:
- `001_column/` - Column widget test
- `002_row/` - Row widget test
- `003_box/` - Box/Stack widget test
- `004_text/` - Text widget test
- `005_button/` - Button widget test
- `006_input/` - Input/TextInput widget test
- `007_image/` - Image widget test
- `008_form_widgets/` - Checkbox/Switch form widgets test
- `010_table/` - Table widget test
- `011_tabs/` - Tabs widget test
- `012_dialog/` - Dialog widget test

Each test case has:
- `input.at` - AURA source file
- `input.expected.ets` - Expected ArkTS output

### Adding Tests

1. Create directory: `XXX_widget_name/`
2. Add `input.at` with widget test
3. Run test (will fail, create `.wrong.ets`)
4. Review output, rename to `.expected.ets` if correct
5. Add test function in `generator.rs`

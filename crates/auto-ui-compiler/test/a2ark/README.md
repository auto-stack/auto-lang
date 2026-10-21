# A2ARK Tests (Auto -> ArkTS)

Test cases for AURA -> ArkTS transpilation.

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

## Test Naming Convention

- `000-099_*`: Core language features
- `100-199_*`: Standard library widgets
- `200-299_*`: Complex UI patterns

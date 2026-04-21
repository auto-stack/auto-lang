# Plan 110: AutoDown Comprehensive Test Suite

## Context

AutoDown implementation is complete (~4,273 lines) but tests are broken (syntax errors in `test/autodown/mod.rs`). This plan establishes a comprehensive test suite covering lexer, parser, transpilers, math, and edge cases.

## Design Decisions

- **Organization**: Hybrid - inline unit tests for lexer/parser, snapshot tests for transpilers
- **Priorities**: All categories (lexer, parser, Typst, HTML, math, edge cases, errors)
- **Fixtures**: Inline for small tests, snapshot files for integration tests

## Test Directory Structure

```
crates/auto-lang/test/autodown/
├── mod.rs                    # Test runner + inline unit tests
├── snapshot/                 # Snapshot-based integration tests
│   ├── 000_basic/
│   │   ├── basic.ad
│   │   ├── basic.expected.typ
│   │   └── basic.expected.html
│   ├── 001_headers/
│   ├── 002_lists/
│   ├── 003_math/
│   ├── 004_control_flow/
│   ├── 005_interpolation/
│   ├── 006_components/
│   ├── 007_tables/
│   ├── 008_code_blocks/
│   ├── 009_edge_cases/
│   └── 010_error_recovery/
└── fixtures/                 # Shared test data
    └── sample.ad
```

## Phase 1: Lexer Unit Tests (Inline)

**File**: `test/autodown/mod.rs`

### 1.1 Text Mode Tests
- `test_lexer_plain_text()`
- `test_lexer_newlines()`
- `test_lexer_blank_lines()`
- `test_lexer_unicode()`

### 1.2 Header Tests
- `test_lexer_h1_header()` through `test_lexer_h6_header()`
- `test_lexer_header_with_text()`

### 1.3 Math Mode Tests
- `test_lexer_inline_math()`
- `test_lexer_block_math()`
- `test_lexer_math_operators()`
- `test_lexer_math_functions()`

### 1.4 Code/Logic Mode Tests
- `test_lexer_dollar_keyword()`
- `test_lexer_interpolation()`
- `test_lexer_if_block()`
- `test_lexer_for_block()`
- `test_lexer_component_call()`

### 1.5 Inline Markup Tests
- `test_lexer_bold()`
- `test_lexer_italic()`
- `test_lexer_inline_code()`
- `test_lexer_links()`

### 1.6 List Tests
- `test_lexer_unordered_list()`
- `test_lexer_ordered_list()`
- `test_lexer_nested_list()`

### 1.7 Mode Transition Tests
- `test_lexer_flip_text_to_code()`
- `test_lexer_flip_code_to_text()`
- `test_lexer_flip_text_to_math()`
- `test_lexer_nested_modes()`

## Phase 2: Parser Unit Tests (Inline)

**File**: `test/autodown/mod.rs`

### 2.1 Document Structure Tests
- `test_parser_empty_document()`
- `test_parser_simple_paragraph()`
- `test_parser_multiple_paragraphs()`
- `test_parser_front_matter()`

### 2.2 Section Tests
- `test_parser_single_section()`
- `test_parser_nested_sections()`
- `test_parser_section_with_content()`
- `test_parser_section_ids()`

### 2.3 Block Tests
- `test_parser_paragraph_block()`
- `test_parser_code_block()`
- `test_parser_blockquote()`
- `test_parser_list_block()`
- `test_parser_table_block()`

### 2.4 Inline Tests
- `test_parser_inline_bold()`
- `test_parser_inline_italic()`
- `test_parser_inline_code()`
- `test_parser_inline_math()`
- `test_parser_inline_interpolation()`
- `test_parser_mixed_inline()`

### 2.5 Control Flow Tests
- `test_parser_if_statement()`
- `test_parser_if_else_statement()`
- `test_parser_for_loop()`
- `test_parser_nested_control_flow()`

### 2.6 Component Tests
- `test_parser_simple_component()`
- `test_parser_component_with_props()`
- `test_parser_component_with_children()`
- `test_parser_nested_components()`

## Phase 3: Transpiler Snapshot Tests

**Directory**: `test/autodown/snapshot/`

| Test Case | Input | Outputs |
|-----------|-------|---------|
| 000_basic | Simple paragraph | `.expected.typ`, `.expected.html` |
| 001_headers | H1-H6 hierarchy | Header formatting |
| 002_lists | Ordered, unordered, nested | List structures |
| 003_math | Inline/block formulas | Typst `$...$`, HTML KaTeX |
| 004_control_flow | `$if`, `$for` | Typst `#if`/`#for` |
| 005_interpolation | `${variable}` | Variable substitution |
| 006_components | `$callout`, etc. | Component rendering |
| 007_tables | Table with headers | Table markup |
| 008_code_blocks | Fenced code | Syntax highlighting |
| 009_edge_cases | Empty, whitespace | Graceful handling |
| 010_error_recovery | Unclosed math | Error messages |

## Phase 4: Math Parser Tests (Inline)

### 4.1 Function Parsing
- `test_math_sum()`, `test_math_prod()`, `test_math_integral()`
- `test_math_sqrt()`, `test_math_trig()`

### 4.2 Operator Tests
- `test_math_superscript()`, `test_math_subscript()`
- `test_math_fraction()`, `test_math_implicit_mult()`

## Phase 5: Error Handling Tests (Inline)

### 5.1 Lexer Errors
- `test_error_unclosed_math()`
- `test_error_unclosed_interpolation()`
- `test_error_invalid_escape()`

### 5.2 Parser Errors
- `test_error_unexpected_token()`
- `test_error_missing_brace()`
- `test_error_invalid_expression()`

### 5.3 Transpiler Errors
- `test_error_unsupported_node()`
- `test_error_circular_ref()`

## Implementation Steps

1. **Fix existing mod.rs** - Remove broken code, add basic structure
2. **Add lexer tests** - Phase 1 (~25 tests)
3. **Add parser tests** - Phase 2 (~30 tests)
4. **Create snapshot directory** - Phase 3 infrastructure
5. **Add snapshot tests** - 11 test cases with `.ad`/`.expected` files
6. **Add math tests** - Phase 4 (~12 tests)
7. **Add error tests** - Phase 5 (~10 tests)
8. **Add test runner** - `test_autodown()` function similar to `test_a2c()`

## Test Runner Pattern

```rust
fn test_autodown(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let base = PathBuf::from("test/autodown/snapshot").join(name);
    let input = std::fs::read_to_string(base.join(format!("{}.ad", name)))?;
    
    let doc = AdocParser::new(&input).parse()?;
    
    // Typst
    let typst = TypstTranspiler::new().transpile(&doc)?;
    let expected_typ = std::fs::read_to_string(base.join(format!("{}.expected.typ", name)))?;
    assert_eq!(typst, expected_typ);
    
    // HTML
    let html = HtmlTranspiler::new().transpile(&doc)?;
    let expected_html = std::fs::read_to_string(base.join(format!("{}.expected.html", name)))?;
    assert_eq!(html, expected_html);
    
    Ok(())
}

#[test]
fn test_000_basic() { test_autodown("000_basic/basic").unwrap(); }
```

## Expected Test Count

| Category | Count |
|----------|-------|
| Lexer Unit | ~25 |
| Parser Unit | ~30 |
| Snapshot Integration | ~11 |
| Math | ~12 |
| Error Handling | ~10 |
| **Total** | **~88 tests** |

## Files to Modify/Create

1. `crates/auto-lang/test/autodown/mod.rs` - Fix and expand
2. `crates/auto-lang/test/autodown/snapshot/` - Create directory + test cases

## Verification

```bash
cargo test -p auto-lang -- autodown
```

## Success Criteria

- All 88+ tests pass
- Zero compilation errors
- Snapshot tests compare correctly
- Error tests produce meaningful messages

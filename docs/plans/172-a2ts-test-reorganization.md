# Plan 172: A2TS Test Suite Reorganization

## Objective

Reorganize the a2ts (Auto-to-TypeScript) transpiler test suite from flat numbered directories into a categorized structure aligned with a2r (Plan 170) and a2c (Plan 171).

## Status: COMPLETE

All tasks implemented and verified. 24 a2ts tests passing, 0 wrong files.

## Current State (Before)

## Current State (Before)

- 24 test directories with flat numbering (000-033, gaps)
- 19 stale `.wrong.ts` files
- No clear grouping by feature category

## Final State (After)

- 24 test cases across 10 categorized directories
- 0 stale `.wrong.ts` files
- Categories aligned with a2r/a2c (01-17 shared, 18+ a2ts-specific)

## Design

### Directory Structure

```
test/a2ts/
  01_basics/           # Hello, functions
  02_types/            # Struct, enum, alias
  03_control_flow/     # if, for, while, nested if, loop, blocks
  05_expressions/      # Object, composition, range
  06_pattern_matching/ # Hetero enums
  07_ownership/        # Union types
  09_option_result/    # Closures
  11_methods/          # Instance methods, struct methods, extension methods
  12_specs/            # Basic spec, full spec
  13_delegation/       # Delegation
  18_ts_interop/       # For-each iteration
```

### Categories and Test Mappings

#### 01_basics (2 tests)

| Old | New |
|-----|-----|
| 000_hello | 01_basics/001_hello |
| 003_func | 01_basics/002_func |

#### 02_types (3 tests)

| Old | New |
|-----|-----|
| 006_struct | 02_types/001_struct |
| 007_enum | 02_types/002_enum |
| 009_alias | 02_types/003_alias |

#### 03_control_flow (6 tests)

| Old | New |
|-----|-----|
| 010_if | 03_control_flow/001_if |
| 011_for | 03_control_flow/002_for |
| 013_while | 03_control_flow/003_while |
| 015_nested_if | 03_control_flow/004_nested_if |
| 017_loop | 03_control_flow/005_loop |
| 019_blocks | 03_control_flow/006_blocks |

#### 05_expressions (3 tests)

| Old | New |
|-----|-----|
| 028_object | 05_expressions/001_object |
| 029_composition | 05_expressions/002_composition |
| 030_range_expr | 05_expressions/003_range_expr |

#### 06_pattern_matching (1 test)

| Old | New |
|-----|-----|
| 014_hetero_enum | 06_pattern_matching/001_hetero_enum |

#### 07_ownership (1 test)

| Old | New |
|-----|-----|
| 013_union | 07_ownership/001_union |

#### 09_option_result (1 test)

| Old | New |
|-----|-----|
| 014_closure | 09_option_result/001_closure |

#### 11_methods (3 tests)

| Old | New |
|-----|-----|
| 008_method | 11_methods/001_method |
| 017_struct_methods | 11_methods/002_struct_methods |
| 033_ext | 11_methods/003_ext |

#### 12_specs (2 tests)

| Old | New |
|-----|-----|
| 016_basic_spec | 12_specs/001_basic_spec |
| 017_spec | 12_specs/002_spec |

#### 13_delegation (1 test)

| Old | New |
|-----|-----|
| 018_delegation | 13_delegation/001_delegation |

#### 18_ts_interop (1 test)

| Old | New |
|-----|-----|
| 018_for_each | 18_ts_interop/001_for_each |

### Summary

| Metric | Before | After |
|--------|--------|-------|
| Test cases | 24 | 24 |
| Category directories | 0 (flat) | 10 |
| Stale .wrong files | 19 | 0 |

# Plan Report: 226-239 Status Audit (2026-05-08)

## Summary

- **Total plans audited:** 18 files (covering plan numbers 226-239)
- **Fully completed and moved to old/:** 13 files
- **Partially complete (kept in plans/):** 3 plans
- **Duplicate/overlapping:** 2 files merged

## Completed Plans → old/

| Plan | Title | Key Deliverables |
|------|-------|-----------------|
| 226 | ABT Bytecode Text Format | ABC↔ABT assembler/disassembler, Playground bytecode tab |
| 227 | Dynamic UI Iced Backend | `run_file()` auto-detects widget/app, iced window |
| 228 | Hetero Enum Tuple Syntax | Parser requires `(T, T)` for multi-field variants |
| 229a | IS_VARIANT Primitive Fix | Engine-level i32 Option compatibility |
| 230 | f64 Struct Literal Fix | PROMOTE_F64 in 5 codegen paths |
| 231 | Nested mut fn Stack Fix | SET_GENERIC_FIELD Void marking + BUILD_FSTR formatting |
| 232 | a2r Lexer Compilation | `.sub()`/`.slice()` handler + post_process() for types |
| 233-P0 | AAVM Parser P0 | tokenize_list() + Pratt parser + 20 tests |
| 233-Full | AAVM Parser P0+P1 | Combined plan document |
| 234-P1 | AAVM Parser P1 | 10 features: closure/fstr/is/enum/use/ext/spec/alias/object |
| 235-a2vue | a2vue Transpiler Gaps | ts_adapter fixes + storage/event/json/math/date/router builtins |
| 235-eval | AAVM Evaluator Plan | Plan doc (implemented in 236) |
| 236 | AAVM Evaluator Impl | Tree-walking eval + AST restructuring + 16 tests |
| 238 | Charts Replica | area/bar/line/donut chart registry + prop mapping |
| 239 | AAVM List/Map Bytecode | BVM heap + 8 opcodes (LIST_NEW/PUSH/GET/LEN, MAP_*) |

## Incomplete Plans (kept in docs/plans/)

### 229: Auto 自举编译器
- **Status:** Phase 1 fully complete (token + lexer + parser + eval + typeinfer + codegen + vm)
- **Remaining:** Phase 2 (a2r transpiler in Auto) + Phase 3 (self-bootstrapping)
- **Next step:** Plan 237 Phase E

### 234: A3UI A2Vue Replica
- **Status:** Phase 0-2 complete, Phase 3 partial
- **Remaining:** Widget Editor full features, Catalog pages, Theater/Icons, State management, Polish
- **Next step:** Phase 3 completion

### 237: AAVM Architecture Gap Closure
- **Status:** Phase A-C complete (value encoding, type inference, bytecode compiler)
- **Remaining:** Phase D (generic monomorphization), Phase E (a2r transpiler)
- **Next step:** Phase D or E

## Overlapping Plans Resolved

| Overlap | Resolution |
|---------|-----------|
| 233-aavm-parser-p0 + 233-aavm-parser + 234-aavm-parser-p1 | All 3 moved to old/; parser fully complete |
| 235-aavm-evaluator + 236-aavm-evaluator | Both moved to old/; evaluator fully complete (236 implemented it) |
| 229-vmtest-08 (fix) + 229-self-hosting (project) | Fix moved to old/; project plan stays active |

## Test Coverage

Total AAVM bootstrap tests: 68 directories (001-068)
- Token/Lexer: 001-007
- Parser P0: 008-027
- Parser P1: 028-037
- Tree-walking eval: 038-053
- Type inference: 054-055
- Bytecode: 060-068

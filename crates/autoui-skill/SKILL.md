---
name: autoui
description: |
  Generate and modify AutoUI (.at) projects safely. Use when creating
  new UI widgets, stores, or pages; when adding AutoDownEditor; when
  debugging UI generation issues. Enforces generator contracts (C1-C9)
  and known-good patterns from 015-notes.
---

# AutoUI Generation Skill

## When to use

- Creating a new AutoUI project (`auto new app`)
- Adding a widget / store / page to an existing project
- Modifying .at files (style, structure, handlers)
- Integrating AutoDownEditor or other stateful components
- Debugging "generated code doesn't work" issues

## Workflow

1. **Classify the task**: new project / add component / modify existing / debug
2. **Select pattern**: match the task to a pattern in `patterns/`
3. **Generate**: produce .at code following the pattern
4. **Validate**: run `auto build` and check warnings (Plan 361 validators)
5. **Update acceptance contract**: if behavior changes, update `tests/acceptance.atd`
   in lockstep (add/modify the T-entry for the affected feature)
6. **Generate test skeleton**: for new features, emit a `.spec.ts` skeleton
   matching the new T-entry
7. **Smoke test**: run `pnpm test:smoke` in `tests/` to verify no regression

## Critical rules (ALWAYS follow)

- **改 .at 时，同步更新 acceptance.atd**
- **新功能必须配测试**——不要让功能"裸奔"
- **修 bug 时加回归测试**（标注"历史教训"）
- **引用 generator-contracts.md 的关键不变量**（C1-C9）
- **AutoDownEditor 必须用单实例 + prop 切换**，不要用双实例 v-if
- **handler 名用 ToggleDarkMode 精确匹配**（生成器只识别这个）
- **store 声明用 `use store: Name` 语法**

## Available tools

- `auto watch` — incremental SFC regeneration (<1s feedback)
- `auto ui inspect <file.at>` — show widget structure + validation warnings
- `auto build` — full project build with validation
- `pnpm test:smoke` — run playwright smoke tests (tests/smoke.spec.ts)
- `cargo test -p auto-lang -- ui_snapshots` — snapshot regression tests

## Reference documents

| Document | Contents |
|----------|----------|
| `reference/generator-contracts.md` | C1-C9: generator implicit assumptions |
| `reference/known-pitfalls.md` | P1-P5: known anti-patterns and fixes |
| `checks/pre-commit-checklist.md` | Pre-commit validation checklist |

## Patterns

| Pattern | Use case |
|---------|----------|
| `patterns/list-detail.md` | List + detail panel layout |
| `patterns/editor-integration.md` | AutoDownEditor integration (most error-prone) |
| `patterns/store-pattern.md` | Store composable declaration and usage |
| `patterns/dark-mode.md` | Dark mode with accent theming |

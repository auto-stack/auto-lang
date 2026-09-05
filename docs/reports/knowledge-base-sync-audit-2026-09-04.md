# AutoLang 知识库同步审计（2026-09-04）

> 审计性质：只读基线；沉淀来源：PLAN-543。
> 代码快照：`plan-543-dev` 创建于 `432e15dabc601c219d9da18e125e445804883e76`。
> 并行边界：PLAN-532、PLAN-536 及其实现 worktree 不在本审计变更范围内。

## 1. 结论

AutoLang 已经形成 Design + Spec + Plan 的完整知识框架，但截至本快照只能评为
“热点模块局部同步”：`docs/specs/` 比根级设计文档更接近代码，Plan 保存了丰富的执行证据，
但当前事实、目标设计、历史决策和机器索引仍存在多个可写副本。读者无法仅依赖一个入口
稳定判断某项陈述是 current-state、target-state 还是 historical snapshot。

## 2. 仓库规模与资产

| 资产 | 本次快照 |
|---|---:|
| Rust workspace package | 约 20 |
| `crates/auto-lang/src/**/*.rs` | 约 528 |
| `docs/design/**/*.md` | 约 117 |
| `docs/specs/**/*.md` | 约 116 |
| `docs/plans/**/*.md` | 约 581 |
| `docs/plans/archive/**/*.md` | 约 528 |

这些数字用于描述数量级，不作为手工维护的长期常量；后续应由 Generated Catalog 从 Git
跟踪文件和 workspace metadata 自动生成。

## 3. 实际模块与主执行路径

```text
Auto source / stdlib
  → frontend: lexer → parser/AST → dialect → resolver
  → semantic: TypeStore → infer/typeck → ownership → comptime
  → compile infra: CompileSession → Database → QueryEngine/cache
  → backends
      ├─ AutoVM: bytecode/codegen → engine/heap/native/FFI/concurrency
      ├─ Transpiler: C/Rust/JS/TS/Python/GDScript/r2a
      └─ AutoUI: AURA schema → ui_gen/VTree → iced/gpui/Vue/headless
  → desktop/system: session/WM/VirtualWindow/protocol/MCP/native integration
```

外围 workspace 还包括 `auto-val`、`auto-atom`、宏、CLI、包管理、缓存、LSP、Playground、
VM 工具、自举编译器、parity workspace、网站和示例应用。

## 4. 已验证的代码—文档漂移

| # | 文档陈述 | 代码/较新 spec 证据 | 判断 |
|---|---|---|---|
| 1 | 根架构把 `eval.rs` Evaluator 列为执行后端 | 旧 evaluator 已删除，公共执行入口统一走 AutoVM | stale |
| 2 | QueryEngine smart caching 标为 deferred/open question | `compile.rs` 明确写有 `QueryEngine integration complete` 并按需复用 | stale |
| 3 | Design 05 描述 32-bit stack、约 120 opcode | VM spec 记录 NaN-boxed `u64` 和 178 opcode | stale |
| 4 | spec overview 记录 `auto-lang` 463 个 src 文件 | 本快照约 528 个 Rust src 文件 | stale |
| 5 | overview 活跃线停在 PLAN-462～465 | 仓库已完成更多后续计划，当前并行计划为 532/536 等 | stale |
| 6 | `.autoos/specs.json` 被称为机器账本 | `*.json` ignore 规则使其未进入 Git，且本地覆盖不完整 | 非共享真相 |
| 7 | Plan 编号由中央取号保证唯一 | 历史上曾出现 active/archive 重复或不同主题共用编号 | 门禁不足 |

## 5. 现有检查能力

基线命令：

```text
python scripts/spec-lint.py --stale-days 7
```

基线结果为 `0 errors, 9 warnings, 0 infos`：3 个相对链接断链，另有 comptime、interpreter、
mcp、runtime、trans、types 六个 module overview 超过新鲜度阈值。

现有 lint 能验证 project/module 结构、活跃 Plan 编号、相对链接和按提交日期计算的 stale；
尚未覆盖：archive 全局编号唯一性、Cargo/npm 清单、源码路径或符号、plan→spec 沉淀覆盖、
Markdown↔`.autoos/specs.json` 一致性、生成 INDEX 的可重现性以及 code owner。

## 6. 同步度判断

| 层 | 判断 | 说明 |
|---|---|---|
| Design | 红/黄 | 意图和历史价值高，但旧实现陈述未显式标为 historical |
| Specs | 黄 | 当前最可信；热点 UI/VM/AAVM 较新，核心旧模块更新不均 |
| Plans | 黄 | 过程证据丰富；历史格式、编号和 active/archive 轨迹不完全一致 |
| Machine ledger | 红 | 本地、ignored、覆盖不完整，不能承担 canonical source |
| Checks | 黄 | 已有 lint 骨架，但缺少 change-based 和 semantic verification |

## 7. 可复现命令

```text
git status --short --branch
git worktree list --porcelain
rg --files crates/auto-lang/src -g *.rs
rg --files docs/design -g *.md
rg --files docs/specs -g *.md
rg --files docs/plans -g *.md
python scripts/spec-lint.py --stale-days 7
python scripts/spec-index.py
git check-ignore -v .autoos/specs.json
git ls-files .autoos/specs.json
```

## 8. 审计边界

本报告不宣称已逐条验证 117 篇 Design 和 116 篇 Spec 的全部语义；它记录的是可复现的
仓库级基线和已经找到的高置信度分歧。逐模块语义 rebaseline、lint/catalog 实现和四技能
升级分别进入 Design 27 定义的后续工作包。

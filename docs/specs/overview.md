# Auto-Lang Monorepo 全局总览

> **用途**：`/auto-plan:new` Step 2 的取材源——新计划的需求分析从这里获得项目现状背景。
> 维护：merge 时如有结构性变化（新模块/crate/状态翻转）须更新（规约 §4）。
> 更新：2026-08-28（Plan 467）

**一句话**：Auto 语言——AI 原生的多目标（VM/C/Rust/TS/Python）脚本与 UI 语言，
"脚本开发 → 转译发布"双形态，配套 AutoUI 跨端 UI 与桌面生态。

---

## 仓库地图

### 语言核心与基础库（crates/，workspace 成员）

| crate | 职责 | 状态 |
|---|---|---|
| [auto-lang](auto-lang/project.md) | 语言核心：lexer/parser/typeck/infer/ownership/comptime/interpreter/VM/trans/ui/mcp（463 个 src 文件，九模块见下） | active |
| [auto-val](auto-val/project.md) | 运行时 Value/Node 值体系（含 serde 双向、nano_value） | active |
| [auto-atom](auto-atom/project.md) | Atom 静态数据结构与解析器 | active |
| [auto-macros](auto-macros/project.md) | 过程宏 `value!`（AutoLang 语法→Value） | active |
| [a2r-std](a2r-std/project.md) | a2r 转译产物的 Rust 运行时库（http/json/task/str…） | active |
| [stdlib](stdlib/project.md) | `.at` 写的语言标准库（auto/aura/c/collections/may/result） | active |
| [aavm](aavm/project.md) | 自举实验：`auto/` 目录用 .at 实现编译器（plans 429–434 系列） | experimental |

### 工具链（crates/）

| crate | 职责 | 状态 |
|---|---|---|
| [auto](auto-cli/project.md)（目录 auto-cli spec） | `auto` CLI 主入口（run/ui/vue/block/docs…） | active |
| [auto-man](auto-man/project.md) | 包管理器/构建调度/工程导出/各前端生态集成 | active |
| [auto-gen](auto-gen/project.md) | AutoGen 代码生成器（模板/guard） | active |
| [auto-lsp](auto-lsp/project.md) | LSP 服务器（补全/诊断/hover/跳转/semantic tokens） | active |
| [auto-vm](auto-vm/project.md) | AutoVM 辅助 CLI（dump_code） | active |
| [auto-cache](auto-cache/project.md) | 全局构建缓存（sqlite/指纹/GC/sandbox/shim 管线） | active |
| [auto-bindgen](auto-bindgen/project.md) | C FFI 头文件清单生成 | active |
| [shim-metadata](shim-metadata/project.md) | rustdoc 元信息→FFI shim 包生成（Plan 430 管线） | active |
| [a2r-actor-tests](a2r-actor-tests/project.md) | actor 行为 parity 慢测试（转译→编译→对拍 VM） | test-only |

### UI/Web 生态

| 资源 | 职责 | 状态 |
|---|---|---|
| [auto-playground](auto-playground/project.md) | Playground Web API 服务（axum+ws+前端） | active |
| [packages/](../design/autoui/examples-app-track.md) | 4 个 JS 包：@auto-ui/widgets、forge-ui、lab-ui、playground-vue | active |
| [blocks/](blocks/project.md) | Skill 级 UI 区块包（Design 17） | active |
| [website/](website/project.md) | 文档站点 + 8 本书籍 + playground 页 | active |
| [autoui-skill](autoui-skill/project.md) | AI agent 技能包（AutoUI 项目生成契约 C1–C9） | active |

### 外围与实验

| 资源 | 职责 | 状态 |
|---|---|---|
| [parity/](parity/project.md) | 独立 workspace：三后端行为一致性验证（30+ 库语料） | active |
| [auto-cosmic](auto-cosmic/project.md) | COSMIC 桌面复刻实验（4 子 crate，Linux 向，无消费者） | experimental |
| examples/ | 51 项示例（ui 应用轨道 0xx 系列/godot/http/charts gallery…） | active |
| schema/ | `aura.at`——AURA 内置组件唯一声明源（Plan 435） | active |
| test/ tests/ | 特性 fixture 与语言级 e2e .at 套件 | active |
| tools/ deploy/ | fstr 转换、shadcn 快照；playground 部署配置 | 辅助 |

### 孤儿与废弃（勿新增依赖，待处置）

- `crates/auto-xml`：非 workspace 成员（Plan 325 注明已迁 ../auto-ai），Cargo.toml 仍用
  workspace 依赖，单建必失败。
- `crates/auto-lang-macros`：auto-macros 的逐字废弃副本，无人依赖。
- `crates/ac-examples`：无 Cargo.toml 的遗留目录（仅 2 个 .at 测试脚本）。
- 根 `auto-shell/`：仅剩 Cargo.lock（crate 已迁独立仓 D:/autostack/auto-shell）。
- `auto/lib-legacy/`：自举库旧版封存。
- `crates/Cargo.lock`：多余（workspace lock 应只在根）。

---

## auto-lang 九模块状态（详见各 module spec）

| 模块 | 职责 | 近期演进（plans） |
|---|---|---|
| frontend | lexer/token/parser/AST/dialect/resolver/宏 | 448 语法改进、442 跨平台闭包 |
| types | infer/typeck/ownership/trait_checker | 453 W5 ext impl 库方法 |
| comptime | 编译期求值 | 稳定 |
| interpreter | TreeWalker 解释器 | 436 setup 前导语义 |
| vm | AutoVM：abt/codegen/engine/debugger/ffi/generic | 446 渲染后端薄弱点清偿、466 测试提速 |
| trans | 转译后端：C/Rust/JS/TS/Python/GDScript/r2a | 400/415 a2r 深化、417/427 parity 修复线 |
| runtime | runtime/scope/session/libs/ffi | 442 平台桥 natives |
| ui | ui/（iced/gpui/headless/interpreter）+ ui_gen/ + a2ui/ + aura/ | 437–465 AutoUI 桌面线主战场 |
| mcp | MCP server 集成 | 稳定 |

## 活跃开发线（2026-08）

1. **AutoUI 桌面轨道**（最热）：虚拟桌面 WM（462）→ 桌面 Shell 自动排布（463）→
   Launcher（464）→ Vue 虚拟桌面（465）；前置：charts（437）/database（439）/
   file-manager（440）/launcher 示例（441）/主题系统（458）/双端 parity（455）。
   设计：Design [20](../design/20-autoui-separation-architecture.md)–[24](../design/autoui/desktop-shell-and-launcher.md)。
2. **a2r parity 线**：功能差距 tracker（242）+ api_gen body 转译（400）+ 剩余大项（415）。
3. **构建与测试基础设施**：466（sccache/cargo t ≤30s/全量门禁收敛 review）已落地。
4. **LSP/VSCode**：416 Phase 5–6（semantic tokens/TS 迁移）。

## 入口索引

- 全局索引：[INDEX.md](INDEX.md)（脚本生成）
- 目标账本：[goals.md](goals.md)
- 技术债：[docs/plans/KNOWN-DEBT-AND-RISKS.md](../plans/KNOWN-DEBT-AND-RISKS.md)
- 计划状态审计：[docs/plans/plans-status-audit-2026-08-20.md](../plans/plans-status-audit-2026-08-20.md)
- 设计文档：[docs/design/00-intro.md](../design/00-intro.md)
- 开发范式：[docs/specs/README.md](README.md)（v2）+ [Design 26](../design/autoplan-spec-ledger.md)

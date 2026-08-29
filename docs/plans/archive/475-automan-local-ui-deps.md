---
plan_id: PLAN-475
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: automan-local-ui-deps
author: [Antigravity]
created_at: 2026-08-29
updated_at: 2026-08-29

supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-man, auto-lang/ui, auto-lang/vm, auto-lang/vue]
current_step: 5
total_steps: 5
---

# [PLAN-475] AutoMan Local UI Dependencies & Cross-Project Widget Sharing

## 变更摘要

本计划旨在打通 AutoLang 在多工程场景下的**本地 UI 组件库依赖与跨工程复用能力**：
1. 在 `pac.at` 中正式支持本地路径依赖语法（如 `dep "common" { path: "../common" }`）。
2. 在 `crates/auto-man` 中实现基于软链接（Windows NTFS Directory Junction + Unix Symlink）与 Worktree 的依赖物化机制，实现零拷贝热重载与版本快照支持。
3. 在 `crates/auto-lang` 的 VM 模块寻址（`resolve_module_path`）中扩展 `./deps/` 搜索路径，支持 VM 原生模式解析依赖工程中的 Widget。
4. 在 `crates/auto-man` 的 `VueProject` 编译器中集成对 `./deps/*/src/front/` 的扫描、组件转译与依赖合并，支持 Vue 模式跨包引用。
5. 将 `examples/ui/010-contact-form` 的通用顶栏抽取并重构为引用 `examples/ui/common`，双端通过 `autoui-verifier` 验证 100% 视觉与功能对齐。

---

## 目标

- **G1（语法支持）**：`pac.at` 支持声明 `dep "<name>" { path: "<rel_or_abs_path>" }`，不依赖远程索引库。
- **G2（零拷贝物化）**：`auto build` / `auto run` 自动将本地依赖通过 Junction/Symlink 链接到 `./deps/<name>`。
- **G3（VM 端无缝解析）**：VM 运行时支持 `use <dep_name>.<module>: <Widget>` 跨包模块导入。
- **G4（Vue 端无缝转译）**：Vue 代码生成器自动转译 `deps` 中的 Widget 并融入 Vite / SFC 组件系统。
- **G5（实机端对端对齐）**：`010-contact-form` 成功通过 `pac.at` 依赖 `examples/ui/common`，并通过 Vue + VM 双端自动化测试。

---

## 详细架构设计

### 1. `pac.at` 本地依赖规范 (`crates/auto-man/src/target.rs`)
- `Target::from` / `Target::extract_origin`：
  - 检查 `node.has_prop("path")` 或 `node.get_str_or("path", "")`；
  - 若存在 `path`，将 `origin` 设为 `TargetOrigin::Local`，`from` 设为 `path` 的值；
  - 默认计算 `at` 为 `deps/<name>`。

### 2. 跨平台物化策略 (`crates/auto-man/src/pac.rs`)
- 未指定固定版本时（本地开发）：
  - **Windows**：调用 Windows Junction API（免管理员权限、免开发者模式）创建 `deps/<name>` 到 `../common` 的目录联接点。
  - **Unix / macOS**：调用 `std::os::unix::fs::symlink`。
  - **Fallback**：若链接创建失败，安全回退到 `copy_dir_all`。
- 若指定了 Git `version:` 且源目录为 Git 仓库：
  - 调用 `git worktree add -d ./deps/<name> <version>`。

### 3. VM 模块搜索路径扩展 (`crates/auto-lang/src/lib.rs`)
- `resolve_module_path(base_dir, module)`：
  - 解析 `use <dep_name>.<sub_module>: <Symbol>`；
  - 探测当前工程根下的 `deps/<dep_name>/src/front/<sub_module>.at`、`deps/<dep_name>/src/back/<sub_module>.at`、`deps/<dep_name>/src/<sub_module>.at`、`deps/<dep_name>/<sub_module>.at`；
  - 找到后作为模块 AST 解析并装载。

### 4. Vue 生成器跨工程扫描 (`crates/auto-man/src/vue.rs`)
- `VueProject::from_workspace(root_dir)`：
  - 探测 `root_dir.join("deps")`；
  - 遍历所有 `deps/*`，若存在 `src/front` 或 `front`，加入扫描队列；
  - 将依赖中的 Widget 同样转译成 Vue SFC，放置到 `gen/front/vue/src/components/`；
  - 合并 `deps/*/pac.at` 中的 `npm_deps` 与 `styles` 到生成的工程中。

---

## 执行步骤

| 序号 | 步骤 | 操作目标与文件路径 | 验收标准 | 状态 |
|---|---|---|---|---|
| T1 | 本地依赖 AST 提取 | 修改 `crates/auto-man/src/target.rs`，支持 `path:` 属性识别为 `TargetOrigin::Local` | `cargo test -p auto-man --lib target` 绿 | [✅] |
| T2 | 跨平台软链接物化 | 在 `crates/auto-man/src/pac.rs` 中实现 `link_local_dep`（Junction/Symlink/Worktree） | `cargo test -p auto-man --lib pac` 绿 | [✅] |
| T3 | VM 模块寻址扩充 | 修改 `crates/auto-lang/src/lib.rs` 中的 `resolve_module_path`，支持 `deps/<dep_name>/` 寻址 | `cargo test -p auto-lang --lib plan339_tests` 绿 | [✅] |
| T4 | Vue 代码生成器支持 | 修改 `crates/auto-man/src/vue.rs`，支持扫描并转译 `deps/*/src/front/` 下的 Widget | `cargo test -p auto-man --lib vue` 绿 | [✅] |
| T5 | 实机示例重构与验证 | 在 `examples/ui/010-contact-form` 中配置 `dep "common" { path: "../common" }`，重构 `app.at` 引入 `ExampleHeader`，运行 Playwright + MCP 验证 | 双端 100% 视觉对齐通过 | [✅] |

---

## 复审记录

### 1. Checklist Audit (逐项核对)
- [x] T1: `crates/auto-man/src/target.rs` 正确解析 `path: "../common"`，设置 `origin = TargetOrigin::Local`，`node.main_arg()` 提取正常，通过 `test_extract_local_path_dep`。
- [x] T2: `crates/auto-man/src/pac.rs` 实现了 `materialize_local_dep`（Windows Junction `mklink /J` + Unix Symlink + Git Worktree 支持），修复了 Windows 路径分隔符问题，通过 `test_pac_dep_local`。
- [x] T3: `crates/auto-lang/src/lib.rs` 的 `resolve_module_path` 支持向上扫描 `deps/<dep_name>/` 并支持 `front/`、`back/` 与根目录，通过 `test_plan475_resolve_module_path_deps`。
- [x] T4: `crates/auto-man/src/vue.rs` 的 `VueProject::from_workspace` 在预扫描与组件生成阶段全面收集 `deps/*/src/front/` 下的 Widget 并生成到 `components/`，合并 `npm_deps` 与 `style_files`，通过 `test_plan475_dep_widgets_scanned_and_compiled`。
- [x] T5: `examples/ui/010-contact-form` 成功将通用 Header 替换为 `use common.header: ExampleHeader`，经 `test_010_vm.py` 和 `test_010_vue.mjs` 测试，双端 14 张截图完整一致，主题切换、色板切换、表单输入与提交动作均 100% 正常。

### 2. Workaround & Debt Scan (遗漏与负债扫描)
- 无临时 hack 或 workaround。
- Windows Junction 路径使用正规反斜杠与引号处理。
- `deps/` 目录已被正确加入 `.gitignore`。

### 3. Health Check (健康检查)
- `cargo check -p auto-man` 干净无阻断。
- 单元测试与端到端自动化测试全部通过。


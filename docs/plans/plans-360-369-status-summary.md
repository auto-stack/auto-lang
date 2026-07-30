# Plans 360–369 状态汇总：未完成工作 & Language/Compiler Workaround 清单

> **审计日期**: 2026-07-30  
> **审计范围**: Plan 360–369（含其引用的更早计划/前置依赖）

---

## 1. 总体完成度概览

| Plan | 状态 | 完成度 | 摘要 |
|------|------|--------|------|
| 360 | ❌ 未开始 | 0% (0/8 AC) | 015-notes UI 现代化 + 主题色切换 |
| 361 | ❌ 未开始 | 0% (0/19) | 生成器加固：不变量检查 + 代码路径收敛 |
| 362 | ❌ 未开始 | 0% (0/18) | 快速反馈链路：auto watch + 增量生成 |
| 363 | ❌ 未开始 | 0% (0/30) | AutoUI Code Generation Skill |
| 364 | ❌ 未开始 | 0% (0/7) | a2r COSMIC 桌面复刻就绪度 |
| 365 | ❌ 未开始 | 0% (4 pending + 1 deferred) | AutoUI 可插拔宿主架构 |
| 366 | ❌ 设计阶段 | 0% (仅 366a 5 项未完成) | 跨平台 UI 测试 DSL（暂不实现） |
| 367 | ✅ 已完成 | 100% | Codegen 质量改进（已归档） |
| 368 | ✅ 已完成* | ~85% (4 项遗留) | Consumer Parity Suite（已归档，有遗留） |
| 369 | 🔧 部分完成 | ~79% (P0–P5 done, P6 6 tasks 未做) | Python Parity Suite |

> \* Plan 368 虽已归档，但有 4 项明确的遗留工作未完成。

---

## 2. 已完成计划详情

### 2.1 Plan 367: Codegen Quality Improvements ✅

**状态**: COMPLETE (2026-07-30)，已归档至 `docs/plans/archive/`

- 所有 P0/P1/P2 任务完成
- sidebar.at 从 337 行重构到 169 行
- Rust UI 代码生成器 015-notes 编译 0 errors
- **无遗留任务**

---

## 3. 部分完成计划详情

### 3.1 Plan 369: Python Parity Suite 🔧

**叙事状态**: P0–P5 完成（69/69 tests 100%）  
**实际遗留**: P6（Tasks 23–28）未执行，88 个 checkbox 全部未勾选

#### P6 待办任务（6 个库，预计 25–35 测试用例）

| Task | 库 | 内容 | 目标用例数 |
|------|-----|------|-----------|
| 23 | py_os | listdir, makedirs, rename, getcwd, os.path | 5–8 |
| 24 | py_re | sub, findall, search, match | 5–8 |
| 25 | py_json | dumps, loads | 3–5 |
| 26 | py_configparser | ConfigParser, read_string, get | 3–5 |
| 27 | py_hashlib | sha256, md5, hexdigest | 3–5 |
| 28 | py_sys | platform, version, executable | 3–5 |

#### Stale 文档（待更新）

- py_struct, py_uuid, py_datetime 的 README 描述已修复的 "限制"
- Task 13 中明确指出的 "stale workaround" 残留

---

### 3.2 Plan 368: Consumer Parity Suite ✅*

**状态**: 已标注 COMPLETE (2026-07-30)，已归档至 `docs/plans/archive/`  
**但仍有 4 项遗留工作**：

#### 遗留 1: R-JSON（VM JSON opaque-handle 运行时）⏸ DEFERRED

- **现状**: VM 的 json 运行时是占位实现：`json.parse(s)` 返回输入字符串本身，每次 `json.get`/`as_int` 重新解析文本
- **正确修复**: 参考 Plan 212 的 opaque-handle 模式，实现 12 个真实 shim 函数（~300 行改动）
- **影响**: c_json_app 测试故意避免 `as_int`/`as_bool`/`as_number`，仅测 String 返回方法
- **文件**: `native.rs`, `native_catalog.rs`, `stdlib.rs`

#### 遗留 2: FU-3（同步 Plan 编号 367→368）⏸ 未执行

- 15 处 "Plan 367" 引用需要更新为 "Plan 368"
- 涉及文件: `parity/crates/auto-parity/src/main.rs`, 4 个 README, `parity/.gitignore`, `a2r-std/src/{fs.rs,env.rs,string_builder.rs}`, `auto-lang/src/a2r_std.rs`

#### 遗留 3: FU-4（mock-server runner hook）⏸ 部分未完成

- V3 说已实现，但 FU-4 列出了具体缺失：
  - `stdlib/auto/http.at` 缺少 `post_sync`/`post_bearer`/`last_status` 声明
  - `runner.rs`/`main.rs` 缺少 mock-server setup/teardown hook 代码

#### 遗留 4: W6（c_process_app workaround 去除）⏸ 源码残留

- 根因修复（bare-name fallback）已合并，但 `.at` 源码中的 workaround 未移除
- c_process_app 仍用 `StringBuilder` 模式代替自然 `str.split`

#### 未开始: R-COV（覆盖率缺口填补）

- `read_bytes`/`write_bytes`/`size`/`delete`/`copy`（fs）
- `env.remove`（env）
- 链式文本调用（text）
- 完整 json 覆盖

---

## 4. Language/Compiler Workaround 清单

以下 workaround 均源于 **Auto 语言或编译器/VM 缺陷**，在 `.at` 源码或生成器中采用规避手段，而非根源修复。

### 🔴 严重：导致 .at 源码写出非自然代码

| ID | 触发条件 | Workaround | 影响范围 | 对应 Plan | 根源 Bug | 修复状态 |
|----|---------|------------|---------|-----------|---------|---------|
| W-SPLIT | VM 中 `str.split` 跨模块调用返回损坏 | 用逐字符状态机代替 `var parts = s.split(" ")` | c_process_app | 368 | codegen CALL 返回类型缺少 bare-name fallback | ✅ 根因已修，workaround 源码未移除 |
| W-INT-RET | 跨模块调用返回整数 | 用 `StringBuilder` 模式代替自然表达式 | c_process_app | 368 | 同上 | ✅ 根因已修，workaround 源码未移除 |
| W-NO-LOOP-RET | `for` 循环内 `return` 值被丢弃 | 避免在 `for` 内使用 `return` | c_process_app | 368 | codegen W6 | ✅ W8 修复后链式解决 |
| W-NO-EMPTY-STR | 跨模块函数不能返回空字符串 `""` | 不用 `return ""` | c_process_app | 368 | 同上 | ✅ 根因已修 |
| W-FLOAT | PyFFI float 返回值无法正确 marshalling（push_f64 2-slot vs 字符串池） | `.to(int)` 截断 | py_random tests | 369 | py_ffi.rs float stringification | ❌ 未修复（DIV-PY-FLOAT-1） |

### 🟡 中等：限制 DSL 表达能力

| ID | 触发条件 | Workaround | 影响范围 | 对应 Plan | 根源 | 修复状态 |
|----|---------|------------|---------|-----------|------|---------|
| W-DROPDOWN | Auto DSL 不支持 dropdown/popover 原语 | 用 5 个内联圆形色块代替下拉菜单 | 015-notes 主题选择器 | 360 | DSL 缺少组件 | ❌ 未修复 |
| W-CSS-RUNTIME | Auto DSL 无法表达运行时 JS 逻辑 | 主题色 CSS 变量注入通过生成器硬编码 TypeScript（`ACCENT_PALETTES` + `applyAccent()`） | vue.rs 生成器 | 360 | 无运行时 JS 注入机制 | ❌ 未修复 |
| W-NO-RAW-CSS | Auto DSL 只能写 Tailwind class，不能写原始 CSS | 所有样式用 Tailwind utilities 表达 | 所有 UI 组件 | 360 | DSL 设计约束 | ❌ 未修复（设计决策） |
| W-LIFETIME | Auto 无命名生命周期语法 | COSMIC 中 `struct MouseArea<'a>` 改为 owned 设计复刻（覆盖率 ~85–90%） | a2r COSMIC 复刻 | 364 | 语言缺少 lifetime 语法 | ❌ 已排除（Route A 设计决策） |
| W-MOVE | `move` 闭包关键字硬编码仅 `thread::spawn` 触发 | 其他闭包无法使用 move 捕获语义 | a2r COSMIC 复刻 | 364 | rust.rs 硬编码特判 | ❌ 未修复（W5 待实施） |
| W-ASYNC-DROP | `~{}` 异步块静默丢弃 `If`/`For`/`Try`/`Is`/`Block`/`Break`/`Continue` 等语句类型 | 生成静默不完整代码 | a2r 异步块 | 364 | rust.rs `_ => {}` fallback | ❌ 未修复（W4 待实施） |

### 🟢 低：已知 Gaps / 设计限制

| ID | 描述 | 影响 | 状态 |
|----|------|------|------|
| GAP-DROPDOWN | Auto DSL 无 Dropdown/Popover 组件 | UI 交互受限 | Plan 360 文档化 |
| GAP-KWARGS | PyFFI 不支持 kwargs | Python 库覆盖率受限 | Plan 369 DIV-PY-KWARGS-1 |
| GAP-AUTOLIST | Auto list 不 marshal 为 Python list | Python 库覆盖率受限 | Plan 369 DIV-PY-AUTOLIST-1 |
| GAP-ITER | PyFFI 不支持 Python iterator for-in 消费 | 文件操作等受限 | Plan 369 DIV-PY-ITER-1 |
| GAP-BYTES | PyFFI bytes 双向转换有问题 | 网络/安全库受限 | Plan 369 DIV-PY-BYTES-1 |
| GAP-CALLBACK | PyFFI 不支持 callback 传递 | email/test 框架受限 | Plan 369 DIV-PY-CALLBACK-1 |
| GAP-WAYLAND | Windows 无法直接运行 Wayland 测试 | COSMIC 验证需 WSL2+WSLg | Plan 365 环境级 workaround |
| GAP-RENDERQ | RenderQueue/共享内存 IPC 推迟 | COSMIC host ③ 暂不实施 | Plan 365 D4 |

---

## 5. 计划间依赖关系

```
Plan 360 (UI 重构)
  └── auto-forge 主题系统（外部参考）
  └── 现有 shadcn token 系统

Plan 361 (生成器加固) ─────────────────────────┐
  └── 015-notes（测试对象）                      │
                                                 │ Plan 361 的 API 是 362 的前置
Plan 362 (快速反馈) ◄───────────────────────────┘
  ├── 硬依赖: Plan 361 generate_component_from_file()
  ├── 硬依赖: Plan 361 validation rules
  └── 硬依赖: Plan 361 smoke tests

Plan 363 (Skill)
  ├── Plan 361（互补：静态验证 vs 预生成指导）
  ├── Plan 362（auto watch 快速验证）
  └── Plan 366（测试 artifact）

Plan 364 (a2r COSMIC 就绪) ◄── 365 的硬前置
  └── Plan 242（被取代的 lifetime 项）

Plan 365 (可插拔宿主)
  ├── Plan 364 W1–W3（属性宏、fn attrs、泛型 bound）必须先落地
  ├── Design Doc 20（架构基础）
  └── Plan 174（headless VTree runner）

Plan 366 (测试 DSL)
  ├── Plan 361（验证规则 + 契约）
  ├── Plan 362（auto watch --test）
  └── Plan 363（Skill 生成测试骨架）

Plan 368 (Consumer Parity)
  ├── Plan 212（opaque-handle 模式——R-JSON 的正确修复路径）
  ├── Phase 359 E4（use auto.http 解析——F6/F7 硬前置）
  └── Plan 367（曾被重名，代码注释残留 15 处）

Plan 369 (Python Parity)
  └── docs/design/python-parity-roadmap.md（架构设计源）
```

---

## 6. 建议执行顺序

基于依赖关系，推荐顺序：

```
Phase A（基础设施）
  361（生成器加固） → 362（快速反馈）

Phase B（UI 体验）
  360（UI 重构）可与 A 并行，但 smoke test 需等 361

Phase C（工具链）
  363（Skill）依赖 A 完成

Phase D（COSMIC 桌面）
  364（a2r 就绪） → 365（宿主架构）

Phase E（测试体系）
  366 → 在 361+362 成熟后再推进 DSL 层

Phase F（完成遗留）
  369 P6 → 368 R-JSON + FU-3/4 + 去 workaround
```

---

## 7. 归档建议

| Plan | 操作 | 理由 |
|------|------|------|
| 367 | ✅ 已归档 | 100% 完成 |
| 368 | ⚠️ 已归档但建议补文档 | 有 4 项遗留（R-JSON, FU-3, FU-4, W6），建议新建 mini-plan 追踪 |
| 369 | ❌ 不归档 | P6 未完成，88 checkbox 未勾选 |
| 360–366 | ❌ 不归档 | 全部 0% 未开始 |

---

*本文档由 plan-360-369 审计自动生成，覆盖 2026-07-30 当前状态。*

# Plans 未归档状态审计（2026-08-01）

> **审计范围**: `docs/plans/` 下 59 个非归档计划文件（不含 `plans-360-369-status-summary.md`，360–369 详情见该文件）
> **审计方法**: 分 6 批读取每个文件的开头状态摘要、checkbox 勾选情况、"遗留/TODO/PENDING/DEFERRED" 标记、末尾"下一步"章节
> **审计日期**: 2026-08-01

---

## 0. 总体状态分布

| 状态 | 数量 | 说明 |
|------|------|------|
| ➡️ 重定向占位 | 2 | `013*` 已迁至 auto-ai 仓库 |
| ✅ 已完成（可归档） | 5 | 无遗留或遗留已明确移交 |
| 🟡 部分完成 | 22 | 核心已落地，有明确遗留 |
| 🔴 仅设计 / 未开始 | 30 | 骨架/方案已写，零或几乎零实施 |

---

## 1. ✅ 已完成（建议归档，5 个）

| Plan | 主题 | 备注 |
|---|---|---|
| **341** | VM 调试方法论文档 | 纯分析，4 条建议非交付物 |
| **345** | 025 差距金丝雀测试 | N1/K2·N4/N2/N3/OOM 全闭环；K1 延后、K3 弃用 |
| **375** | SCU001 config/build | 与基线字节级一致；device 子 group 顺序差异明确"不修" |
| **378** | `.to_uint()` u64 槽位修复 | 9 叠加缺陷全修；§10.5 `str.lower()` heap string 回归已于 2026-08-01 修复（根因：engine.rs inline str 分发表漏 `upper/lower` 别名），遗留全清 |
| **379** | 放宽 `route` 关键字 | ✅ 已归档（2026-08-01）。§5 遗留已由 Plan 383 彻底解决（函数引用作方法实参），不再绕开 |
| **383** | 命名函数引用作为值 | ✅ 已完成（2026-08-01）。VM 复用 CLOSURE 让 `let f = handler`/`f()` 工作；a2r 修 `is_copy_type` 认 `Type::Fn` 使 `.route("/", handler)` 输出干净 handler。零新增回归 |

> ⚠️ **归档前复核**: ~~`378` §10.5 发现 `str.lower()` 在 heap string 上返回垃圾值~~
> ——**已于 2026-08-01 修复**（实际根因：`engine.rs` CALL_SPEC inline str 分发表漏了
> `upper/lower/to_upper/to_lowercase` 别名，receiver 为 `Expr::Index` 时误入 `_ => push null`）。
> `test_26_str_method_on_heap_001_lower_on_split` 已转绿，零新增回归。378 现可干净归档。

---

## 2. 🟡 部分完成（22 个）

### 2.1 🔥 高优先级阻塞项（红色失败 / 卡下游）

| Plan | 主题 | 关键未完成 |
|---|---|---|
| **317** | VM 异步调度统一 | Phase 4（HTTP 异步 server 接入）待评审；Phase 2 `~{}.await` 取值缺陷推迟；Phase 1/3 各有 3-4 项已知遗留 |
| **322 → 318** | List\<Struct\> 渲染 | **核心 generic constructor bug 未修**（`List<Item>.new` 误创建类型）→ 015-notes VM 列表渲染阻塞点；`test_list_struct_push_then_len` 仍 ❌ |
| **333** | VM UI CompileSession 接入 | 核心达成，但子组件 `EditorPanel` `.Delete/.Edit/.Save` 报 `Undefined variable: self`（遗留） |
| **335** | List 运行时根因②修复 | `read_state_as_vec` 不解引用 `VmRef`；多个 shim（pop/get/set/insert/len/contains）的 `heap_objects/arrays` 双查**待复核**；`shim_list_len` 可能返回错误长度 |
| **348** | parity workaround bug 修复 | 17 bug 已修，但 **Task 20（模块级 const/var str 全局损坏，高优先级）+ Task 21（位运算链 sha2 mk32，高优先级）** 未补全 |
| **358** | 生成器缺陷 D1-D10 | 仅 4/10 修复；**D9（autodown_editor 卡住生成器）阻塞 354 阶段 C** |
| **372** | a2r 3 系统性缺陷 | A/B/C 已修，但**单文件 transpile 路径不过 Phase 1.5 预注册**，跨模块 spec 仍解析为裸 `Type::User`（建议补出路 2） |

### 2.2 一般部分完成

| Plan | 主题 | 关键未完成 |
|---|---|---|
| **242** | a2r 功能差距 tracker | 17 项仅 #14 完成（Cookbook 163 .at）；16 项未做（planned/partial/workaround 三类） |
| **243** | LSP/VSCode 现代化 | Phase 1 完成，**Phase 2-6 全待做**（QueryEngine、workspace、Rename/CodeActions、TS 迁移、CI） |
| **288** | notes 全栈 API | Phase 1 完成；Phase 2（动态 API 发现）/ Phase 3（rustvm 后端模式）待做 |
| **311** | rust 模式 DevTools MVP | Phase 1 合并；**Phase 2（P2-A async-init 应用 F12 + P2-B Canvas 检视）待做** |
| **323** | 016-calendar 完整 app | VM 跑通已合并；Phase 3-6（拆子 widget、后端节假日、事件侧栏、打磨）全未做 |
| **327** | 015-notes VM 渲染 | 阻断点 #1 + Phase 1.1 已修；**Phase 1.2（限定调用解析）+ Phase 2（模块级 var，需用户拍板方案 α/β）未做** |
| **340** | List 方法对 `ListData<Value>` 支持 | push/len 已修；filter/map/get/find/contains + 辅助函数待实现 |
| **347** | 8 库 Rust 复刻 | 7 库 257 测试已验证；**reqwest（Task 29-31）疑似未完成**；最终报告全 TBD；checkbox 全未回填 |
| **349** | HTTP Roadmap | VM 侧 done；**a2r 适配 8 步全未做**（TLS/multipart/download/WebSocket 异步等） |
| **350** | WebSocket | client + server echo done；`#[ws(path)]` 声明式路由 + a2r 客户端生成未做 |
| **354** | 015-notes 真实化 | 后端 schema/store/标签/搜索 done；**布局重构（v3 两栏树）+ AutoDown 编辑器 + block 体系未做**（依赖 358 D9） |
| **357** | 015-notes v4 UX | v1-v3 done；**v4 三项待做**（Tag 编辑独立、导航栏 tag 动态化、Pin 改 hover 图标） |
| **359** | "Auto 作 Rust 脚本层"发布 | 庞大计划；Phase E 前置部分 fixed，**主体 Phase A/B/C/D + DoD V1-V7 全未勾** |
| **369** | Python parity suite | P0-P5 done（69/69）；**P6（Task 23-28，6 个新库 os/re/json/configparser/hashlib/sys）未开始** |
| **373** | a2r B1 papercuts | MVP 达成（0 errors + cargo run 通）；**re-transpile 可重现性未达成（移交给 376）** |
| **376** | a2r 类型流分析 | MVP 达成（re-transpile 0 错误）；**lib.rs 仍手写**；37 警告清理 |

---

## 3. 🔴 仅设计 / 未开始（30 个）

### 3.1 设计已成文、待实施（优先级由依赖关系决定）

| Plan | 主题 | 备注 |
|---|---|---|
| **300** | Python FFI 运行时成熟化 | 全 4 Batch 待做（全 Auto 签名、dict→Obj、无 items 导入、REPL） |
| **308** | Godot demo 逆向翻译 | 6 fixture + 11 测试函数待加；含大量 deferred gaps |
| **319** | 统一 VM/Rust 渲染 + `View::Grid` | 3 Phase 设计完成未实施 |
| **320** | 单 VM widget 树重构 | 4 Phase 未实施（322 是其排查产物） |
| **324** | npm Vue 组件库战略 | "待评估"，§7 决策清单 5 项待团队确认 |
| **325** | enum 方法 + 跨模块 bug | **自评阻断性，阻塞所有后端 Auto 代码** |
| **328** | a2r HTTP server 架构 | "设计完成待实施"，6 环节未做 |
| **329** | Tauri IPC SSE 支持 | "设计完成待实施"，4 改动未做 |
| **330** | agent 友好调试工具链 | 纯设计，4 Phase + 验收全未勾 |
| **331** | `@auto-ui/widgets` npm 库 | "设计已确认实施待执行"，8 Phase 待做 |
| **332** | `#[derive(ToAtom)]` 宏 | "草案待评审"，5 Phase 未实现 |
| **334** | vm+vm 合并跳过 HTTP | Phase 1 未实现（333 的直接收尾） |
| **336** | vue-gallery showcase | "设计待确认"，依赖 331 |
| **337** | gallery↔widgets 同步层 | "设计待确认"，依赖 331+336 |
| **338** | 015-notes M1 基准 | 已重定范围到 `025-notes-extended`；routing 明确 blocked（依赖 351） |
| **339** | VM Symbol 命名空间系统 | 6 Phase 设计未实现（335 部分依赖本计划） |
| **342** | Block 层 Phase A（包基础） | "设计待确认"，5 Phase 全未勾 |
| **343** | Block 层 Phase B（CLI） | "设计待确认"，依赖 342 |
| **346** | Web Framework 差距调研 | 调研文档，20 项差距待后续 Plan 实施 |
| **351** | SharedStore（Rung-4 共享状态） | **阻塞多页路由**；DoD 7 项全未勾 |
| **352** | 中间件/Session/SSR/OpenAPI | "设计文档"，四能力全未实施 |
| **355** | a2r async/await 转译 | "设计文档/TODO"，4 项未做（中优先级） |
| **364** | a2r COSMIC 就绪 | **W1-W7 全 pending**；阻塞 365 W3（整个 COSMIC 复刻） |
| **365** | AutoUI 可插拔 Host | W1-W4 pending，W5 deferred（依赖 364 W1-W3） |
| **366** | 跨平台 UI 测试 DSL | "设计阶段暂不实现"；366a 5 项验收未勾（1-2 天可落地） |
| **374** | a2r store/viewfn parity | 4 Task 全未实施（Rust 模式 015-notes 功能 parity） |
| **377** | 统一值表示消除 2-slot | "⏳ 待实施"，5 阶段架构重构（高风险）；**378 的全局 u64 大值截断遗留移交至此** |

### 3.2 ⚠️ Stale 风险（久未更新）

| Plan | 主题 | 风险 |
|---|---|---|
| **277** | Plan 审计修复（LSP/MCP bug） | **"状态：待实施"，日期 2026-06-12（约 7 周前）**；全 5 Phase 未做，含 UTF-16 偏移中文崩溃等 P0 |

---

## 4. 关键发现 & 建议

### 4.1 最该立即处理的红色/阻断项

- **`322/318` generic constructor bug**——阻塞 015-notes VM 列表渲染
- **`325` 跨模块基础缺陷**——自评"阻断性，阻塞所有后端 Auto 代码"
- **`348` Task 20/21**——高优先级，影响 base64/regex/sha2 workaround 移除

### 4.2 强依赖链（动一处解锁一片）

- `351` SharedStore → 解锁 `338` routing / `354` 多页
- `364` W1-W3 → 解锁 `365` W3 → 整个 COSMIC 桌面复刻
- `358` D1+D9 → 解锁 `354` 阶段 C（AutoDown 编辑器）
- `331` → `336` → `337`（Vue 组件库整条链）
- `342` → `343`（Block 层整条链）

### 4.3 可立即归档的 5 个

341 / 345 / 375 / 378 / 379 ——其中 **378、379 已于 2026-08-01 归档**（378 的 `str.lower()` 回归已修，379 的遗留已由 Plan 383 解决）。

### 4.4 文档债

`347` 全部 checkbox 从未回填（实施进度其实由 348 追踪），建议在 347 顶部加一句重定向到 348，或补回填。

---

*本文档由 2026-08-01 plans 状态审计生成，基于各计划文件自述状态。实际代码进度请以仓库为准。*

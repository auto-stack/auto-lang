# Plan 415: a2r 剩余大件拆粒度实施（242 tracker 收尾批）

> **状态**: 🟡 A/D 已收口——A ✅（2026-08-22 落地，golden 006_map_literal）/ D ✅（2026-08-24 Plan 433 收口）；B/C/E 真待办（2026-09-01 Plan 513 C 组刷新；**2026-09-15 漂移核查**：B 前提复核成立可直接续做，E 被 Plan 610 部分取代需重定方案，C 前置决策改锚虚拟桌面——详见各节括注与 §2）
> **来源**: Plan 242（a2r 功能差距 tracker,持续维护不归档）剩余未做项;审计判定"均为大件,独立立项"
> **前置核查**: #8 闭包推断根因已由 audit-A6 修复（`c2bd1d0c`,golden 004_closure_infer）,不在本计划范围

---

## 0. 拆解原则

242 剩余项彼此独立、体量差异大,按"可独立验收的最小粒度"拆为 5 个子项,
每项独立 worktree + 分支(`plan-fix/415-<id>`),全绿才合并。跨仓验证环
(a2r 改动必做:worktree 构建 → auto-ai 三 retranspile + cargo check 零错)
对 #2/#10 强制,#15/#16/#17 视触及面。

## 1. 子项清单

### 415-A `HashMap::from` 字面量发射（242 #2,预估 1-2 天）— ✅ 已落地（2026-08-22，Map 类型+对象字面量 → HashMap::from，golden 006_map_literal）

- **现状**: `let m Map<str,int> = {"a": 1}` 发射 struct/object 语法而非惯用
  `HashMap::from([...])`（242 §行 59-63 已给出期望形态）。
- **入口**: `crates/auto-lang/src/trans/rust.rs` map 字面量发射臂 +
  a2r golden 新用例。
- **验收**: golden 双向通过 + auto-ai 重生成零 diff。

### 415-B Redis/SQLite a2rs backend stdlib（242 #10,预估 3-5 天）

- **现状**: Plan 121 交接的 6 个 cookbook DB stub + Plan 240 Phase 10 交接
  4 stub,均为 VM 侧占位;a2r 路径无对应发射。
- **拆粒度**: B1 先做 SQLite(rusqlite 已在依赖树,auto-man 已用)——
  a2r 侧 stdlib 声明 + 发射映射;B2 再做 Redis(需引入 redis crate,
  涉及 build 脚本与平台验证,单独立分支)。
- **入口**: `stdlib/auto/` 新增 `sqlite.rs.at`/`redis.rs.at` +
  `crates/a2r-std/` 对应手抄副本(注意 KNOWN-DEBT 396 条目:手抄漂移风险,
  本项落地时应顺带建立签名比对)。
- **[2026-09-15 漂移核查]**: ✅ 前提全部复核成立——rusqlite 0.30.0 仍在
  Cargo.lock、redis 仍缺席、a2r-std/stdlib 入口结构稳定(15 commits/月,
  无破坏性重构);KNOWN-DEBT 396 仅修了 time.rs 单点(i32→i64,8164e93a9),
  通用签名比对环仍开放,骑乘项仍待做。可直接续做;开工前轻量预检
  Plan 121/240 交接 stub 清单是否仍为最新基线。

### 415-C GPUI a2r UI generator（242 #15,预估 ≥1 周,⭐⭐⭐⭐⭐）

- **现状**（2026-09-15 刷新）: GPUI renderer 骨架已迁移至
  `crates/auto-lang/src/ui/gpui/`（auto_render/renderer/vnode_entity
  + style/gpui_adapter,合计 ~3.8k 行）,以 `ui-gpui` feature 门控
  （gpui 0.2.2 + gpui-component 0.5.0）,19 个 `examples/ui_*.rs`
  支持 `--features ui-gpui` 运行。Plan 365 已知限制仍在
  （`View::Image` → `[img: src]` 文本占位、`View::Grid` 行列分解
  无原生 grid,KNOWN-DEBT 365 条目在案）。a2r(ui 生成器侧)仍完全未接。
- **前置决策**（2026-09-15 刷新）: ~~Plan 386 的启动条件式评估("≥3 个
  COSMIC app 跑通")是否放宽~~ 已失效——386 归档,RenderCommand 重定位为
  虚拟桌面 AppWindow 接缝的渲染叶子(路线 B,Design 23 / Plan 452,
  2026-08-26),启动条件改挂虚拟桌面计划(宿主=Plan 455 桌面进程,仪表盘
  `docs/plans/autos-desktop-program.md`)。go/no-go 应对新锚点重估;
  若重启,保留"1 天 spike(AURA → GPUI 映射层 PoC)先行"建议。

### 415-D 自举 Phase 2/E（242 #16,预估 ≥1 周）— ✅ 已收口(2026-08-24,Plan 433)

- **现状**: ~~Plan 355 完成 a2r 发射侧(#12),自举(用 Auto 写 Auto 工具链)
  Phase 2(编译器自身)与 Phase E 待做。~~
- **收口**: Phase 2 = AAVM v2 六层管线(432,Auto 写的编译器+VM,AutoVM 内
  自举);Phase E = 433(Rust 版 a2r 转译 AAVM → 纯 Rust 零 a2r_std,
  rustc metadata 零错,corpus 30/30 与参考一致,四向矩阵全绿)。
  纯 Auto 闭环(Auto 版 a2r 转译器,五向)→ Plan 434 余力项。
- **依赖**: ~~415-A/B 落地后再评估~~(A 已落地;B/C 不阻塞)。

### 415-E dep cc + memmap2 FFI（242 #17,预估 2-3 天）

- **现状**: Plan 240 Phase 13 交接 4 个 cookbook stub。
- **入口**: build-time codegen(`build.rs` + cc 编译 C 桥)+ memmap2
  FFI 声明;Windows/MSVC 工具链验证是主要风险点。
- **[2026-09-15 漂移核查]**: ⚠️ 原方案被 Plan 610(已归档,a2r C ABI
  两形态)部分取代——610 已建成 manifest IR(auto-bindgen link 字段)/
  `use.c` 静动态双形态/`#[export]` cdylib 导出/auto_cabi_kit 指针桥,
  且经 597 驱动器实编实证。不建议按原文手搭独立 build.rs+cc 路径,应先
  出一页基于 610 机制的重定方案,避免两套 FFI 路径并存。另:cc/memmap2
  已作为传递依赖进入 Cargo.lock,"需引入"前提已松动。

## 2. 执行顺序建议

~~A（最小、独立）→ E → B1 → B2；C/D 各自 spike 后重估~~
**2026-09-15 漂移核查后调整**: **B1 → B2**（前提复核成立,当前最新鲜、
最自包含的下一步）→ **E** 先出基于 Plan 610 机制的重定方案再排期 →
**C** 待虚拟桌面计划(Plan 455)给出新信号后重做 go/no-go。每项合并后
回填 242 tracker 对应行 + 本文档勾选。

## 3. 验证矩阵

| 子项 | 单测/golden | 跨仓环 | 实机 |
|---|---|---|---|
| A | golden 新用例 ×2 | 必须 | — |
| B1/B2 | a2r-std 契约测试 | 必须 | cookbook demo |
| C | 映射层单测 | 视触及 | GPUI 窗口冒烟 |
| D | 自举产物 diff | 必须 | `auto build` 自举 |
| E | FFI 冒烟 ×4 | 视触及 | Windows 构建 |

# Plan 442: 跨平台合龙——musk 五域端口接线 + VM 渲染能力补缺 + 后端 AutoVM 激活

> **状态**: ⬜ 未开工（2026-08-23 立项草稿；**整体 gated**——前置未竟期间不执行）
> **来源**: auto-musk PLAN-038 待澄清 #7（接线边界划出后无人承接）+ PLAN-041 裁定
> （web 轨退役等迁移完成）+ auto-musk KNOWN-DEBT-AND-RISKS 028 ③（VM 渲染目标
> "归 VM 渲染目标立项"）+ auto-musk pac.at 头注（"后端用 AutoVM 脚本运行"激活线）。
> **关联前置**: Plan 429–434（AAVM v2 移植/a2r 闭环）、Plan 436（setup 相位）、
> auto-musk PLAN-038（第三方库 Auto 版）、auto-down Plan 008（渲染库 Auto 化/
> markstream 消灭/编辑库）
> **跨仓说明**: 本计划主跟踪在 auto-lang，但缺口 A/C 的动作面在 auto-musk 仓
> （ports adapter 文件、pac.at 目标切换）——任务表显式标注仓库归属。

## 1. 目标与缺口

迁移线两侧基础计划（auto-lang AAVM 系列 + musk 前端单源化/第三方库）完成后的
**合龙段**：让 auto-musk 真正以 Auto/vm 或 Auto/rust 形态跑起来。三个缺口：

- **缺口 A（musk 侧动作）——五域端口 VM/Rust adapter 接线**：PLAN-037 落定的
  `ports/{platform,composables,icons,renderer,upload}` 目前只有 `.web.at` 适配器；
  各域需要 rust/vm 目标的同名 adapter 并切换绑定（musk 038 已划出、无计划承接）。
- **缺口 B（auto-lang 侧动作）——VM 渲染目标能力补缺**：KNOWN-DEBT 028 ③ 登记的
  store facade 概念缺失（VM 渲染对 `store` 合成物报 Undefined variable）+ TS ext
  显式 link 错误；以及两个 038 canary 将暴露的能力前置项——svg 节点（auto-icons
  渲染层）、调度/定时器原语（markstream 流式行为）。
- **缺口 C（musk 侧动作，gating 在 auto-lang）——后端 AutoVM 激活**：pac.at 头注
  "后端用 AutoVM 脚本运行（待 #[api] server 修复后激活）"；hw/ag 双轨中 ag 轨
  转正、`musk serve` 以 VM 跑后端的切换与验证。

## 2. 现状盘点（2026-08-23 立项时已核实）

- musk 五域端口调用面已纯 Auto（`use pac.ports.<域>: *`，调用面 `use.web` 非
  `.at` 目标零命中——PLAN-037/424 收口）；`resolve_at_adapter` 的目标门控机制
  （`X.at` 端口 → `X.<target>.at` adapter，缺失显式报错）已在 auto-man 落地并有
  platform 域拆分 proof（PLAN-037 Phase 6 T22-T23）。
- VM 渲染目标对 musk 源的现有报错清单见 auto-musk KNOWN-DEBT-AND-RISKS 028 行
  （store facade / ext link）；svg 与调度原语能力未实测（musk 038 T9/T16 canary
  会给出结论，届时回填本节）。
- `#[api]` server 修复状态**待核验**（pac.at 注释可能已过时——auto-lang 429-434
  前进很多）；ag 轨休眠镜像清单见 auto-musk KNOWN-DEBT 018（tools/spec_tools/
  orch_tools/server_serve，已评估收益为零、不阻塞）。
- auto-down Plan 008 Phase 3 的调度端口（VM adapter）与本计划缺口 B 的调度原语
  是同一能力的两个消费面。

## 3. 前置门（全部满足才开工）

| 前置 | 计划 | 状态 |
|---|---|---|
| AAVM v2 移植 + a2r 闭环 + AA2R | auto-lang 429–434 | draft |
| setup 相位解释器/a2r | auto-lang 436 | draft |
| 第三方库 Auto 版（i18n/icons/渲染切换/高亮决策） | auto-musk 038 | drafting |
| 渲染库 Auto 化 + markstream 消灭 + 编辑库定版 | auto-down 008 | 草稿 |

> musk PLAN-041（web 轨退役）**不在前置门内**——它与本计划互为对侧：本计划合龙
> 完成 = 041 的"迁移完成"条件达成，041 随即解挂启动。

## 4. 任务分解（gated；仓库归属标注）

### Phase A — VM 渲染能力补缺（auto-lang，先行）

- A1 核验 #[api] server 现状：AAVM 系列产物上重放 pac.at 后端激活路径，确认修复
  或登记剩余缺口（产出回填 §2）。
- A2 store facade：VM 渲染目标引入 store 合成物概念（或显式报错指引改写），消除
  Undefined variable 警告；musk 30 widget 源作回归语料。
- A3 ext link：TS ext 依赖在 VM 目标的显式 link 错误改为可配置跳过/挂平台桩。
- A4 svg 节点能力：按 musk 038 T9 canary 结论决定（语言层支持 / 挂账）。
- A5 调度/定时器原语：按 musk 038 T16 与 auto-down 008 Phase 3 的需求面定接口。

### Phase B — 五域端口接线（auto-musk 动作，auto-lang 机制配合）

- B1 platform 域：`ports/platform.rust.at`（inject_styles 空实现/去化、
  setup_auth_fetch→rust fetch 注入、relay_command_runner rust 版）+ 构建双目标验证。
- B2 composables 域：`ports/composables.rust.at`（useT→auto-i18n 直绑、gate_router
  rust 版）——依赖 musk 038 Phase 1 产物。
- B3 icons 域：`ports/icons.rust.at`（auto-icons 数据层直绑；渲染层依 A4 结论）。
- B4 renderer/upload 域：`ports/renderer.rust.at`（auto-down 008 产物）、
  `ports/upload.rust.at`（rust http 客户端版）。
- B5 musk `auto build` 双目标全绿（vue 产物对拍不变 + rust/vm 目标产物生成）。

### Phase C — 后端 AutoVM 激活（auto-musk 动作）

- C1 pac.at `api` 目标切换试验（rust→vm 路径），暴露的转译缺口登记回 auto-lang。
- C2 `musk serve` 以 VM 后端起服：HTTP/SSE 契约测试（复用既有 parity 测试面）
  对照 hw 后端全绿。
- C3 双后端并行观察期与切换/回滚开关（env 级），收口后 pac.at 头注的
  "待激活"改为已激活记录。

## 5. 验收标准

1. musk 前端 `auto build` 在 vue 与 rust/vm 双目标下全绿，五域端口各有非 web
   adapter（或显式降级登记）。
2. VM 渲染目标对 musk 30 widget 源零 Undefined variable 级报错（或每条有登记的
   能力缺口条目）。
3. `musk serve` VM 后端通过既有 HTTP/SSE 契约测试（与 hw 后端对照）。
4. 本计划完成 = musk PLAN-041 解挂条件达成（041 启动记录回填）。

## 6. 待澄清事项

1. **VM vs Rust 目标优先序**：B 阶段 adapter 先落 `rust.at`（a2r 路线成熟）还是
   直接 VM（AAVM 路线）——依 429-434 完成时的成熟度定，两个都做则排序待定。
2. **A4/A5 能力项归属**：若 auto-lang 决定不做 svg/调度原语，musk/auto-down 侧的
   降级路径为唯一路线——需要显式拍板而不是默认沉默。
3. **C 阶段 ag 轨休眠镜像**（KNOWN-DEBT 018：tools/spec_tools/orch_tools/
   server_serve）维持"不激活"结论还是借机激活——建议维持（收益为零结论仍成立）。
4. **观察期与回滚策略**：C3 的并行观察期长度与回滚开关形态。

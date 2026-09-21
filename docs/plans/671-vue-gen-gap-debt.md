---
plan_id: PLAN-671
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vue-gen-gap-debt
author: [zcode]
created_at: 2026-09-21
updated_at: 2026-09-21

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 ui overview vue 生成器自完备契约, SD-02 aura schema menubar 族 props 吸收]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN（review 时定）

affects: [docs/specs/auto-lang/ui/overview.md]
current_step: 0
total_steps: 11
---

# [PLAN-671] vue 生成器缺口批（auto-edit 七类补件上游清偿）

## 0. 变更摘要

auto-edit PLAN-003（已 delivered）vue 轨以 `specs/auto-edit/scripts/regen_vue.py`
七类生成后补件维持"生成 → 补件 → build 绿"链，围栏禁用 `auto run -r vue`
（其内置再生成会静默覆盖补件）。本计划把七类缺口在上游**按通用机制清偿**，
使 vue 生成器裸产出即自完备可构建，regen 补件逐段退化直至摘除。

用户裁定（2026-09-21）：**修复一律按机制（内建注册表/形参表/类型系统/值域
映射），不针对特殊函数名单打补丁**——auto-edit 的 21 个内建清单、四个自调
别名等只是通用机制的项目投影，不得照抄进上游代码。

**先手勘定警示**：PLAN-668（baseline-red-clearance，已 archived）R-21/R-22/
R-23 已撞同型——R-21 vite/client（env.d.ts）、R-22 auto-sources 阶段序、
R-23 038 store 名未解析 + 013 负标量随检（vue.rs:17979-17988 `store_init_to_js`
已有 Unary(Sub/Add) 臂，即原补件 4「int 负初值 ref<number>(null)」大概率已
清偿）。七类现状**必须在当前 master 三态重验**（已清偿/部分清偿/仍破），
勘定结论分流后续任务，本计划不预设全部仍破。

## 1. 目标

- **G1（勘定）**：七类缺口在当前 master（≥496a7df92，含 668/669/670）上
  逐一三态判定，产出决策档（每类：现状/生成物证据/上游锚点/修法归属）。
- **G2（通用修复）**：仍破/部分清偿类按机制修复——natives 声明按内建注册
  表全量发射、事件 handler 形参按 on 声明形参表装配、插值界符按表达式
  文法补齐、store 自调按 self_bare 机制改写、bp 函数内联覆盖 store/
  composable 文件、variant 按值域映射、menubar 族 props 进 aura schema。
- **G3（端到端）**：auto-edit 仓裸 `auto build --gen-only -r vue`（**不带
  --lenient**）产出的工程 vue-tsc + vite build 绿，regen 补件全摘。
- **G4（交接）**：auto-edit 侧复跑收口通知（669 §10-4 模式：merge 后通知
  消费方，auto-edit 零上游改动自解）。

**非目标（Non-goals）**：

- **不做 vue 轨运行期内建实现**——dialog_open/console_*/code_editor_* 读回
  族的运行期缺口（vm 宿主无 vue 运行时）维持 auto-edit README 登记限制，
  归深度双轨（jade-edit 换基承接）。本计划只管**生成面自完备**（构建绿 +
  工程完整 + 转译保真）。
- 不动 auto-edit 仓代码（auto-edit 是发现方与复跑验收方；补件摘除归其
  复跑小改）。
- 不动 a2r/rust 轨（PLAN-670 已覆盖 F-R1/F-W3）。
- 不做 `auto run -r vue` 入口恢复的 auto-edit README 改写（auto-edit 侧
  后续）。

## 2. 架构方案

七类缺口分布在四个发射层，修复各归其层：

| 层 | 文件 | 涉及类 |
|---|---|---|
| 声明层（ts_adapter） | `crates/auto-lang/src/ui_gen/ts_adapter.rs` | ①natives 声明（对象级 `__vmOnly` 白名单 :1025-1039 已有先例，函数级裸标识符族缺声明发射） |
| 发射器（vue.rs） | `crates/auto-lang/src/ui_gen/vue.rs`（29606 行） | ②store 自调（self_bare/`with_store_bare_heads` :17355-17364 消费侧未覆盖 store 文件）③bp 内联（组件文件内联、store 文件裸调）④int 负初值（:17976-17988 已有 668 R-23 臂，待验）⑤事件形参装配（坐标型 handler 误套列表型单参模板）附a 多段插值界符（626 修了 vm 侧扁平化，vue 侧残留）⑥button variant（:13634 值域白名单注释一带） |
| 契约层（validators+schema） | `crates/auto-lang/src/ui_gen/validators.rs`（S001 :1033）+ `schema/aura.at`（:4502 MenubarItem vue 映射已有、props 未声明） | 附b S001 menubar 族 props 漂移（`--lenient` 债） |
| 工程完整性（gen-only 路径） | `crates/auto-man/src/vue.rs`（PLAN-646 auto-sources 写入 :1540-1572 + P657-D2 空占位兜底 :1590-1603） | ⑦gen-only 不写 auto-sources 真值/占位、不发 env.d.ts |

**分流规则（显式，非静默缩面）**：T-00 勘定判「已清偿」的类，对应任务标
`[x] 已清偿（证据）` 后跳过执行，AC 不删；判「部分清偿」按残缺面执行；
判「仍破」全量执行。修订记录入 §9。

## 3. 技术栈

Rust（crates/auto-lang ui_gen + crates/auto-man vue 路径）、aura schema
（schema/aura.at）、fixture 验证链（auto build --gen-only -r vue + pnpm
vue-tsc/vite）。复跑消费方：auto-edit（D:/autostack/auto-edit，兄弟仓）。

## 4. 需求分析与背景调查

**授权记录**：2026-09-21 用户指定 /auto-plan:new 起草本计划，裁定修复
形态为「通用方案，不针对特殊函数做处理」；上游锚点初勘与七类定性分析
于当日本会话完成（auto-edit 侧逐类核对源码证据：app.at:244 双参
`.EditorCtx(x, y)`、editor_store.at:67 `var confirm_idx int = -1`、
editor_store.at:14/423 bp `toggle_id` store 文件消费等）。

**七类清单与上游锚点初勘**（消费方补件全录：auto-edit
`specs/auto-edit/scripts/regen_vue.py:45-158`）：

| # | 补件（auto-edit 侧） | 上游锚点初勘 | 定性 |
|---|---|---|---|
| ① | natives.d.ts 21 声明（dialog/console/code_editor_*/Env/Process/file_basename 裸标识符 TS2304） | ts_adapter.rs:1025-1039 仅对象级（fs/File/image）拦 `__vmOnly` 桩；函数级无声明发射 | 通用能力缺口 |
| ② | store 自调别名 `const store = {SyncCursor, RemoveAt, TabActivate, CloseRequest}` | vue.rs:17355-17364 self_bare 集合含 store 名+"store"，`with_store_bare_heads` 下游消费面待勘 | 通用；**668 R-23 同型** |
| ③ | toggle_id bp 内联体追加 store 文件尾 | vue.rs bp 函数内联机制只覆盖组件文件（tree_util 在 FileTree.vue 有内联、store 文件裸调）——内联点代码锚点待 T-00 钉死 | 通用覆盖面缺口 |
| ④ | `ref<number>(null)` → `(-1)` | vue.rs:17976-17988 `store_init_to_js` 已有 `Expr::Int`/`Unary(Sub/Add)` 臂（668 R-23 013 随检）——**大概率已清偿，待验** | 通用纯 bug |
| ⑤ | EditorCtx 双参签名 + `$event.clientX/clientY` 实参 | on 声明形参表（app.at:244 双参铁证）未驱动 vue 发射；contextmenu 坐标事件缺事件类型→实参映射 | 通用装配 bug |
| ⑥ | button `text` variant 补进 shadcn cva 联合 | vue.rs:13634 一带 variant 值域处理；vm 专属扁平样式值 vs shadcn 封闭 cva 联合 | 生态契约缺口 |
| ⑦ | auto-sources 空 stub + env.d.ts（vite/client） | auto-man/src/vue.rs:1540-1603 PLAN-646 真值写入 + P657-D2 空占位兜底已存在，gen-only 路径未接 | 通用；**668 R-21/R-22 同型**；**646 jade 跨项目复现** |
| 附a | StatusBar 多段插值 `{{`/`}}` 界符补齐 | vm 侧 Plan 626 已修扁平化同型，vue 发射器插值拼装路径残留 | 通用；**626 同型** |
| 附b | `--lenient` 固定（命令层，非补件） | validators.rs:1033 S001（未声明 prop Info）；schema/aura.at:4502 MenubarItem 映射在但 PLAN-630 menubar 族 props（title/icon/shortcut/checked/enabled）未进 props 声明 | 纯上游 schema 债 |

（auto-edit 侧另有 StatusBar 多段插值补件，脚本内编号 5b，即本表附a。）

**先例与交集**：626（vm 侧同型已修，vue 侧漏）；646（jade-edit 跨项目同款
补件——gen-only 工程完整性缺口第二消费方）；668 R-21/R-22/R-23（同型已修
但按 038/013 项目清仓形态，auto-edit 2026-09-21 复验工具链含 669 时补件
链仍需跑——是否机制级修透即 T-00 核心问题）；669 §10-4（消费方交接模式）。

## 5. 详细设计

各类通用修法（机制级，含设计裁定项标 §10）：

- **①natives 声明层**：ts_adapter 增设函数级内建处理——按 vm 内建注册表
  （非名单硬编码）对顶层裸调用内建发射 `natives.d.ts` 声明文件；运行期
  语义二选一（§10-1）：纯 `declare`（运行期 ReferenceError，与 auto-edit
  补件现形态一致）或 `__vmOnly` 抛错桩（fail-fast 报错信息，与对象级
  fs/File 先例一致）。
- **②store 自调**：self_bare/with_store_bare_heads 消费侧扩展——store/
  composable 文件体内 `store.X()` 改写为本地 handler 直调（或统一别名
  发射），按 store 方法表驱动，非名单。
- **③bp 内联覆盖面**：bp 函数（`use bps.*: fn` 导入）内联发射从组件文件
  扩展到 store/composable 文件；方向二选一（§10-3）：就地内联同体 vs
  发射共享 util 模块（多文件消费同一 bp 时去重）。
- **④int 负初值**：预期已由 668 R-23 清偿——T-00 以负标量 fixture 复验
  即闭环，不预设新改动。
- **⑤事件形参装配**：vue 发射器按 on 声明的形参表发射 handler 签名；
  事件实参按**事件类型→实参映射表**装配（contextmenu→`$event.clientX,
  $event.clientY`，列表型→循环变量，input→既有 Plan 062 T9 语义），
  替换现"一律单参 (i)"模板。
- **⑥variant 值域**：方向二选一（§10-2）：vm variant 值映射到最近似
  shadcn variant（text→ghost，视觉有损）或生成器对捆绑 ui/button 自动
  扩 cva 联合（vm 视觉零回退）。倾向后者（双轨零回退原则），上游裁定。
- **⑦gen-only 工程完整性**：gen-only 路径接入既有 P657-D2 空占位兜底
  （auto-sources）+ env.d.ts 发射，使 `--gen-only` 产出与 `auto run` 完整
  流同等可构建（真值/占位一致化，646 语义收口）。
- **附a 插值界符**：vue 发射器插值拼装路径按 626 vm 侧同款修法补齐界符
  （多段插值混合文本场景通用）。
- **附b S001**：schema/aura.at Menubar/Menu 族 props 声明吸收 PLAN-630
  组件族（title/icon/shortcut/checked/enabled/menubar-separator 语义），
  S001 Info 消失后消费方可摘 `--lenient`。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
| --- | --- | --- | --- | --- | --- |
| SD-01 | modify | docs/specs/auto-lang/ui/overview.md vue 生成器段 | before：vue 轨生成缺口/限制散记（natives 声明/gen-only 完整性/自调/形参装配等无契约）/ after：vue 生成工程自完备契约（裸产出可构建：声明层完备/gen-only 工程完整/转译保真三条规则 + 各机制归属） | 修复落地的规范面收口，消费方（auto-edit/jade-edit）围栏解除依据 | AC-04/06 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md（menubar/aura schema 段） | before：menubar 族 props 未进 aura schema（S001 漂移靠 --lenient 规避）/ after：PLAN-630 menubar 组件族 props 声明吸收契约 | schema 债清偿登记 | AC-04/06 |

## 6. 测试设计

- **fixture 化（通用性红线的产品）**：每修类配一个最小 .at fixture
  （tests/ 下新目录），含对应特性**但不复现 auto-edit 业务名**（如负标量
  初值/双参坐标 handler/bp store 消费/多段插值文本/non-shadcn variant）；
  验证 = `auto build --gen-only -r vue`（无 --lenient）+ pnpm build 绿。
- **现有回归**：cargo test（ui_gen 模块内嵌测试，vue.rs:19313 一带已有
  `ref<number>(0)` 断言先例）+ examples 现有 vue 生成面。
- **端到端**：auto-edit 仓复跑（T-10 交接，669 §10-4 模式）。
- **通用性静态门**：修复面 grep 不得出现 auto-edit 特定标识符
  （EditorCtx/toggle_id/useEditorStore/confirm_idx/SyncCursor 等）。

## 7. 验收标准

- **AC-01（勘定决策档）**：七类 + 附a/附b 各有三态结论与证据（生成物
  diff/build 日志/代码锚点行号），落 docs/plans/671 附件或 §4 增补。
- **AC-02（通用性红线）**：上游修复代码零消费方特定名单（grep 门通过，
  §6 清单）。
- **AC-03（fixture 证明）**：每个执行修复的类有对应 fixture 且生成工程
  build 绿；已清偿类以 fixture 复验代证明。
- **AC-04（auto-edit 端到端）**：auto-edit 仓裸 `auto build --gen-only
  -r vue`（无 --lenient、无补件）产出工程 vue-tsc + vite build 绿；
  S001 不再触发（附b 清偿实证）。
- **AC-05（回归）**：cargo test ui_gen 面绿；examples vue 生成面无回退。
- **AC-06（spec/交接收口）**：SD-01/SD-02 落账；auto-edit 复跑通知发出。

## 8. 执行步骤

| ID | 任务 | 依赖 | 落点（文件/符号） | 意图 | AC | 验证命令/预期 |
|---|---|---|---|---|---|---|
| T-00 | 七类三态勘定：当前 master 上 auto-edit 逐段摘补件跑 regen + 上游锚点现状比对，产出决策档 | — | 决策档落 docs/plans/671（附件或 §4 表增补）；worktree D:/autostack/.wt/lang-671/auto-lang | 分流依据（已清偿/部分/仍破） | AC-01 | auto-edit gen/ 逐段 diff + build 日志留档；④类负标量 fixture 复验 |
| T-01 | gen-only 工程完整性：auto-sources 占位接入 gen-only 路径 + env.d.ts 发射 | T-00 | crates/auto-man/src/vue.rs（:1540-1603 兜底机制接入 gen-only 分支） | ⑦清偿 | AC-03/04 | fixture + auto-edit 裸 gen 后 pnpm build exit 0 |
| T-02 | 多段插值界符补齐（626 同款修法 vue 侧） | T-00 | crates/auto-lang/src/ui_gen/vue.rs 插值拼装路径（T-00 钉死行号） | 附a 清偿 | AC-03 | fixture（含 `行:列` 型多段插值文本）build 绿 |
| T-03 | 事件 handler 形参装配：按 on 形参表发签名 + 事件类型→实参映射表 | T-00 | crates/auto-lang/src/ui_gen/vue.rs handler 发射路径 | ⑤清偿 | AC-02/03 | fixture（双参坐标 handler）build 绿 + grep 门 |
| T-04 | store 自调改写：self_bare 消费侧覆盖 store/composable 文件 | T-00 | crates/auto-lang/src/ui_gen/vue.rs :17355-17364 消费链 | ②清偿 | AC-02/03 | fixture（store 内自调）build 绿 + grep 门 |
| T-05 | bp 函数内联/共享发射覆盖 store 文件 | T-00 | crates/auto-lang/src/ui_gen/vue.rs bp 内联点（T-00 钉死） | ③清偿 | AC-02/03 | fixture（store 消费 bp fn）build 绿 |
| T-06 | natives 声明层：函数级内建按注册表发射 natives.d.ts（§10-1 裁定后） | T-00 + §10-1 | crates/auto-lang/src/ui_gen/ts_adapter.rs（对象级白名单 :1025-1039 旁） | ①清偿 | AC-02/03 | fixture（front 层用 vm 内建）build 绿；声明按注册表全量非名单 |
| T-07 | button variant 值域对齐（§10-2 裁定后） | T-00 + §10-2 | crates/auto-lang/src/ui_gen/vue.rs :13634 一带 | ⑥清偿 | AC-03 | fixture（non-shadcn variant）build 绿，vm 视觉零回退（若裁定扩 cva） |
| T-08 | aura schema 吸收 PLAN-630 menubar 族 props | T-00 | schema/aura.at（:4502 MenubarItem 族）+ validators S001 面 | 附b 清偿（摘 --lenient） | AC-04 | strict 模式（无 --lenient）auto-edit 生成零 S001 |
| T-09 | 测试/fixture 全量收口 + cargo 回归 | T-01..T-08 | tests/ fixture 目录 + crates 内嵌测试 | 证明集中 | AC-03/05 | cargo test 绿 + fixture 矩阵绿 |
| T-10 | 文档收口（SD-01/SD-02）+ auto-edit 交接通知 | T-09 | docs/specs/auto-lang/ui/overview.md + 交接记录 | 规范落账 | AC-06 | spec 回读 + 通知留档 |

## 9. 复审记录

- 2026-09-21 stage: new / PLAN-671 r1 起草交付评审。
  outcome: pass（评审通过即待「开工」授权进入 work）。
  next: review → work（授权后从 T-00 起；T-00 三态结论可能缩面 T-02..T-07
  中若干任务为「已清偿跳过」，属显式分流非缩面违约）。
  备注：上游锚点初勘来自 auto-edit 会话逐类源码核对；670 已覆盖 a2r/vm
  事件面，无重叠。

## 10. 待澄清事项

1. **natives 运行期语义**（阻塞 T-06 设计）：纯 `declare`（运行期自然
   ReferenceError，auto-edit 补件现形态）vs `__vmOnly` 抛错桩（fail-fast
   显式报错，对象级先例）。倾向抛错桩（错误信息可携带内建名与"vue 轨
   运行期缺口"指引），上游裁定。
2. **variant 值域对齐方向**（阻塞 T-07）：映射最近似 shadcn variant
   （text→ghost，视觉有损）vs 自动扩 cva 联合（零回退）。倾向扩 cva。
3. **bp 内联 vs 共享 util 模块**（阻塞 T-05 细化）：单文件内联同体简单；
   多文件消费同一 bp 时共享模块去重更净。T-00 勘定时按消费频度定。
4. **auto-edit 补件摘除归属**：默认 merge 后 auto-edit 复跑时随收口小改
   摘除（669 §10-4 模式）；若复跑发现补件锚点先于通知已退化（⚠ 无害
   警告形态），以复跑实测为准。

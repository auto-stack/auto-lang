---
plan_id: PLAN-671
status: reviewed          # drafting → executing → execution_done → reviewed → archived
feature_name: vue-gen-gap-debt
author: [zcode]
created_at: 2026-09-21
updated_at: 2026-09-21
plan_revision: 2
worktree: D:/autostack/.wt/lang-671/auto-lang
base_commit: ebfeef30c

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 ui overview vue 生成器自完备契约, SD-02 aura schema menubar 族 props 吸收]
touched_goals: [GOAL-007]      # AutoUI 跨端视觉一致（vue 轨生成自完备/双端 parity 锁定面）

affects: [docs/specs/auto-lang/ui/overview.md]
current_step: 14
total_steps: 14
---

# [PLAN-671] vue 生成器缺口批（auto-edit 七类补件上游清偿）

## 0. 变更摘要

auto-edit PLAN-003（已 delivered）vue 轨以 `specs/auto-edit/scripts/regen_vue.py`
七类生成后补件维持"生成 → 补件 → build 绿"链，围栏禁用 `auto run -r vue`
（其内置再生成会静默覆盖补件）。本计划把七类缺口在上游**按通用机制清偿**，
使 vue 生成器裸产出即自完备可构建，regen 补件逐段退化直至摘除。

> **r2（2026-09-21）**：附加 **Phase 2——1631 漂移增补批**（供料 §8
> 承接；F-1 勘误收窄后实际范围 = **natives 吸收覆盖对象形态内建
> （Process/Env）** + 消费方冷重生成复收口，见 §4.2 + T-11..T-13 +
> AC-07/08）；r1（T-00..T-10 / AC-01..06）任务与 AC 零改动，其
> review/merge 先行。

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

### 4.1 T-00 三态勘定决策档（2026-09-21，基线 = master ebfeef30c worktree 裸生成实测）

勘定方法：worktree `D:/autostack/.wt/lang-671/auto-lang`（HEAD ebfeef30c）
构建 `auto` 二进制，auto-edit 仓裸 `auto build --gen-only [--lenient] -r vue`
（无补件）逐类检查生成物 + 上游锚点代码比对。生成日志留档
`/tmp/p671-raw-gen.log`（无 --lenient，EXIT=1）与 `/tmp/p671-lenient-gen.log`。

| # | 类 | 三态 | 生成物证据 | 上游锚点（当前 master） | 分流 |
|---|---|---|---|---|---|
| ① | natives 声明 | **仍破** | `src/lib/` 仅 api.ts/utils.ts，无 natives.d.ts；19 个平名内建裸用于 2 文件（DataTableCrudDialog.vue、useEditorStore.ts）+ `Process` 1 文件（Env 0 命中） | 平名注册表 = vm `intrinsics` 表 `vm/codegen.rs:494+`（名→NATIVE_ID，无签名）；对象级 `__vmOnly` 白名单 `ts_adapter.rs:1025-1039` 仅 fs/File/image（Env/Process 不在） | T-06 执行 |
| ② | store 自调 | **已清偿** | 生成 useEditorStore.ts 零 `store.X()` 自调（唯一 `store\.` 命中为 `editor_store.at` 字符串子串） | 668 R-23 self_bare（store 名+`"store"` 字面）`vue.rs:17355-17364` → `ts_adapter.rs:1098` 裸调改写 | T-04 标已清偿，T-09 fixture 复验代证明 |
| ③ | bp 内联 store 文件 | **仍破** | useEditorStore.ts:308 调 `toggle_id(...)`，全文无定义无 import（TS2304） | 组件路径 `emit_use_module_fns` `vue.rs:4609`（:4087 消费，池拉取+闭包传递+冲突过滤）；store 路径 `generate_store_composable_full`（:17200 一带）无同款调用 | T-05 执行 |
| ④ | int 负初值 | **已清偿** | 生成 `const confirm_idx = ref<number>(-1)`（editor_store.at:67 `= -1` 正确落 -1） | `store_init_to_js` `Expr::Int` + `Unary(Sub/Add)` 臂 `vue.rs:17984-17990`（668 R-23） | 标已清偿，T-09 fixture 复验代证明 |
| ⑤ | 事件形参装配 | **仍破** | App.vue:99 `function EditorCtx(i: any): void`（源声明 `.EditorCtx(x, y)` 双参）；:298 `@contextmenu="EditorCtx(i)"`（循环参抢占，body 引用未绑定 x/y） | 双因：`code_editor_event_payload_call` `vue.rs:16778-16781` 循环早退（"loop-var stays authoritative"）拦截了 Plan 421 载荷转发；签名优先级 `vue.rs:3904` loop_param_handlers 先于声明形参 | T-03 执行 |
| ⑥ | button text variant | **仍破** | 生成 `ui/button/index.ts` cva variant 联合 = default/primary/submit/destructive/outline/secondary/ghost/link，无 `text` | cva 联合字面量 `vue.rs:18382-18416`（PLAN-571 与 Rust 侧互锁注释）；Rust 侧 `button_variant_preset("text") == ""`（chromeless）`ui/style/variants.rs:19,43` | T-07 执行 |
| ⑦ | gen-only 工程完整 | **已清偿** | gen-only 产出 `src/auto-sources.ts`（44KB 真值非 stub）+ `src/vite-env.d.ts`；tsconfig `types: ["vite/client"]`（668 R-21 根修）双保险，include `src/**/*.ts` 覆盖 d.ts | `prepare_vue_sources` 共享段（build 与 gen-only 同走）`auto-man/src/vue.rs:5087-5088` 调 `write_auto_sources_ts` + `ensure_vue_type_stubs`（038 Phase B T8 / P657-D2，75cf73bde） | T-01 标已清偿，T-09 fixture 复验代证明 |
| 附a | 多段插值界符 | **仍破** | StatusBar.vue:32 `store.line }}:{{ store.col`（缺外层 `{{ ` 与 ` }}`；源 `text "${.store.line}:${.store.col}"`） | `expr_to_vue_text_raw` `Expr::Str` 臂 `vue.rs:11298-11304`：`convert_template_to_vue` 后无条件 strip `{{ `/` }}` 前后缀——单段假设，多段被剥外层 | T-02 执行 |
| 附b | S001 menubar props | **仍破** | 无 --lenient EXIT=1（S001 阻断生成）；40 条 = menubar-item title×13/icon×11/shortcut×8/enabled×1 + **`text` on `<text>` ×7**（超出计划枚举的同族面，AC-04 零 S001 口径须一并收） | `schema/aura.at:4496-4508` menubar_item props 仅 text/disabled/onclick/class；`element text` :1551-1563 仅 selectable | T-08 执行（含 text widget text prop） |

**§10 裁定落定（执行期，依计划记录倾向 + 勘定证据）**：

1. **§10-1 natives 运行期语义 → 抛错桩形态**（计划倾向）：类型面
   `natives.d.ts`（vue-tsc 构建绿）+ 运行期 fail-fast。函数级按
   `intrinsics` 注册表驱动发射声明（TS 签名泛化 `(...args: any[]): any`，
   不手写逐名签名表）；对象级 `Env`/`Process` 进 `ts_adapter` 对象级
   `__vmOnly` 白名单（复用 fs/File/image 先例，含内联 throwing fn 发射，
   调用点改写 `__vmOnly('Env.get', ...)`）。
2. **§10-2 variant 方向 → 扩 cva 联合**（计划倾向）：`text: ''` 空差量类
   = Rust 侧 `button_variant_preset("text") == ""` chromeless 语义的 vue
   镜像（vm 视觉零回退：两侧均零附加样式），互锁注释同步。
3. **§10-3 bp 内联 vs 共享模块 → 就地内联同体**：auto-edit 实测消费
   频度 = 每模块单消费（toggle_id 于 store 文件 1 处 + FileTree.vue 1 处
   各自内联），无跨文件共享需求；复用 `emit_use_module_fns` 同一拉取
   发射器，零新发射形态。多文件消费同一 bp 的去重需求未来另立计划。
4. **§10-4 维持默认**（669 §10-4 模式，auto-edit 复跑时随收口小改摘补件）。

**范围外新发现（登记不扩面）**：menubar 族 vue 渲染保真——menubar_menu/
menubar_item 发射为裸 `<div :title=...>`（仅外层 Menubar 组件，:15046 臂
仅处理 text/disabled/onclick），构建绿不受影响、消费方从未补件此面，属
渲染保真债（vue 轨 menubar 组件化映射），非本计划七类范围，后续计划候选。

### 4.2 证据增补与 Phase 2 立项（2026-09-21 r2，供料 §8 承接；同日 F-1 勘误收窄）

- **来源**：auto-edit 上游供料包（M1 批 2026-09-21）§8 附记（原件
  `docs/plans/attachments/673-674-m1-supply.md`，勘误版）+ 其 PLAN-004
  T-04 复验收据与复审 F-1 勘误（commit b0b4676，vue exit 0 修复后
  诉求收窄）。
- **勘误后双症状定性**（工具链 `v0.4.2-1631-ge82b95b22` 实测；1588+669
  时代绿 → 1631 红）：
  - ① **TS1117 button——上游无诉求（撤回）**：1631 生成器**已自带**
    text variant（空预设语义 + "Keep in sync with ui_gen/vue.rs"
    注释——本计划 ⑥ 修面形态）；重复属性是**下游补件⑥非幂等双插**，
    消费方已加在场守卫修复（b0b4676）。本计划仅登记归属，无动作项。
  - ② **TS2304 `Cannot find name 'Process'` ×3——真实上游缺口
    （保留）**：1631 生成器已自发射 `src/natives.d.ts`（函数形态内建，
    `(...args: any[])` 宽松签名——671 中间态），但**漏对象形态内建
    `Process`/`Env`**；且 tsc 程序只收录 src 根该文件（下游
    `src/lib/natives.d.ts` 不入程序，listFilesOnly 实证——与 r1 T-06
    「d.ts 落 src 根」实测一致）。**诉求 = r1 T-06 natives 吸收须覆盖
    对象形态内建**（声明发射 + 抛错桩/调用点改写）；下游过渡补件
    （regen 1b 追加式）已先行修复，上游落地后退化为 no-op。
- **T-11 勘定回填（2026-09-21，r1 worktree 7a34a1a67 冷重生成实测）**：
  r1 对 ②症状 = **部分覆盖**——调用点改写在（`__vmOnly('Process.exit', 0)`
  ×3 于 useEditorStore.ts:156/241/249，与消费方 1631 实测 ×3 TS2304 逐一
  对应——其 e82b95b22 二进制为缺白名单改写的 671 中间态，裸
  `Process.exit` 残留即其红面真身）；声明发射缺（`src/natives.d.ts` 零
  `declare const`，对象形态不在函数形态注册表面内）。Env 在 auto-edit
  生成面零真实引用（PLAN-003 T-02 已改道 back api），诉求为前瞻覆盖。
  ①button 撤回项归属：1631 生成器已自带 text variant（= 本计划 ⑥ 形态），
  TS1117 为下游补件非幂等双插（消费方 b0b4676 已修），**上游无动作项，
  登记在案**。→ T-12 执行补修（对象形态声明发射 + 抛错桩，单源五名表）。
- **Phase 2 范围**：核对 r1 T-06 修面对对象形态内建的覆盖完整性 →
  缺则补 → 消费方冷重生成+摘补件复收口（含 1b 退化 no-op 验证）。
- **用户裁定（2026-09-21）**：671 已 execution_done，新需求**不回改已
  完成任务/AC，以附加 Phase 2 承接**（T-11..T-13 + AC-07/08）；r1 的
  review/merge 先行，Phase 2 随后执行。

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
- **AC-07（Phase 2：对象形态内建覆盖）**：§4.2 ②症状（Process/Env
  对象形态）有 r1 覆盖判定结论；残缺项已补修或按分流规则记录在案；
  ①button 撤回项归属登记在档（1631 已自带，无动作项）。
- **AC-08（Phase 2：消费方复跑收口强化）**：交接通知更新为「冷重生成 +
  七类补件全摘」形态并发出；auto-edit 复跑收据回传（build 绿）或复跑
  窗口登记在案（669 §10-4 模式——通知即达标，跨仓收据不阻塞本计划
  终态）。

## 8. 执行步骤

| ID | 任务 | 依赖 | 落点（文件/符号） | 意图 | AC | 验证命令/预期 |
|---|---|---|---|---|---|---|
| T-00 | [x] 七类三态勘定：当前 master 上 auto-edit 逐段摘补件跑 regen + 上游锚点现状比对，产出决策档（2026-09-21，基线 ebfeef30c worktree 裸生成：三态结论=①③⑤⑥附a附b 仍破 / ②④⑦ 已清偿，§4.1 决策档 + §10 四项裁定落定；vue-tsc 基线 95 错全落七类面：store 83+App 7+组件 5） | — | 决策档落 docs/plans/671 §4.1；worktree D:/autostack/.wt/lang-671/auto-lang | 分流依据（已清偿/部分/仍破） | AC-01 | auto-edit gen/ 逐段 diff + build 日志留档；④类负标量 fixture 复验 |
| T-01 | [x] 已清偿（跳过执行）：⑦ gen-only 工程完整——基线实测 gen-only 产出 auto-sources.ts 44KB 真值 + vite-env.d.ts + tsconfig `types:["vite/client"]` 三保险（038 Phase B T8/P657-D2 `prepare_vue_sources` 共享段 :5087-5088 + 668 R-21 tsconfig 根修）；T-09 fixture 复验代证明 | T-00 | crates/auto-man/src/vue.rs（:1540-1603 兜底机制接入 gen-only 分支） | ⑦清偿 | AC-03/04 | fixture + auto-edit 裸 gen 后 pnpm build exit 0 |
| T-02 | [x] 多段插值界符补齐：`expr_to_vue_text_raw` Expr::Str 臂 strip 收窄为「整串恰一段插值」（多段外层界符是内容本身，剥掉即残缺）；auto-edit StatusBar 生成 `{{ store.line }}:{{ store.col }}` 界符完整 + fixture 同型 build 绿 | T-00 | crates/auto-lang/src/ui_gen/vue.rs 插值拼装路径（T-00 钉死行号） | 附a 清偿 | AC-03 | fixture（含 `行:列` 型多段插值文本）build 绿 |
| T-03 | [x] 事件形参装配：删除 `code_editor_event_payload_call` 的循环早退——声明形参的载荷事件（cursor/contextmenu）载荷转发优先于循环参（`(0,_) => None` 保持列表语义循环参权威）；auto-edit 生成 `function EditorCtx(x: any, y: any)` + `@contextmenu="EditorCtx($event.x, $event.y)"`（优于补件的 clientX——脚手架 CodeEditor emit 载荷为 {x,y}）+ fixture OnCtx 同型 | T-00 | crates/auto-lang/src/ui_gen/vue.rs handler 发射路径 | ⑤清偿 | AC-02/03 | fixture（双参坐标 handler）build 绿 + grep 门 |
| T-04 | [x] 已清偿（跳过执行）：② store 自调——668 R-23 self_bare（store 名+`"store"` 字面）已覆盖，基线实测生成 useEditorStore.ts 零 `store.X()` 自调；fixture `.Reset` 内 `store.Bump()` → 裸调 `Bump()` 复验 | T-00 | crates/auto-lang/src/ui_gen/vue.rs :17355-17364 消费链 | ②清偿 | AC-02/03 | fixture（store 内自调）build 绿 + grep 门 |
| T-05 | [x] bp 内联覆盖 store 文件（§10-3 裁定就地内联同体）：`generate_store_composable_full` 增池参数，handler/watcher/computed/module_fn 体引用集 ∩ 显式导入名按池闭包拉取，模块级 function 声明发射（冲突面 R013 同款警告）；api.rs 池传递 + STORE_EXTRA_FILES stash 复用同串（双写路径字节一致）；auto-edit useEditorStore.ts:384 内联 `function toggle_id` + fixture 同型 | T-00 | crates/auto-lang/src/ui_gen/vue.rs bp 内联点（T-00 钉死） | ③清偿 | AC-02/03 | fixture（store 消费 bp fn）build 绿 |
| T-06 | [x] natives 声明层（§10-1 裁定抛错桩）：vm codegen `Codegen::new` 的 55 项平名 intrinsics 表移 `build_bare_native_intrinsics` 单源 + `bare_native_intrinsics()` OnceLock 导出；auto-man `ensure_natives_layer` 四站点接入（gen-only/build/run/scaffold）——注册表 ∩ 生成文件裸用 token 集发射 `src/natives.d.ts`（d.ts 落 src 根：实测 vue-tsc include 只收根级 .d.ts）+ `src/lib/natives.ts` globalThis 抛错桩 + main.ts 装载，JS 保留字过滤（export 等），空集清退；对象级 Env/Process 进 ts_adapter `__vmOnly` 白名单（lifecycle 预检 walker 同步五名表）；auto-edit 23 声明 + fixture 3 声明实证 | T-00 + §10-1 | crates/auto-lang/src/ui_gen/ts_adapter.rs（对象级白名单 :1025-1039 旁） | ①清偿 | AC-02/03 | fixture（front 层用 vm 内建）build 绿；声明按注册表全量非名单 |
| T-07 | [x] button variant 值域（§10-2 裁定扩 cva）：`text: ''` 空差量类 = Rust `button_variant_preset("text")==""` chromeless 互锁镜像；双面同步——WidgetTemplate variants.ts（ui_gen/vue.rs，plan571 互锁测试锚定）+ 脚手架资产 crates/auto-man/assets/shadcn-ui/button/index.ts；auto-edit 冷树 TS2322×12 清零 | T-00 + §10-2 | crates/auto-lang/src/ui_gen/vue.rs :13634 一带 | ⑥清偿 | AC-03 | fixture（non-shadcn variant）build 绿，vm 视觉零回退（若裁定扩 cva） |
| T-08 | [x] aura schema 吸收：menubar_item 增 title/icon/shortcut/checked/enabled（union 形态）+ `element text` 增 text prop（位置内容伪 prop，基线 7×S001 同族）+ `element menubar_checkbox_item` 新入册（vm view_builder 既有臂的 schema 补册，backends 对齐族惯例 iced:none + menubar sub_widgets 收编 + element_coverage 登记 + docs 三围栏同步再生 core.md/kitchen-sink.at〔auto-os 侧 d21bee3 联动提交〕）；auto-edit strict 生成零 S001/S002（EXIT=0） | T-00 | schema/aura.at（:4502 MenubarItem 族）+ validators S001 面 | 附b 清偿（摘 --lenient） | AC-04 | strict 模式（无 --lenient）auto-edit 生成零 S001 |
| T-09 | [x] 测试/fixture 收口：tests/vue-gen-gap 通用 fixture（九特性集中、零消费方业务名）strict gen EXIT=0 + pnpm build 双绿；AC-02 grep 门通过（命中全为注释层案例引注或先在内容，可执行逻辑零名单）；cargo t 全量对基线 diff = 零新增红（预存红 20 = musk_vm_track×7 + ui::layout×13，干净 HEAD 同红在案）；docs_gen/schema_drift 四围栏绿 | T-01..T-08 | tests/ fixture 目录 + crates 内嵌测试 | 证明集中 | AC-03/05 | cargo test 绿 + fixture 矩阵绿 |
| T-10 | [x] 文档收口 + 交接：SD-01/SD-02 落 docs/specs/auto-lang/ui/overview.md（worktree 提交，随 merge 落账）；auto-edit 交接通知留档 §9（669 §10-4 模式：merge 后通知消费方复跑摘补件，见复跑要点） | T-09 | docs/specs/auto-lang/ui/overview.md + 交接记录 | 规范落账 | AC-06 | spec 回读 + 通知留档 |
| **—— Phase 2 分界（r2，2026-09-21 供料 §8 增补批；r1 以上零改动）——** | | | | | | |
| T-11 | [x] 对象形态内建覆盖勘定（2026-09-21，r1 worktree 7a34a1a67 冷重生成实测，§4.2 已回填）：r1 = **部分覆盖**——调用点改写在（`__vmOnly('Process.exit', 0)` ×3，与消费方 1631 实测 ×3 症状逐一对应，其 e82b95b22 二进制为缺白名单的 671 中间态）；声明发射缺（natives.d.ts 零 `declare const`）；Env 在 auto-edit 生成面零真实引用（PLAN-003 T-02 改道 back）；①button 撤回项归属登记（1631 已自带 text variant = 本计划 ⑥ 形态，TS1117 为下游补件非幂等，上游无动作项） |r1 worktree（plan-671-dev @ 7a34a1a67）构建生成消费方 vue 工程，核 `src/natives.d.ts` 是否含 `Process`/`Env` **对象形态**声明（+ `__vmOnly` 抛错桩/调用点改写）——1631 实测只覆盖函数形态；①button 项按勘误撤回，仅登记归属（1631 已自带 text variant，冲突为下游补件非幂等）。覆盖判定（r1 已覆盖 / 残缺）回填 §4.2 | — | r1 worktree 生成物 diff + auto-man natives 层代码 | 覆盖勘定 | AC-07 | 结论在档（§4.2 回填） |
| T-12 | [x] 残缺补修：ts_adapter 白名单五名表单源化 `VM_ONLY_OBJECT_NATIVES`（matches! 臂 / vue.rs lifecycle walker / auto-man 声明层三消费同源）+ `ensure_natives_layer` 增对象形态面（「标识符+成员访问」用面扫描，改写后 `__vmOnly('X.y'` 串同样命中）→ `declare const NAME: { [key: string]: (...args: any[]) => any }` 索引签名（注册表驱动不手写方法面）+ globalThis Proxy 抛错桩（任取成员即抛错带「对象名.成员名」指引）；SD-01 规范面同步增补；验证 = auto-edit 冷重生成 strict gen EXIT=0 + pnpm build EXIT=0（`declare const Process` 实证）+ fixture 增 Env.get 用例双绿（Env+Process 双声明 + `__vmOnly('Env.get','HOME')` 实证）+ cargo t 全量对基线零新增红 |若 r1 T-06 注册表驱动发射只覆盖函数形态（intrinsics 平名表无对象形态面），补对象形态发射（单源纪律：与 ts_adapter 对象级 `__vmOnly` 白名单五名表同源，不手写名单）；已覆盖则标 [x] 已覆盖跳过（证据注记） | T-11 | auto-man natives 层 + ts_adapter.rs | 补修/分流 | AC-07 | 分流记录入 §9 |
| T-13 | [x] 消费方复跑收口强化：§9 交接通知更新为强化版（冷重生成 + 七类补件全摘，含 1b——上游落地后其 typed `declare const Env/Process` 追加与上游索引签名声明会撞重复标识符 TS2300，必须随补件链一并摘除，退化 no-op 验证）；冷重生成等价收据 = T-12 验证（worktree 二进制冷重生成 build 退出码 0，即供料 §8 复验条等价形态）；复跑窗口登记：r1+Phase 2 merge 后消费方执行 |交接通知更新为「**冷重生成**（rm gen/front/vue 后 `auto build --gen-only -r vue`，可摘 --lenient）+ 七类补件全摘（含新增 1b——上游落地后应退化 no-op，一并验证）」形态——补件链重跑在生成器漂移后不可靠（TS1117 形态即证据）；复跑收据回传（解阻判据 = 供料 §8 复验条冷重生成等价形态，build 退出码 0）或复跑窗口登记 | T-12 | §9 交接通知更新 | 复收口 | AC-08 | 通知留档 + 收据/窗口登记 |

## 9. 复审记录

- 2026-09-21 stage: new / PLAN-671 r1 起草交付评审。
  outcome: pass（评审通过即待「开工」授权进入 work）。
  next: review → work（授权后从 T-00 起；T-00 三态结论可能缩面 T-02..T-07
  中若干任务为「已清偿跳过」，属显式分流非缩面违约）。
  备注：上游锚点初勘来自 auto-edit 会话逐类源码核对；670 已覆盖 a2r/vm
  事件面，无重叠。


- 2026-09-21 stage: work / PLAN-671 r1 / outcome: pass（全部 11 任务闭环，
  T-01/T-04 显式分流为已清偿跳过）。
  code_commit: worktree plan-671-dev @
  D:/autostack/.wt/lang-671/auto-lang（c1f986110 实现批+fixture 批 +
  7a34a1a67 specs 批两提交，base ebfeef30c）+ auto-os d21bee3（kitchen-sink 围栏联动再生成成物）。
  task_ids: T-00..T-10（T-00 勘定 §4.1；T-01/T-04 已清偿标注；T-02/03/05/06/07/08
  执行修复；T-09 fixture+回归；T-10 文档+交接）。
  evidence: auto-edit 冷树裸 `auto build --gen-only -r vue`（无 --lenient）
  EXIT=0 + pnpm build（vue-tsc+vite）EXIT=0（AC-04 达成）；基线 95 错 → 清零；
  tests/vue-gen-gap 通用 fixture strict gen + pnpm build 双绿（AC-03）；
  cargo t 全量对干净 HEAD diff 零新增红（预存红 20：musk_vm_track×7 +
  ui::layout×13，与本案无关在案）；docs_gen/schema_drift 围栏绿（AC-05）；
  AC-02 grep 门通过（可执行逻辑零消费方名单）。
  blockers: 无。
  next: review（/auto-plan:review 独立复审 → merge）。

  **auto-edit 交接通知（G4，669 §10-4 模式——merge 后发出）**：PLAN-671
  已清偿 vue 生成器七类缺口中的六执行类（②④⑦为 668/038 既有清偿复验）。
  复跑要点：① upstream merge 后**冷重生成**（rm gen/front/vue 后
  `auto build --gen-only -r vue`，**可摘 --lenient**——strict 零 S001）；
  ② 七类补件全数摘除（regen_vue.py 退化为裸生成调用直至摘除——natives.d.ts/
  store 别名/toggle_id 内联/null→-1/EditorCtx 签名/button text variant/
  auto-sources+env.d.ts 均已上游自备；注意 vue 轨运行期内建缺口仍按 README
  登记——natives.ts 为抛错桩非实现）；③ 存量树脚手架资产 write-if-missing
  （PLAN-457 契约）——冷重生成是取新 button cva 的必要条件；④ jade-edit
  同款补件链（646 同型）同受惠，复验窗口同开；⑤ auto-os 侧 kitchen-sink
  再生成（d21bee3）若画廊 golden 快照受 text 样例行影响，按其围栏提示
  同步重采样。

- 2026-09-21 stage: revision / PLAN-671 **r2（Phase 2 附加）** / 授权=
  用户指令（"上游的计划 671 已经做完了，我们要加新修改需求得在后面附加
  新的 phase"）· 依据 = auto-edit 供料 §8（**勘误版**，F-1/b0b4676 收窄：
  button 撤回、Process/Env 对象形态保留）+ PLAN-004 T-04 收据（§4.2）·
  变更 = **仅追加**（§4.2 + T-11..T-13 + AC-07/08；r1 任务/AC 零改动）·
  status: execution_done → executing（语义修订回退规则）· outcome:
  pass（r2 待执行）· next: **review（r1 范围 T-00..T-10 / AC-01..06）
  → merge → work（Phase 2 自 T-11 起）**。


- 2026-09-21 stage: work / PLAN-671 **r2（Phase 2）** / outcome: pass
  （T-11..T-13 全闭环）。
  code_commit: worktree plan-671-dev 追加 Phase 2 提交 102818dc5（单源
  五名表 + 对象形态声明/Proxy 桩 + fixture Env 用例 + SD-01 增补；r1 两
  提交之后）。
  task_ids: T-11（勘定：部分覆盖结论 §4.2 回填）/ T-12（补修：对象形态
  声明发射 + 抛错桩，单源纪律）/ T-13（交接强化 + 冷重生成等价收据）。
  evidence: auto-edit 冷重生成 strict gen EXIT=0 + pnpm build EXIT=0
  （`declare const Process` + Proxy 桩实证）；fixture 双绿（Env+Process 双
  对象形态声明 + `__vmOnly('Env.get','HOME')` 改写实证）；cargo t 全量对
  基线 diff 零新增红（同 20 预存红）。AC-07（覆盖判定+补修+撤回项登记）
  / AC-08（通知强化+等价收据+窗口登记）达成。
  blockers: 无。
  next: review（r1+Phase 2 全范围 T-00..T-13 / AC-01..08）→ merge。

  **auto-edit 交接通知（G4 强化版，取代 r1 版——merge 后发出）**：
  PLAN-671 r1+Phase 2 已清偿 vue 生成器七类缺口（六执行类修复 + ②④⑦
  既有清偿复验）**及 1631 复验的对象形态残缺**（Process/Env：调用点
  `__vmOnly` 改写 + 防御性 `declare const` 索引签名 + globalThis Proxy
  抛错桩）。复跑要点：
  1. upstream merge 后**冷重生成**（rm gen/front/vue 后 `auto build
     --gen-only -r vue`，**可摘 --lenient**）——补件链重跑在生成器漂移后
     不可靠（TS1117 形态即证据：下游补件⑥对已自带 text variant 的 1631
     生成器非幂等双插）。
  2. **七类补件全摘**（含 PLAN-004 新增 1b）：natives.d.ts/store 别名/
     toggle_id 内联/null→-1/EditorCtx 签名/button text variant/
     auto-sources+env.d.ts 上游均自备；**1b 的 typed `declare const
     Env/Process` 追加必须摘除**——与上游索引签名声明撞重复标识符
     TS2300；摘除后 regen 退化 no-op 一并验证。
  3. vue 轨运行期内建缺口仍按 README 登记限制（natives.ts/Proxy 为抛错
     桩非实现）；存量树脚手架资产 write-if-missing（PLAN-457 契约）。
  4. jade-edit 同款补件链（646 同型）同受惠；auto-os 侧 kitchen-sink
     再生成（d21bee3）golden 如受影响按围栏提示重采样。
  冷重生成等价收据（供料 §8 复验条形态）：本计划 T-12 已以 worktree
  二进制冷重生成 + build 退出码 0 实证；消费方正式复跑窗口 = merge 后。


- 2026-09-21 stage: review / PLAN-671 r2 / outcome: **pass**。
  reviewed_commit: 102818dc58238ba21bf2bd46ae42a03c60fdb810（worktree
  plan-671-dev；r1 = c1f986110 实现+fixture、7a34a1a67 specs，Phase 2 =
  102818dc5；diff 15 文件 +687/−102）。
  base_commit: ebfeef30c。dependency_revisions: auto-os d21bee3
  （kitchen-sink 围栏联动再生成成物，已落 auto-os main）。
  spec_inputs: docs/specs/auto-lang/ui/overview.md（SD-01 §「vue 生成工程
  自完备契约」/ SD-02 §「menubar 族 props schema 吸收」，worktree 提交在案）。
  独立性声明：与实施同会话——结论自工件重建（关键验证全部在 reviewed
  commit 上重跑），不采信执行侧叙述。

  acceptance_results（复审重跑实证）：
  - AC-01 pass：§4.1 九类三态决策档（生成物证据+锚点行号）+ §4.2 Phase 2
    增补与 T-11 勘定回填在档。
  - AC-02 pass：`git diff ebfeef30c..HEAD` 15 文件 grep 消费方特定标识符
    （EditorCtx/useEditorStore/confirm_idx/SyncCursor/toggle_id 等）——仅
    2 行注释层案例引注（repo 惯例形态），可执行逻辑零命中。
  - AC-03 pass：fixture 冷重跑 strict gen EXIT=0 + pnpm build EXIT=0 零 TS
    错；natives.d.ts `declare const Env/Process` 双对象形态 + 22/23 函数
    形态声明实证；②④⑦已清偿类 fixture 复验在 T-09 记录。
  - AC-04 pass：auto-edit 冷树（rm gen/ 后裸 strict 生成）EXIT=0——消费方
    widget 零 S001/S002（20 条 S001 全在 bps 依赖 reference 语料，Warning
    容忍面）；pnpm build EXIT=0 零 TS 错；生成物断言全中（EditorCtx(x,y)
    签名 + $event.x/y 实参 / toggle_id 内联 / {{line}}:{{col}} 界段 /
    text:"" variant / ref<number>(-1) / auto-sources+vite-env /
    __vmOnly('Process.exit') ×3）。
  - AC-05 pass：**cargo tf 3685/3685、cargo tv 3832/3832、cargo tb
    3732/3739**（tb 7 红 = book_listing ch02_05/ch03_08/ch06_05/06/08/09/
    ch09_02，**detached ebfeef30c 基线同红实证=全预存**）；日常档 cargo t
    对基线 diff 零新增红；examples 面 = 002-counter strict gen EXIT=0 抽检
    （natives 层空用面干净清退）+ docs_gen/schema_drift 围栏绿。
  - AC-06 pass：SD-01/SD-02 落账（worktree 提交）；通知留档 §9 强化版
    （发出时点 = merge 后，669 §10-4 模式即计划设计）。
  - AC-07 pass：覆盖判定在案（r1 部分覆盖：调用点改写在/声明发射缺）+
    T-12 补修 + ①button 撤回项归属登记（1631 已自带，TS1117=下游非幂等）。
  - AC-08 pass：强化版通知（冷重生成 + 七类补件全摘，1b 撞 TS2300 警示）
    + 冷重生成等价收据（worktree 二进制冷重生成 build 退出码 0）+ 复跑
    窗口登记（merge 后）。

  findings（全部非阻塞，已处置）：
  - F-1（info）：master 预存红三簇（日常档 20 = musk_vm_track×7 +
    ui::layout×13，full 配置下不存在；tb book_listing×7）——668 全绿口径
    后引入，非本批（对基线零新增）→ 债册 P671 条目在案，需独立归因计划。
  - F-2（enhancement）：menubar 族 vue 渲染保真（裸 div 发射）→ 债册登记。
  - F-3（enhancement）：bps reference 语料 S001 容忍面 → 债册登记。
  - F-4（process）：复审期裸 `git stash` 共享栈事故——干净树上 pop 弹掉他
    人 stash@{0}（plan-637 026-final）留 UU 冲突态；已 `git restore` 还原
    HEAD、stash 栈三条原样保全（冲突 pop 不消费栈条目）。教训与既有记忆
    「裸 git stash 共享栈禁用」一致：基线比对一律用 detached checkout。

  evidence: 门禁数字与生成物断言已内联上文（持久于本计划文件）；tmp 日志
  为辅助。auto-edit/fixture 复跑命令与退出码见 §9 work 记录 + 本记录。
  next: merge（/auto-plan:merge 五 checkpoint）。

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

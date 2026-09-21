---
plan_id: PLAN-682
status: executing
feature_name: mcp-snapshot-determinism-and-vue-generator-fixes（快照投影确定性 + vue 生成器四缺陷修复批）
author: [zhaopuming]
created_at: 2026-09-22T00:00:00+08:00
updated_at: 2026-09-22T00:00:00+08:00
plan_revision: 1
current_step: 0
total_steps: 4
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
---

# [PLAN-682] MCP 快照投影确定性 + vue 生成器四缺陷修复批

> 来源：jade-edit PLAN-002（execution_done）执行期实证登记 + 2026-09-22
> 四路源码级根因分析（jade-edit `docs/upstream/2026-09-jade-supply.md`
> 供料包 + parity-ledger D-17/D-18）。669 先例：下游实证与根因已备，
> 本计划为上游本体修复。worktree = `D:/autostack/.wt/lang-682/auto-lang`，
> branch `plan-682-dev`（基点 3f2eb0b56）。

## 1. 背景：四个下游实证缺陷

| # | 下游登记 | 现象 |
|---|---|---|
| F1 | jade parity-ledger D-18 / 供料包 §4 | MCP `autoui_snapshot` 的 style/onclick 属性行随实例双态发射（state 逐字节一致、vnode id 不变）——消费侧字节基线间歇漂移（auto-edit F-RV6 家族） |
| F2 | jade D-03/D-17 + regen-vue 补件③守 | widget `key:` 绑定表达式被发射为计数器字面量（`:key="'AutoDownEditor-8'"`），切档不重挂载 |
| F3 | jade D-13 + 垫片后装绕过 | 生成的 natives.ts R-tier 实装块无条件覆写 `console_*`/`file_basename`（S-tier 有先到先得、R/B-tier 没有）；且 `join('\\\\n')` 转义过度一层（实发反斜杠+n 非换行） |
| F4 | auto-lang bd64d8df6 在案（Plan 676 外部在途）/ jade 补件④守 | `--gen-only` 路径零 wrapper 物化检测——GalleryShell.vue（dep leg 写盘）import `@/components/ui/popover` 而未物化 → vue-tsc TS2307 |

## 2. 根因与修复设计

### F1 快照投影双态（iced/renderer.rs + mcp_server.rs）

根因：`StyledNodeSnapshot` 两个写入者按帧时序「最后写入者冻结」：
- 写入者 1：view() 的 MCP 同步块（renderer.rs ~21395-21405，
  PLAN-062 T11 门控内）——`view_with_debug_gated(false)` 把探针丢弃为
  `_probe`，computed 恒写 `HashMap::new()` ⇒ 结构上恒空（状态 B）；
- 写入者 2：update() 的 `__bounds_collected` 臂（~16890-16991）
  `from_live` 填 computed，但仅 `capture_debug && dirty` 帧运行 ⇒
  非脏帧（如纯 resize 的 gate_ws 帧）后永不覆盖。

修复（恒发射 = 状态 A）：
- **F1a**（renderer.rs ~21395）：接住探针——
  `let (view, id_map, probe) = state.component.view_with_debug_gated(true);`
  用 `probe.snapshot()`（与 ~21802-21808 既有合并逻辑同构的
  path→vnode id 派生）填充 computed 的 raw_class/events 再
  `set_styled_vtree`。bounds 字段仍留给写入者 2 回填
  （ComputedNodeLite 全 Option）。
- **F1b**（防御性，mcp_server.rs `set_styled_vtree` ~377）：拒绝
  「降级覆盖」——新快照 computed 为空且 vtree 与现有一致时跳过写入。

### F2 `key:` 绑定表达式直发（ui_gen/vue.rs）

根因：`autodown_editor` 走 `generate_shadcn_attrs` curated 臂（白名单
制，`key` prop 静默丢弃），随后 Plan 360 catch-all（~8085-8097）只查
生成串中无 `:key=` 就补发计数器字面量；组件早退臂的 explicit-key 契约
（~7372-7374 `has_explicit_key = props.contains_key("key")`）从未推广
到 shadcn curated 路径（对照 `code_editor` curated 臂 ~13115-13129 有
正确的动态 key 转发）。

修复：Plan 360 处（单点覆盖全部 curated 臂）——
`props.contains_key("key")` 时先经 `expr_to_vue_bound_value` 发
`:key="<expr>"`，仅当不存在才走计数器。jade 补件③（:key 绑回）在
上游落地后按 pattern-断言 fail 撤除。

### F3 natives.ts R-tier 守卫 + join 转义（auto-man/src/vue.rs）

根因：`ensure_natives_layer`（~2066-2073）R-tier/B-tier 无条件
`g['x'] = ...`（同文件 S-tier ~2044-2046/2057-2065 均有先到先得）；
~2067 `out.join('\\\\n')`（Rust 源双转义）发射 JS 串值 `'\n'` =
反斜杠+n 两字符（上方注释却写 "joined with a newline"）。

修复：R-tier/B-tier 四名外包 `if (!(n in g))`（与 S-tier 同构）；
`'\\\\n'` → `'\\n'`。jade 垫片可回退先装语义（后装保留亦可，语义
统一后不再敏感）。

### F4 gen-only wrapper 物化（auto-man/src/vue.rs）

根因：wrapper 物化 = 检测语料驱动（`detect_shadcn_components`
~189-221 扫 `@/components/ui/*` → `vue_shadcn::materialize`
write-if-missing，bundle 内含 popover）——但检测+物化只挂全量构建
路径；`--gen-only` 入口 `prepare_vue_sources`（~5698-6342）全程零
detect 零 materialize，dep leg（~6047-6109）写盘的 GalleryShell.vue
从不进语料；另有 components/bps 子目录通道（~3691-3702）漏调 detect
（兄弟臂 ~3731/~3879 均有）。

修复：`prepare_vue_sources` 尾部对本次写盘的全部 SFC 内容跑
detect 并集 + 直接调 `crate::vue_shadcn::materialize(&output_dir,
&needed)`（幂等）；子目录臂补一行 detect 对齐。

## 3. 执行步骤

| # | 任务 | 产出 | 验证 |
|---|---|---|---|
| T-01 | F1a+F1b 快照确定性 | renderer.rs / mcp_server.rs | cargo check；jade vm_matrix 对新 exe 双跑快照逐字节稳定（属性态恒定） |
| T-02 | F2 :key 直发 | ui_gen/vue.rs | cargo check；jade regen 产物 App.vue 含 `:key="store.active_key"` 原生发射、jade 补件③ pattern 断言 fail 提示撤除 |
| T-03 | F3 natives 守卫+转义 | auto-man/vue.rs | cargo check；生成 natives.ts R-tier 带守卫 + join('\n') 真换行 |
| T-04 | F4 gen-only 物化 | auto-man/vue.rs | cargo check；jade `--gen-only` 冷删 components/ui 后 popover/ 物化、vue-tsc 零 TS2307、补件④自然 no-op |

## 4. 验收标准

- AC-01 F1：同一应用两实例快照文本逐字节一致（属性恒发射）；
  `autoui_action`/`autoui_find` 的 onclick 断言面不回归（jade 九检查
  全绿对新 exe）。
- AC-02 F2：声明 `key: <expr>` 的 widget 生成动态 `:key` 绑定；
  无 key 声明维持计数器形态（既有快照/生成物零漂移面不扩大）。
- AC-03 F3：R-tier 赋值带先到先得守卫；`console_lines` join 真换行。
- AC-04 F4：`--gen-only` 独立路径物化全部被引用 wrapper（含 popover）。
- AC-05 jade-edit 侧全门（gate + e2e 九检查 + bench）对新 exe 全绿；
  三个补件（③④/垫片后装/孤儿清理）进入可撤除态（按各自 pattern
  断言提示逐个撤除）。

## 5. 非目标

- D-17 引擎键入发射（auto-down engine 侧，另计划）；PLAN-673 rope/
  分块读主线；a2r 词汇门（681 在途）。

## 6. 复审记录

- 2026-09-22 work 入场（auto-plan-work，用户直接授权上游修复批）：
  根因分析 + 修复设计见 §2（jade-edit 四路勘察报告在案），T-01 起步。
- 2026-09-22 T-01..T-04 实施完成（commit 5f46d99fb）+ 验证实录：
  - **AC-01 (F1)**：新 exe 下三全新实例快照恒带 style/onclick、逐字节
    同尺寸（5201B×3——旧双态消失）；jade vm_matrix 双臂对新 exe
    10/10+9/9 ALL GREEN，基线 v2（id 序列）零漂移。
  - **AC-02 (F2)**：裸 gen-only 产物 App.vue 编辑器原生发射
    `:key="store.active_key"`（零补件状态）。
  - **AC-03 (F3)**：生成 natives.ts 含 6 处先到先得守卫（S-tier 2 +
    R-tier 4）+ `join('
')` 真换行。
  - **AC-04 (F4)**：冷删 popover/ 后 gen-only 重新物化（Popover 三件 +
    index.ts）；裸产物 vue-tsc 出现 3×TS1117（DataTableCrud 族 dep
    demo 组件）——**归属界定**：worktree 基点 3f2eb0b56 落后主检出
    dirty 态中另一会话的未提交生成器修复，与本批四修复正交（四修复
    产物面各自直验通过）；主检出会话落地后 rebase 复验。
  - cargo check auto-lang + auto-man lib 绿（auto-man bin 空壳为上游
    既有态）。

## 7. 待澄清事项

- F1a 的 `view_with_debug_gated(true)` 会让每次 view() 都启用探针
  （成本面）——若回归可改条件 `mcp_active`（同 ~21539 口径），执行期
  按实测定。

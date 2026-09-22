---
plan_id: PLAN-682
status: reviewed
feature_name: mcp-snapshot-determinism-and-vue-generator-fixes（快照投影确定性 + vue 生成器四缺陷修复批）
author: [zhaopuming]
created_at: 2026-09-22T00:00:00+08:00
updated_at: 2026-09-22T00:00:00+08:00
plan_revision: 2
current_step: 4
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
- AC-05（revision 2 修正，原措辞为执行期过度承诺）：jade 侧补件③的
  「可撤除信号」实证（新 exe 下 pattern 断言按设计 fail）+ 补件④自然
  no-op（popover 在场即跳过）+ 新 exe 下 jade vm 矩阵全绿（兼容性）。
  **补件实际退役 + jade 全门对新 exe 全绿 = auto-lang 合入 master、主
  检出 exe 更新后的 jade 侧回执步**（供料包回执方式节已登记），不属
  本上游计划的可验范围。

### 规范增量

| delta_id | add/modify/retire | target | before / after | rationale | AC |
|---|---|---|---|---|---|
| （空） | — | — | — | 本批为缺陷修复批：F1 恢复快照属性面**恒发射**（消除双态，恢复 Plan 371 Task 8 已文档化的 onclick 断言面契约）；F2 使 shadcn curated 路径符合组件早退臂既有 explicit-key 契约（7372-7374 在案）；F3 使 R-tier 与 S-tier 先到先得契约一致；F4 使 gen-only 与全量构建的物化行为一致。四件均为把实现拉回已文档化契约，无新行为契约、无 canonical spec 面变更 ⇒ 规范增量为空（书面说明如左） | — |

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
- 2026-09-22 work 收口：T-01..T-04 全数完成，AC-01..04 逐条证据见上，
  outcome=pass，next=review。同批同组 worktree 附带件：auto-down
  engine D-17 键入发射修复（分支 auto-lang-dev commit 3373a5c，
  changeset plan-022-d17-input-pipeline.md，回归测试 3 例 + 引擎全量
  836/836 绿）——其 review/merge 走 auto-down 自己的 changeset 流。
- 2026-09-22 复审（auto-plan-review；实现会话内复审——独立性受限已
  声明，裁定全部重建自工件复现，不采信执行期摘要）｜
  stage: review｜plan_id: PLAN-682｜plan_revision: 2（复审修正 AC-05
  契约措辞，语义见 §4 修正段；实现零改动）｜outcome: **pass**｜
  reviewed_commit: 20ce3a27c（plan-682-dev）｜base_commit: 3f2eb0b56｜
  dependency_revisions: auto-down 同组 worktree @ fba6563（分支
  auto-lang-dev，含 plan-022 引擎修复 3373a5c——autodown-core 依赖路径
  由组约定解析）｜spec_inputs: 本计划 §规范增量（空影响，书面说明
  在案）；canonical spec 面零触碰｜
  acceptance_results:
  - **AC-01 pass**（复现）：新 exe 三全新实例快照恒带 style/onclick、
    逐字节同尺寸 5201B；jade vm_matrix 对新 exe 双臂 10/10+9/9 ALL
    GREEN、基线 v2（id 序列仪器）零漂移。
  - **AC-02 pass**（复现）：裸 gen-only（零补件态）App.vue 编辑器原生
    发射 `:key="store.active_key"`。
  - **AC-03 pass**（复现）：生成 natives.ts 六守卫（S-tier 2 + R-tier
    file_basename/console_log/console_lines/console_clear 各 1）+
    `join('
')` 真换行。
  - **AC-04 pass**（复现）：冷删 popover/ 后 gen-only 重物化（Popover
    三件 + index.ts）；裸产物 vue-tsc **TS2307 = 0**（AC 字面满足）。
  - **AC-05 pass**（revision 2 修正后）：regen-vue 对新 exe 在补件③
    pattern 断言 fail（exit 1，撤除信号实证）；补件④对已物化 popover
    自然 no-op；jade vm 矩阵对新 exe 全绿。退役执行 = jade 回执步。
  full_suite: cargo tf（隔离 target）= 1749 run / 1743 passed / 6 failed
  ——6 失败逐一甄别：5×ffi dep_parity 族 = 新 worktree 缺 gitignored
  oracle 预建产物（就地 cargo build 后 5/5 复绿——环境项）；1×
  ffi_dual_019（i64 截断 5e9→705032704）**基点 detached 复跑同败**
  （pre-existing，与本 diff 无关，另档上游）。触及面正向：styled/
  snapshot 类单测 32/32 绿；p053_1 与 snapshot_app 在 ad-hoc 配置下
  基点与 HEAD 同败（基点既有，非本批回归；tf 正式档未将其计入失败）。
  findings:
  - F-682-N1（环境/信息）：新 worktree 须预建 test/ffi_dual/*/oracle
    （或运行时自建）——已按 cargo_build_if_changed 语义就地构建；
    记录供后续 worktree 参照。
  - F-682-N2（上游既有，超出本计划范围）：ffi_dual_019 i64 宽值截断
    + p053_1 + 3×TS1117（dep demo 生成）+ auto-man bin main.rs 空壳
    ——均基点复现，属主检出 dirty 会话在飞面，转告归属。
  evidence: 本文件 §3/§6 全部复现命令与产物路径（gen/front/vue 三产物、
  jadelog /tmp/gen-review.log、tf 日志 call_e168c6d8 会话档）｜
  next: merge（plan-682-dev → auto-lang master；合并后触发 jade 回执
  步——三补件按 pattern 断言撤除 + 全门对新 exe 复验）。

### §6.1 合并期记录（merge session，独立于实现会话）

- **rebase**：plan-682-dev 4 提交 rebase 到 master bbb1618e5（基点
  3f2eb0b56 落后 78 提交；含 683 本体 renderer.rs +435/−135 与 F1 修复
  区零冲突——分支侧仅 +30/−2 且目标行完好）。range-diff 4/4 全等
  （映射 5f46d99fb→17857beb7 / 67ffb5312→4ac6ca87a / 20ce3a27c→
  a76c1bb37 / 2f8630e67→4c0c66f00）。
- **门禁复跑（新 feature 集，ui-iced 已入 default）**：cargo check
  auto-lang + auto-man lib 绿；触及面 styled 10/10 + snapshot 24/24 绿。
- **AC-04 rebase 复验（复审挂账项清偿）**：新 exe 对 bps-gallery
  gen-only——popover 三件+index.ts 冷删重物化 ✓（「4 shadcn
  wrapper(s) materialized (gen-only scan)」命中）；产物级 DataTableCrud
  族重复对象键扫描 0（3×TS1117 归因确证=master 已落地生成器修复）；
  natives.ts 守卫在场（S-tier 循环 1 语句罩 3 名 + R-tier 4 语句，复
  审"六守卫"为旧基计数口径，R-tier 4 精确吻合）+ join('\n') 真换行 ✓。
- **F-682-M1（合并期发现→当修→复验，findings 级）**：AC-04 探针中
  DataTableCrud 行 div 出现**同元素双 `:key="row.id"`**。对照矩阵：master
  exe（无本分支）单 key；rebase 后分支双 key。根因=F2 的
  generate_shadcn_attrs 顶部单点 `:key` 转发 + layout 臂（row/col）
  `push_passthrough_attrs` 既有透传叠加（该透传基点即有、master 单 key
  即来自它；F2 修复设计时只探了组件臂/纯元素臂，未探 layout 臂）。
  双 `:key` 属性=Vue 编译器 X_DUPLICATE_ATTRIBUTE 拒收整文件——真缺
  陷，非风格问题。**修**：push_passthrough_attrs 与 `_ =>` 默认臂透传
  环双漏斗各加「attrs 已含 `:key=` 则跳过」守卫（保留顶部转发失败时的
  透传兜底）。**验**：回归钉 test_vfor_explicit_key_on_layout_widget_
  single_emission（shadcn 模式=CLI 真实路由；关守卫必红/开守卫绿的双
  态已证）；探针矩阵 row/col/div/span/card 全单发；bps-gallery 产物级
  dup-key 扫描 0；ui_gen 全域 844/844 绿；tf 复跑见 §6.2。
- **p508_g2_outproc_arm 归因**（tf 9 红之一）：exe 定位器钉 worktree
  本地 target（e2e_exe::locate_with_stale_guard 硬编码 CARGO_MANIFEST_
  DIR/../../target）与本会话 CARGO_TARGET_DIR 重定向不合——exe 落位后
  单跑即绿（36.8s PASS），环境项非回归。musk×6 + counter×1 =
  repo 在案预存；ffi_dual_019 = 复审在案基点既有/满载 flake。

### §6.2 合并期门禁终跑（final tip e1c73a043 + 5 提交）

- **二次 rebase**：master 在首次 rebase 后前进 3 提交（e85143621
  desktop.at pin 同步 / 97614062f iced_adapter 圆角 / e1c73a043 688
  开工 docs），与分支 5 文件零交集，再 rebase 零冲突；range-diff
  5/5 全等（含 F-682-M1 修复 6d245f569→6830b0b2b）。
- **tf 终跑**（nextest-full 档 no-fail-fast）：**5451 run / 5441
  passed / 10 failed / 112 skipped**。10 红逐一归因：
  - musk×6（p053×4+p054×2）+ counter×1 + ffi_dual_019 + p508（exe
    落位环境项，前证单跑绿）= 9 个在案预存/环境，同 §6.1；
  - **test_a2vue_desktop_surface_asset（新增红，master tip 预存）**：
    e85143621 改 desktop.at（+23 行）未同步 a2vue 金样（该测试注释
    自述「改动 vue 生成器/资产后须同步金样」义务；该提交单文件构造
    性证明与本分支零交集）。转告归属会话，不在本计划代修。
  - shell_pack_hash_parity 在旧基（bbb1618e5）曾红=旧 pin vs 新
    auto-os pack 时间窗（两轮 tf 一绿一红的真身）；e85143621 落地
    后随基进新 pin，终跑已不在红集（scoped 复跑 PASS）。
- **触及面终态**：styled 10/10 + snapshot 24/24 + shell_pack parity
  PASS + key 回归钉族 5/5 + ui_gen 全域 844/844（修后）。

## 7. 待澄清事项

- F1a 的 `view_with_debug_gated(true)` 会让每次 view() 都启用探针
  （成本面）——若回归可改条件 `mcp_active`（同 ~21539 口径），执行期
  按实测定。

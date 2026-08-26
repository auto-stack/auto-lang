# Plan 443 — defineModel 降级收窄（PLAN-037 T4 修正：深 mutation 响应性断裂）

状态: complete ✅（已合并 master `38adb1ef4`，2026-08-26 finish-plan 复审通过并归档）
创建: 2026-08-24
来源: auto-down DEBTS.md 015 阻断行 / plans/015-auto-lang-dsl-capability-debt.md Phase 4 PLAN-037 联动项上报
基线: master 41e7985c（问题①TS2440 已在 73861f8d 修复；本计划修问题②）

## 验证结果（2026-08-24，全部通过）

- auto-lang：`cargo test -p auto-lang --lib` **3136 绿**（vm::ui_console ring
  buffer 偶发 flaky 与本计划无关，单跑稳定绿）；`--test vue_capabilities`
  **62 绿**（60 基线 + 新增 cap_model_channel_bound_downgrades_child /
  cap_model_channel_unbound_keeps_ref；预存红 cap_widget_map_model_init
  **自动回绿**，未改动该测试本身）；a2vue golden 10/10（8 例 defineModel 行
  回翻 ref）。
- auto-man：**227 绿**（新增 test_plan443_cross_file_bound_model_channels：
  app.at 父绑定 panel.at 子的 doc channel → 子 defineModel + 工厂默认；
  board.at 未绑定 → ref，全链路 from_workspace + generate 落盘断言）。
- 三仓 regen（worktree auto.exe）：
  - jade：regen OK（gen 树 vue-tsc 零错）+ **e2e 23/23**（白板 #21
    "add note" 回绿——阻断解除）；WhiteboardPage.vue 回
    `const doc = ref<any>({})`。
  - demo：regen OK + **e2e 9/9**。
  - editor：regen OK + **vue-tsc -b 零错**（exit 0）。
- auto-down 侧三仓部署产物已随验证部署（工作树修改未提交，由 auto-down
  侧决定收编时机）。

## finish-plan 复审（2026-08-26）

- 交付物核验：A（vue.rs:443/450/736 `bound_model_channels`+`emitted_model_bindings`+三分支发射）、
  B（api.rs:302/356 双 pass 选项与聚合）、D（auto-man vue.rs:1958 "Plan 443 prescan"）、
  E（scenario-dialect.md:36-37 语义行）全部在 master 落地；运行时 canary
  `examples/capability-tests/041-model-deep-reactivity/` 存在（ab34fa9f4）。
- 验证重跑（2026-08-26，master `779b2db87`+）：`--test vue_capabilities` **72 绿**
  （含 cap_model_channel_bound_downgrades_child / unbound_keeps_ref，超集于声称的 62）；
  a2vue golden **10/10**（与声称一致）；`cargo test -p auto-man` **229+6 绿**（超集于声称的 227）。
- 判定：**A 类全完成**，无遗漏子项、无 workaround（双 pass 成本风险 §风险 已按构造排除）。
  遗留边界（增量路径/pages 跨文件 channel、widgets-gallery 存量 gen 产物刷新）均为
  计划 §边界与不做 中明示的范围外项，其中后者无测试守卫不影响门禁。


## 问题

PLAN-037 T4（c696a729）把 widget 的 model var **无条件**编译为
`defineModel<T>("name", { default })`。T4 注释声称"未绑定 = 本地 ref，行为不变"，
在 Vue 3.5 运行时**不成立**：

- defineModel 底层是 useModel（@vue/runtime-core 3.5.38 :4305-4350）。父级未绑定
  该 channel 时，get 返回裸内部值（customRef 的 localValue），**不是 reactive 代理**：
  `doc.value.shapes.push(x)` 无依赖追踪、不触发 computed/渲染。
- 对照：`ref({})` 的 `.value` 是 reactive 代理，同样代码正常触发。
- 实证：jade e2e 白板 22/23 红 —— `WhiteboardPage` 的 `var doc map = {}`（未被任何
  父级绑定；main_area 只传 path/key/class），"Add note" 后 `addNoteShape(.doc)` 内的
  `doc.shapes.push` 不渲染。种子 shape 因 `.Init` 整体赋值走 setter 正常。
- 附带风险：`{ default: {} }` 对象/数组**字面量**默认值跨组件实例共享
  （Vue props default 需工厂函数）。

阻断三仓（demo/editor/jade）regen 采用 T4 产物。

## 根因定位（auto-lang 侧代码现状）

- T4 降级点：`crates/auto-lang/src/ui_gen/vue.rs` 状态变量发射（~:2265-2281，
  master 行号 2252-2274 附近），无条件 defineModel。
- T5 已有项目级 channel 信息：`VueGenerator::sub_widget_models`（widget 名 →
  model var 名列表），父侧 `try_model_channel_attr`（~:11294）据此在调用点把
  `channel: .slot` 折叠为 `v-model:channel`（非 writable slot 是硬错误）。
  - 两个发射点：已知子 widget 元素路径（~:3955，要求 `is_known_sub_widget`）与
    `AuraNode::Component` 路径（~:4707）。
  - 跨文件收集：auto-man `VueProject::from_workspace` Phase 1（vue.rs:1882-1909，
    front 兄弟文件）；同文件合并：`generate_component_from_file`（api.rs:479-490）。
  - pages/ 路径（scan_pages_dir）与增量路径（incremental_compile_changed）今天
    **不传**跨文件 map —— 跨文件 v-model 仅在 front 兄弟与 app.at 间有效（预存 T5 范围）。
- jade 全部 model var 均无父级绑定（GraphPage 的 centerPath/depth 是 props 非 model）
  → 收窄后 jade 全部回 ref，白板恢复深响应。

## 方案：绑定感知的降级（bound-aware downgrade）

判定原则：**"channel 被绑定" = 某父侧调用点实际发出 `v-model:channel`**。
收集机制不做独立的 AST 走查器，而是**复用父侧 codegen 自身的判定**（在
`try_model_channel_attr` 命中时记录），保证收集与发射逐字节一致、永不漂移。

### A. VueGenerator（ui_gen/vue.rs）

1. 新字段 `emitted_model_bindings: Vec<(String, String)>`（(tag, channel)），
   `try_model_channel_attr` 返回 Some 的分支里 push。
2. 新字段 `bound_model_channels: HashSet<String>`（**当前 widget** 的被绑定
   channel 名）+ builder `with_bound_model_channels(Vec<String>)`。
3. 状态变量发射改为三分支：
   - channel 被绑定 → `const x = defineModel<T>("x", { default: INIT })`；
     若 `state.initial` 是 `Expr::Object(_) | Expr::Array(_)` 字面量 →
     `default: () => (INIT)`（工厂，防跨实例共享）。
   - 未绑定 → 回到 T4 前形态 `const x = ref<T>(INIT)`（深响应语义）。
   - JS（非 TS）模式同理。
4. `needs_ref` 恢复：`widget.state_vars.iter().any(|s| !bound.contains(&s.name))`
   并入现有条件（plan 015 的收窄注释同步修正）。

### B. generate_component_from_file（ui_gen/api.rs）

1. `ComponentGenOptions` + `bound_model_channels: Option<HashMap<String, Vec<String>>>`
   （跨文件绑定，调用方预扫描汇总后传入）。
2. `GeneratedComponent` + `bound_model_channels: HashMap<String, Vec<String>>`
   （**本文件**发现的绑定，供调用方聚合）。
3. 双 pass：
   - pass 1：若合并后的 channel map 非空，用现有配置（无 bound）逐 widget 生成一遍，
     harvest 全部 `emitted_model_bindings` → 本文件绑定 B_sf。
     （父侧 v-model 判定只依赖 sub_widget_models/known_sub_widgets，两 pass 完全
     一致，故 pass1 的绑定集合 == 真实发射集合；生成失败直接上抛。）
   - pass 2：`bound = opts.bound ∪ B_sf`，逐 widget
     `.with_bound_model_channels(bound.get(name))` 正式生成；校验警告取 pass 2。
   - channel map 为空时跳过 pass 1（多数文件的快路径）。

### C. auto-lang 公共入口（lib.rs）

- `ui_build_shadcn_with_sub_widgets_and_stores_full` + `bound_model_channels` 参数
  （app.at 路径）。
- `ui_build_shadcn_all_widget_codes` + bound 参数（pages 路径；或加 `_with_bound`
  变体以免扰动既有调用者 —— 实现时按调用面决定）。

### D. auto-man from_workspace（auto-man/src/vue.rs）

1. Phase 1 之后新增**预扫描**：对 app.at、每个 front 兄弟 .at、pages/ 递归每个 .at，
   各自用与其真实生成**相同的** channel map（app.at/front 兄弟 = phase1 map +
   同文件；pages = 仅同文件，与现状一致）调 `generate_component_from_file`（丢弃
   产物，只取 `bound_model_channels`），并集 → B。
2. 正式生成接线 B：app.at 的 full 调用、app.at 附加 widget 手工 regen、pages 的
   all_widget_codes 调用、front 兄弟手工 regen（`.with_bound_model_channels`）。
3. 增量路径（incremental_compile_changed）不引入跨文件 map（维持 T5 预存范围），
   同文件绑定经 B 的双 pass 自动正确。

### E. 语义文档

`docs/specs/auto-lang/ui/design/scenario-dialect.md` 的 model-var 段补一行：
未绑定 model var = ref（深响应）；仅父级 v-model 绑定的 channel = defineModel。

## 测试

1. 单测更新（T4 时代断言回翻）：vue.rs :14141（Counter count → ref）、:16764
   （content → ref）、:21242（accent_color → ref）、SlashMenu import 断言
   （ref 回到 import 列表）。
2. a2vue golden 8 例：defineModel 行回翻为 ref（跑测试 → input.wrong.vue 覆盖
   expected）。
3. 新增锁定测试（vue_capabilities.rs）：
   - 同文件父绑定 → 子 channel 发 `defineModel` + 调用点 `v-model:x`
     （经 generate_component_from_file 双 pass，临时文件）。
   - 绑定 channel + `var doc map = {}` → `default: () => ({})`。
   - 未绑定保持 `ref<any>({})`（既有 `cap_widget_map_model_init` 自动回绿，不改）。
4. 基线：`cargo test -p auto-lang --lib` 全绿；`--test vue_capabilities` 60 绿
   （cap_widget_map_model_init 回绿）；`cargo test -p auto-man` 既有用例不回归。
5. 三仓 regen（auto-down 侧，build 本仓 auto.exe 后）：
   - jade：`bash gen/regen.sh` → e2e 23/23（白板 Add note 场景回绿）。
   - demo：`E2E_PORT=5199 pnpm exec playwright test --workers=1` 9/9。
   - editor：`pnpm exec vue-tsc -b` 绿。

## 边界与不做

- 不改父侧 T5 判定（writable-slot 硬错误、v-model 折叠规则不变）。
- 不扩增量路径/ pages 的跨文件 channel 可见性（T5 预存范围，另行立项）。
- 不动 `cap_widget_map_model_init` 以外的预存失败（本计划无 —— 该例将自动回绿）。
- widgets-gallery / musk 等仓内存量 gen 产物（defineModel 行）不在本计划刷新，
  作为后续 housekeeping（无测试守卫，不影响门禁）。

## 风险

- 双 pass 生成成本 ×2（仅当 channel map 非空；widget 生成毫秒级，可忽略）。
- 既有 SFC 产物大面积回翻 ref（预期内：T4 产物本就未被三仓采用）。
- pass1/pass2 判定不一致风险：按构造排除（父侧判定不读 bound 集合）。

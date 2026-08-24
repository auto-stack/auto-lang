# Plan 444 — Vue codegen 五类缺陷修复（解锁 auto-shell ash-gui Vue 构建，auto-shell-057）

状态: in_progress
创建: 2026-08-24
来源: KNOWN-DEBT-AND-RISKS.md `auto-shell-057`（41f4a7b7 登记）+ auto-shell DEBTS.md「Vue 产物构建引擎侧阻塞（Plan 057 Phase 5 T-B）」外部报告
基线: master 8c2a2d4a
验收: 下游 `ash-gui-auto` 复现流程 `auto gen && cd gen/front/vue && npx vue-tsc` **零错误**（无需 pnpm add / rm 手工补丁），`pnpm run build` 绿；本仓 lib / vue_capabilities / auto-man 测试全绿。

## 问题（13 错 / 5 类 + 模板缺口，逐类根因）

复现基线：`auto gen` 后 `gen/front/vue` 内 `npx vue-tsc` 报 13 错。逐类定位
（行号为 2026-08-24 生成物）：

### ① 子组件回调 props 通道断裂（App.vue 168/176/183、BlockList.vue 69，4 错）

三个独立缺陷叠加：

- **1a 子侧必填 snake prop**：`on_delete: msg` 无同名 Pascal 变体时
  （BlockItem 的 Msg 是 `DeleteBlock` 而非 `Delete`），`prop_is_emitted_callback`
  判否 → `on_delete: () => void` 以**必填**形态留在 defineProps；父级
  `@Delete` 传入的是 `onDelete` → TS2345。msg 型 prop 本质是回调通道，
  从不应作为数据 prop 声明。
- **1b 父侧事件名与子侧 emit 永不相配**：父级绑定 `on_delete: .DeleteBlock`
  经 `sub_widget_event_to_vue` 一律按 prop 名派生 `@Delete`，而子侧空
  handler 桥实际 `emit('DeleteBlock')` —— 类型与运行时双双断裂。
  语料两套习惯并存：k2/k3/015/017 子侧按 prop 名派生 emit（`on_select`↔
  `Select` 变体/`props.on_send()` 改写），ash-gui 子侧透传自身同形 msg
  （`on_delete`↔自身 `DeleteBlock`）。区分信号只有子侧 emits 名册。
- **1c 文本拼接解析丢接收者**：`text "#" + j.id` 只吃 `"#"` 作 primary
  prop，`+` 与 `j.id` 沦为兄弟文本节点，`{{ j.id }}` 退化为 `{{ id }}`
  （App.vue 168/176 的 TS2339）。VM 侧 STR_CAT 本就支持 str+int，修复
  解析后双端一致。

### ② 可空变体字段脚本访问（BlockItem.vue 65/99，2 错）

handler 体内 `cell.Tagged.text`（api.ts 中 `Tagged?: TaggedCell | null`）
报 TS18049。else 分支的变体不变量保证运行时非空 —— 发射非空断言
`cell.Tagged!.text`（`?.` 在 strict 下引入 `string|undefined` 赋值新错，
不可用）。规则：ts_adapter 的 Dot 发射，接收者为「PascalCase 变体字段
访问」链时补 `!`（api.ts 惟 PascalCase 可选字段即变体 payload，snake
数据字段不受影响）。

### ③ 多参 msg emit 签名（BlockItem.vue 206/209，2 错）

`Sort(int,int)`/`Filter(str)` 无 on-block handler → defineEmits 门控
（需 handler_params）不放 payload 类型（`Sort: []`），模板引用的未定义
handler 桩也无参（`function Filter(): void { // TODO }`）。修复：emit
payload 类型一律取自 msg 变体；未定义 handler 桩升级为**带参 emit 桥**
（参数 `arg0..argN: any`，与既有空 handler 桥同构）。

### ④ VM-only stdlib 泄漏 + async 误判（PromptBar.vue 488/494/509×2，4 错）

- walker 三处（`stmts_contain_api_call_with` 的 check_stmts、
  `extract_api_calls_from_ast_stmts`、`stmts_call_complete`）都不下探
  **else 分支 / while / for 循环体**：`complete(...)` 深藏 else 链 →
  async 误判为 false（TS1308）、api 导入漏发（TS2304）、debounce 包装
  漏识别。
- `fs.read_dir`/`File.is_dir` 无 JS 对应物时原样输出（TS2304/TS2339）。
  按「显式报错优于静默坏代码」：改写为 `__vmOnly('<name>', ...)` 抛错
  桩（`never` 返回型，可处任意表达式位），gen 期发 R 级警告。

### ⑤ str 字段动态变体读（useShellStoreStore.ts 178，1 错）

`.__sse_status.Failed`：运行时二态（裸串或 `{"Failed": msg}`），.at 靠
VM 动态求值；TS 侧 `ref<string>` 读 `.Failed` 报 TS2339。规则：ts_adapter
Dot 发射，接收者为 **str 型 state ref**（typed_strings 已有该信息）时走
any 通道 `(__sse_status.value as any).Failed`。

### ⑥ gen 模板缺口（复现需 2 步手工补丁）

- `VueDependencyUsage`：@vueuse 只在 carousel/sidebar 时声明，但
  progress/scroll-area/table 脚手架同样 import @vueuse → fresh gen 丢依赖。
- `sync_code_editor_shell` 清理条件是「与当前模板逐字节相等」：旧版脚手架
  残留（ash-gui 现场）永不匹配 → 带着未声明的 codemirror import 存活。
  放宽为「unused 且含 vue-codemirror/codemirror import 签名」即删。

## 方案

### P1 回调通道（1a+1b+1c）

1a：`prop_is_emitted_callback` 放宽为「任意 `on_*: msg` prop」——全部
不进 defineProps；`props.on_x(...)` 改写链（musk-022/PLAN-037 T3）对
未匹配变体的 prop 同样生效（emit 名 = prop 名 Pascal 化，017-chat 的
`on_send` 习惯）。

1b：新增「子 widget emits 名册」跨层接线：

- `VueGenerator::widget_emit_set(widget)`（pub）：变体名 ∪ handler 体内
  `.on_x(...)` 自调用的 Pascal 化名（子侧实际会 emit 的全量）。
- `ComponentGenOptions.sub_widget_msgs` + `with_sub_widget_msgs`；
  api.rs 同文件 widgets 自动并入 + opts 跨文件合并。
- 父侧 `on_X: .Y` 绑定（PascalCase 子组件/known sub-widget 三处调用点）
  派生规则：`PascalOf(X) ∈ 名册` → `@PascalOf(X)`（语料习惯，含未知子
  组件回退）；否则 `Y ∈ 名册` → `@Y`（ash-gui 透传习惯）+ R 级警告；
  都不在 → `@PascalOf(X)` + 警告（现状回退）。
- auto-man `vue.rs` Phase-1 预扫描顺带收获名册，app.at/_full 入口与
  兄弟文件再生成（2016/2175 两处 VueGenerator）均传入；lib.rs 新增
  opts 直通入口避免再堆 positional 参数。

1c：`parse_view_node` primary prop 为 Str 且**后随 `+`/`-`** 时改为
`parse_expr()` 吃完整二元链（`"#" + j.id` → Bina）。VM 走 STR_CAT、
Vue 走 `{{ "#" + j.id }}`。

### P2-P5（见上「问题」各节，均为 ts_adapter/vue.rs 局部规则）

### P6（auto-man vue.rs 局部）

## 任务

- [ ] P1a prop_is_emitted_callback 放宽 + props.on_x 改写覆盖未匹配变体
- [ ] P1b emits 名册接线（widget_emit_set / opts / 三绑定点 / auto-man 预扫描）
- [ ] P1c text 拼接解析折叠
- [ ] P2 变体字段访问非空断言
- [ ] P3 emit payload 门控放开 + 未定义 handler 带参 emit 桥
- [ ] P4 walker 补全（else/while/for）+ __vmOnly 桩 + R 警告
- [ ] P5 str ref 点访问 any 通道
- [ ] P6 @vueuse 检测扩展 + CodeEditor 鲁棒清理
- [ ] 测试：vue_capabilities 新增各类 canary；auto-man 新增 444 依赖/清理测试
- [ ] 下游复现验证（auto gen → vue-tsc 0 错 → pnpm build 绿，无手工补丁）
- [ ] 全量回归：`-p auto-lang --lib` / `--test vue_capabilities` / `-p auto-man` / schema drift
- [ ] KNOWN-DEBT-AND-RISKS.md auto-shell-057 标 ✅ + commit 号

## 风险与边界

- 1a 放宽影响面：全语料 msg prop 均为通道（无 `:on_x=` 值绑定用例），
  k2 canary 兜底（props.on_select 改写覆盖）。
- 1b 未知子组件（无 名册）保持 prop 名派生 —— k2/k3/015/017 行为不变。
- P3 带参桥对「模板裸引用带 payload 变体 handler」的新 TS2554 风险：
  语料无此形态（loop-var/显式参全覆盖），新增 canary 固定。
- merged/VM 目标零改动（P4 桩仅 Vue/ts 适配层；1c 解析修复对 VM 是
  STR_CAT 正常路径）。

---
plan_id: PLAN-528
status: execution_done
feature_name: widgets-gallery 检查问题跟踪与修复
author: [zhaopuming, ZCode]
created_at: 2026-09-03T11:30:00+08:00
updated_at: 2026-09-03T15:10:00+08:00

# Leave these EMPTY here — /auto-plan:review fills them:
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 2
total_steps: 2
---

# [PLAN-528] widgets-gallery 检查问题跟踪与修复

## 变更摘要

用户对 widgets-gallery 的 Vue 版（master 最新源码，`auto run -r vue`，端口 3024）
进行人工逐页检查；本计划作为**滚动跟踪计划**：检查中发现的每个问题在此登记
（问题清单 §9），逐条转为执行步骤修复并验证。首项 W1（alertdialog 页接
onclick + toast）已在计划创建前完成并验证，本文档回填记录。

## 目标

1. 为 widgets-gallery Vue 版人工检查发现的问题提供统一的登记、修复、验证跟踪。
2. W1：alertdialog 文档页的 `alert-dialog-action` / `alert-dialog-cancel`
   注册 onclick 事件，点击后用 toast 显示用户的选择。
3. 后续问题（W2+）按同一流程追加：登记 → 修复 → 双端（或 vue 单端，按问题定）
   验证 → 勾销。

## 架构方案

- 纯示例资产层修改：只动 `examples/widgets-gallery/src/front/**`（`.at` 页面/组件），
  不改 `crates/` 编译器与 VM。若后续发现的问题必须改 codegen 才能修，再在该问题
  条目中显式标注 "needs-codegen" 并升级改动范围评审。
- 修复流程：改 `.at` 源 → `AUTOUI_MCP_PORT=9347 auto run -r vue` 重新生成 +
  起 vite（3024）→ 生成产物 grep 断言 → Playwright 交互验证（复用
  `.agents/skills/autoui-verifier/scripts/test_vue_playwright.mjs`）→ 截图存档。

## 技术栈

- Auto 语言 `.at` 页面源码（widget / model / on / view 语法）
- vue 生成轨：vite 5 + Vue 3 + shadcn-vue（reka-ui）+ vue-sonner + tailwind
- 验证：Playwright 1.62（标准脚本）、hash 路由 URL `http://localhost:3024/#/<route>`
- 备忘：pac.at `render: "vm"`，故 vue 版必须显式 `auto run -r vue`

## 需求分析与背景调查

- 规约台账（.autoos/specs.json v1）：本项目 goals/architecture/designs/tests/
  reviews/reports 六节，历届 gallery 相关沉淀（PLAN-408 §9、PLAN-412 layout、
  PLAN-437 toast 严格模式修复等）为本计划的先行上下文；本计划属于示例资产维护，
  预期不产生新 spec 组件（review 阶段复核）。
- toast 页面模式（PLAN-437 确立）：`on { .h -> { toast(...) } }` + 页面挂
  `toast-provider {}`（vue 端映射 `<Toaster/>`，vue-sonner）。
- 事件附着语法：`widget (onclick: .Handler) {}`；handler 在 `on {}` 块定义。
- vue codegen 已内置 alert-dialog action/cancel 的 onclick 通路
  （`crates/auto-lang/src/ui_gen/vue.rs` snake 臂 "alert_dialog_action" |
  "alert_dialog_cancel" 注记 "Handled by event handlers below"，通用事件映射
  onclick → @click），W1 无需改编译器。

## 详细设计

### W1 alertdialog 页事件 + toast（已完成，回填记录）

`examples/widgets-gallery/src/front/pages/alertdialog.at`：

1. widget 头部新增 `on` 块：
   - `.confirmAction -> { toast("AlertDialog choice: Continue") }`
   - `.cancelAction -> { toast("AlertDialog choice: Cancel") }`
2. `alert-dialog-footer` 内：
   - `alert-dialog-cancel "Cancel" { onclick: .cancelAction }`
   - `alert-dialog-action "Continue" { onclick: .confirmAction }`
3. `preview-card` 内追加 `toast-provider {}`（vue 端渲染 `<Toaster/>`）。

### W2+ 追加问题的模板

每个问题登记于 §9 问题清单（现象 / 复现 URL / 期望），转为执行步骤时：
- 明确目标文件（`src/front/pages/<x>.at` 或 `src/front/components/<x>.at`）
- 精确改动内容与验证命令（grep 断言 + Playwright 动作序列 + 截图路径）
- 涉及 VM 端对照的问题补 `auto run -r vm` 双端核验（VM 崩溃问题见 §9- OBS-1）

## 测试设计

- 门禁分级：Category A（纯示例资产）——不跑 `cargo t` / `docs_gen`。
- 每项修复的验证（vue 轨）：
  1. 生成产物断言：`grep -n "@click\|toast" gen/front/vue/src/pages/<x>.vue`
  2. 交互验证：`node .agents/skills/autoui-verifier/scripts/test_vue_playwright.mjs
     "http://localhost:3024/#/<route>" /tmp/out.png --actions '[...]'`
  3. 截图存档（不入库）：`D:\d\tmp\`（脚本以 cwd 解析 `/d/tmp` 前缀）
- W1 实证：continue/cancel 两次点击 toast 文案均正确出现（ad_3/ad_4 截图）。

## 验收标准

- [x] W1：`/alertdialog` 页 action/cancel 点击触发对应 toast，文案含用户选择
      （已验证，见复审记录前执行记录）
- [ ] W2+：逐条按登记的期望验收，完成一条勾一条
- [ ] 全程零编译器改动（如破例须在条目中显式记录 reasons）
- [ ] `git status` 中 examples/widgets-gallery 改动仅限本计划登记的文件
      （注意 `.auto/ui-cache.json` 为生成伴随缓存，属预期副产物）

## 执行步骤

### W1 alertdialog onclick + toast（已完成 2026-09-03）

- [x] T1 编辑 `examples/widgets-gallery/src/front/pages/alertdialog.at`：
      on 块两 handler + 两标签 onclick + `toast-provider {}`
      验证：`grep -n "AlertDialogAction @click\|AlertDialogCancel @click\|vue-sonner"
      gen/front/vue/src/pages/alertdialog.vue` → 3 处命中
- [x] T2 重新生成并启动：`AUTOUI_MCP_PORT=9347 auto run -r vue`（后台），
      `curl -s -o /dev/null -w "%{http_code}" http://localhost:3024/` → 200
- [x] T3 Playwright 交互验证：Show Dialog → Continue → toast
      "AlertDialog choice: Continue"；Show Dialog → Cancel → toast
      "AlertDialog choice: Cancel"。截图 `D:\d\tmp\ad_3_continue.png`、
      `D:\d\tmp\ad_4_cancel.png`

### W2 深浅主题切换：移植 019 统一 SettingsPanel（已完成 2026-09-03）

- [x] T1 新建 `examples/widgets-gallery/src/front/components/settings.at`：
      SettingsPanel 按 019 忠实移植（⚙ Header + ✕ 关闭 + Theme 分段切换 +
      Accent 五色点 + 零参消息 prop 回传），调色板对齐 Plan 409 §8
      indigo/coral/ocean/sage/amber。
      验证：`grep -n "SettingsPanel" gen/front/vue/src/App.vue` → import +
      `<SettingsPanel :accent_color... @ToggleDark...>` 命中；
      `ls gen/front/vue/src/components | grep -i settings` → SettingsPanel.vue
- [x] T2 编辑 `examples/widgets-gallery/src/front/app.at`：
      `use { package: official from "./components" }`；model 增
      `dark_mode bool = true`；on 增 `.ToggleDark`/`.closeThemePicker`/
      `.SetIndigo/.SetCoral/.SetOcean/.SetSage/.SetAmber`（019 零参消息模式，
      避免带参消息过 prop 的 arity 风险）；`if .themeOpen` 弹层内容替换为
      `SettingsPanel (...)` 调用（绝对定位壳保留）。
      验证：`grep -n "watch(dark_mode\|classList.toggle" gen/front/vue/src/App.vue`
      → App.vue:174 `watch(dark_mode, v => classList.toggle('dark', v) +
      applyAccent 重刷)`（Plan 458 自动生成，codegen 零改动）
- [x] T3 重新生成 + Playwright 全流程（`AUTOUI_MCP_PORT=9347 auto run -r vue`，
      http 200）：palette 打开面板（深色：Dark 高亮+五色点，indigo ring 选中）
      → ☀ Light 全站翻白（首页/侧栏/面板白底、Light 高亮）→ 浅色下导航
      /button 主题保持 → 🌙 Dark 回程全站复暗。截图
      `D:\d\d\tmp\w2_1_panel_dark.png`、`w2_2_light.png`、
      `w2_4_button_light.png`、`w2_3_back_dark.png`。

已知边界（登记为 OBS-4/OBS-5，不再属于本条验收）：
- 页面内硬编码深色样式（如 codeblock 代码体 bg-zinc-900）在浅色下保持深色
  （preview-card 代码区可见）；token 驱动区域（bg-background/border 等）全部正确翻转。
- 主题状态为 App 内存态：SPA 内路由切换保持，深链直载/刷新回默认深色。

### W3 命名议题（⏸ 已裁决：搁置——2026-09-03 用户裁决记录）

用户观察：`alert-dialog`（模态）替代的是浏览器 `alert()`，比 inline 的
`alert` 更常用，建议互换命名：`alert`=现在的 alert-dialog，inline alert 改用
单词别名（message/info 等）。

**用户裁决（2026-09-03）**：搁置，仅记录。理由与战略背景——
- 当前已发布的 widgets 是仿照 shadcn 做的，与 shadcn 命名/结构统一是现阶段的
  一致性锚点（ZCode 评估同向：生态主流约定即 alert=inline，反转成本高）。
- **远期（独立大议题，组件库稳定后统一讨论）**：跨平台需求会要求更多组件；
  web 端实现也可能脱离 shadcn 基底改为自己实现。届时命名体系可整体重议，
  不在组件库磨合期单点翻转。
- 届时讨论的输入保留在本节：互换提案原文、生态对照、增量别名方案
  （`confirm`/`callout`）均可作为远期命名体系的备选项。

原评估结论（存档）：

现状事实（2026-09-03 调研）：
- `alert`：inline 提示条，schema Content 类；vue 端 shadcn-vue Alert；**VM 端
  fallback 未实现**（render_support.rs:237 "alert/toast not implemented"）。
- `alert-dialog`：模态确认（shadcn/Radix AlertDialog parity）；vue 端完整；
  **VM 端同样 fallback**（overlay 家族降级为 Column，render_support.rs:255）。
- 存量使用面极小：官方示例仅 gallery 的 alert.at / alertdialog.at 两页。

评估要点：
- 生态主流约定恰是 alert=inline（Bootstrap/shadcn/Radix/Chakra/MUI/AntD/
  Element Plus 全部如此；模态确认各自叫 confirm/MessageBox/Modal.confirm）；
  AutoUI 现名与 shadcn/Radix 严格对齐。
- 反转 = 破坏性 API 变更：schema（alert/alert_title/alert_description/
  alert_dialog_* 全套）+ vue.rs 两 match 臂 + registry WidgetSpec + VM 端 +
  gallery 两文档页（含 npx shadcn-vue 安装命令对照）+ 存量用户代码（需别名
  兜底→长期双名负担）；且 gallery 立身之本是 1:1 文档化 shadcn 名。
- 建议路线（保持 parity 的增量方案）：
  A. 两页面互加"何时用哪个"交叉说明（纯示例层，无争议）；
  B. 可选：给 alert-dialog 增单词别名 `confirm`（语义=浏览器 confirm()，
     符合"弹出的 alert"直觉；同 toast/toaster 别名先例，schema+双端小改）；
  C. 可选：inline alert 增别名 `callout`（优先级低）。
- 若用户坚持反转：应作为 L2 级 API 契约变更另立 design doc + 迁移方案，
  不在本计划（示例检查修复）范围内执行。

### W4 Auto 示例代码两缺陷（已完成 2026-09-03）

- [x] T1（crates，Category B）修 `..handler` 双点打印 bug：
      `crates/auto-lang/src/ui_gen/vue.rs` 两处（node_to_auto_code 约 6653、
      generate_codeblock_html 约 6744）由 `format!("{}: .{}", ...)` 改为
      条件补点（handler 已含前导点则原样）。
      验证：`cargo check -p auto-lang` 通过；`cargo t ui_gen` 734/734 绿；
      `cargo build -p auto` 重建 CLI。
- [x] T2（示例层）`alertdialog.at` preview-card 增 `auto:` 覆盖，展示含
      `on {}` 块的完整可运行 widget Demo（handler 声明+实现+view+toast-provider）。
- [x] T3 全量重生成验证：`.auto/ui-cache.json` 为内容寻址缓存（touch 无效，
      需删除强制重建）——删除后 85 页全部重生成；
      `grep -c "onclick: \.\." gen/front/vue/src/pages/*.vue` 全零；
      alertdialog.vue Auto 串含 `.confirmAction`/`.cancelAction`（单点）与
      on 块；Playwright 截图 `D:\d\d\tmp\w4_1_full.png` 目检确认。

附带发现：
- OBS-6（存量，非本计划引入）：`cargo t --test gallery_golden` 中
  `plan370_015_behavior_tests::d8_toggle_dark_mode` 在 master 即失败
  （断言 initial dark_mode 应为 false 实为 true；已用 git stash 验证与
  W4 改动无关）。待单独排查。
- 生成缓存坑：`.auto/ui-cache.json` 按源码内容 hash 命中，mtime touch 不触发
  重生成——改 codegen 后必须删缓存（登记 OBS-3 同类的启动摩擦事实）。

### W5 charts 5 页打不开（已完成 2026-09-03）

用户判断"charts-gallery 未移植"——**证伪**：四个 chart 组件
（line/bar/area/donut_chart.at）与 examples/charts-gallery 逐字节 diff 一致
（Plan 498/499 "三副本同步"机制在位），页面/路由/产物齐全。真因是
**codegen 文本转义缺失**：

- 根因：文档表格里的泛型文本 `List<str>` 被 node_to_html 原样写进 vue 模板，
  `<str>` 解析为未闭合元素 → SFC 编译 500（"Element is missing end tag"，
  line-chart.vue:210）→ 路由懒加载失败整页打不开。四张 chart 页同根因；
  flow-diagram 实测未命中（用户操作时点疑为 vite 瞬断，实机验证正常）。
- 修复（crates，Category B）：`ui_gen/vue.rs` 三处文本出口接既有
  `escape_html_text`（Plan 451 P2 助手，菜单/导航一直在用，此三臂漏接）——
  `expr_to_vue_text` 纯文本返回、独立 Text 节点臂、提升进 slot 的 Text 子节点。
  含 `{{` 的插值结果不转义。
- 门禁：`cargo check` 通过；`cargo t ui_gen` 734/734 绿；`cargo t -p auto-man vue`
  332 绿；重建 CLI；删 `.auto/ui-cache.json` 全量重生成。
- 验证：5 页全部 http 200 + Playwright 零 console error + 截图目检
  （`D:\d\d\tmp\w5_line-chart.png` 折线/坐标轴/图例齐全、`w5_flow-diagram.png`
  LR/TD 双布局、bar/area/donut 同过）；产物中 `List&lt;str&gt;` 转义正确。
- **全站 69 页扫描**顺带揪出 W6 两个新 500（见下）。

### W6 kitchen-sink / navitem 页 500（W5 全站扫描发现，已完成 2026-09-03）

**(a) navitem**：model 用逗号分隔声明 `model { activeTab str = "preview", picked
str = "inbox" }` → 解析器吃出空名变量（生成 `const , = ref`）+ 变量名错位
（`const str = ...`）→ TS 编译 500。修复：页面源改家规换行式多变量声明。
（逗号支持属 parser 增强，登记 OBS-8 待立项。）

**(b) kitchen-sink**：三层问题，逐层修：
1. `autodown_editor` 需要 `@autodown/*` 引擎包：pac.at 补 `npm_deps`
   （015-notes 同款 inline-array link 形式）。**发现并修复机制缺口**：
   `package_json_deps_drifted`（auto-man/vue.rs:354）只对比 OPTIONAL_DEPS
   用量、不查 npm_deps → 声明了永远不重写；且 `auto run` 走
   `run_vue_project` 增量路径，根本不经过 regenerate_source_files 的漂移
   重写。修复两处：drift 检查纳入 extra_deps；run 路径按 Plan 458
   index.html 自愈先例加 package.json 每次运行自愈块（"✓ Updated
   package.json (npm_deps sync)"）。门禁：`cargo check -p auto-man
   --all-targets` + `cargo t -p auto-man vue` 332 绿 + 重建 CLI。
2. `avatar/image (src: "sample")`：裸词 src 被当静态资源 import（
   `import _imports_0 from "sample"`）→ 解析 500。修复：指向实际公共
   资源 `/icon.png`。
3. `window_thumbnail {}`：桌面壳（Plan 526 域）专属 widget，`@/wm/*`
   脚手架在 web 画廊不存在。修复：从 web 画廊 kitchen-sink 移除，注明
   归属桌面壳 demo。

验证：69 页全量扫描 0 个非 200；navitem/kitchen-sink Playwright 实机零
console error（`D:\d\d\tmp\w5_*.png`）。

### W7 togglegroup 渲染缺陷：纵向裸 B/I/U（已完成 2026-09-03）

用户观察：ToggleGroup 页只显示 B/I/U 字符且上下排列,shadcn 原型是横向
"三连按钮"(outline 变体、aria-label、单选/多选 UX)。

根因(三层全缺,互相叠加):
1. schema/aura.at:toggle_group/toggle_group_item 的 `backends.web: "none"`、
   无 aliases、无 `vue:` 映射 → apply_schema_vue_mappings 无从附加 vue 通道;
2. WidgetRegistry:无 ToggleGroup/Item spec → map_tag 的
   shadcn_component_name 查空 → 塌缩为裸 div;
3. generate_shadcn_attrs 臂拼写死区:normalization 把 `-` 折成 `_`,页面
   书写 togglegroup/togglegroup-item 归一为 togglegroup/togglegroupitem,
   而既有臂名为 toggle_group/toggle_group_item → 恒不命中。

修复(三处联动):
- schema/aura.at + schema.rs:web "component"+aliases(PascalCase/kebab/
  无连字符)+vue 映射(ToggleGroup/Item @/components/ui/toggle-group)+
  variant(default,outline)/size(default,sm,lg)契约;item 增 aria-label
  (替代 label)。
- registry.rs:注册 ToggleGroup/ToggleGroupItem WidgetSpec(Form 组,
  with_alias kebab 形式;canonical 小写自动覆盖无连字符拼写)。
- vue.rs:臂名归并三种拼写 + variant/size/aria-label 发射(item 的
  aria-label 兼容 label 旧拼写;size=default 不发射)。
- auto-man:detect_shadcn_components 增 SCAFFOLD_DEPS 闭包表——
  toggle-group 的 bundled 物化连带 ui/toggle(ToggleGroupItem.vue import
  toggleVariants,缺它 vite 裸屏报错)。

门禁:cargo check(ui-gen+auto-man)零新错误;cargo t ui_gen 735/735 绿
(含新增回归 test_plan528_toggle_group_registry_mapping);cargo t -p
auto-man vue 333 绿;重建 CLI;删 ui-cache 全量重生成。
验证:产物 import { ToggleGroup, ToggleGroupItem } + <ToggleGroup
type="multiple" variant="outline">;Playwright 实机三连按钮横排渲染
(D:\d\d	mp\w7_final.png 零 console error);全站 69 页扫描 0 非 200。

排查附带发现:
- 排查中一度误判"新二进制未生效",实为 `cargo build | tail` 管道吞退出码
  + 旧 vite 实例堆积占用 3024→3033 端口(vite 自动跳港)。已清理 10 个
  僵尸 vite;OBS-2 升级认知:静默退出=原生崩溃 exit 127,与端口堆积互相
  放大,重跑可续(缓存增量)。

### W8 togglegroup 连体样式 + 单选示例（已完成 2026-09-03）

用户观察：三连按钮没有合在一起（三个独立圆角块+缝隙）；要求做成 shadcn
原型的连体样式,并补单选示例。

根因：bundled 快照是旧版 shadcn-vue toggle-group——root 写死
`flex items-center justify-center gap-1`,item 各自 rounded-md+border,
即"分离三按钮"观感;新版 shadcn 的连体(依赖闭包/首尾圆角)未包含在该
快照版本中。

修复（示例层,W7 的 codegen 链路已就绪）:togglegroup.at 两组示例 root
注入 tailwind-merge 可覆盖的连体 class——`gap-0` 消缝 + `[&>*+*]:-ml-px`
边框叠压 + 首尾/中段圆角分配;新增 Single (alignment) 示例:
model.alignment + `value: .alignment` 走 codegen `v-model` 通路(type
single 单选互斥)+ f-string "Selected: ${.alignment}" 实时回显。
非目标:bundled 快照升级到新版连体设计(影响所有工程,登记 OBS-11)。

验证:产物 `<ToggleGroup v-model="alignment" type="single" variant=
"outline" class="gap-0 ...">`;Playwright 实机:Multiple 三连按钮合体
(w8_joined.png),点 Right→激活高亮+"Selected: right"联动
(w8_single_right.png),零 console error。

### W9 VM popover 面板裸透明+全宽（已完成 2026-09-03）

用户观察:VM 版点击按钮弹框出现,但无边框无背景(且全宽拉满)。

根因:VM 端面板 chrome 设计上来自 `popover` 标签的 class(Plan 422
"content 元素自带 visual wrap"),而 shadcn 语义是 PopoverContent **组件
自带** chrome(bg-popover border rounded-md shadow-md p-4 w-72)——页面
popover 未写 class,Vue 端靠组件自带 chrome 正常,VM 端面板裸透明;
且 overlay content 列无宽度约束,被 viewport 上限拉满整窗。

修复(crates/aura_view_builder.rs convert_popover):class 缺省时给
shadcn PopoverContent 同款默认面板 chrome
`w-72 bg-popover border border-border rounded-md shadow-md p-4`
(显式 class 整体覆盖,坐标锚 contextmenu 形态同样受益)。VM Style 对
未知 token 容错(filter),border-border 若不可解析仅该条丢弃。

门禁:cargo check 零新错误;cargo t -p auto-lang iced 160/160 绿;
重建 CLI。验证(VM 实机,MCP 驱动):导航 /popover → 按 Open Popover →
面板 w-72 锚定按钮下方,背景/边框/圆角/内边距齐全,不再全宽
(src/front/tests/screenshots/w9b.png;截图目录不入库)。

W9 追加打磨(用户反馈:贴按钮太近/对齐方式/防撞边界):
- DEFAULT_GAP 0→4px(对齐 shadcn/radix sideOffset=4,0 缝时边框粘连);
- shadcn 嵌套形态缺省对齐 BottomStart→Bottom(锚中,shadcn popper
  align=center 默认);坐标锚(contextmenu)保持 BottomStart,menubar 用
  自带显式 BottomStart 不受影响;显式 placement 恒优先;
- 越界翻转(垂直):下方放不下且上方放得下→翻到锚上方,反之亦然;
  剩余越界由既有 snap_within_viewport 钳制兜底(默认开启)。
  复验截图 w9c.png:居中+4px 间隙。

附带:VM 模式本次未复现 OBS-1 启动崩溃(721GB alloc),疑似页面相关。

### W10 VM mobile 断点内容双份（2026-09-03 定位,待 VM shell 专项修复）

现象：窗口收窄过 md 断点（≤768）后,页面内容整体 ×2 重叠（两种不同换行
宽度=两种布局约束的两份绘制叠印）,同时 mobile 色素正确出现（汉堡菜单、
底部导航、桌面侧栏隐藏）。

对分实验结论（pac.at window 700x900 冷启动复现,排除 resize 过渡）：
- **结构无双份**：MCP snapshot 树中页面节点仅一份（"Popover"×1、
  "Installation"×1,"Open Popover"×2 中一处在展示用代码串里,合法）;
- **绘制层双通道**：同一棵树被以两种宽度约束各绘制一遍叠印（两份文案
  换行点不同=两种 max 宽度）,且 mobile 断点门控本身工作正常
  （Plan 527 T7 的类过滤对）。
- 定位：绘制合成层——虚拟窗口 fit 缩放/表面管理（Plan 512/463）与
  Plan 527 T7 响应式重建的交互,疑似旧表面未随断点重建移除（双表面
  叠印）。

修复建议（专项）：VM shell 表面生命周期排查——断点翻转时虚拟窗口表面
是否复用/重建、fit 缩放通道与直绘通道是否同时上屏。涉及 iced 0.14
overlay/virtual_window/dock 合成,非示例层可修。
复现配方：pac.at `window: "700x900"` 冷启动 → MCP 截图即现（本次实验
已还原 pac.at）。

> 2026-09-03：W10（问题 A）与 OBS-1（问题 B）已立项 **PLAN-530**
> （`docs/plans/530-vm-mobile-paint-crash.md`）专项深挖,本计划保留
> 定位结论与复现配方。

### W2+ （待问题清单追加）

（占位：每追加一问题，在此按 T 粒度展开，并同步 total_steps/current_step。）

## 复审记录

（待 /auto-plan:review 填写。）

## 待澄清事项

1. vue 路由为 hash 模式（`createWebHashHistory`），直接访问 `/alertdialog`
   静默落回首页而非 404/跳转——是否算问题、要不要改 history 模式，待用户定夺。
2. 生成器把页面 demo 在产物中渲染两份（preview + 代码串旁边的一份完整拷贝），
   Playwright `text=` 选择器会命中未打开的那份，需要 `[role=alertdialog]` 作用域
   限定——属预期结构还是冗余渲染，待确认。

## §9 问题清单（滚动登记）

### 已修复

| # | 问题 | 来源 | 状态 |
|---|------|------|------|
| W1 | alertdialog 页 action/cancel 无 onclick，应用 toast 显示选择 | 用户（人工检查） | ✅ 已修复并验证（本文档 W1） |
| W2 | 有主题色选择器、无深/浅主题切换；采纳 019 统一 SettingsPanel 移植 | 用户（人工检查） | ✅ 已修复并验证（本文档 W2） |
| W3 | 命名议题：alert（inline）vs alert-dialog（模态）名字是否应互换；modal 场景更高频，建议 alert=模态、message/info=inline | 用户（人工检查） | ⏸ 已裁决搁置（shadcn 对齐期保持现名；整体命名体系留待组件库稳定后随"脱离 shadcn 基底"大议题统一讨论，见 W3 节） |
| W4 | alertdialog 页 Auto 标签页示例代码两个问题：(a) 缺 `on {}` 块——cancelAction/confirmAction 无声明与实现，复制不可运行；(b) `onclick: ..cancelAction` 双点（源码写的是单点 `.cancelAction`） | 用户（人工检查+截图） | ✅ 已修复并验证（本文档 W4；a=示例层 auto: 覆盖，b=codegen 双点 bug） |
| W5 | charts 相关 5 页打不开：LineChart/BarChart/AreaChart/DonutChart/FlowDiagram；疑似近期 chart 重实现（charts-gallery）未完整移植到 widgets-gallery | 用户（人工检查） | ✅ 已修复并验证（本文档 W5；移植完整，真因=文本转义缺失） |
| W6 | （W5 全站扫描发现）kitchen-sink、navitem 页 500：npm_deps 不生效 / model 逗号解析缺陷 / 裸词 src 静态 import / 桌面专属 widget 误入 web 画廊 | ZCode（扫描） | ✅ 已修复并验证（本文档 W6） |
| W7 | togglegroup 页纵向裸排 B/I/U,应为 shadcn 横向三连按钮(outline/aria-label/单选多选 UX) | 用户（人工检查+截图对照 shadcn 原型） | ✅ 已修复并验证（本文档 W7） |
| W8 | 三连按钮未合体（旧脚手架 gap-1 分离观感）;需补单选示例 | 用户（人工检查+截图） | ✅ 已修复并验证（本文档 W8;示例层 class 注入连体） |
| W9 | VM 版 popover：点击按钮有弹框但无边框无背景 | 用户（人工检查） | ✅ 已修复并验证（本文档 W9;默认面板 chrome w-72 bg-popover border rounded-md shadow-md p-4） |
| W10 | VM 版窗口收窄触发 mobile 断点后内容成双份（文字/按钮/代码块全部 ×2 重叠），底部导航栏出现 | 用户（人工检查+截图） | 🔍 已立项 **PLAN-530** 深挖（根因定位见 W10 节） |

### 观察（OBS，未定是否立项）

| # | 现象 | 影响 | 状态 |
|---|------|------|------|
| OBS-1 | `auto run`（VM 模式）原生窗口崩溃：`memory allocation of 721554505560 bytes failed`，无 backtrace | VM 版 widgets-gallery 无法启动检查 | open 已立项 **PLAN-530**（问题 B） |
| OBS-2 | `auto run -r vue` 外层包装进程曾静默退出（exit 1，无报错日志），vite 子进程存活 | 干扰长会话验证流程 | open |
| OBS-3 | AutoUI MCP 默认端口 9247 与本机 musk.exe 服务冲突，需 `AUTOUI_MCP_PORT` 绕开 | 启动摩擦，非缺陷 | open（环境事实） |
| OBS-4 | 浅色主题下，页面内硬编码深色样式区域（如 codeblock 代码体）不随主题翻转（token 驱动区域正常） | 浅色观感割裂，待立项逐页适配 | open |
| OBS-5 | 主题状态为 App 内存态：SPA 内路由切换保持；深链直载/刷新回默认深色 | 可考虑 localStorage 持久化 | open |
| OBS-6 | master 存量测试失败：`plan370_015_behavior_tests::d8_toggle_dark_mode`（initial dark_mode 应 false 实 true；git stash 验证与 PLAN-528 改动无关） | 影响全量测试门禁 | open |
| OBS-7 | `.auto/ui-cache.json` 按源码内容 hash 命中，mtime touch 不触发生成；改 codegen 后须删缓存重建 | 开发摩擦（codegen 迭代易误判） | open |
| OBS-8 | model 声明的逗号分隔式（`model { a str = "1", b str = "2" }`）解析缺陷：空名变量 + 名字错位 | parser 增强候补（家规为换行式） | open |
| OBS-9 | master 存量测试失败共 4 个（git stash 验证与本计划无关）：`d8_toggle_dark_mode`、`plan055_strip_html_tests::strips_tags_and_decodes_entities`、`schema_drift_fence`、`gallery_vue_golden` | 影响全量测试门禁；golden 需重采样评估 | open |
| OBS-10 | 战略方向（用户 2026-09-03）：现阶段的 shadcn 对齐是过渡态——组件库成熟后可能与 shadcn 分道扬镳：①跨平台需求催生更多自有组件；②web 端实现或脱离 shadcn 基底改为自研。届时命名体系（含 W3 的 alert/alert-dialog 互换、confirm/callout 别名）整体重议 | 远期 L2 级架构议题，需独立 design doc | open（远期） |
| OBS-11 | bundled shadcn-ui 快照为旧版:toggle-group 无新版连体样式(其余组件亦可能滞后于上游 shadcn-vue) | 快照升级属独立工程,影响面全工程 | open（远期,关联 OBS-10） |

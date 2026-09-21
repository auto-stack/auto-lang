---
plan_id: PLAN-676
status: execution_done        # drafting → executing → execution_done → reviewed → archived
feature_name: gallery-shell-bp
author: [agent]
created_at: 2026-09-21
updated_at: 2026-09-21

# /auto-plan:review 结束时填写：
supersedes_spec_components: [blueprint/project#gallery-discovery-PLAN-640]
new_spec_components: [blueprint/layout-gallery-shell]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/specs/blueprint/contract.md, docs/specs/blueprint/project.md]
current_step: 7
total_steps: 7
---

# [PLAN-676] gallery-shell-bp

## 0. 变更摘要

综合三个成熟画廊的框架代码（auto-os `widgets-gallery` 727 行 routes 文档站、
auto-os `ui-gallery` 356 行 demo 浏览器、本仓 `charts-gallery` 123 行内容卡栈），
提炼一套 **vue/vm 双臂通用的 `layout/gallery-shell` BP**（新包，18/18），然后
用它重写 `examples/bps-gallery`——手写 Vue 壳退役，bps-gallery 升级为 Auto 化
双臂画廊（`auto run` / `auto run -r vm`）。

## 1. 目标

1. **新 BP 包 `blueprints/layout/gallery-shell/`**：header（brand slot + 搜索 +
   设置弹层）+ 侧栏（筛选 pills + scroll 条目列表 + active 态）+ content slot
   的画廊骨架；主题三件套（dark_mode / accent_color / SettingsPanel）按
   PLAN-528 W2 组件契约收编进 reference。骨架家族准入（PLAN-075 和解注记
   先例，`layout/sandwich` 同款：骨架+slot 为契约本体）。
2. **catalog 数据通道**：新增 `auto bp list --format at` 发射器，把
   `blueprints/**` 目录扫描结果物化为 `.at` 静态表（PLAN-625 registry.at
   先例：生成产物、双端同源）。**显式取代** PLAN-640 的 import.meta.glob
   零产物裁定（理由：glob 只服务 vue 臂；Auto 化后 VM 臂无法读盘，双臂
   同源要求静态表）。
3. **bps-gallery Auto 化**：改造为 auto 项目（pac.at + src/front/app.at，
   charts-gallery 同款形态），壳消费 gallery-shell BP（L1 通道），内容页
   （spec 正文 / 变体切换 / 参考源码 / gotchas）迁入 `.at`；手写
   src/App.vue + vue-router + Prism 依赖退役。

**非目标**：
- 参考实现 live render（a2vue 单件编译进预览）仍 deferred（bps-gallery 既列 scope note）；
- 不改动 widgets-gallery / ui-gallery（它们是参考源；自身迁移吃 BP 另立计划）；
- 不触碰 PLAN-675 routes-in-embed 线（routed demo 内嵌，互补不重叠）;
- 主题持久化（P672 T-16 THEME_PREFS OS 层机制）不在本计划——BP 只收 UI 形态，
  主题态按契约 #3 归宿主单例。

## 2. 架构方案

```
blueprints/layout/gallery-shell/
├── spec.md                  # 契约：props/actions/dataSource/palette/acceptance
├── reference/default.at     # 骨架实现（双臂词汇面）
└── gotchas.md               # 引擎教训锚（aside h-full / sidebar 族契约 / popover VM 位）

examples/bps-gallery/
├── pac.at                   # auto 项目声明（dep bps + registry 生成产物路径）
├── src/front/
│   ├── app.at               # 壳=use bps.layout.gallery_shell.reference.default + slot 填充
│   └── registry.at          # 生成产物：auto bp list --format at > 此文件（提交+CI 漂移校验）
└── src/ → 退役（App.vue / bps.ts / router.ts / Prism 依赖）
```

**三画廊 → BP 映射**（综合来源）：

| BP 构件 | 吸收自 | 形态 |
| --- | --- | --- |
| header 骨架（sticky/z/边框） | widgets-gallery :120-148 | `header` + brand slot + 右侧动作区 |
| 搜索框 | ui-gallery :58-61 | `input (value: .search_query)`，query 经 action 上抛 |
| 设置弹层 | widgets-gallery PLAN-528 `settings.at` 组件契约 | dark_mode/accent_color props + on_toggle_dark/on_set_* msgs；触发通道 T-01 勘定（popover=ui-gallery 先例 vs absolute+Overlay=PLAN-536 T6 VM 先例） |
| 侧栏骨架 | widgets-gallery :178-197 | `aside w-72` + `sidebar_provider/menu` 族（P573 裁定：aside 只留布局外壳） |
| 筛选 pills | ui-gallery :136-160 | `sidebar_header` 内按钮行，style-if active |
| 条目列表 | ui-gallery :163-196 | scroll + `sidebar_menu_button` 双行卡（主行 id+badge、副行 title），active 绑定 |
| content | 纯 slot | 消费方形态（ui-gallery 视口卡+tabs / bps-gallery spec+源码+gotchas 各自填充） |
| 主题三件套契约 | Plan 458 约定 | dark_mode 声明即 codegen 翻转 html.dark / VM iced 种子；accent 五色板 indigo/coral/ocean/sage/amber |
| 视口守卫 | ui-gallery :128-132 教训 + 663 契约 | aside `h-full` 必写；外层 `h-screen overflow-hidden`；bp-gate 单元先报警 |

**导航形态裁定（D1，采纳 ui-gallery 先例）**：条目选择走 `active_id` 状态 +
`select(item)` action，**不进 routes{}**——VM 臂状态切换是已验证形态（ui-gallery
28 demo 同款），routes 文档站形态（widgets-gallery）留后续 promotions。

**契约六问自答**（摘要，全文见 spec.md）：
1. 输入 props：`items []GalleryItem`（id/title/category/badge）、`active_id str`、
   `dark_mode bool`、`accent_color str`；
2. 输出 actions：`select(item)` / `search(query)` / `set_theme(mode)` /
   `set_accent(name)`；
3. 状态归属：search_query/settings_open 为 bp scoped；**主题态归宿主 model**
   （契约 #3 平台服务禁 bp 私有副本，bp 经 props+actions 收发）；
4. 变体：`default` 单变体起步，结构性差异走 promotions；
5. 打包：kind=layout（词表既有），磁盘 kebab 名 `gallery-shell`；
6. 双形态：词汇面全部为双臂已验证 widget（sidebar 族 P573 / popover P672
   维护中实证 / scroll P656），验收走 bp-gate 同款门禁。

## 3. 技术栈

Auto (.at) / 现有 BP 体系（BlueprintRegistry, L1 通道, bp-gate）/
`auto bp` CLI（cmd_bp.rs 扩展发射器）/ CI：build-bps-gallery.yml（改造）。
无新外部依赖（Prism 随退役删除）。

## 4. 需求分析与背景调查

**授权记录**（2026-09-21 用户裁定）：参考 widgets-gallery / ui-gallery /
charts-gallery 三画廊框架综合成 vue/vm 通用 Auto 代码，作为 gallery-bp 代码
基础；做好后 bps-gallery 用它实现（替代手写框架），使 bps-gallery 成为
Auto 化双臂画廊。本计划即该裁定的执行契约。

**实勘证据**：
- `D:/autostack/auto-os/widgets-gallery/src/front/app.at`（727 行）+ `components/settings.at`（PLAN-528 W2）：header/侧栏/SettingsPanel 组件契约、routes 形态、Plan 458 dark_mode 约定、PLAN-536 T6 VM Overlay 悬浮层先例。
- `D:/autostack/auto-os/ui-gallery/src/front/app.at`（356 行）：selected_id 状态切换、分类 pills、双行卡列表、popover 设置弹层、registry.at 数据源（PLAN-625 生成产物双端同源）、aside h-full 教训注记。
- `examples/charts-gallery/src/front/app.at`（123 行）：内容卡模式（标题+描述+card demo）。
- `examples/bps-gallery/`：import.meta.glob 目录发现（PLAN-640）、BpPage.vue 渲染面=spec 纯文本 pre + Prism 源码高亮 + gotchas、kindOrder 偏好序（bps.ts）。
- `crates/auto-lang/src/ui_gen/bp/registry.rs`：17 包扫描、palette_drift 合法集（AURA registry ∪ package_origin）、with_defaults 三级根解析（PLAN-645）。
- `crates/auto/src/cmd_bp.rs`：List/Show/Add/Check 四子命令（发射器挂 List）。
- `docs/specs/blueprint/contract.md`：六问契约、Q5#5 kind 词表（layout 既有）、palette 合法集、L1/L2/L3 通道、PLAN-075 和解注记（骨架 bp 例外）。
- `docs/specs/blueprint/`无 bps-gallery 发现机制专节——PLAN-640 裁定散落
  contract.md 版本面规则⑤（CI paths 触发）与 bps.ts 注释，SD-01 收拢改写。

**已知坑（执行纪律）**：
- `auto build` 物化 dep junction（bp-gate R-B / P661 先例）：worktree 内构建
  bps-gallery 前允许 junction 落地，**合并前** `cmd /c rmdir /s /q` 摘净再过
  wt-guard；
- prismjs 1.30.0 全域阻断 vue dev（钉 1.29.0）：本计划直接退役 Prism，不新增；
- registry.at 体积：17 包 spec+gotchas+reference 全量内嵌约数百 KB 字符串——
  ui-gallery registry.at 与 lib 470KB 形态先例在案，T-02 实测确认 VM 臂可承受。

## 5. 详细设计

### 5.1 gallery-shell BP 契约面（T-01 产出）

spec.md frontmatter（草案，T-01 落盘时对照 WidgetRegistry 校 palette）：

```
kind = "layout"
name = "gallery-shell"
palette = ["header","aside","sidebar_provider","sidebar_header","sidebar_menu",
           "sidebar_menu_item","sidebar_menu_button","scroll","button","text",
           "input","icon","col","row","separator","badge", <popover 三件或 fallback 形态>]
extension_points = ["brand","header_actions","filters","content"]
variants = ["default"]
props = ["items","active_id","dark_mode","accent_color"]
actions = ["select","search","set_theme","set_accent"]

[dataSource]
# 无远程 fetcher——列表数据经 props 注入（画廊目录在编译期已知）
```

acceptance（草案）：
1. 骨架无应用侧文案：brand/条目/内容全经 slot 与 props 注入；
2. 条目点击上抛 `select(item.id)`，active 态随 `active_id` 渲染；
3. 搜索输入防抖上抛 `search(query)`（VM 臂即按键上抛，语义一致）；
4. 设置弹层切换 dark_mode / accent_color，经 action 通知宿主，bp 不持主题态；
5. `compact` 未声明；侧栏宽度 w-72 钉骨架内（sandwich gotchas#4 同款裁定）。

### 5.2 `auto bp list --format at`（T-02 产出）

cmd_bp.rs List 路径追加 `--format at`：输出

```
// GENERATED by `auto bp list --format at` — do not edit.
widget BpCatalog { ... }   // 或 const 表形态，按 .at 语法勘定
// entries: [{kind,name,spec,gotchas,references:{variant:source}}...]
```

形态二选一在 T-02 现场裁定（const 字符串表 vs widget 静态字段），判据：
解析器支持面 + 双臂读取一致性。同步更新 bps.ts 注释所述"authoritative
registry view"语义（SD-01）。

### 5.3 bps-gallery Auto 化（T-03/T-04 产出）

- pac.at：`scene: "ui"` / `render: "vue"` / `dep bps`（解析链
  base_dir→examples→repo root 命中 `blueprints/`，Q5#5 既有链）；
- app.at：`use bps.layout.gallery_shell.reference.default: GalleryShell`
  （L1 通道），slot 填充——brand=标题、filters=kind 偏好序 pills
  （bps.ts kindOrder 迁入 .at 静态表）、content=现 BpPage 三区
  （spec 正文 pre / 变体切换按钮行 / 参考源码 pre / gotchas pre）；
- 变体切换交互沿用 ui-gallery 手搓 tab 形态（button + style-if），零新词汇；
- 纯文本 pre 保双臂一致；Prism 高亮不迁移（Vue 臂增强留 promotions）。

### 5.4 验证位（T-05/T-06）

- bp-gate 新单元 `gallery_shell`（units.mjs 登记，vue needles/clip + vm state/snapshot）；
- bps-gallery 双臂实测走 autoui-verifier 双端模式；
- CI build-bps-gallery.yml：build 通道改 `auto build`；增 registry 漂移校验步
  （重跑 `auto bp list --format at` 与提交产物 diff，非空即红）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
| --- | --- | --- | --- | --- | --- |
| SD-01 | modify | docs/specs/blueprint/contract.md（Q5#5 邻接，发现机制注记） | before: bps-gallery 目录发现=Vite import.meta.glob 零产物（PLAN-640）；after: bps-gallery 目录发现=`auto bp list --format at` 生成产物 registry.at（提交+CI 漂移校验），PLAN-640 裁定就此退役并注记原因 | Auto 化后 VM 臂无法 glob 磁盘，双臂同源要求静态表（PLAN-625 先例） | AC-05 |
| SD-02 | add | docs/specs/blueprint/project.md（包目录清单） | 新增 `layout/gallery-shell` 条目（18/18），骨架家族 + Q5 六问自答摘录 | BP 入册惯例 | AC-01 |

## 6. 测试设计

| 层 | 测试 | 归属 |
| --- | --- | --- |
| registry | `scan_tests` 增 `layout/gallery-shell` key + palette 零漂移断言（既有 `palette_has_no_drift` 自动覆盖） | T-01 |
| CLI | `--format at` 发射 golden 测试（17 包输入 → 稳定输出，含排序稳定性） | T-02 |
| bp-gate | `gallery_shell` 单元双臂绿（vue 截图基线 + vm snapshot 断言） | T-05 |
| e2e | bps-gallery `auto run` / `auto run -r vm` 双端一致性（autoui-verifier） | T-05 |
| CI | build-bps-gallery.yml 改造后绿 + 漂移校验步空转绿 | T-06 |

门禁分级：T-01/T-02 属 Rust 面（Category B：`cargo check -p auto-lang` +
`cargo t bp`（registry/plan64x 系）+ cmd_bp 归属 `cargo t` 滤串）；T-03 以后
纯示例资产（Category A：严禁 cargo t/docs_gen，验证=auto build + 双臂实测）。

## 7. 验收标准

- **AC-01** `auto bp list` 含 `layout/gallery-shell`；palette 漂移为零；
  registry 扫描测试含新 key。验证：`cargo t bp`（registry 系）绿。
- **AC-02** gallery-shell 双形态同语义：bp-gate `gallery_shell` 单元
  `pnpm gate` 双臂绿。验证：`cd examples/bp-gate && pnpm gate`。
- **AC-03** bps-gallery Auto 化完成：`auto build` 零 error；页面功能面
  （kind 分组目录 / spec 正文 / 变体切换 / 参考源码 / gotchas）在生成产物
  中可走查。验证：`auto build` + 生成物走查截图。
- **AC-04** 双臂 parity：`auto run` 与 `auto run -r vm` 渲染一致，
  autoui-verifier 双端证据入档。验证：技能脚本产出。
- **AC-05** registry 通道：`auto bp list --format at` 重跑与提交的
  registry.at 零 diff；blueprints/** 变更经 CI 漂移校验步拦截。
  验证：发射器 golden 测试 + CI 步。
- **AC-06** CI 绿：build-bps-gallery.yml 适配后跑绿（含 Rust build 面）。
  验证：workflow 运行记录。
- **AC-07** 手写壳退役：examples/bps-gallery/src 下 App.vue/bps.ts/
  router.ts/BpPage.vue 与 Prism 依赖删除，仓内零 import.meta.glob 残留
  （grep 证据）。

## 8. 执行步骤

- [x] **T-01** 新建 `blueprints/layout/gallery-shell/{spec.md, reference/default.at, gotchas.md}`：按 §5.1 落契约与骨架实现；popover vs Overlay 通道在此任务实勘裁定（ui-gallery popover 双臂现状 → 定）；registry 测试名单更新。依赖：无。→ AC-01。验证：`cargo check -p auto-lang && cargo t bp`（registry 系滤串）。
  [✅ 已完成] 2026-09-21 worktree lang-676。D2 裁定=非受控 popover（Plan 422
  anchored 双臂在册 + ui-gallery donor 形态：trigger 原生管理开合，不绑
  onclick/open；Overlay fallback 不触发，gotchas#3 落档）。props 较 §5.1 草案
  增 empty_text（row-list 空态先例，显式 UI 态）。palette 19 项逐一核
  WidgetRegistry alias 键（sidebar 族 snake/popover 族 kebab）零漂移。
  证据：cargo check 绿（177 警告全存量）+ `cargo t bp` 47/47
  （palette_has_no_drift 覆盖新包 + scans_default_packages 含
  layout/gallery-shell）；worktree 提交 T-01。注：worktree 组内补建
  auto-down 兄弟位（detached @fba6563，crates/auto-lang Cargo.toml:132
  path 依赖解析序组内命中）。
- [x] **T-02** cmd_bp.rs List 增 `--format at` 发射器 + golden 测试；输出形态（const 表 vs widget 字段）现场勘定。依赖：T-01。→ AC-05（发射器半边）。验证：`cargo t bp_list`（或滤串）+ 手跑 `auto bp list --format at`。
  [✅ 已完成] 2026-09-21 worktree lang-676 cda85a826。输出形态裁定=PLAN-625
  registry.at 先例函数表（`pub fn all_bps() List`），非 const 表——解析器
  双臂已验证面。CLI 落 `--format at` 全局旗（OutputFormat 增 At 变体）：
  子命令字段与 global value_parser 同名 downcast 冲突，复用全局旗为实测
  最稳解（语义收敛：bp list 只消费 At）。字节稳定双保=变体排序+读盘 LF
  规范化（防 Windows/CI EOL 假漂移）。证据：golden 2/2
  （bp_list_at_golden_stable：18 包逐行/序稳定/双跑相等）+ auto 包 bin
  12/12 + 手跑 188KB 实产物（ui-gallery 470KB 先例内）；文本模式零回归。
  注：`cargo t` 档钉 `-p auto-lang`，cmd_bp 测试走
  `cargo nextest run -p auto --bin auto`。
- [x] **T-03** bps-gallery 项目化：pac.at（dep bps）+ registry.at 生成产物落地 + app.at 壳消费 GalleryShell（slot 填充）+ `auto build` 双臂通过。依赖：T-01/T-02。→ AC-03（壳半边）。验证：`auto build`。
  [✅ 已完成] 2026-09-21 worktree lang-676 8a03a5c95。`auto build` 零 error
  （vue-tsc+vite exit=0）。执行期炸出三枚 vue 臂转译边界并当场修复：
  ①**spec 是 .at 保留字**——记录键名 `spec` 解析必炸（样张二分定位），
  发射器字段改 `spec_text`（ui-gallery pac→pac_text 同型陷阱，T-02 产物
  增量修正随本提交）；②**跨模块 fn 内联只走一跳**（app→catalog→registry
  链丢 all_bps）→ helpers 收 bps List 参数、app.at 一跳 use 直调；
  ③SFC 对与响应式同名的形参/局部盲目加 `.value`（TS2551）+ int.to_str/
  view 内 computed 列表 .len() 无转译 → 改名避让+度量收进 fn 体。
  以上三条已写入 catalog.at 头注（装配纪律）。S003 WARNING
  （deps/bps/**/reference/components 扫描噪音）非阻断在案。
- [x] **T-04** 内容页迁移：spec 正文/变体切换/参考源码/gotchas 三区入 app.at（数据源=registry.at），kindOrder 偏好序迁入；信息架构与现版对齐走查。依赖：T-03。→ AC-03/AC-07（功能面）。验证：双臂走查。
  [✅ 已完成] 2026-09-21 worktree lang-676（代码随 T-03 提交 8a03a5c95，
  走查证据随 T-05）。三区+kindOrder 全部在 app.at/catalog.at 落地；走查
  核对与现版 IA 对齐（侧栏 kind 分组/条目→spec 正文 frontmatter 剥壳/
  变体 tab/参考源码/gotchas），vue 臂交互链实证：条目切换→三变体 tab
  渲染→two_column 源码切换（文本量变+特征串）→kind pill 过滤（form/
  signup 消失）→搜索 "sandwich" 过滤（form/login 消失）→设置弹层
  Light/Dark 翻转落 html.dark→coral 色点 ring 态。证据截图
  docs/reports/p676/vue-walkthrough-login.png。
- [x] **T-05** 双臂验证：bp-gate `gallery_shell` 单元（units.mjs + 基线）+ bps-gallery autoui-verifier 双端证据。依赖：T-04。→ AC-02/AC-04。验证：`pnpm gate` + verifier 产出。
  [✅ 已完成] 2026-09-21 worktree lang-676 ae01fed23。bp-gate 第五单元
  gallery_shell 双臂绿：vue 臂 7/7（新基线 gallery-shell.png，既有 4 张
  零扰动；视口 1040→1456，AC-04 贴底断言随最底单元转移改写）+ VM 臂
  state（gs_active/gs_dark/gs_accent）+ snapshot（bp 子树 slot 填充
  needle）全绿。bps-gallery 双端：vue 臂交互链走查（见 T-04）+ VM 臂
  iced 渲染截图 parity（docs/reports/p676/vm-walkthrough.png——brand/
  pills/双行卡/三区/frontmatter 剥壳全同构）。执行期修复三枚（详见
  T-03/T-05 记录与提交 ae01fed23）：slot 直挂 widget 转译误读、循环
  变量括号形态 text 错合并、catalog.at 函数体 query 残留（vue 臂闭包
  捕获侥幸过、VM 臂硬错——双臂语义差的活标本）。
- [x] **T-06** CI 适配：build-bps-gallery.yml 改 `auto build` 通道 + registry 漂移校验步；README 更新（发现机制/跑法/退役说明）。依赖：T-02/T-03。→ AC-05/AC-06。验证：workflow 绿。
  [✅ 已完成] 2026-09-21 worktree lang-676 805cf936c。workflow 两道闸落地
  （漂移校验 diff 步 + auto build 全链通道，paths 扩发射器面）；形态沿
  build-ui-examples.yml 既有先例（bare cargo build -p auto）。AC-06 的
  workflow 运行记录须待 merge 后 GitHub Actions 首跑确证（worktree 内
  无法触发）——复审/合并时盯一次。README 终态重写（双臂跑法/SD-01/
  再生命令）。
- [x] **T-07** 手写壳退役：删 src/App.vue、src/bps.ts、src/router.ts、src/pages/、Prism 依赖与 vite 手写工程面；`grep -r "import.meta.glob" examples/bps-gallery` 零命中。依赖：T-05 全绿后。→ AC-07。验证：grep + `auto build` 复跑。
  [✅ 已完成] 2026-09-21 worktree lang-676 805cf936c。删 14 文件
  （App.vue/bps.ts/router.ts/main.ts/app.css/pages/{BpPage,Home}.vue +
  index.html/vite.config.ts/tsconfig.json/package.json/pnpm-lock/
  pnpm-workspace.yaml/shims.d.ts）；`auto build` 复跑零 error；
  import.meta.glob 调用形态仓内零命中（残余=README/pac.at 退役裁定
  注记文本，属 retirement 记录非功能残留）。
- （每步完成后追加 `[✅ 已完成]` 证据行；worktree=`D:/autostack/.wt/lang-676/auto-lang`，合并前摘 junction 过 wt-guard）

## 9. 复审记录

- 2026-09-21 draft/rev1 handoff：stage=new, PLAN-676 rev1, outcome=pass（三项
  默认裁定待用户确认，见 §10；确认后 next=work）。
- 2026-09-21 work 接令：用户指示"实施它"，§10.4 三项默认裁定（①selected_id
  状态切换非 routes ②layout/gallery-shell 单变体 default ③SD-01 取代
  PLAN-640 glob）随批确认通过，status drafting→executing，开建 worktree
  lang-676。
- 2026-09-21 work 收口：stage=work, PLAN-676 rev1, outcome=pass,
  code_commit=8b7ca6dda（worktree plan-676-dev，base 4469076e9），task_ids=
  T-01..T-07 全闭环（current_step 7/7）。evidence：
  - AC-01 `cargo t bp` 47/47（scans_default_packages 含 layout/gallery-shell
    + palette_has_no_drift 零漂移）+ auto 包 bin 12/12（发射器 golden）；
  - AC-02 bp-gate 第五单元双臂绿（gate.mjs 全量复跑 vue ✓ vm ✓，新基线
    gallery-shell.png，既有 4 张零扰动）；
  - AC-03 `auto build` 零 error（T-07 退役后复跑再证）+ 功能面走查
    （docs/reports/p676/vue-walkthrough-login.png）；
  - AC-04 双端 parity 证据（vue 交互链走查 + VM iced 截图
    docs/reports/p676/vm-walkthrough.png）；
  - AC-05 发射器 golden（bp_list_at_golden_stable/escapes_strings）+
    漂移校验实证（T-05 reference 修复后 diff 非空→再生成 8b7ca6dda→零
    diff 复验 ✓——闸门真工作）；
  - AC-06 workflow 改造就绪；**GitHub Actions 首跑绿=merge 后待确证项**
    （worktree 内无法触发，非阻塞）；
  - AC-07 手写壳 14 文件退役 + import.meta.glob 调用形态零命中。
  blockers=无。next=review（/auto-plan:review）。
  merge 纪律注记：worktree 内 examples/bps-gallery/{deps,gen} 有构建
  产物，合并前 `cmd /c rmdir /s /q` 摘 deps/bps junction 再过 wt-guard
  （bp-gate R-B/P661 先例）。

## 10. 待澄清事项

1. **D2 触发通道**——已解（T-01 实勘）：非受控 popover（Plan 422 anchored
   双臂在册 + ui-gallery donor 形态，trigger 原生管理开合不绑
   onclick/open）；Overlay fallback 未触发（P672 维护实证）。已落
   gotchas#3。
2. **D5 消费通道细节**——已解（T-03）：`dep bps{path:"../../blueprints"}`
   + `auto build` 托管 junction 物化；worktree 纪律=合并前摘净（见 §9
   merge 注记）。
3. **registry.at 输出形态**——已解（T-02）：PLAN-625 函数表形态
   （`pub fn all_bps() List`），非 const 表；字段避名 spec_text（spec 是
   .at 保留字，pac_text 先例同型）。
4. **三项默认裁定**——已随"实施它"指令确认（§9 work 接令记录）。
5. **AC-06 CI 首跑**（merge 后非阻塞项）：build-bps-gallery.yml 改造后
   首跑绿须在 merge 后于 GitHub Actions 确证；若 `cargo build -p auto`
   在 lone checkout 缺 auto-down 兄弟位，则是与 build-ui-examples.yml
   共同的既有 CI 面问题，另立基建处理不属本计划。

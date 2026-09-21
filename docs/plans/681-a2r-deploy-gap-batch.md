---
plan_id: PLAN-681
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: a2r-deploy-gap-batch
author: [zcode]
created_at: 2026-09-22
updated_at: 2026-09-22

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 shell-a2r-seams S1 生成域扩容（handler 体语句/内建函数转译面 + 伴生模块/route A 契约）, SD-02 trans 内建映射面扩容契约]
touched_goals: []

affects: [docs/specs/auto-lang/ui/design/shell-a2r-seams.md, docs/specs/auto-lang/trans/overview.md]
current_step: 0
total_steps: 7
---

# [PLAN-681] a2r 部署面缺口清偿批：F-R1-B 四缺口 + P674-D3 前面子族（消费方 `auto build -r rust` 全编译过）

> 来源：P670-D1（KNOWN-DEBT 在册，勘定件 `docs/reports/p670-fr1b-survey.md`
> §2 四缺口表）+ P674-D3（PLAN-674 复审前置实测，2026-09-22 用户裁定
> 扩 P670-D1 并入清偿面）。目标态一句话：**auto-edit `auto build -r
> rust`（merged 主形态）生成物 cargo check 错误清零，`perf.py a2r`
> exit 0**。

## 0. 变更摘要

a2r 编译轨两部署面的机制清偿：

1. **server 面（F-R1-B 四缺口，~210-350 行）**——①伴生模块通用转译
   （api_gen.rs 只认 `src/back/db.at` 硬编码，fsys.at 不进转译面）；
   ②route A 接线（端点 .at fn 体降体进 handler——`ApiEndpoint.body`
   生产不读）；③trans/rust.rs 内建面（`Env.get` 大写零覆盖、`fs.tree`
   无映射）；④a2r-std 新宿主 `fs.tree` 且 JSON 形状与 VM 内建逐字节
   对齐。
2. **front 面（P674-D3 实测 ~135 错五子族，词汇门 23 错已由 PLAN-674
   清偿后浮出）**——①内建函数面：`console_log` ×23 / `console_lines`
   ×25（VM 内建，vm/native.rs shim 在档——a2r handler 体转译零映射）
   + `file_basename` ×6 + `code_editor_select_all/cut/copy` ×6 +
   `Process`/`json` 对象面；②生成器型面/作用域：循环变量 `i`/`id`/
   `x`/`y` ×8、`list` 类型 ×3、`TreeIcon` 子件类型 ×2、
   `TabActivate(i)` unit 变体带参（E0618）×3；③E0308 参数形 ×33 /
   E0599 ×8 / E0433 ×2 / E0277·E0061（T-00 逐错归类）。

## 1. 目标

- **G1（server 四缺口）**：伴生模块通用转译 + route A 接线 + trans
  内建面 + a2r-std tree 宿主落地——无 Db 契约（fsys.at 形）端点真体
  降体（空体桩退役）。
- **G2（front 子族全清）**：handler 体内建函数/语句转译面机制化
  （映射表驱动，非逐消费方名补丁）+ 生成器型面/作用域修正——生成物
  cargo check 错误清零。
- **G3（消费方解阻判据）**：`auto build -r rust` exit 0；
  `python tools/perf/perf.py a2r` exit 0（auto-edit 判据，供料 §7 终
  复验态）。
- **G4（机制红线沿 671/674）**：内建面扩容一律按机制（映射表/别名
  表/宿主函数族），消费方名零硬编码；grep 门零命中。

**非目标（Non-goals）**：

- 不动 vue 轨（PLAN-671/680 域）；不动 VM 解释轨语义（解释轨全绿
  现状零变化——a2r 生成物专属面）；不动 RQ/渲染面（PLAN-674 已收口）。
- 消费方 auto-edit 仓**零改动**（669 先例：上游修复后消费方零改动或
  最小改动复验）；本仓 examples 不扩（消费方语料即验证面）。
- `--server rust`（split 形态 HTTP 面）的**运行期**保真（端点真体后
  的 HTTP 语义对拍）不在本批验收——本批验收=生成物编译过 + 端点体
  非空（真体降体在档）；运行期对拍登记后续。

## 2. 架构方案

| 件 | 现状锚点（实读） | 目标形态 |
|---|---|---|
| 伴生模块转译 | auto-man/src/api_gen.rs:657 只认 `src/back/db.at`；fsys.at（`src/back/`伴生，Env/fs/File 薄封装）不进转译面 | `src/back/*.at` 通用扫描 + 逐模块转译发射（db.at 既有行为保持——零回归锚） |
| route A 接线 | `ApiEndpoint.body` 生产不读（api/types.rs 注记 "route B shipped"；仅测试读） | no-types 有体契约 → transpile api.at（剥 `#[api]` 属性行）→ 端点 handler 直体/委派 ~100-150 行 |
| trans 内建面 | trans/rust.rs :5687 小写 `env.get` 有映射、`Env.get` 大写零覆盖；`("fs","tree")` 无映射 | 大写别名 + tree 映射 ~20-40 行（**动 trans/ 本体**——tt 档门） |
| a2r-std 宿主 | a2r-std/src/fs.rs 无 tree | `fs.tree(path,depth)` 宿主 ~30-60 行 + **JSON 形状与 VM 内建逐字节对齐**（front 侧 json.to_value 解析——双轨分叉风险，对拍测试钉） |
| front 内建函数面 | ui_gen/rust.rs `generate_handler_body` 零 console_*/file_basename/code_editor_* 映射（VM 侧 vm/native.rs:1773 shim_console_log 等 in 档） | handler 体语句/函数转译**映射表驱动**（VM 内建名 → a2r 宿主调用）；a2r-std 补宿主函数族（console 缓冲/文件基名/code_editor 原语委派注册表） |
| 生成器型面 | E0425 循环变量闭包作用域 / E0308 参数形 / list·TreeIcon 类型映射 / E0618 unit 带参 | 逐错根修（T-00 归类后逐族定位——作用域穿透/类型映射表/变体带参发射） |

**机制红线**（沿 671 T-06/674 §2 同款）：内建函数面 = 「VM 内建名 →
宿主实现」映射表单源（VM 侧 shim 注册表为语义源对齐参照），禁止逐
消费方调用点打补丁；`console_log` 只是通用机制的消费方投影。

## 3. 技术栈

Rust 三 crate 面：auto-man（api_gen.rs）、auto-lang（ui_gen/rust.rs +
trans/rust.rs）、a2r-std（宿主函数族）；验证链 = 消费方语料再生成 +
rust-workspace cargo check 逐错清零 + `cargo t`/`cargo tt`（动 trans
本体必跑 tt）+ perf.py a2r 终判据。

## 4. 需求分析与背景调查

**授权记录**：
- P670-D1 立项裁定：2026-09-22 用户指令「括P670吧」——P674-D3 五子族
  并入 P670-D1 清偿面（债册双条同步在档）。
- 本计划起草授权：2026-09-22 用户指令「走 /auto-plan:new」（drafting）；
  **执行另行授权**。

**消费方证据**（实测 2026-09-22，auto-edit main@dc99328，worktree
auto.exe @ PLAN-674 reviewed HEAD）：
- `cargo check`（rust-workspace）错误分类：E0425 ×86（console_lines
  ×25 / console_log ×23 / file_basename ×6 / code_editor_* ×6 /
  i·id·x·y ×8 / list ×3 / Process ×3 / TreeIcon ×2 / json ×1 …）+
  E0308 ×33 + E0599 ×8 + E0618 ×3 + E0433 ×2 + E0277·E0061 ×2；
  词汇门 compile_error! = 0（PLAN-674 已清）。
- `console_log` 确证为 VM 内建（vm/native.rs:1773 shim + codegen.rs:552
  intrinsic + parser 在案）——非消费方自定义 fn。
- server 面：auto-edit `src/back/fsys.at` 六端点空体桩现状（P670-D1
  在册）。

**先例与依赖**：PLAN-670（F-R1-A 已交付=E0432 消失；B 半缩面登记）；
PLAN-674（词汇门/menubar 降层臂——前置清偿件，reviewed）；PLAN-669
（按名绑定四装配点——`--server rust` 运行期面的既有底座）；勘定件
`docs/reports/p670-fr1b-survey.md`（四缺口尺寸表 §2 + 三支路同根
注记 §3）。

**specs 现状**：`docs/specs/auto-lang/trans/`（architecture/overview/
design/）在档但内建映射面无成文契约（T-00 勘定落点）；`shell-a2r-seams.md`
S1 生成域（PLAN-674 刚转正词汇门节——本批扩 handler 体面）。

## 5. 详细设计

- **T-00 全错误集勘定（决策档先行，沿 674 T-00 范式）**：消费方
  workspace 逐错归类（E0308 ×33 / E0599 ×8 两大未归类桶逐错定位）+
  console_*/file_basename/code_editor_* 声明位归因（VM 内建 vs .at
  fn）+ 映射表落点裁定（ui_gen 内建面 vs trans 面 vs a2r-std 三层
  分工）；产 §9 决策档。
- **server 四件套**（survey §2 尺寸序）：伴生模块通用化 → route A
  接线 → trans 内建面（Env.get/fs.tree）→ a2r-std tree + JSON 对齐
  对拍。
- **front 内建函数面**：映射表驱动（语义源 = vm/native.rs shim 注册
  表）；a2r-std 宿主族（console 缓冲线程本地/进程级、file_basename、
  code_editor_* 委派 CODE_EDITORS 注册表）。
- **生成器型面**：T-00 归类后逐族根修（作用域/类型映射/变体带参
  发射）；预期 E0308 大桶与内建面缺映射同根（类型推断回落 f64 缺省
  的既有注释在档——ui_gen 多处）。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
| --- | --- | --- | --- | --- | --- |
| SD-01 | modify | docs/specs/auto-lang/ui/design/shell-a2r-seams.md（S1 生成域） | before：S1 = 词汇门（674 转正）+ 裸 popover 等臂 / after：+ handler 体语句/内建函数转译面契约（映射表单源、VM 内建名→宿主调用、grep 门）+ 伴生模块通用转译与 route A 接线契约（server 面） | 生成域新面契约化（防逐名补丁回潮——671/674 同款纪律） | AC-02/04/05 |
| SD-02 | modify | docs/specs/auto-lang/trans/overview.md（内建映射面节；T-00 勘定后定精确落点） | before：trans 内建映射无成文契约（大小写别名/tree 族缺席零记载） / after：映射面契约（别名规则、宿主对齐、双轨分叉防护=JSON 逐字节对拍） | trans 本体面（tt 档门）契约化；a2r-std/VM 双轨形状对齐是分叉风险主位 | AC-01/03/05 |

## 6. 测试设计

- **消费方再生成判据**：`auto build -r rust` → rust-workspace
  `cargo check` 错误数**逐批递减至 0**（每任务后收据入 §9）；终判据
  `perf.py a2r` exit 0。
- **机制单测**：内建映射表逐条（VM 内建名→宿主调用发射）；a2r-std
  tree 与 VM `fs.tree` JSON 逐字节对拍（双轨测试）；route A 端点体
  降体金样（fsys 形契约——无类型有体）。
- **通用性静态门**：修复面 grep 零消费方标识符（同 674 §6）。
- **回归**：`cargo t` + **`cargo tt` 必跑**（动 trans 本体）；db.at
  既有 route B 行为零回归（既有测试面）。

## 7. 验收标准

- **AC-01（server 四缺口）**：伴生模块通用转译 + route A 接线落地；
  fsys 形契约端点真体降体（空体桩退役，生成物中端点体非 TODO）；
  trans 内建面 Env.get/fs.tree 映射在档。
- **AC-02（front 子族全清）**：消费方生成物 `cargo check` 错误清零
  （0 错——含 E0308/E0599 大桶，逐错归因档在 §9）。
- **AC-03（消费方终判据）**：`auto build -r rust` exit 0 且
  `perf.py a2r` exit 0（消费方零改动或最小改动复验）。
- **AC-04（机制红线）**：内建函数面映射表驱动单源落地；grep 门零
  消费方标识符命中。
- **AC-05（双轨对齐）**：a2r-std `fs.tree` 与 VM 内建 JSON 逐字节
  对拍测试绿；db.at route B 既有测试零回退。
- **AC-06（spec+账）**：SD-01/SD-02 落盘；P670-D1/P674-D3 债条清偿
  标记；消费方复跑通知留档。

## 8. 执行步骤

| ID | 任务 | 依赖 | 落点（文件/符号） | 意图 | AC | 验证命令/预期 |
|---|---|---|---|---|---|---|
| T-00 | 全错误集勘定：E0308/E0599 大桶逐错归类 + console_*/file_basename/code_editor_* 声明位归因 + 映射表三层分工（ui_gen/trans/a2r-std）裁定——决策档入 §9 | — | 消费方 workspace + 本计划 §9 | 设计先行 | AC-02/04 | 决策档在档（子族 × 修法 × 落点） |
| T-01 | 伴生模块通用转译（`src/back/*.at` 扫描通用化，db.at 零回归） | T-00 | auto-man/src/api_gen.rs:657 邻域 | server ① | AC-01 | db 既有测试绿 + fsys 进转译面 |
| T-02 | route A 接线：`ApiEndpoint.body` 消费 + no-types 有体契约降体 | T-01 | api_gen.rs + api/types.rs | server ② | AC-01 | fsys 端点体非 TODO 金样 |
| T-03 | trans 内建面：`Env.get` 大写别名 + `fs.tree` 映射（动 trans 本体） | T-00 | auto-lang/src/trans/rust.rs:5687 邻域 | server ③/front 共基 | AC-01 | 映射单测 + `cargo tt` 绿 |
| T-04 | a2r-std 宿主族：`fs.tree` + console_*/file_basename/code_editor_* 宿主函数 | T-00 | a2r-std/src/fs.rs 等 | server ④/front 宿主 | AC-01/02/05 | 宿主单测 + JSON 逐字节对拍绿 |
| T-05 | front 转译面接线 + 生成器型面根修：ui_gen handler 体内建映射表 + 循环变量作用域/list·TreeIcon 类型/E0618 带参/E0308 逐族 | T-00..T-04 | ui_gen/rust.rs generate_handler_body 等 | front 全清 | AC-02/04 | 消费方 cargo check 错误数递减收据 → 0 |
| T-06 | 消费方终验 + 回归 + grep 门：`auto build -r rust` exit 0；perf.py a2r exit 0；cargo t/tt；grep 门 | T-05 | 消费方仓 + 本仓门禁 | 终判据 | AC-03/04/05 | 全绿收据 |
| T-07 | spec 落账 + 债清偿标记 + 交接：SD-01/SD-02；P670-D1/P674-D3 销账；消费方通知 | T-06 | docs/specs/ + KNOWN-DEBT | 收口 | AC-06 | spec 回读 + 账本回读 |

## 9. 复审记录

- 2026-09-22 · stage: new · r1 起草 · 授权=用户指令（「括P670吧」
  裁定 P674-D3 并入 P670-D1 清偿面 + 「走 /auto-plan:new」立项）·
  范围=**立项起草（drafting）；执行另行授权** · outcome: **pass
  （待执行授权）** · next: **work**（授权后自 T-00 起；§10-1 映射
  分工与 §10-2 E0308 大桶根因 T-00 勘定后定）。

## 10. 待澄清事项

1. **映射表三层分工**（T-00 裁定）：front handler 体内建函数转译落
   ui_gen 映射表 vs trans 面（与 server 缺口③同表）vs 全落 a2r-std
   宿主——倾向：转译发射在 ui_gen（handler 体域）+ 宿主实现集中
   a2r-std + 跨层共用名（Env.get/fs.tree）走 trans 面，T-00 依
   `generate_handler_body` 结构实勘定。
2. **E0308 ×33 大桶根因**（T-00 归类）：预期与内建面缺映射同根
   （类型推断回落 f64 缺省——ui_gen 既有注释在档），若含独立型面
   缺口则分流登记。
3. **console 缓冲宿主形态**（T-00 附带）：console_lines 读面在 a2r
   编译进程的承载（线程本地 ring vs 进程级——消费方 console_panel
   挂载形态实勘）。

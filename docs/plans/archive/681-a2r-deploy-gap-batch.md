---
plan_id: PLAN-681
status: archived                  # drafting → executing → execution_done → reviewed → archived
feature_name: a2r-deploy-gap-batch
author: [zcode]
created_at: 2026-09-22
updated_at: 2026-09-22

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 shell-a2r-seams S1 生成域扩容（handler 体语句/内建函数转译面 + 伴生模块/route A 契约）, SD-02 trans 内建映射面扩容契约]
touched_goals: []

affects: [docs/specs/auto-lang/ui/design/shell-a2r-seams.md, docs/specs/auto-lang/trans/overview.md]
current_step: 7
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
| T-00 | 全错误集勘定：E0308/E0599 大桶逐错归类 + console_*/file_basename/code_editor_* 声明位归因 + 映射表三层分工（ui_gen/trans/a2r-std）裁定——决策档入 §9 | — | 消费方 workspace + 本计划 §9 | 设计先行 | AC-02/04 | [✅ 已完成] 2026-09-22 勘定：基=plan-681-dev@1f8828d19（叠 674 reviewed），消费方 auto-edit@dc99328 再生成 cargo check=**134 错**（E0425×86/E0308×33/E0599×8/E0618×3/E0433×2/E0277×1/E0061×1，全在 front main.rs，back 骨架桩编译干净）；六族谱系+三层分工裁定入 §9 决策档（含计划外发现：bps 依赖解析面 F2 + front CRUD mock 桩契约错配 F4） |
| T-01 | 伴生模块通用转译（`src/back/*.at` 扫描通用化，db.at 零回归） | T-00 | auto-man/src/api_gen.rs:657 邻域 | server ① | AC-01 | [✅ 已完成] 2026-09-22 r2：build 路径 back 刷新（R-1 修正）后消费方 back=fsys.rs+api_impl.rs+真体 handler 在档（mtimes 10:46 新鲜）；tf 3722/tv 3869/tt 4091 全绿含 db 面零回归 |
| T-02 | route A 接线：`ApiEndpoint.body` 消费 + no-types 有体契约降体 | T-01 | api_gen.rs + api/types.rs | server ② | AC-01 | [✅ 已完成] 2026-09-22 r2：api.rs 端点体非 TODO=0（Query 提取+crate::api_impl 委派金样在档）；`pub async fn tree(Query(query): Query<TreeQuery>) -> JsonResponse<String> { JsonResponse(crate::api_impl::tree(&query.path, query.depth)) }` |
| T-03 | trans 内建面：`Env.get` 大写别名 + `fs.tree` 映射（动 trans 本体） | T-00 | auto-lang/src/trans/rust.rs:5687 邻域 | server ③/front 共基 | AC-01 | [✅ 已完成] 2026-09-22：Env→env 归一化+两派发表 tree 臂；cargo tt 4091/4091 绿 |
| T-04 | a2r-std 宿主族：`fs.tree` + console_*/file_basename/code_editor_* 宿主函数 | T-00 | a2r-std/src/fs.rs 等 | server ④/front 宿主 | AC-01/02/05 | [✅ 已完成] 2026-09-22：fs_tree_parity 双轨逐字节对拍 2/2 绿+迷你库 re-export+console/code_editor=auto_lang 直连+dialog rfd 宿主 |
| T-05 | front 转译面接线 + 生成器型面根修：ui_gen handler 体内建映射表 + 循环变量作用域/list·TreeIcon 类型/E0618 带参/E0308 逐族 | T-00..T-04 | ui_gen/rust.rs generate_handler_body 等 | front 全清 | AC-02/04 | [✅ 已完成] 2026-09-22：134→72→55→49→**0** 递减收据（F1 映射表/F2 bps 解析/F3 载荷推断/F4 真契约桩/F6/N1/N2 动态记录面 P1-P5/N3 形状表；子代理专段 f12574230） |
| T-06 | 消费方终验 + 回归 + grep 门：`auto build -r rust` exit 0；perf.py a2r exit 0；cargo t/tt；grep 门 | T-05 | 消费方仓 + 本仓门禁 | 终判据 | AC-03/04/05 | [✅ 已完成] 2026-09-22：auto build -r rust exit 0 + perf.py a2r「a2r 绿」EXIT=0 + cargo t 唯一停跑红=musk p053_4（P645-D2 在册预存豁免）+ signature parity 兜带扩容绿 + grep 门代码位零消费方标识符 |
| T-07 | spec 落账 + 债清偿标记 + 交接：SD-01/SD-02；P670-D1/P674-D3 销账；消费方通知 | T-06 | docs/specs/ + KNOWN-DEBT | 收口 | AC-06 | [✅ 已完成] 2026-09-22：SD-01/SD-02 worktree 落盘提交；P670-D1/P674-D3 债册清偿标记 master 在案；消费方复跑通知=merge 后以新 master 重建 auto.exe 复跑 perf.py a2r |

## 9. 复审记录

### T-00 决策档：全错误集勘定（2026-09-22）

**基线复现**：消费方 auto-edit@dc99328 + worktree auto.exe（plan-681-dev@1f8828d19
= plan-674-dev reviewed tip 叠基）再生成 → rust-workspace cargo check = **134 错**
（E0425×86 / E0308×33 / E0599×8 / E0618×3 / E0433×2 / E0277×1 / E0061×1），
与立项基线一致；词汇门 0（674 覆盖生效）。**全部错误在 front
`auto-edit/src/main.rs`；back `auto-edit-back` 骨架空体桩编译干净**。

**六族谱系（子族 × 根因 × 修法 × 落点）**：

| 族 | 错误面 | 根因（实读锚点） | 修法 | 落点 |
|---|---|---|---|---|
| **F1 内建函数面** | E0425×62：console_log ×23/console_lines ×25/console_clear ×1/file_basename ×6/code_editor 8 成员 ×11/dialog_open·save ×2/Process ×3/json ×1 | VM 内建（vm/native.rs shim 族 + native_catalog 词位）在 ui_gen 发射器裸函数名零映射 | **映射表单源**（VM 内建名→宿主调用，per-name 表驱动）挂 ui_gen `ast_expr_to_rust` 调用臂 | ui_gen/rust.rs（发射）+ 宿主见三层分工 |
| **F2 bps 依赖解析面**（计划外） | E0425 toggle_id/flatten_tree ×2 + E0433 TreeIcon ×2 | `use bps.navigation.filetree.tree_util: flatten_tree/toggle_id`（pac.at `dep bps{path:../../../auto-lang/blueprints}`）与裸 widget `TreeIcon`（bps tree_icon.at 70 行）——生成器只收集本地 `src/front/**.at`（rust_ui.rs:2278），bps 依赖面零解析（vue 轨靠 regen 补件、VM 轨 import_aliases 各自为政） | 生成器依赖解析扩域：pac.at dep 声明 → bps 包按名解析 use-fn 源与 widget 源 → 转译内联进生成物（**通用机制，消费方名零硬编码**） | auto-man/rust_ui.rs 收集面 + ui_gen 转译面 |
| **F3 生成器型面/作用域** | E0425 循环变量 i×4/id×2/x·y×1（main.rs:188-224 根视图闭包）+ E0618×3（:250 TabActivate/CloseTab unit 变体带参）+ E0308×2（:250 expected () found View） | for 索引/嵌套闭包参数未绑定（enumerate 缺失）；msg 枚举载荷判定与 on_click 发射点不一致；view 链 if/else 臂型 | 逐点根修：索引绑定/变体载荷契约对齐/臂型收口 | ui_gen/rust.rs 视图发射 |
| **F4 front api 桩契约错配**（route A 前半） | E0308 大桶主体（:660/:661/:696/:699/:703/:748）+ E0599/E0277 Display 级联 ×7 + E0061×1 | `generate_merged_api_client` CRUD 兜底臂（PLAN-648 T-03 明示保留，优先级①db ②viewer ③CRUD）把标量服务端点发成 `fn tree(id: i32)->Option<Value>` API_DATA mock——与真契约（`tree(path str, depth int) str`）错配 | CRUD 兜底前插**伴生模块吸收臂**（merged_db_impl `pub mod db` 先例同款 → `pub mod fsys`）+ 端点真契约桩（参数/返回型取 ApiEndpoint，体=api.at 端点体转译委派 fsys::*） | auto-man/rust_ui.rs + api_gen.rs（route A） |
| **F5 server 四缺口** | （back 编译过但六端点空体桩——AC-01 非编译错） | survey §2 表：伴生模块只认 db.at/route A 未建/Env.get·fs.tree 零映射/a2r-std 无 tree | 照 survey 执行（T-01..T-04） | api_gen.rs + trans/rust.rs + a2r-std |
| **F6 实参形态** | E0308 ×3-4（:729-731 code_editor_cursor String→&str） | 既有映射（code_editor_cursor）实参发射缺借用收缩 | 映射表实参形状统一（&str 位 `.as_str()`/`&`） | ui_gen/rust.rs |

**三层分工裁定（§10-1 落定）**：

1. **ui_gen/rust.rs**（handler 体域，**裸函数名**内建映射表单源）：console 族
   （→`auto_lang::vm::ui_console::{ui_console_push/ui_console_lines(DEFAULT_LINES)/ui_console_clear}`
   ——**§10-3 裁定：宿主=auto_lang 直连**，VM shim 与 a2r 生成物同一实现零分叉，
   code_editor 先例同款；console_log 块表达式 `{push; true}` 保 bool 语义）；
   file_basename（→`a2r_std::fs::basename`，rsplit(['/','\\']) 同式）；
   code_editor 8 成员（→`auto_lang::ui::code_editor::<同名>` 直连，生成物已依赖
   auto_lang，code_editor_text :1678 先例）；dialog_open/save（→新增
   `auto_lang::ui::dialog::{open/save}` 宿主，rfd 同式——VM shim 体上提为宿主
   fn 双消费；attach 父窗锚 a2r 形态 v1 从简，边界登记）；Process.exit
   （→`std::process::exit`，auto.process.exit 语义直连）；json.to_value
   （→`serde_json::from_str::<Value>(x).unwrap_or(Value::Null)`，VM 无效串
   行为实现期核对）。
2. **trans/rust.rs**（obj.method 跨层共用名，server 伴生模块转译面）：`Env.get`
   大写别名（`File`/`fs` delete 别名先例 :5792/:8132）+ `("fs","tree")` 映射
   （两族派发表 :5713/:5914/:5965/:8032-8141 均补）。**tt 档门**。
3. **a2r-std 独立 crate 宿主**：`fs::tree`（JSON 逐字节对齐 VM `fs_tree_walk`
   ：{id,label,children,kind,icon,is_leaf,badge} 嵌套、dirs-first+大小写不敏感
   排序、跳过表 .git/target/build/node_modules/gen/dist/__pycache__+dotfile、
   depth clamp 1-8、空→`[]`）+ `fs::basename`；console/code_editor/dialog 宿主
   =auto_lang 直连（零分叉优先，**不落 a2r-std**）。

**⚠️ qualify_a2r_std 保护**：api_gen 后处理把裸 `a2r_std::` 重限定进
auto_lang::a2r_std **迷你库**（仅 List/May/StringAsStr）——`a2r_std::{env,fs}::`
须沿 `sys` 先例（rust_ui.rs PLAN-668 R-24）保护直连独立 crate（back 模板已含
a2r-std dep :801；front 模板实现期核对）。

**E0308 ×33 大桶归因（§10-2 落定）**：主根=F4 桩契约错配（级联 Display
Option<Value> ×7 + E0061）+ F1 映射缺失类型回落（String.iter :320=flatten_tree
缺席级联）；独立型面残余=F3（闭包绑定 ×8/E0618 ×3/View 臂 ×2）+ F6 实参
形态 ×3——分流在册，无计划外独立型面缺口族。

**工作量注记**：F2（bps 依赖解析）为计划外新增面（原计划把 toggle_id 等 4 名
宽归"内建函数面"桶），预估 +100-200 行；F4 修正了 route A 的形状认知——
route A 不止 back 端点真体，还包括 front merged 桩的真契约化（同一转译源
两发射位）。


- 2026-09-22 · stage: new · r1 起草 · 授权=用户指令（「括P670吧」
  裁定 P674-D3 并入 P670-D1 清偿面 + 「走 /auto-plan:new」立项）·
  范围=**立项起草（drafting）；执行另行授权** · outcome: **pass
  （待执行授权）** · next: **work**（授权后自 T-00 起；§10-1 映射
  分工与 §10-2 E0308 大桶根因 T-00 勘定后定）。
- 2026-09-22 · stage: work · r1 开工 · 授权=用户指令「计划681: auto-plan-work 开工」·
  worktree=`D:/autostack/.wt/lang-681/auto-lang` branch=`plan-681-dev` ·
  **基=plan-674-dev@1f8828d19（叠基裁定**：错误基线"词汇门 23→0 后 ~135 错"依赖
  PLAN-674 reviewed 实现，674 未 merge 前唯一可复现基线；674 先 merge 则本支
  顺延干净**）** · 代码落点确认=三 crate 全在本仓 workspace
  （crates/auto-man、crates/auto-lang、crates/a2r-std）单仓计划 ·
  主检出预检=WIP 三簇（examples/rust-workspace 再生成漂移 015-notes 族 +
  examples/ui 001/003 两行变 + Design 32 拆仓调研未提交文档）——他会话所有，
  本计划不动不并 · outcome: 执行中 · next: T-00 全错误集勘定。
- 2026-09-22 · stage: work · r1 收口 · outcome: **pass → execution_done** · code_commit=worktree plan-681-dev（主体批 + N2/N3 f12574230 + spec 增量 + 兜带扩容；基=plan-674-dev@1f8828d19 叠基） · task_ids=T-00..T-07 全勾 · evidence=`auto build -r rust` exit 0 / perf.py a2r「a2r 绿」EXIT=0 / cargo tt 4091/4091 / cargo t 唯一停跑红=musk p053_4（P645-D2 在册预存豁免）/ fs_tree parity 2/2 / signature parity 绿 / grep 门代码位零消费方标识符（命中仅映射表 VM 内建名位=机制本体） · blockers=无 · next=review（/auto-plan:review；worktree 叠基于 plan-674-dev——674 先 merge 则本支 rebase master 干净，否则携带 674 两 commit 顺延）· 边界登记：dialog 父窗锚 v1 从简（674 降级同族）；--server rust 运行期 HTTP 对拍不在本批（计划非目标）；code_editor View 预制槽载荷参型默认值（事件真值 store 臂自查，坐标默认化=674 I3 同族）；AUTO_LANG_CRATE=PLAN-025 dev 环路口子（merge 后自然解除）
- 2026-09-22 · stage: work · r1 主体批落地 · worktree=lang-681 ·
  探针递减收据：**134 →(F1 映射表+back route A)→ 72 →(E 载荷推断+F2 bps
  解析+F4 merged 真契约桩+宿主直连)→ 55 →(N1 写臂 Dot 基座形态+F6 借用
  收缩+A1 裸 List→Vec<Value>+json! push gate)→ 49**（E0070/E0425/E0433/
  E0618 全清；余=E0308×25/E0609×22/E0599×1/E0282×1——纯 trans 动态记录
  面 N2 残差 + N3 view bool 读 ×5）· 实现批已提交 worktree（T-01..T-05
  主体+fs_tree 对拍 2/2 绿）· N2 五子机制（P1 Value 派生推断/P2 字段索引
  访问/P3 push json! 包裹/P4 typed 收口/P5 形参收口）+N3 形状表已派
  专段子代理执行中 · outcome: 执行中 · next: 子代理收敛 → T-06 终验。

- 2026-09-22 · stage: review · PLAN-681 · r1 · outcome: **needs_fix** · reviewed_commit=90cc1fab2 · base_commit=1f8828d19（叠 674-dev） · findings:
  **R-1（major，AC-01）**：`auto build -r rust` UI 路径只走 build_rust_ui→（full→generate_rust_ui / code→regenerate_code_only）——两者均不再生成 back crate；generate_api（含本批 T-01/T-02 的伴生转译+route A back 半）仅 run 路径 start_api_server 触发。消费方 back 成员停留在 9-21 20:37 桩版（api.rs mtime 铁证；6×TODO 空体桩在档），此前「auto build exit 0」证据实为旧桩可编译——**AC-01 空体桩退役判据在 back 工件上未验证**。修正：build 路径补 back 刷新（build_rust_ui 在 regen 后、cargo build 前调 generate_api——该入口自注 idempotent；resolve_back_api 门控无契约纯前跳过），重验=消费方再生成后 back api.rs mtime 新鲜+fsys.rs/api_impl.rs 在档+端点体非 TODO+workspace cargo check 0+perf.py a2r 复跑。余项（tf/tv 门禁跑批中）无发现。
  next: work 修 R-1 → 复审 r2。

- 2026-09-22 · stage: review · PLAN-681 · r2 · outcome: **pass → reviewed** · reviewed_commit=worktree tip（R-1 修正 commit 后） · base=1f8828d19（叠 674-dev） · spec_inputs=SD-01/SD-02（worktree commit 574e89065——描述现行行为与持久决策，无执行日记残留；frontmatter new_spec_components=SD-01/SD-02，touched_goals 空=既有 spec 文档内新增契约节、非目标级变更） · acceptance_results=AC-01..AC-06 全 pass（R-1 修正后复验：back 工件 mtime 10:46+TODO 0+Query 委派金样；workspace cargo check 0；auto build exit 0；perf.py a2r 绿 EXIT=0；tf 3722/3722+tv 3869/3869+tt 4091/4091；fs_tree parity 2/2；grep 门代码位零消费方标识符） · findings=R-1 已修正闭环（build_rust_ui 补 generate_api 刷新，idempotent+门控） · evidence=消费方 rust-workspace 工件（再生成复现）+本计划 §9 探针递减链+门禁四组数字 · next=merge（/auto-plan:merge；worktree 叠基注记：674 先 merge 则 rebase 干净，否则携带 674 两 commit 顺延——merge 顺序须 674 先）

## 10. 待澄清事项

1. ~~映射表三层分工~~ **已裁定（T-00 §9 决策档）**：ui_gen 裸函数名映射表
   （console/code_editor/dialog/Process/json.to_value/file_basename 发射）+
   trans 跨层共用名（Env.get/fs.tree）+ a2r-std 宿主（fs.tree/basename）；
   console/code_editor/dialog 宿主=auto_lang 直连（零分叉优先）。
2. ~~E0308 ×33 大桶根因~~ **已归因（T-00）**：主根=front CRUD mock 桩契约
   错配（generate_merged_api_client 兜底臂，PLAN-648 T-03 保留契约的错配面）
   + 内建缺失类型回落级联；独立残余=闭包绑定/E0618/实参形态（F3/F6 在册）。
3. ~~console 缓冲宿主形态~~ **已裁定（T-00）**：auto_lang::vm::ui_console 直连
   （VM shim 同一 ring，DEFAULT_LINES=200 newest-first '\n' join，进程级
   Mutex<VecDeque> CAP=500——a2r 生成物进程内同语义）。

## 11. 合并收据（PLAN-681:r1）

- prepared：reviewed 基线=r2 pass@worktree tip（R-1 修正后）；SD-01/SD-02 worktree 落盘（574e89065→rebase a5582a8ed 同补丁）；账本投影=P681-1(designs)/P681-2(reviews)+ui/trans plans.md 双行+INDEX 再生（projection-only descendant，588b2eca9）。
- landed：rebase onto master（674 两 commit 已在 master ancestry=自然跳过；四实现 commit patch-id 逐枚相等 /tmp old/new 比对在案；账本 commit 冲突=EOF 双追加取并集——P674-1/2+P681-1/2 共存回读证实）；master ff-only 至 588b2eca9（零 merge commit；他会话 WIP 三簇原样保留）；master 冒烟 cargo check -p a2r-std -p auto-lang --lib --features ui-iced 零 error。
- ledger_refreshed：.autoos/specs.json（indent=1 无尾换行）P681-1/P681-2 插入+回读；docs/specs/auto-lang/{ui,trans}/plans.md 双行；scripts/spec-index.py 再生 INDEX。
- archived：docs/plans/archive/681-a2r-deploy-gap-batch.md（本文件）status=archived，completion_kind: delivered。
- cleaned：wt-guard clean（reparse 零命中）→ worktree lang-681/auto-lang 移除+分支 plan-681-dev 删除（was 588b2eca9=交付 tip）+依赖位 lang-681/auto-down 移除+组目录删除（GROUP_CLEANED 实测）。
- outcome: pass · 边界/债注记随 §9 r2 记录在档（dialog 父窗锚/HTTP 对拍/预制槽默认参/AUTO_LANG_CRATE dev 口子 merge 后解除）。

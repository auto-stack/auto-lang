---
plan_id: PLAN-701
status: drafting
feature_name: edit-consume-supply-pack-v1（auto-edit 消费供料包六件——save 直写/光标滚动端点/F-RV6 稳定化/time 族/vue strict/跳转列表 shim）
author: [agent]
created_at: 2026-09-24T17:45:00+08:00
updated_at: 2026-09-24T17:45:00+08:00
plan_revision: 1
current_step: 0
total_steps: 9
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/code-editor-session-endpoints.md（SD-01：save 直写+set-cursor+scroll 三端点契约）"
  - "docs/specs/auto-lang/vm/design/time-natives.md（SD-02：time 族 VM 运行时 shim 契约）"
  - "docs/specs/auto-lang/ui/design/shell-recent-integration.md（SD-04：SHAddToRecentDocs shim 契约）"
touched_goals: [GOAL-003]
affects: [crates/auto-lang/src/ui/code_editor/core/mod.rs, crates/auto-lang/src/vm/native.rs, crates/auto-lang/src/vm/native_catalog.rs, crates/auto-lang/src/ui_gen/ts_adapter.rs, crates/auto-lang/src/ui_gen/rust.rs, crates/auto-lang/src/a2r_std]
---

# [PLAN-701] auto-edit 消费供料包六件 v1（形态 A 供料件）

## 0. 变更摘要

auto-edit PLAN-014（M2 尾件+上游端点消费，形态 A 消费件）的**供料
件**——两计划执行前置互绑。六件均为下游（auto-edit）实勘登记的上游
want：**供③ F-RV6 矩阵竞态稳定化**（下游一切判据可信度前置）→**供①
`code_editor_save` rope 直写落盘端点**（013 大文件护栏解除件）→**供②
`code_editor_set_cursor`+editor scroll 读/写端点**（会话恢复光标/滚动
转正件）→**供④ time 族 VM 运行时 shim**（bench 毫秒值 app 内来源）→
**供⑤ vue strict 三缺**（menubar-sub 三元素 schema/helper 族 TS 类型
注解/print 映射改道）→**供⑥ SHAddToRecentDocs shim**（任务栏跳转列表
内核面——PLAN-014 T-01 裁定 (b) 用户确认改道件，2026-09-24）。每件
独立可验、可独立 delivered；消费侧验收（端点形探针/矩阵 T18/光标断
言等）属 PLAN-014 不在本件。契约漂移处理：下游 T-00② 探针复核门，
实际端点形与下游预记漂移时双方向各有界修订回 new。

## 1. 目标

- **G-1 F-RV6 稳定化（供③）**：desktop_mcp 矩阵 app 实例中途死亡
  竞态（下游 1631+ 形态=早崩 exit1 2/5、死亡点逐跑漂移、MCP 端口
  WinError 10061）根因定位+修复；判据=下游矩阵 5 连跑零早崩——下
  游自此可废「早崩重跑条款」固化正式基线。
- **G-2 save 直写端点（供①）**：`code_editor_save(key, path)`——
  rope 直写磁盘（零全文 VM 往返），下游 big 态 save 从「拦截+提示」
  改道直调（50MB 域 1.3–4.2s → <100ms 量级预期）。
- **G-3 光标/滚动端点（供②）**：`code_editor_set_cursor(key, line,
  col)` 写端点（`code_editor_cursor` 读侧已备 core/mod.rs:2064）+
  editor scroll 读/写双端点（per-tab 滚动偏移上报/按偏移应用）——
  下游会话恢复 cline/ccol 前向兼容位转正+scroll 条件形。
- **G-4 time 族接线（供④）**：`auto.time.now_ms/now_sec/now` VM 轨
  运行时 shim（catalog 已登记 native_catalog.rs:1192 但无 shim——裸
  /use 两形态均返 0；a2r-std/src/time.rs 已实装）+`Instant.elapsed()`
  返回约定——下游 bench BENCH 标记升 app 侧毫秒值。
- **G-5 vue strict 三缺清偿（供⑤）**：menubar-sub/sub-trigger/
  sub-content 三元素进 vue schema（现 rust 轨有 rust.rs:3360、vue 轨
  缺=strict gen S002「unknown element」）；helper 函数族 TS 类型注解
  （App.vue TS7006×12 build 红）；print 映射改道 `globalThis.console.
  log`（现硬编码 console.log 与下游 store `console` state 遮蔽
  TS2339×4）——下游 strict gen+pnpm build 双 exit 0。
- **G-6 跳转列表 shim（供⑥）**：Windows native 一调用形 shim（
  `SHAddToRecentDocs(SHARD_PATHW, path)`）进 native 族+catalog+a2r
  映射——下游 recents 落盘挂点直调后打开文件进壳 Recent（任务栏跳
  转列表「最近」）。ICustomDestinationList 自定义任务类=非目标（下
  游 T-01 裁定成文）。

### 非目标

- 消费侧任何改动（PLAN-014 T-02/T-04..T-11：WriteFidelity 改道/会话
  扩展/矩阵 T18/SD 消费侧落账——其仓其计划）。
- 大文件分块解码/1GB 线（013 两段式注记维持）。
- a2r 视图-状态联动缺口（§10 登记，单 iced 下游 L2 open 段另行件）。
- 文件关联注册（HKCR/Classes）——跳转列表 Recent 面不依赖注册表关
  联（PLAN-014 T-01 勘定：pac `opens` 是 auto 桌面壳内部注册）。
- EOL 逐行保真（§11）/单处替换族（§12）其余 want——不在本包。

## 2. 架构方案

| 件 | 落点（2026-09-24 实勘锚） | 形态 | 依据 |
|---|---|---|---|
| 供③ | 竞态面=mcp_server.rs+app 生命周期（死亡点漂移、端口拒连） | T-01 有界勘定（对照法：拆分前代码×同工具链）→T-02 修复+判据 | upstream §5 |
| 供① | core/mod.rs:2104 `code_editor_load_file` 同族新函数 `code_editor_save`；cosmic-text Buffer→磁盘直写 | rope/Buffer 序列化直落盘（bom/eol 包装语义归属=端点侧裸写字节+下游 front 包装保留——T-00 勘定后契约成文） | upstream §10-2/§16 |
| 供② | core/mod.rs:2064 `code_editor_cursor` 读侧姊妹写位；scroll 偏移=iced widget 滚动面 | set_cursor(line,col)（0/1 基随 SyncCursor 对齐勘定成文）+scroll_offset(key)读/scroll_to(key,x,y)写 | upstream §12/§13 |
| 供④ | vm/native.rs shim 族（shim_console_log:1798 先例）+native_catalog.rs:1192 已登记位 | shim_time_now_ms/now_sec/now（Instant elapsed 毫秒）+elapsed 约定 | upstream §9 |
| 供⑤ | ui_gen/rust.rs:3360（rust 轨 menubar-sub 在档）；ui_gen/ts_adapter.rs（vue 轨 schema+helper+print） | 三元素 vue schema+发射；helper 族类型注解；print→globalThis.console.log | upstream §9 附记/§14/§15/§16 解锁③ |
| 供⑥ | vm/native.rs 新 shim+catalog 登记+a2r 映射（687 先例：vm_builtin_host_call 双轨同步） | cfg(windows) `SHAddToRecentDocs(SHARD_PATHW, path)` 一调用；返回 bool | PLAN-014 T-01 裁定+probe_jumplist_report.txt |

**双轨铁律**（本仓惯例）：每端点/native 三面同步——VM shim、catalog
登记、a2r 映射（vm_builtin_host_call/trans File 表），缺一即下游 a2r
轨断（687 E0308 教训在案）。

## 3. 技术栈

rust（code_editor core 无 iced 依赖面+iced widget 滚动面、vm/native.rs
shim 族、ui_gen vue 生成器、a2r_std facade）；cargo test 门禁；下游
desktop_mcp 矩阵=供③判据仪器（跨仓只读执行）。无新依赖（供⑥直调
shell32 既有系统库）。

## 4. 需求分析与背景调查

- **授权记录**：用户 2026-09-24 形态 A 双子计划裁定（「可以合起来做
  一个跨仓库的计划吗？OK，那么用形态A吧」——供料件 auto-lang 立号
  +消费件 auto-edit 014）；2026-09-24 用户确认供⑥ 改道并入（「OK，
  这么改可以。不过等会儿给 auto-lang 仓库要新建的计划时要记得这个
  事情」）。本计划=该授权的供料侧落位。
- **契约真源**：auto-edit `docs/upstream/2026-09-m1-supply.md` §5
  （F-RV6）/§9（time 族+print 遮蔽）/§10-2（save 直写 want）/§12
  （set-cursor want）/§13（scroll 读+写 want+set-cursor 消费注记）/
  §14（helper TS7006+menubar-sub 快照缺）/§15（gen S002+build 16 错
  分解）/§16（save=护栏解除件）；auto-edit `docs/plans/014-m2-tail-
  upstream-consume.md` §5 供料回执表（六件契约预记，r2）+§9 复审记
  录；probe_jumplist_report.txt（供⑥ 五面勘定）。
- **重叠核查**：DEBTS.md 零命中六件关键词；活跃计划 242（a2r gap
  tracker）/699（VM HTTP transport）零覆盖；archive 至 700——无重
  复立项。.next-id=701。
- **代码锚**（实勘 2026-09-24，HEAD 70bcd5246）：上文 §2 表。供②
  读侧 `code_editor_cursor(key) -> Option<(usize,usize,usize)>` 在
  档=写侧语义对齐锚；供① 唯一 save 路径现状=下游 front 读出+write_
  text（端点为新路径）；供④ a2r 轨已实装（crates/a2r-std/src/time.rs）
  =VM 轨对齐锚。
- **风险登记**：F-RV6 根因未知（T-01 有界勘定件；不达修复判据则记
  录性降级+漂移登记，不硬凑）；直写端点字节保真语义（bom/eol 归属
  ——T-03 勘定成文，防下游双重包装）；scroll 回声行为漂移（§14 观
  察件：程序化 scroll onscroll 回声随构建漂移——供②写端点语义需与
  该行为裁定合流）；并行会话施工（本仓主检出 blueprints WIP 在场——
  执行期 worktree 纪律照旧+钉版核验）。
- **下游判据仪器**：供③ 验收用 auto-edit 仓 desktop_mcp 矩阵 5 连
  跑（跨仓只读执行；worktree 消费面=其组内依赖惯例，执行期按
  auto-plan-work 勘定记录接线）。

## 5. 详细设计

### 供料回执（供→消接口契约面）

| 供件 | 端点/件 | 契约（T-00 形探针=下游门；本表为设计预记） | 下游消费位 |
|---|---|---|---|
| 供③ | 矩阵稳定化 | 5 连跑零早崩（004 §5 判据形）；修复合入后通知下游重定基线 | 下游 T-09 口径评估 |
| 供① | `code_editor_save(key, path) -> bool`（返回形预记，勘定可调） | rope/Buffer→磁盘直写；零全文 VM 往返；bom/eol 包装归属=T-03 勘定（预记：端点侧裸写字节——front 包装逻辑保留） | 下游 T-05 WriteFidelity big 分支改道 |
| 供② | `code_editor_set_cursor(key, line, col) -> bool` + `code_editor_scroll_offset(key) -> (x,y)`/`code_editor_scroll_to(key, x, y) -> bool` | 基面（0/1）随 SyncCursor 对齐勘定成文；scroll 语义与 onscroll 回声裁定合流 | 下游 T-06 恢复链应用+会话 scroll 条件形 |
| 供④ | `auto.time.now_ms() -> i64`/`now_sec`/`now` + elapsed 约定 | VM 轨=Instant elapsed 毫秒（单调钟）；与 a2r-std/src/time.rs 三方一致（GOAL-003） | 下游 T-07 bench 改源 |
| 供⑤ | vue schema 三元素+helper 类型+print 改道 | strict gen exit 0+pnpm build exit 0（下游 vue 臂验收） | 下游 T-08 复验+缓解件退役判定 |
| 供⑥ | `shell_add_recent(path) -> bool`（命名供料侧定，契约随 T-09 成文） | SHAddToRecentDocs(SHARD_PATHW)；非 Windows 返 false no-op；零注册表面 | 下游 T-02 recents 落盘挂点直调 |

### T-01 F-RV6 根因勘定（有界调查件，决策件）

复现最小化：下游矩阵 5 连跑采证（死亡点/端口态/stdout 尾帧/日志）+
拆分前代码×同工具链对照跑（§5 排障法）+668 快照面嫌疑位审查（mcp_
server 绑定/app 生命周期/窗口关闭链）。产出=根因报告落 §9+修复方案
定形。判不达（不可稳定复现/根因在本包界外）→记录性结论+漂移登记，
T-02 改记录性件（不改退出码语义）。

- AC 关联：AC-01。验证：根因报告在档（含对照跑数据）。

### T-02 F-RV6 修复+五连跑判据（前置 T-01）

按 T-01 根因形修复（工具链侧）。验证=下游 desktop_mcp 矩阵 5 连跑
零早崩（exit0×5；测试级失败集漂移另计——判据=早崩清零非全绿）。
T-01 判不达分支：记录性件——竞态边界成文+下游条款维持注记。

- AC 关联：AC-01。验证：五连跑谱+exit 码。

### T-03 code_editor_save 直写端点（供①）

core/mod.rs 新 `code_editor_save(key, path) -> bool`：storage_key 取
Buffer → 序列化直写磁盘（零全文 VM 字符串往返——与 code_editor_text
读出+write_text 链的本质差）；bom/eol 包装归属勘定成文（预记=端点侧
裸写，下游 front 包装保留——探针对拍防双重包装）；VM shim+catalog+
a2r 映射三面同步（687 形）；单元测试（写通/缺席 key false/字节对拍）。

- AC 关联：AC-02。验证：`cargo test -p auto-lang code_editor_save`
  绿+a2r 映射在册 grep。

### T-04 set-cursor 端点（供②a）

core/mod.rs 新 `code_editor_set_cursor(key, line, col) -> bool`：
cursor 读侧（:2064）姊妹写位——定位+caret 应用；基面勘定（cursor 读
返 (line,col,?) 语义 vs 下游 SyncCursor +1 面——契约成文）；三面同
步+单元测试。

- AC 关联：AC-03。验证：cargo test 绿+契约节。

### T-05 editor scroll 读/写端点（供②b）

`code_editor_scroll_offset(key)`（per-tab 滚动偏移上报）+
`code_editor_scroll_to(key, x, y)`（按偏移应用）——iced widget 滚动
面（core/draw 中立形+iced adapter 应用位）；onscroll 回声行为与 §14
观察件裁定合流（写端点应用是否触发回声事件——契约成文）；三面同
步+单元测试。

- AC 关联：AC-03。验证：cargo test 绿+契约节。

### T-06 time 族 VM 运行时 shim（供④）

vm/native.rs 新 shim_time 族（now_ms=Instant elapsed 毫秒单调钟/
now_sec/now）接 catalog 已登记位（:1192 `("auto.time.now_ms", 1200,
I64)` 等）；裸/use 两形态通（现均返 0）；elapsed() 返回约定补（.at
侧 None 现状→Option 形语义成文）；与 a2r-std/src/time.rs 对拍测试
（三方一致 GOAL-003）。

- AC 关联：AC-04。验证：cargo test -p auto-lang time 绿+.at 侧探针
  （auto eval 形）返非零。

### T-07 menubar-sub 三元素 vue schema（供⑤a）

ui_gen vue 轨补 menubar-sub/sub-trigger/sub-content（rust 轨 :3360
语义对齐——prop/事件发射同形）；strict codegen validation S002
「unknown element」清零。验证：下游 pac（auto-edit）strict gen
exit 0（跨仓只读构建证）。

- AC 关联：AC-05。验证：strict gen exit 0+schema 单测。

### T-08 helper 族类型注解+print 映射改道（供⑤b）

ts_adapter helper 发射位补 TS 类型注解（App.vue TS7006×12 清零——
参数/返回显式形）；print 发射 `console.log`→`globalThis.console.log`
（TS2339×4 清零——store `console` state 遮蔽解）；验证=下游 strict
gen+pnpm build 双 exit 0（供⑤整体验收面，与 T-07 合流）。下游
regen_vue.py 遮蔽缓解件随之可退役（下游退役判定其计划 T-08）。

- AC 关联：AC-05。验证：双 exit 0。

### T-09 SHAddToRecentDocs shim（供⑥）

vm/native.rs 新 shim（cfg(windows) `SHAddToRecentDocs(SHARD_PATHW,
path.encode_utf16()+NUL)` 一调用；非 Windows no-op 返 false）+catalog
登记（id 段随族）+a2r 映射三面同步；命名 `shell_add_recent`（契约节
成文：返回语义/非 Windows 形/与注册表零关联边界）。单元测试
（Windows 上冒烟+签名对拍）。

- AC 关联：AC-06。验证：cargo test 绿+catalog/a2r grep。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | auto-lang/ui/design/code-editor-session-endpoints.md | before：cursor 读-only（load_file/edit/text 族在档，save/set_cursor/scroll 缺位）。after：save 直写（字节保真归属成文）+set_cursor（基面对 SyncCursor 成文）+scroll 读/写（回声语义成文）三端点契约节 | 下游 013/010/009 三期 want 集中清偿 | AC-02/AC-03 |
| SD-02 | add | auto-lang/vm/design/time-natives.md | before：catalog 登记无 VM 运行时（返 0），a2r-std 单轨实装。after：三面一致 shim 契约（单调钟毫秒/秒/elapsed 约定） | §9 观察清偿+GOAL-003 三方一致 | AC-04 |
| SD-03 | modify | auto-lang/ui/design/vue-use-fn-emission.md | before：helper 发射无类型注解+print 硬编码 console.log。after：helper 显式 TS 形+print→globalThis.console.log（遮蔽解） | §9 附记/§14/§15/§16 解锁③ | AC-05 |
| SD-04 | add | auto-lang/ui/design/shell-recent-integration.md | before：内核零 Windows shell 面（T-01 五面勘定）。after：SHAddToRecentDocs shim 一调用契约（非 Windows no-op/零注册表关联/跳转列表 Recent 面边界；ICustomDestinationList 非目标成文） | PLAN-014 T-01 裁定 (b) 消费前置 | AC-06 |

## 6. 测试设计

- **供③**：下游矩阵 5 连跑（判据仪器；对照跑数据在根因报告）。
- **供①②⑤⑥ 单元/契约**：cargo test -p auto-lang 按 filter；catalog
  一致性（登记↔shim↔a2r 三面 grep 门）；字节对拍（save 直写 vs 参
  考实现）。
- **供④**：cargo test time 族+a2r-std 对拍+auto eval 探针（VM 轨返
  非零）。
- **供⑤ 跨仓证**：auto-edit 仓 strict gen+pnpm build 双 exit 0（只
  读执行，产物不入本仓）。
- **回归面**：本仓既有门禁（cargo test 全档在执行期按仓库惯例跑相
  关 crate 档）；六件互零扰动（分件提交）。

## 7. 验收标准

- **AC-01 F-RV6**：根因报告在档（对照跑+嫌疑位审查）；修复后下游矩
  阵 5 连跑零早崩（或 T-01 判不达分支：竞态边界成文+记录性降级，下
  游口径注记）。验证：五连跑谱/报告。
- **AC-02 save 直写**：`code_editor_save` 三面同步在册（VM shim/
  catalog/a2r）；单元测试绿含字节对拍；bom/eol 归属契约成文。验证：
  cargo test+grep。
- **AC-03 光标/滚动**：set_cursor+scroll 读/写三面同步；基面与回声
  语义契约成文；单元测试绿。验证：cargo test+契约节 grep。
- **AC-04 time 族**：VM 轨 now_ms/now_sec/now 返真实单调钟值（探针
  非零）；elapsed 约定成文；与 a2r-std 对拍一致。验证：cargo test+
  探针。
- **AC-05 vue strict**：下游 strict gen exit 0+pnpm build exit 0
  （S002/TS7006×12/TS2339×4 三集清零）。验证：跨仓构建退出码。
- **AC-06 跳转列表 shim**：shell_add_recent 三面同步+cfg(windows)
  形态+非 Windows no-op；契约节成文。验证：cargo test+grep。

## 8. 执行步骤

| 步 | 任务 | 依赖 | 产出/验证 |
|---|---|---|---|
| 1 | T-01 F-RV6 根因勘定 | 无 | 根因报告（对照跑数据） |
| 2 | T-02 F-RV6 修复+五连跑 | T-01 | 下游矩阵 5×exit0 |
| 3 | T-03 code_editor_save 直写 | 无（可与 T-01 并行） | cargo test+三面 grep |
| 4 | T-04 set-cursor 端点 | 无 | cargo test+契约节 |
| 5 | T-05 editor scroll 读/写 | 无 | cargo test+契约节 |
| 6 | T-06 time 族 shim | 无 | cargo test+探针非零 |
| 7 | T-07 menubar-sub vue schema | 无 | strict gen exit 0 |
| 8 | T-08 helper 类型+print 改道 | T-07 | 双 exit 0 |
| 9 | T-09 SHAddToRecentDocs shim | 无 | cargo test+三面 grep |

（优先序=下游消费解锁序：供③先行→供①→供②→供④⑤⑥并行可选；
T-03..T-06、T-09 与 T-01 无依赖可并行。worktree 纪律=组内
`.wt/lang-701/auto-edit` 无涉、本计划 worktree=`.wt/lang-701/
auto-lang` branch `plan-701-dev`；本仓主检出另有并行会话 blueprints
WIP——零触碰零依赖；钉版纪律=执行期核 `auto --version`+mtime。）

## 9. 复审记录

- **2026-09-24 stage: new（r1，drafting → 预备交接）**：形态 A 供料
  件立项（消费件=auto-edit PLAN-014，两件执行前置互绑）。六件契约
  面取自下游 upstream m1-supply §5/§9/§10-2/§12/§13/§14/§15/§16+
  PLAN-014 §5 回执表（r2）+probe_jumplist_report.txt；代码锚实勘
  （HEAD 70bcd5246）：code_editor core 读侧在档（cursor:2064/
  load_file:2104）写侧全缺、time 族 catalog:1192 登记无 shim（a2r
  轨 a2r-std/src/time.rs 已实装）、menubar-sub rust 轨 rust.rs:3360
  在档 vue 轨缺、native shim 族先例 shim_console_log:1798、a2r 映射
  687 先例。重叠核查：DEBTS/242/699 零覆盖，archive 至 700，
  .next-id=701 一致。四 SD（三 add 一 modify）。供⑥=用户 2026-09-24
  确认改道件（PLAN-014 Q-1 闭环）。outcome: pass（drafting 授权完
  备——执行分段优先序=供③→供①→供②→供④⑤⑥）。next: work
  （auto-plan-work，worktree `.wt/lang-701/auto-lang` branch
  plan-701-dev）。

## 10. 待澄清事项

- **Q-1 save 直写字节保真归属**（T-03）：端点侧裸写（预记）vs 端点
  侧承载 bom/eol 包装——下游 WriteFidelity 包装逻辑保留前提下的双
  重包装风险=T-03 对拍勘定后契约成文；漂移则双方向有界修订。
- **Q-2 scroll 写端点回声语义**（T-05）：程序化 scroll_to 的
  onscroll 回声随构建漂移（§14 观察件）——写端点应用是否发回声需
  与该观察裁定合流（二选一稳定+成文）；不阻塞 T-04。
- **Q-3 F-RV6 根因在界外分支**（T-01）：若根因定位落在本包界外（如
  iced/系统层），T-02 转记录性件（竞态边界成文），下游重跑条款维持
  ——不硬凑修复判据。

---
plan_id: PLAN-710
status: reviewed
feature_name: a2r 生成缺口残余清偿（auto-edit M4 供料包供① 承接——G-C code_editor_delta ui_gen 臂/G-B try-catch trans 臂/G-A envelope 成员访问投影 三件+corpus 穷尽性判定）
author: [agent]
created_at: 2026-09-30T01:00:08+08:00
updated_at: 2026-09-30T01:00:08+08:00
plan_revision: 1
current_step: 7
total_steps: 7
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/a2r-app-mapping-completeness.md（SD-01：三类映射语义契约+同源纪律+corpus 判定面）"
touched_goals: []             # 无 goals.md 正式 GOAL-NNN 锚定本面（供料驱动面，703 先例注记式）
affects: [crates/auto-lang/src/ui_gen/rust.rs, crates/auto-lang/src/tests/]
---

# [PLAN-710] a2r 生成缺口残余清偿（auto-edit 供① 承接）

## 0. 变更摘要

auto-edit 2026-09-29 落档的 **M4 解阻供料包**
（`auto-edit/docs/upstream/2026-09-m4-perf-unblock-supply.md` §1 供①）
的**承接件**。下游动机：L2 主形态（a2r release 直拉——战略 §5 唯一
预算效力形态）自 PLAN-007 装载链引入起 blocked，018/019 两轮勘定
**regen 探针 133 错实录**（`perf.py a2r` exit 3——生成物三类占位/
裸调，日志在 auto-edit tools/perf/logs/a2r-20260929-110102.log 入仓）；
下游 M4 全预算行正式判定（steady/warm/open/idle 的 L2 断言化+
diff_100mb 硬门禁收口+对比表我方列）以本件为最后一公里。三子件按
供料定义：**G-C** `code_editor_delta` ui_gen 臂（唯一名字解析错
E0425——`vm_builtin_host_call` 单源表增臂，镜像 `code_editor_edit`
同款直调形态）→ **G-B** try/catch 块 trans 臂（4 处 `/* unhandled
stmt */`——catch_unwind 形态，back_proxy 先例）→ **G-A** envelope
成员访问投影（17 行 `/* expr */`——`json.to_value` 结果 `v.field ??
default` → serde_json::Value 读取+缺省回退）。**穷尽性裁定**=T-00
对 133 错全量普查（三类修毕 corpus regen exit 0 即穷尽性实证；现
第四类=记录回补范围）。判定面=auto-edit 工程副本（tmp 生成树——
pristine 纪律）`auto build -r rust` exit 0+生成物 grep 零占位零裸调
+生成 workspace cargo check 过。下游消费件（perf.py a2r 复验+L2
锚点补跑+矩阵回归）属 auto-edit 后续件不在本件（669 模式）。

## 1. 目标

- **G-1 G-C `code_editor_delta` ui_gen 臂**：`vm_builtin_host_call`
  表（ui_gen/rust.rs:9673 实锚——单源 a2r 内建面）增
  `"code_editor_delta"` 臂，直调 core 实现
  `code_editor_delta(key) -> Option<String>`（core/mod.rs:2068 实锚
  ——**不绕 VM shim**，703 直调纪律）；返回形
  `Option<String>→String` 对齐 VM shim 同源语义（native.rs:709
  destructive read——未注册键错误形同源；unwrap 映射形 T-01 对拍
  定案）。清偿生成物 3 裸调用点（018 实勘）。
- **G-2 G-B try/catch 块 trans 臂**：stmt 发射器占位
  `_ => format!("/* unhandled stmt */")`（ui_gen/rust.rs:9019 实锚）
  增 Try 形映射——`.at` 语义=运行时错误兜底走 catch 块；生成形默认
  **catch_unwind 包裹**（back_proxy.rs 先例在册）——try 块正常路径
  零语义扰动、panic 兜底进 catch 块（变量逃逸形=生成器侧 let 预置
  +赋值回写，T-02 实勘定形）。清偿生成物 4 处（editor_store.at:499
  会话恢复/:897 file_size 探测/badge probe ×2——018 实例集）。
- **G-3 G-A envelope 成员访问投影**：expr 发射器占位 `_ =>
  "/* expr */"`（ui_gen/rust.rs:10951 实锚；trans/r2a.rs:1363 同形
  占位——T-00 判定生成链路归属）增 `json.to_value` 结果成员访问族
  ——`v.field ?? default`（缺省=?? 右值）、嵌套成员
  `v.a.b ?? d`、索引+成员混合形（018 实例集 17 行：`v.err ?? ""`/
  `v.hunks ?? []`/`r.lo ?? 0`/`h.a1 ?? 0` 族）；实现形=serde_json::
  Value 读取 helper（a2r_std 或生成内联——T-03 定）+类型投影
  （str/int/bool/list 按右值类型）。
- **G-4 穷尽性裁定（T-00 决策件）**：018 日志 133 错全量分类普查
  ——三类归因计数+非三类残余枚举（预期零；若有=§10 记录回补本件
  范围）；生成链路定界（`auto build -r rust` 实际发射路径=ui_gen/
  rust.rs 为主，r2a.rs 旧面是否在链=T-00 实证）；每类最小复现
  .at 用例构造（上游单测基）。
- **G-5 corpus 集成判定**：auto-edit 工程副本（main@执行日 钉版
  tmp 拷贝——016 tmp 生成树先例，pristine 纪律零触碰下游检出）：
  `auto build -r rust` **exit 0**+生成物 grep 三占位零命中
  （`/* expr */`/`/* unhandled stmt */`/裸 `code_editor_delta`）
  +生成 workspace `cargo check` 过。
- **G-6 语义同源对拍（上游单测）**：三类各构造用例（供料验收建议
  原文）——try 成功/异常两臂（成功零扰动+异常走 catch+变量回写）、
  envelope 嵌套成员+缺省（缺省=?? 右值逐形）、delta 未注册键错误
  形；对拍口径=VM 轨与 a2r 轨同输入同输出（673 同源对拍惯例）。
- **G-7 规范+账本**：SD-01 新册落档+242 tracker 补行（living doc
  惯例——三类完成行+核实日期）+specs.json reviews 段 P710-1 外科
  插入（703 先例）。

### 非目标

- 下游消费件（auto-edit `perf.py a2r` exit 0 复验+L2 锚点补跑
  [bench --mode l2 全链]+装载链/diff 视图面矩阵回归——669 模式
  下游件，本件 delivered 后立项）。
- 同包他件（供② 帧时间戳插桩/供③ 卡死确认复核/供④ tree-sitter/
  §5 two-face 子集——各自承接另立）。
- 性能/体积优化（本件=生成完备性；瘦身与 L2 数字=下游 L2 链打通
  后件）。
- 语言级特性扩展（242 tracker 语言面条目不扩——本件三类=生成器
  完备性非新语义；`.at` 语义变更零容忍[frozen]）。
- r2a.rs 旧 trans 面的超出链路必需的清理（T-00 判定不在链=不动）。

## 2. 架构方案

分层落点（2026-09-30 实勘，auto-lang master@4dae03122）：

| 面 | 现状 | 本期形态 | 依据 |
|---|---|---|---|
| a2r 内建表 | `vm_builtin_host_call`（ui_gen/rust.rs:9673）单源表——`code_editor_edit` 臂在（:9716 直调形+max(0) 换算）；delta 无臂（grep 零命中） | G-C 增臂（镜像 edit——直调 core `code_editor_delta`；Option→String 映射形对齐 shim） | 供料 §1 G-C（行级定义）；703 直调纪律（「实现体单源 diff::envelope，不绕 VM shim」同款） |
| stmt 发射 | `_ => format!("/* unhandled stmt */")`（:9019）——Try 形落占位 | G-B 增 Try 臂（catch_unwind 形——back_proxy.rs 先例；变量逃逸=let 预置+回写形 T-02 定） | 供料 §1 G-B；back_proxy catch_unwind 在册 |
| expr 发射 | `_ => "/* expr */"`（:10951；r2a.rs:1361 同形）——Value 成员访问族落占位 | G-A 增投影臂（serde_json::Value 读取 helper+?? 缺省+类型投影） | 供料 §1 G-A（17 行实例集） |
| corpus 判定 | 下游 018/019 探针 exit 3（133 错日志入仓）；上游无 corpus 集成面 | G-5：auto-edit 工程副本 tmp 生成——exit 0+grep 零占位+cargo check（上游侧自含判据，不依赖下游会话） | 016 tmp 生成树先例；pristine 纪律 |
| 对拍 | 673 同源对拍惯例在册（VM/a2r 同输入同输出） | G-6 三类对拍单测（plan710 探针族——703 time 族探针形态） | 703 验证锚先例（plan703_supply_probes.rs） |
| 规范 | a2r 域册=shell-a2r-seams（seam 面）；完备性无专册 | SD-01 新册 a2r-app-mapping-completeness（三类语义契约+判定面）；242 tracker 补行 | 703 SD 三册家族；242 living doc 惯例 |

**关键设计约束（frozen）**：
① **语义同源纪律**（供料原文）——G-C 直调 core 实现（不绕 VM
shim）；G-A/G-B 与 VM 轨语义逐形对拍（envelope 缺省=?? 右值；try
异常=catch 块承接）。② **.at 语义零变更**（本件=生成器完备性，
语言语义面不动——冲突即 discrepancy 回供料方）。③ corpus 验证经
tmp 副本（下游检出零触碰——016 pristine 先例；钉版哈希记录在案）。
④ 判据机读：exit 0+grep 零命中+cargo check 过（三重判定缺一不可）。

## 3. 技术栈

Rust workspace（crates/auto-lang ui_gen 面+tests/plan710 探针族）；
corpus=auto-edit 工程副本（worktree 侧钉版拷贝）+`auto build -r
rust`（worktree 构建工具链——016/018 纪律：判据前核构建源；PATH
二进制坑三度应验在案）；cargo tf 全量门+tv（VM 面零触及则 tv 抽档
——T-00 判定）。

## 4. 需求分析与背景调查

**授权记录**：用户 2026-09-30 会话指令「计划 20 已经完成；请规划
下一个计划文件」+**AskUserQuestion 裁定**「auto-lang 710：供① a2r
映射承接（推荐）」——授权=**起草本件**（auto-lang 仓）；执行/work
待用户另行启动（建议 auto-lang 会话——707 复审在途/706 rev2/708/
709 三稿并行，执行启动时核 710 编号与建组空闲）。范围=auto-lang
crates/docs/specs/docs/plans；**auto-edit 零改动**（corpus 经 tmp
副本只读消费）。无预算/自动续跑授权。

**来源与版本**：

- 供料包：`auto-edit/docs/upstream/2026-09-m4-perf-unblock-supply.md`
  §1 供①（2026-09-29 落档，PLAN-018 T-01）——三类行级定义+动机+
  期望形态+验收建议；证据基=`auto-edit/specs/auto-edit/tests/
  evidence-p018-survey.md` §①-b 三类表+§⑤ 探针实录（133 错/
  perf.py a2r exit 3/首错 `expected expression, found ';'`
  @main.rs:1258=G-A 占位）+019 复核（regen 探针 133 错维持，
  last-good 基面裁定在案）。
- 承接时点复核（2026-09-30，auto-lang master@4dae03122）：
  `vm_builtin_host_call` 表在位（:9673）+edit 臂形（:9716——直调+
  max(0) 换算+unwrap_or 形先例 :9724 load_file）+delta 无臂 grep
  实证；stmt 占位（:9019）/expr 占位（:10951）发射点实锚；core
  `code_editor_delta`（core/mod.rs:2068，Option<String>）靶函数
  在位；back_proxy.rs catch_unwind 先例在册；242 tracker 为 living
  doc（a2r 域归口，补行惯例在案）。
- 历史关联：PLAN-673/687（端点族 a2r 臂分批清偿——18/19 内建先例，
  供料原文「上游已分批清偿大半」）；PLAN-703（直调纪律+探针形态+
  SD 册家族先例）；PLAN-707 work 完待复审/706 rev2/708/709 并行
  （域无冲突——本件 ui_gen 发射面，707=HTTP 传输/708=gallery 渲染
  响应/709=槽位交互；706 rev2 含 ui_gen 面注记[aura_view_builder
  大宗]——执行期 rebase 协调在 §10 Q-2）。

## 5. 详细设计

### T-00 错误普查与链路定界（决策件，产物=普查报告）

1. **133 错全量分类**：018 日志（入仓）+本仓重跑复核——按三类
   归因计数（G-C E0425×N/G-B unhandled stmt×N/G-A expr 占位×N）
   +非三类残余枚举（预期零；有=§10 记录回补）。
2. **生成链路定界**：`auto build -r rust` 发射路径实证（ui_gen/
   rust.rs 主链 vs trans/r2a.rs 旧面——是否在链决定 G-A 实施落点
   单侧或双侧）。
3. **最小复现族**：每类 .at 最小用例（try 两臂/嵌套成员+缺省/
   delta 键两形）——单测与对拍基。
4. **corpus 钉版**：auto-edit main@执行日 哈希记录+tmp 拷贝脚本。

### T-01 G-C delta 臂（G-1）

`vm_builtin_host_call` 增 `"code_editor_delta" => Some(format!(
"auto_lang::ui::code_editor::code_editor_delta(&({a0}))\
.unwrap_or_default()", ...))`（unwrap 形对拍 shim native.rs:709
定案——错误形字符串 vs 空串，以 shim 实勘为准）；五面注册惯例核
（catalog/bigvm/intrinsics/a2r/静态表——703 注册面家族中 ui_gen 臂
即本面，其余四面 delta 已在[701/703 期]——T-00 复核）。

### T-02 G-B try/catch trans 臂（G-2）

stmt 发射器 Try 形：`std::panic::catch_unwind(AssertUnwindSafe(
|| { try 块 })）`——`Ok` 丢弃走 try 尾、`Err` 进 catch 块；try 块
内变量逃逸=发射器静态预置 let（外层可见）+闭包内赋值（Copy/引用
形 T-02 实勘——editor_store 实例集四处的变量面：bool 旗标+int 计
数，逃逸形简单）；panic 载荷不消费（.at catch 无绑定形——实例集
实证）。**成功路径零语义扰动**（非 try 块零改动——frozen）。

### T-03 G-A envelope 投影臂（G-3）

expr 发射器 Value 成员访问族：`v.field ?? default` →
`a2r_std::json_get(&({v}), "field").unwrap_or({default})` 形
（helper 落 a2r_std[sys 族旁]或生成内联——T-03 依助面最小化定）；
类型投影按 ?? 右值（str→String/int→i64 as i32/bool/list→Value
数组——实例集四族覆盖）；嵌套成员/索引混合形递归展开。r2a.rs 面
若在链（T-00②）同臂落位。

### T-04 corpus 集成判定（G-5）

tmp 副本（钉版拷贝）→`auto build -r rust`（worktree 构建工具链）
→**三重判据**：exit 0+grep 三占位零命中+生成 workspace cargo
check 过。产出=判定报告（错误普查对照——133→0 全量在档）。

### T-05 对拍单测+回归（G-6）

tests/plan710_supply_probes.rs（703 探针形态）：三类各最小用例
VM 轨 vs a2r 轨对拍（同输入同输出）+delta 未注册键错误形；回归=
cargo tf 全量（预存红对账零新增——703 惯例）+ui_gen/a2r 既有
测试族全绿。

### T-06 规范+账本（G-7）

SD-01 新册（三类映射语义契约：delta 直调形/try-catch catch_unwind
形含变量逃逸形/envelope 投影形含类型投影表+同源纪律+corpus 三重
判定面）；242 tracker 补行（三类完成+核实日期+710 链接——living
doc 惯例）；specs.json reviews 段 P710-1 外科插入（703 先例——
roundtrip 字节等价先证+前缀零扰动回读断言）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/design/a2r-app-mapping-completeness.md | before：a2r 域册=shell-a2r-seams（seam 面）；生成完备性无专册（三类占位/裸调为已知 blocked 面） / after：三类映射语义契约册——G-C delta 直调形（Option→String 同源映射）/G-B try-catch catch_unwind 形（变量逃逸形+成功路径零扰动）/G-A envelope 投影形（类型投影表+?? 缺省语义）+同源纪律（VM/a2r 逐形对拍）+corpus 三重判定面（exit 0/grep/cargo check） | 供① 落账；下游 L2 链的内核侧真源 | AC-01..04 |

## 6. 测试设计

- **对拍单测（plan710 探针族）**：G-C 两形（正常 delta JSON/未注册
  键错误形——对齐 shim）；G-B 两臂（try 成功=旗标不变+try 异常=
  catch 承接+变量回写）；G-A 族（平/嵌套/索引混合 ×缺省命中/未命
  中——右值类型四族）。VM 轨 vs a2r 轨同输入同输出断言。
- **corpus 判定**：tmp 副本三重判据（exit 0+grep+cargo check）+
  错误普查对照表（133→0）。
- **回归**：cargo tf 全量（预存红对账零新增）+a2r 既有测试族
  （144+ 用例族——242 tracker 在册口径）。
- **pristine 断言**：auto-edit 检出零触碰（corpus 全程 tmp——
  双仓 porcelain 验证）。

## 7. 验收标准

- **AC-01 G-C 清偿**：delta 臂在位+对拍两形绿+生成物 grep 零裸
  `code_editor_delta`。验证：`cargo test -p auto-lang plan710` 绿
  +corpus grep。
- **AC-02 G-B 清偿**：Try 臂在位+两臂对拍绿+生成物 grep 零
  `/* unhandled stmt */`。验证：同上。
- **AC-03 G-A 清偿**：投影臂在位+四族对拍绿+生成物 grep 零
  `/* expr */`。验证：同上。
- **AC-04 corpus 三重判定**：auto-edit 副本 `auto build -r rust`
  exit 0+grep 三占位零命中+cargo check 过（普查对照 133→0 在档）。
  验证：判定报告+钉版哈希。
- **AC-05 回归门**：cargo tf 预存红对账零新增+a2r 测试族全绿。
  验证：tf 输出对账表。
- **AC-06 规范+账本**：SD-01 册+242 行+specs.json P710-1 回读
  True。验证：文件在档+账本断言+grep 锚。

## 8. 执行步骤

| # | 任务 | 依赖 | 落点（实勘锚） | 产出/意图 | AC | 验证（命令/预期） |
|---|---|---|---|---|---|---|
| 0 | T-00 错误普查+链路定界 | — | 本件 §5 T-00 节+普查报告 | 三类穷尽性+复现族+钉版 | AC-04 | [x] 普查报告在档（133 全分类）✅ docs/reports/p710-census.md——基线 133 错逐数吻合（17/4/3+E0425×1+PLAN-027 门×1）；链路定界=ui_gen/rust.rs 单链（r2a.rs 不在链）；corpus 钉版 auto-edit main@70c5c60 组内 tmp 拷贝（零 junction） |
| 1 | T-01 G-C delta 臂 | T-00 | ui_gen/rust.rs:9673 表 | 直调臂+同源映射 | AC-01 | [x] 对拍绿+grep 清 ✅ commit 5c1d43b52——直调臂+shim 同消息 panic! 形；probe 11/11 绿（vm_delta envelope/watermark/错误消息三断言） |
| 2 | T-02 G-B try/catch 臂 | T-00 | ui_gen/rust.rs:9019 族 | catch_unwind 形+逃逸形 | AC-02 | [x] 两臂对拍绿+grep 清 ✅ commit 5c1d43b52——catch_unwind(AssertUnwindSafe) 形+catch(e) panic_message 绑定（VM STORE_LOCAL 同源）；probe 成功臂/错误臂绿 |
| 3 | T-03 G-A 投影臂 | T-00 | ui_gen/rust.rs:10951（+r2a 视链路=T-00 判定不在链，单侧落位） | Value 成员访问族 | AC-03 | [x] 四族对拍绿+grep 清 ✅ commit 5c1d43b52——a2r_std::json get_*_or/at_owned/truthy 路径切片族（含 Unary(Sub) 负缺省+str-with 计算缺省）；probe 五段投影+七族发射形绿 |
| 4 | T-04 corpus 集成判定 | T-01..03 | tmp 副本+worktree 工具链 | 三重判据+普查对照 | AC-04 | [x] exit 0+grep 零+cargo check 过 ✅ r22（工具链 v0.4.2-2243-gb310acafd+AUTO_LANG_CRATE 钉 worktree）：exit 0+三占位零命中（直调臂×3 在位）+生成 workspace cargo check 过（56.55s 0 error）——133→0 穷尽性实证（回补 D-4..D-8 归因在 census 报告） |
| 5 | T-05 对拍单测+回归 | T-01..03 | tests/plan710_supply_probes.rs | 探针族+回归门 | AC-01..03/05 | [x] plan710 绿（11/11）✅ commit fb47aea61；AC-05 全集 tf=用户裁定豁免（部分证据+musk p053 预存归因在 §9 deviation/R-2） |
| 6 | T-06 规范+账本 | T-04/05 | SD-01+242+specs.json | 落账+投影 | AC-06 | [x] 文件在档 ✅ commit 4d79b52d3（SD-01 册+242 #18 行+census 报告）；specs.json P710-1 主检出账本投影随 §9 收尾（703/704 master 直投惯例） |

## 9. 复审记录

- 2026-09-30 起草 handoff：`stage: new`，PLAN-710，plan_revision 1。
  `outcome: pass`（起草完备：供① 三类行级定义直承供料档（018 勘定
  证据基）+承接时点三面实锚复核[表/占位点/靶函数]；穷尽性裁定与
  corpus 三重判定面成文；同源纪律 frozen；下游消费件边界清晰[669
  模式]；路径/符号经 auto-lang@4dae03122 与 auto-edit 供料档双源
  锚定；授权=起草[用户 AskUserQuestion 裁定在录]，执行待用户启动
  ——建议 auto-lang 会话，注意 706 rev2/707-709 并行协调[§10 Q-2]）。
  `next: work`。

- 2026-09-30 work handoff：`stage: work`，PLAN-710，plan_revision 1，
  `outcome: pass`，code_commit=**plan-710-dev@4d79b52d3**（实施链
  5c1d43b52→fb47aea61→4d79b52d3，基=b310acafd；worktree
  .wt/lang-710/auto-lang + auto-down 组内兄弟@3373a5c detach）。
  task_ids=T-00..T-06 全落（§8 逐项证据）。
  evidence：① corpus 三重判据（AC-04）——auto build -r rust **exit 0**
  +生成物 grep 三占位零命中（delta 直调臂×3）+生成 workspace cargo
  check 过（56.55s 0 error），133→0 穷尽性实证
  （docs/reports/p710-census.md，基线 133 错逐数吻合 018 口径；018
  「类型推断塌方」误并档更正=回补 D-4..D-8 五独立残面逐一归因）；
  ② 语义同源对拍（AC-01..03）——plan710_supply_probes **11/11 绿**
  （G-C VM envelope/watermark/错误消息、G-B 两臂+绑定、G-A 五段投影
  +truthy 逐格、发射形七断言）；③ SD/账本（AC-06）——SD-01 册+
  242 tracker #18 行（worktree commit）+specs.json reviews P710-1
  外科插入（主检出；roundtrip 字节等价先证[indent=2/ascii=false/尾
  换行]、175→176、前缀+他五段零扰动回读断言，sha 7e65c8be9e271ac5）
  +INDEX 再生（26 projects）。
  blockers=无。
  deviation：**AC-05 全集 tf 由用户裁定豁免**（2026-09-30 会话指令
  「本计划相关测试已全绿——不必全集 tf，直接 review/merge」）；已留
  部分证据=tf fail-fast 跑 1564 例 2 红（musk p053 族）+master@b310acafd
  双检出隔离复跑同数（35/4 一致）归因**预存非本件回归**；第三次全集跑
  执行至尾部由同裁定中止（尾部 SLOW=default_headers 环境 Red——
  P707 在册预存）。AC-05 状态=豁免（非 pass），复核请对照本注记与
  R-2（musk p053 稳定化另件）。
  边界登记：R-1 route-A 深修（merged back-api 运行期真值桩形维持，
  下游 L2 按需立项）。
  `next: review`。

- 2026-09-30 review（复审会话=实现会话同 context，独立性局限在案——
  结论从工件/复跑证据重建而非执行者转述）：`stage: review`，PLAN-710，
  plan_revision 1，`outcome: pass`，reviewed_commit=**plan-710-dev@
  1cb3ad662**（rebase 后新链 e891ca323/8e41c3214/48f9f3649/1cb3ad662，
  基=c80887ab7 测试规则改版 master；旧基 b310acafd 上的原链
  5c1d43b52/fb47aea61/4d79b52d3 语义等价重放）。dependency_revisions：
  auto-down@3373a5c（组内兄弟 detach）；corpus=auto-edit main@70c5c60
  组内 tmp 钉版。spec_inputs：SD-01
  docs/specs/auto-lang/ui/design/a2r-app-mapping-completeness.md
  （plan 分支 48f9f3649 落档，合并时随分支入 master——canonical 发布
  流 intact）。
  acceptance_results（新门禁口径=裸 cargo t+触面档；AC-05 旧 tf 口径
  由门禁改版自然取代——tf 已改判批量回归档，归 merge 收尾到期判定）：
  - AC-01/02/03（G-C/G-B/G-A 语义+清偿）：`cargo test --lib plan710`
    **11/11 绿**（@1cb3ad662 复跑）；corpus grep 三占位零命中。
  - AC-04（corpus 三重判据）：**r24 复跑 @1cb3ad662 工具链**——exit 0
    + expr/stmt/裸 delta=0 + 生成 workspace cargo check 过（1m28s，
    fix-ui-tier 动过 ui_gen 后在新基重立）。
  - AC-05（回归门）：按改版门禁执行——裸 `cargo t --no-fail-fast`
    4903 例 4892 绿 11 红：9 例=已知红基线逐名对上（musk p053×4/
    p054×2/plan606 gallery/projector/e4 default_headers 30s 有界）；
    2 例 app_registry（scan_examples_ui_curation_set/
    launch_three_real_apps）**归因=corpus 判定副作用污染**（生成
    workspace 产物按 Stage B 共享 target-dir 落仓根 target/debug/
    auto-edit.exe→041-auto-edit 被 exe 背书挤出策展集）——产物清理后
    隔离复跑 **26/26 绿**，非代码回归；`cargo tu` 855/856 绿，1 红=
    a2vue desktop 金样（4f123a50e 在案）。**零新增红**。
  - AC-06（规范+账本）：SD-01/242 #18 行 plan 分支在档（grep 锚）；
    specs.json P710-1 回读 True（176 项；插入时 roundtrip 字节等价
    先证+前缀/他五段零扰动断言，sha 7e65c8be9e271ac5）。
  findings：
  - F-710-1（已解决，工具性）：corpus regen 与 app_registry 策展
    扫描共享仓根 target 的干涉面（Stage B 共享 target-dir 设计副
    作用）——corpus 判定后须清 target/debug/auto-edit.exe 再跑族测；
    已清理并断言。非阻塞。
  - F-710-2（记录，非阻塞）：D-7 merged 桩运行期真值=记录+缺省形，
    route-A 深修=R-1 后续件（§10）。
  - F-710-3（记录，非阻塞）：scroll_to 臂按 corpus 轴串形发射，
    坐标形态零实例（误用由生成物编译错显式拦截，代码注记在案）。
  `next: merge`（含批量回归到期判定——收据 .last-batch-regression.json
  缺失=首次 merge 触发主检出单实例 tf，设计行为）。

## 10. 待澄清事项

- **Q-1 G-B 变量逃逸形与 panic 语义边界（T-02 内定，非阻塞）**：
  实例集四处变量面简单（bool/int Copy 形）；若 corpus 普查现非
  Copy 逃逸（String/list 回写）——catch_unwind+回写形扩展或记录
  为生成器已知边界（实例集外不扩[frozen ⑤ 语言面零容忍]）。.at
  catch 块无绑定形=实例集实证；若语言有绑定形语法在册而未用——
  不在本件扩（242 tracker 归口）。
  **执行裁定（2026-09-30）**：实例集逃逸面实证=bool/int Copy 形+闭包
  外层 String 局部（clone 语义臂覆盖，fsize/path 株）；catch(e) 绑定形
  落 panic_message 载荷串（与 VM STORE_LOCAL 同源）——plan710 探针
  断言在档。try 体内 return/break/continue 不出闭包=SD-01 §2 边界
  注记（实例集零实例）。
- **Q-2 并行会话协调（执行启动时核，非阻塞）**：707 复审在途/
  706 rev2（ui_gen 面大宗——rebase 协调）/708/709 三稿并行——
  启动 work 时核 710 编号仍空闲+建组（.wt/lang-710 惯例）+master
  基线重勘（本件起草基 4dae03122，执行时取当日 tip）。
  **执行裁定（2026-09-30）**：710 编号空闲确认；基组 .wt/lang-710
  已建（auto-lang@plan-710-dev + auto-down@3373a5c detach——path dep
  解析需组内兄弟）；master 基线=b310acafd（执行起点）；ui_gen 面与
  706 rev2 的合流=rebase 协调项留给 merge 步（本件发射臂改动均
  局部新臂，冲突面预期小）。
- **Q-3 corpus 钉版时点（执行期定，非阻塞）**：auto-edit main@
  执行日哈希（起草时=70c5c60）；副本只读消费+tmp 生成树——下游
  检出零触碰（pristine 断言在 AC 面）。
  **执行裁定（2026-09-30）**：钉版=main@70c5c603dcaf…（起草时点
  哈希即执行日 main——auto-edit 无新提交）；`git archive` 快照落
  .wt/lang-710/p710-corpus（组内兄弟布局，pac.at dep 与
  compute_auto_lang_rel_path 漫步自然解析 worktree；deps 两 dep 预置
  真实拷贝规避 mklink /J materialize——零 junction 红线兑现）；
  pristine 断言=census 报告 §6（auto-edit 检出前后一致）。
- **R-1（执行新登记，非阻塞）D-7 route-A 深修边界**：merged back-api
  GET/POST 桩已按契约签名收口（本件 D-7），但 route A（api_impl
  真实现委派——fsys.at 端点体转译）因 transpile 链 `.ok()?` 静默
  失败未启用——运行期真值仍为「记录+缺省」桩形。下游 L2 链
  （perf.py a2r→release→直拉）的 env_str/exists 运行时真值需求
  =后续件（merged back 服务端供真值或 route-A 转译链修复——
  auto-edit 侧 perf 复验时按需立项）。
- **R-2（执行新登记，非阻塞）musk p053 族预存红**：tf 对账中
  musk_vm_track_p053 族 4 例红（widget_computed ×2/merged_api_warning/
  widget_content）——master@b310acafd 双检出隔离复跑同数归因
  **预存非本件回归**（P707 收据在册「musk p053 双向 flaky」族的
  现势成员）。稳定化=独立件（同 707 F-1 建议）。

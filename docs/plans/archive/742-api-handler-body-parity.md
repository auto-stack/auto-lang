---
plan_id: 742
title: "api.rs handler body transpile parity（NOTES-001 T-04 跨仓）"
status: archived
author: [Codex]
created_at: 2026-10-04T21:20:00+08:00
plan_revision: 1
current_step: 3
total_steps: 3
base_branch: master
base_commit: a6c0f9691
depends_on: []
supersedes_spec_components: ["docs/specs/auto-man/design/api-generation-integrity.md", "docs/specs/auto-man/project.md", "docs/specs/auto-lang/trans/overview.md", "docs/specs/a2r-std/project.md", "docs/specs/auto-lang/ui/overview.md"]
new_spec_components: []
touched_goals: ["GOAL-003", "GOAL-007"]
---

# 742：api.rs handler body transpile parity

## 0. 背景与授权

auto-os/apps/015-notes 的 NOTES-001 T-04（legacy CRUD → repository adapter）把
多语句业务体写进 api.at 端点后，api_gen 的 handler 体发射产出不可编译 Rust。
用户已批准治本（方案1）。来源实录见
auto-os/apps/015-notes/docs/plans/001-durable-inbox.md §9 work 记录 3/4。

## 1. 缺陷清单（三例均有 Notes 实际编译错误佐证）

- D1 List 构造原样发射：`var out List<Note> = List<Note>.new([])` →
  `let mut out: Vec<Note> = List<Note>.new(vec![]);`（类型转换了、值表达式没走
  构造降级；repository.rs 同构语句产出 `vec![]` 正常——api.rs 体路径状态缺失）。
- D2 体引用的 crate 模块缺 `use crate::repository;` 导入（api.rs 头固定
  axum/types/std 三行，不收集 api.at 的 use 语句）。
- D3 `==` 链式比较发射（实错行待复现后补录）。

## 2. 方案

try_transpile_body 已走 RustTrans::stmt 全量发射器；差异在初始化状态与
api_gen 的导入收集。修复点：

- T1（D1）：api.rs 体路径的 List 构造降级——按 repository.rs 模块路径的对齐
  方式补齐 RustTrans 初始化（register_type 字段类型/所需集合），或在该路径
  显式处理 `List<T>.new(args)`；以失败测试锁行为。
- T2（D2）：generate_api_rs 收集体/来源文本中的 crate 模块引用并发射
  `use crate::<mod>;`（api.rs 头部）。
- T3（D3）：复现后修对应发射（comparison 链）。

### 规范增量

遗留计划在独立审核中补齐知识增量元数据，不改变原实现/验收语义及 plan_revision。
SD-742-01～04 的逐条 before/after、实际目标与验收映射见
[冻结稿](attachments/742/review-r1/spec-delta.md)；SHA256
`0B6E59EA24DA9131ACF7A29B591FBBE9DBEFF371EAE66A525BEA00AFD9C03EB0`。
API 规则已由 e35f9e99b 先行沉淀，但本次反例表明体路径仍未完全正确；
SD-742-01/02 不能批准为完成。new_spec_components 为空，因为更新既有模块规则即可。
复审不改 canonical Specs 或 live ledger；待修复与复审后按 merge 收口。

第二次 final review 的 [冻结增量](attachments/742/review-r2/spec-delta.md) SHA256
`9B4CA3167BBA44D5389B7840AF07499A025B63EDFD9B1B171D5D9E32B507AF28`。
旧反例已转绿，但完整发射链字面量保持、绑定作用域、File 入口及返回出口仍有缺口；
本轮 SD-742-01/02 继续不可批准为完成，目标路径和原 AC 不变。

第三次复审 [冻结增量](attachments/742/review-r3/spec-delta.md) SHA256
`9DF237808934A93E5D5DCB21A8790EFC1925C4093F18AAF3EAA89A8E076AA4FF`。
旧例及新增矩阵通过，placeholder 恢复与嵌套借用仍有反例，T-3 必要 oracle 未闭合；
SD-742-01/02 仍不可批准为完整 parity，原目标路径/AC/revision 不变。

第四次复审 [冻结增量](attachments/742/review-r4/spec-delta.md) SHA256
`D013A579F74D0861B9FD376C8016A4C7B7E5F229288A93710F5ABD5A0621FAC0`。
旧17例、嵌套组合及当前S10/Vue通过；合法companion模块名后缀仍被误识别为标准库，
SD-742-01/02继续partial。VM交互未验，不新增固定19场景门禁；原目标/AC/revision不变。

第五次复审 [冻结增量](attachments/742/review-r5/spec-delta.md) SHA256
`9691E5B5BA2D146ECE88E16765135EB2BCEBFBBE11DFD2AF6F7D340EA6B0A675`。
R4模块名反例和旧20输入原样闭合，新增多行字符串保持；但比较符被当泛型角括号，
后续str实参漏借用（F-742-R5-01），SD-742-01/02继续partial；原目标/AC/revision不变。

第六次复审 [冻结增量](attachments/742/review-r6/spec-delta.md) SHA256
`04AA21D108255F07A1F55D65EEB38A7B7D6F9F78506539E5EBD3C70013DA8803`。
F-742-R5-01 关闭（glued-ident 判别 + 独立重放 + 两枚审核方新探针），
SD-742-01/02 批准为完成；SD-742-03 目标已被 PLAN-773 沉淀改写，merge 沉淀须
基于当前 canonical 合入。原目标/AC/revision 不变。

## 3. 任务

- [x] T-1: 失败测试钉三缺陷（api_gen tests，extract_api_lenient + generate_api_rs 路径）
  —— r1 存量：`test_handler_body_module_calls_and_list_construct`
  （crates/auto-man/src/api_gen.rs，提交 152c832bc）；r2 增实编 oracle：
  `test_handler_body_string_survival_and_compile_oracle`——真 generate_api →
  提取生成 handler → rustc 实编 → noop-waker poll 运行 → 返回值对拍（七端点：
  control/list/brace/module 四字符串 + list/get(?T)/update(E0308 形)），
  文本断言降级为辅助。D3 归因补录：原始记录为类别级（NOTES-001 r2 work
  记录 4 "(c) `==` 链式比较"，auto-notes 归档 001-durable-inbox.md）；触发
  源 api.at WIP 在 3bcf5bf 回滚前未提交，无可考源行；单 `==` 发射现由
  oracle 实编覆盖。
  r3 增矩阵 oracle：`test_handler_body_r2_matrix_compile_oracle`——11 值
  编译+运行矩阵（escape 复合串/borrow-marker 串+真实借用正例/嵌套遮蔽/
  void 三态/Option 变量双臂/File 模块），覆盖 r2 全部反例与 path 变体。
- [x] T-2: 修复使测试绿——r1 账（D1~D9，见下）+ r2 修复四发现（提交
  6d93cfdb7，worktree plan-742-dev）：
  - F-742-R1-01：`code_region_mask`（转义感知的字符串/字符/行+块注释）门控
    `split_glued_statements`/`lower_list_constructs`（trans/rust.rs）——业务
    字符串逐字节保留，仅代码区构造降级/分割。
  - F-742-R1-02：companion 点调用限定化从整文件文本 pass 移入
    generate_api_rs 逐端点体循环（endpoint 局部名遮蔽模块、调用形态
    `mod.fn(` 必需、字段位排除）；导入按实际限定发生发射；退役 pass 同时
    不再误改模板体 `db.lock()`。
  - F-742-R1-03：`InlineRetWrap` 五态枚举——needs_result 非空返回一律
    `Ok(JsonResponse(x))`（update_note E0308 关闭）；void 体补状态尾
    （无显式 return 时合成）。
  - F-742-R1-04：删 NL/wrap_inline_return 旧包装/borrow_string_db_call_args_old，
    742 新增 dead-code 告警清零。
- [x] T-3: 端到端——r1 历史账（NOTES-001 r3/r4 交付，D8/D9 即实测产物；
  auto-os c4c848d 电池记录）+ r2 重验（绑定修复提交 6d93cfdb7 同源码的工作树
  CLI）：再生成 auto-notes 后端（generation.json 2026-10-13T15:05）→
  cargo build 通过 → `tests/probe/run_v1_battery.sh` 全绿（fails=0，36 断言 ✓，
  含 S10 崩溃恢复/S15 修复/S19 混态；NOTES_TEST_MODE=1 + 隔离
  NOTES_DATA_DIR，NOTES_SEED 未设）。审核 replay.ps1 -Build 于提交态重放
  四用例 ALL-PASS（compile_exit=0 + 返回值与预期字面值逐字节一致；D7/D9
  运行时控制同轮 PASS）。r3 重验（绑定提交 8c9b2d3d9 重建 CLI SHA256
  5e5b146ca7f2…）：再生成后端（generation.json 2026-10-14T01:40:08）→
  cargo build → v1 电池 fails=0 + **实驱 S10**（setup 隔离目录→建
  s10-final→taskkill /F→确认下线→重启→同目录 setup→GET 存活）；
  r2 replay 13/13 compile_exit=0 逐项对拍。Vue 面阻塞（pnpm 镜像网络
  两败）、VM 面阻塞（需有头桌面 MCP 9247）如实记录，见 work-r3。

上述为历史 work 申报；2026-10-14 第三次复审为 **needs_fix**：r2 的 13 个输入
确已转绿，但标准库 placeholder 恢复仍改字符串、嵌套模块调用漏补内层借用。
Vue/VM 验证在交接时即未完成，S10 存档只有 setup/GET，缺创建、强杀、下线、
重启过程；不能将 T-3 计完成。T-1/T-2/T-3 重开，当前 0/3。详见 §4.4。

以上 r1 提交账与交付记录保留为历史；2026-10-13 独立 final review needs_fix
后，r2 修复在 worktree `D:/autostack/.wt/lang-742/auto-lang`（branch
plan-742-dev，base c151935f1 = reviewed_commit，修复提交 6d93cfdb7）完成。
完整复现、修正方向、验收映射见 [复审报告](attachments/742/review-r1/REVIEW.md)，
修复证据包见 [work-r2](attachments/742/work-r2/README.md)。

以上 r2 为 work 申报与旧例闭合事实；第二次独立 final review 仍为 **needs_fix**，
不能将具体输入通过视为全部不变量闭合。R1-04 告警已清偿；R1-01/02/03 旧例已绿，
但新的完整路径反例见 F-742-R2-01～04。T-1/T-2 重开，T-3 在修复提交上补齐联动，
当前 0/3。下一次 work 交接须先给出诊断与覆盖/设计调整（见 §4.3 和本轮报告）。

## 4. 验证

- cargo t api_gen（本 plan 测试）
- cargo tt（trans 档）
- 015-notes：auto run + v1 电池全绿 + legacy 冒烟（T-04 联动）

执行时点验证（2026-10-04~06，提交内记录）：c5d54f125 记 tt 29 红对基线 33（净 -4，
新增红隔离复跑均绿=并行 flake）；015-notes r4 电池全绿交付。

追认复核（2026-10-13，主检出 f799be601）：`cargo test -p auto-man --lib api_gen`
45/45 绿（含钉行为测试）；`cargo tt` 5649/5649 绿（1 slow）；批量回归收据
（.last-batch-regression.json，覆盖至 PLAN-766，2026-10-12）tt 腿 5629/5629 绿、
new_reds 空——742 代码合入后至今无回归归因于本计划。

### 4.1 当前独立审核（2026-10-13）

为旧任务保留原义补充 stable AC IDs（未变更计划合同）：

| AC ID | 对应 Task / 要求 | 本次结果 |
|---|---|---|
| AC-742-01 | T-1/T-2；D1 List 构造降级、可编译且保持业务值 | fail，F-742-R1-01 |
| AC-742-02 | T-1/T-2；D2/D4 companion 调用、导入与借用 | fail，F-742-R1-02 |
| AC-742-03 | T-1/T-2；D3 比较发射及回归检测能力 | partial，原错误行/归因与实编 oracle 未闭合 |
| AC-742-04 | T-1/T-2；D5/D6 返回包装与语句分界 | fail，F-742-R1-01/03 |
| AC-742-05 | T-3；Notes 电池、legacy 冒烟与双端联动 | partial；历史证据存在，修复后重验 |
| AC-742-06 | T-2/T-3；追加 D7/D8/D9 的实际行为 | partial；D7/D9 独立 Rust 控制及 tu 通过，当前 Notes 主题/VM 联动待验 |

当轮门禁：api_gen **45 passed / 1 ignored**；日常 t **5277/5278 + 1 timeout**
（plan707 独立复跑 1/1 pass）；tt 首跑有 plan730 红，完整 `--no-fail-fast`
复跑 **5649/5649 pass**，该红定向 1/1 pass；tu **862 passed / 2 ignored**；
check/build exit 0。本次真实生成/编译/执行反例仍失败，绿套件不抵消该失败。
健康检查发现 742 新增 dead-code 告警；五触面文件 rustfmt 仅 fs.rs 的存量 687 段落报格式差异。
命令、原始输出、指纹、范围与待验边界均见 [证据包](attachments/742/review-r1/REVIEW.md)。

### 4.2 r2 修复轮（2026-10-13，worktree plan-742-dev @ 6d93cfdb7）

发现闭合表（verification-contract 形态）：

| Finding | 违反不变量 | 触面路径/状态 | 修复与永久用例 | 当前结果/余缺 |
|---|---|---|---|---|
| F-742-R1-01 | 体后处理只许改代码区，字面量/注释载荷逐字节保留 | trans/rust.rs split_glued_statements / lower_list_constructs / transpile_body_stmts 全部体发射消费者 | code_region_mask 门控；oracle 用例 list_literal+brace_literal（实编+值对拍）+ control | 全关；掩码边界=单行发射面（跨行块注释/raw string 不在发射面） |
| F-742-R1-02 | 模块限定只许作用于真实调用节点：局部绑定遮蔽、字段位排除、字符串排除 | auto-man/api_gen.rs generate_api（退役整文件 pass）→ generate_api_rs a2r 体循环；borrow pass 依赖 marker 顺序不变 | 逐体限定化（endpoint 局部名/调用形态/掩码）+ 导入按实发；oracle 用例 module_literal + 既有 repository 正例 | 全关；同拼写跨形态（如局部变量恰名 db 且调模块）由局部名集遮蔽规则覆盖 |
| F-742-R1-03 | needs_result 签名的每个 return 必须类型匹配；回归须实编证明 | generate_api_rs 返回包装（?T/普通值/void × has_path/PUT/DELETE）| InlineRetWrap 五态 + void 尾合成；oracle 用例 update_note（实编+值对拍） | 全关；D3 原始错误行不可考（WIP 回滚前未提交），归因补录见 §3 T-1 |
| F-742-R1-04 | 计划引入的告警必须清偿，不得混入存量 | rust.rs NL / api_gen.rs wrap_inline_return 旧包装 / borrow_string_db_call_args_old | 三项删除；告警面与基线对照确认零残留 | 全关 |

门禁：api_gen **46/46（1 ignored，三连跑稳定）**；`cargo tt --no-fail-fast`
**5649/5649**（首跑 fail-fast 红=ffi_dual 满池负载 flake，批量回执两轮在案，
建议升 KNOWN-DEBT）；auto-man 全 lib 与 master 基线同款 3 预存红
（merged_api_client_crud_fallback / test_shell_pack_lib_freshness /
plan593_index_css——**候选 KNOWN-DEBT 登记，非本计划面**），零新增红；
rustfmt 触面两文件干净；审核 replay 提交态重放 ALL-PASS；Notes v1 电池
fails=0（36 断言）。测试基建：generation_consumer_identity 未持
a2r_env_lock 即 set_var AUTO_RUST_WORKSPACE 的并行 receipt 竞争已补锁
（bp-root-env-lock 先例形态）。

### 4.3 第二次独立 final review（2026-10-13）

绑定同工作树 `plan-742-dev` 的已提交 `6d93cfdb73f5178b624846d46f88614ed999c158`，
base `c151935f1b50a9aefb7164771bcc70ccf4b66c9d`；实现与验证输入无漂移。
详见 [报告/反例/诊断矩阵](attachments/742/review-r2/REVIEW.md)。

| AC ID | 本次结果 | 证据 / 剩余 |
|---|---|---|
| AC-742-01 | fail | 构造/拆分旧例与转义正例通过，但下游 borrow pass 仍改字符串（R2-01） |
| AC-742-02 | fail | 字面量借用、子块名字误遮蔽外层模块、File 漏限定化（R2-01/02/04） |
| AC-742-03 | pass（可考边界） | 类别级 D3 来源/失落 WIP 归因接受；原 fixture 单比较实编；不冒称原失落输入重放 |
| AC-742-04 | fail | 原 E0308 样例实编/运行关闭；Option 变量、void 分支/fallthrough 仍 E0308，File 调用 E0423（R2-03/04） |
| AC-742-05 | partial | work 有电池摘要；本轮未运行 Notes 双端/legacy 冒烟，S10 外部驱动记录缺失；修复后绑定新提交重验 |
| AC-742-06 | partial | 同指纹 r1 D7/D9 Rust 控制及 tu 可重用；当前 Notes 主题/VM 联动待验 |

本轮 `api_gen` **46 passed / 1 ignored**；日常 `t --no-fail-fast`
**5278/5278**；`tt --no-fail-fast` **5649/5649**；check/build/两触面文件格式门 exit 0。
独立生成的 13 case 为 **8 符合预期、1 值错误、4 编译失败**（所有生成入口 exit 0）；
另精确重放旧 E0308 fixture 三 handler 实编并运行 `list=0/get=404/update=ok:0`。
四旧字符串例的 api.at/repository.at bytes 与 r1 一致。

剩余诊断：只保护 qualifier/两个体 pass 未保护下游借用；局部名集丢作用域；
整文件 qualifier 退役只迁移 JSON 入口；返回按 Some/None 文本及“出现任意 return”
判断类型与全路径完成。下一次 work 必须对完整链/入口/binding/出口矩阵作出调整并留
实际实编/值对拍证据，不能仅累加四个失败例。第二次 final needs_fix 的累计轮次保留。

### 4.4 第三次复审（2026-10-14，r3 修复提交）

实现树 `D:/autostack/.wt/lang-742/auto-lang`，`plan-742-dev`，
reviewed_commit `8c9b2d3d96d648e3a0ffa7c40b4c25694a6da85d`，
base `6d93cfdb73f5178b624846d46f88614ed999c158`，实现已提交且无漂移。
报告：[review-r3](attachments/742/review-r3/REVIEW.md)。

入口已声明缺必要 Vue/VM oracle，按技能就绪规则不进行另一完整 final 验收周期；
本轮依用户明确请求做代码调查和旧例独立重放，最终不签 pass。
`api_gen` **47 passed / 1 ignored**；新构建 exit 0；两触面格式和 crates diff 门通过。
r2 的13个api.at/repository.at精确输入 **13/13** 实编运行符合预期；本轮有效补充四例
为两通过、字符串值错误、嵌套借用 E0308。合计 **17有效case：15符合预期、1值错误、1编译失败**。
日常/trans/Notes完整双端未作为本轮新门禁通过。

| AC ID | 本次结果 | 当前依据 / 剩余 |
|---|---|---|
| AC-742-01 | fail | 旧构造/拆分通过；placeholder恢复破坏合法str（R3-01） |
| AC-742-02 | fail | 旧作用域/限定化例通过；marker身份与嵌套调用借用漏遍历（R3-01/02） |
| AC-742-03 | pass（可考边界） | 沿用已接受的归因；新实编oracle通过，不声称恢复失落WIP |
| AC-742-04 | fail | Option/void/File旧例已关；合法嵌套调用生成体仍E0308（R3-02） |
| AC-742-05 | partial | work电池有摘要；Vue/VM未完成，S10记录不足以核验强杀恢复（R3-03） |
| AC-742-06 | partial | D7/D9和Vue种子宿主源码未变，已有Rust控制可重用；当前主题/VM联动待验 |

两连败诊断及实际接线已核查，仍须调整 placeholder 恢复身份和借用遍历覆盖。
本轮没有宣布未完成联动已通过，亦未因已有缺口重复运行完整日常/trans测试。

### 4.5 第四次独立 final review（2026-10-14）

审面 `2c40e1fe4673c2041ae3c366037d7eab38017d81`，基面
`8c9b2d3d96d648e3a0ffa7c40b4c25694a6da85d`；同plan-742-dev工作树。
结论 **needs_fix**，原17输入hash相同并全部通过，R3-01/02具体旧例关闭。
当前20有效输入实编运行19通过、1失败：F-742-R4-01（P2）标准库扫描在较长
标识符后缀命中a2r_std，将合法custom_a2r_std.ping()改成
crate::custom_auto_lang::a2r_std::ping()，实编E0433；custom模块控制通过。
原模块身份/handler可编译要求尚未闭合；T-1/T-2及T-3依赖证据重开，当前0/3。

本轮独立 api_gen48 pass/1 ignored，tt5649/5649；daily5277 pass/1 fail
（state lock在案99≠100，定向复跑1/1）；格式及crates diff whitespace通过。
独立S10完成创建→强杀/下线→同制品同目录重启→值读回，Vue smoke13/13。
当前后端008f0045…与work早先制品不同，身份和runtime主检出路径如实绑定；
DB同路径前后文件hash不同，不将恢复过程不写文件作为新要求。
VM标准驱动45秒未ready，保留交互未验边界，不判产品故障、不新增VM19门禁。
未验Upload/db候选维持原调查范围。完整映射、命令、输入/生成物、身份/限制见
[review-r4报告](attachments/742/review-r4/REVIEW.md)。已通过事实保留，不以记账抹去。

### 4.6 第五次独立 final review（2026-10-14）

审面 `2174f0de28dd786369b2e44e94bfa1a1ac806499`，基面
`2c40e1fe4673c2041ae3c366037d7eab38017d81`；同plan-742-dev工作树。
结论 **needs_fix**。R4-01后缀模块名反例/控制关闭；旧20输入源hash及返回值相同。
新增四组合后，24有效例23实编运行通过、1失败：F-742-R5-01（P2）
split_top_level_args把表达式 `<` 当泛型角括号，repository.choose(1 < 2, s)
漏补第二实参&s，实编E0308；true与2 > 1控制通过，直接include实际repository.rs。
原AC02/04借用/handler可编译要求剩余缺口，非r5新增回归。T-1/T-2与T-3依赖版本
证据重开，当前0/3；旧例/当前Notes通过事实全部保留。

api_gen48 pass/1 ignored；daily5278/5278；tt5648 pass/1 timeout（plan707
在案同款，定向1/1通过），不将整档称全绿。格式与crates whitespace通过。
当前CLI4af3e449…/后端6b4bcb6a…独立S10强杀重启值读回及Vue13/13通过。
收口Notes有并行源码/文档WIP，制品hash未变；上述运行结果不冒称当前Notes源码工作区
整体通过，漂移详情见review-r5/notes-drift.json。742审面/配置保持稳定。
VM交互沿用未验边界，本轮未跑、不新加19场景门禁；Upload/db候选仍原范围。
诊断、原始输入/生成物/真实companion实编与门禁见
[review-r5报告](attachments/742/review-r5/REVIEW.md)。

### 4.7 第六次独立 final review（2026-10-14，r6 修复提交）——**pass**

审面 `8d59f5e88962a51a7e608b5ffe9bd8d03ff22086`，基面
`2174f0de28dd786369b2e44e94bfa1a1ac806499`；同 plan-742-dev 工作树，diff =
api_gen.rs 单文件 +72/-2。审核人 Kimi Code 独立会话（非前轮 Codex 上下文，
未参与实施）。

| AC ID | 本次结果 | 依据 |
|---|---|---|
| AC-742-01 | pass | 旧例全绿，replay 24/24 独立重放逐字段一致 |
| AC-742-02 | pass | F-742-R5-01 关闭：重放 compare_less 转 ok + 审核方新探针 cmp_glued_source（用户无空格源 `1<2` 经发射器归一化为 `1 < 2`，证明 glued-`<` 启发式在真实源下安全）与 cmp_nested_call_arg（比较+嵌套借用组合 `choose(2 > 1, &crate::repository::path())`）均实编运行通过 |
| AC-742-03 | pass（可考边界） | 沿用已接受归因 |
| AC-742-04 | pass | cmp 双端点实编运行=2，精确形态 `choose(1 < 2, &s)` |
| AC-742-05 | pass（绑定注记） | 提交态电池 S0–S19 fails=0 收据在案；api.rs mtime 绑定被并行会话再生成覆盖，持久日志/制品保留，不冒称 Notes 当前工作区整体通过 |
| AC-742-06 | pass（边界沿用） | Vue 13/13、S10 为已验事实；VM 交互未验边界沿用，不新增门禁 |

门禁：api_gen 49 passed/1 ignored（正确拼写 `cargo nextest run -p auto-man
--lib api_gen`）；tt 5648/5649（plan707 满池超时 flake 定向 1/1 绿，同 r5
处置）；check exit 0；auto-man 告警 27 ≤ master 基线 29；触面 rustfmt 干净。
发射器形态规律经源码实证（rust.rs:3960 二元操作符恒 `" {} "` 空格填充）。
零新 finding；R1–R5 历史 finding 全部保持关闭。T-1/T-2/T-3 重计 3/3。
完整证据、探针细节、债务候选（plan734 非 lib 测试目标预存编译失败等）见
[review-r6 报告](attachments/742/review-r6/REVIEW.md) 与
[baseline](attachments/742/review-r6/baseline.json)。

## 5. 已知延后

- D-742-1: `use api: Note` 于 db.at/repository.at 报 'Note' undefined
  （api.at 本地定义+构造正常）——跨模块导入类型构造解析缺陷，单独计划处理；
  本计划以"映射留在 api.at"绕开。

## 6. 合并血缘与追认说明

实现于 2026-10-04~06 在 lang-742 worktree（已退役）`plan-742-dev` 分支完成，
a6e108f60 合入 v0.6-dev，再经 PLAN-775 收敛（774951052 integrate v0.6-dev on v0.5
main）进入 master；知识沉淀 e35f9e99b（auto-man api-generation-integrity）。
PLAN-745 交接注记（0bc1bea01）确认 742 合并面对其 27 处共现预存红零贡献。
计划簿记当时未随合并回写，2026-10-13 本记录为对账追认（reconcile）：代码/测试/
端到端证据已在 master 逐项核验（见 §3/§4），非重新执行。

## 7. 复审记录

- 2026-10-13 work 追认对账（stage: work | 742 | rev 1 | pass | worktree 已退役
  （lang-742，历史 plan-742-dev）| base a6c0f9691 | code 至 67d655f76，现随
  v0.6-dev 收敛在 master f799be601 | dep: a2r-std 同分支面 | T-1/T-2/T-3 全勾
  | 证据：§3/§4（钉行为测试 api_gen.rs:5449 绿、tt 5649/5649、auto-os c4c848d
  r4 电池交付记录、批量回回收据覆盖）| blockers: 无 | next: review（/auto-plan:review
  按 §3 任务清点 + §4 验证面独立复核；已知延后仅 D-742-1）

## 9. 复审记录

- stage: review | review_scope: final | reviewer_identity: Codex /root（独立审核上下文，
  未参与 742 实施，无子 agent）| plan_id: 742 | plan_revision: 1 | outcome: needs_fix |
  worktree_path: D:/autostack/auto-lang（historical landed review；原
  D:/autostack/.wt/lang-742/auto-lang 已退役，merge 祖先检查 exit 0）| branch: master |
  reviewed_commit: c151935f1b50a9aefb7164771bcc70ccf4b66c9d |
  base_commit: 59f2fbc2a0e6f216f6bfa5b4cd3436ba068927dc（742 历史 merge 第一父）|
  dependency_revisions: a2r-std=同 HEAD，auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，
  Notes=1391bd7eb2f6b3b08e9841d30b25125e47ced058（历史 r4=3fecc99）|
  spec_inputs: baseline.json 内 canonical 五文件 SHA256 |
  acceptance_results: §4.1（fail:01/02/04；partial:03/05/06）|
  findings: F-742-R1-01(P1 字符串被构造/语句后处理改写)、
  F-742-R1-02(P1 模块限定 pass 改业务文本)、F-742-R1-03(P2 回归样例 E0308/oracle 缺口)、
  F-742-R1-04(P3 新增 dead code 告警) |
  pending_closeout_gates: 修复提交绑定后的 T-1/T-2/T-3 及 scoped gates/Notes 联动，
  后续 merge 的知识沉淀/归档/批量到期判定 |
  evidence: [报告](attachments/742/review-r1/REVIEW.md)、
  [基线](attachments/742/review-r1/baseline.json)、
  [冻结规范增量](attachments/742/review-r1/spec-delta.md) |
  next: /auto-plan:work 742，修复并提交后再独立 review。
  入口 HEAD c394969b5 → c151935f1 的差异仅为 774 Plan 簿记，运行输入无漂移；
  742 进度重新计为 0/3，历史完成/合并事实及旧工作记录全部保留。

- stage: work（r2 修复轮） | plan_id: 742 | plan_revision: 1 | outcome: pass |
  worktree_path: D:/autostack/.wt/lang-742/auto-lang | branch: plan-742-dev |
  base_commit: c151935f1（=r1 reviewed_commit）| code_commit: 6d93cfdb7 |
  dependency_revisions: auto-down=组内兄弟 detached c39ac7f9（同 r1 记录），
  a2r-std=同分支源码 | task_ids: T-1/T-2/T-3 全勾（r2 语义）|
  evidence: §3/§4.2 + [work-r2 证据包](attachments/742/work-r2/README.md)
  （实编 oracle 绿 ×稳定三跑、tt 5649/5649、replay 提交态 ALL-PASS 四用例
  值对拍一致、Notes v1 电池 fails=0 36 断言含 S10/S15/S19、dead-code 清零、
  D3 归因补录=NOTES-001 r2 work 记录 4 类别级记录+WIP 回滚无源行）|
  blockers: 无在案未决发现；开放项两笔候选 KNOWN-DEBT（auto-man 3 预存红、
  ffi_dual 满池 flake 升级登记）随复审处置 | next: review（/auto-plan:review
  r2，独立上下文；基面=本记录 base/code，worktree 冻结待审；已知延后仅
  D-742-1 另案）

- stage: review（第二次 final） | review_scope: final |
  reviewer_identity: Codex /root（曾做 r1 审核，未参与 r2 实施，无子 agent）|
  plan_id: 742 | plan_revision: 1 | outcome: needs_fix |
  worktree_path: D:/autostack/.wt/lang-742/auto-lang | branch: plan-742-dev |
  reviewed_commit: 6d93cfdb73f5178b624846d46f88614ed999c158 |
  base_commit: c151935f1b50a9aefb7164771bcc70ccf4b66c9d |
  dependency_revisions: auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，
  a2r-std=同提交源码，Notes=1391bd7eb2f6b3b08e9841d30b25125e47ced058 |
  spec_inputs: [baseline.json](attachments/742/review-r2/baseline.json) 五 canonical SHA256 |
  acceptance_results: §4.3（fail:01/02/04；pass:03可考边界；partial:05/06）|
  findings: F-742-R2-01(P1 borrow pass 改字面量)、F-742-R2-02(P2 子块名字遮蔽外层模块)、
  F-742-R2-03(P2 Option 变量/void 出口 E0308)、F-742-R2-04(P1 File 入口调用漏迁移) |
  prior_findings: R1-01/02/03 旧例关闭但完整不变量 partial，R1-04 closed |
  pending_closeout_gates: work 的完整链/入口/出口诊断及矩阵调整、修复提交上的 T-1/T-2/T-3，
  Notes 联动；后续 merge 沉淀/归档/批量到期判定 |
  evidence: [报告与重放](attachments/742/review-r2/REVIEW.md)、
  [冻结规范增量](attachments/742/review-r2/spec-delta.md) |
  next: /auto-plan:work 742，在同树修复并提交后再独立复审。
  执行申报 3/3 退回 0/3、executing；旧记录不覆盖，累计第二次 final needs_fix。

### 9.1 两连败诊断（verification-contract §两连败检查点，r3 修复前完成）

| 剩余不变量 | 前轮修复/测试为何漏掉 | 触面路径/状态 | 设计/oracle 调整 | 当前结果/阻塞 |
|---|---|---|---|---|
| 完整发射链字面量保持 | r1 只掩码化了 qualifier 与 split/lower；borrow pass 仍整文件 find(marker)；oracle 字面量不含已限定调用 marker | api.rs 链：qualify_a2r_std/borrow_string_module_call_args；db/companion 链：strip_collection_new | 全链盘点（work-r3 README 表）；borrow pass 掩码+调用形态；qualify_a2r_std/strip_collection_new 掩码化；矩阵 oracle 加 borrow-marker 串+真实借用正例 | 全关；db.rs 三个 Plan 399 遗留正则 pass 同类暴露登记债务候选 |
| binding 真实作用域 | r1 局部名集合按整个 endpoint 扁平化；oracle 无同名子块 | qualify（JSON/File/Upload 体循环） | BodyQualifyScope 栈式帧（params 帧+每代码区 brace 一帧）；oracle 加嵌套遮蔽用例（外层限定、块内方法调用保持） | 全关；match 臂/if-let 多 pattern 绑定形态为已注记边界 |
| 全入口调用可达 | 整文件 pass 退役时只迁移了 JSON 循环；测试全 JSON | File（3931）/Upload（4054）独立体循环 | 三入口共享 scope 化限定化；oracle 加 File 模块用例；Upload 源码级接线+明示待查 | File 全关；Upload 无独立实编反例（审核亦仅要求源码检查） |
| 类型/出口完整返回 | 按 Some/None 拼写分类；saw_return 把任意 return 当全覆盖；void 裸 return 实际发射为 `return None;` | wrap_inline_return 五态 + 尾合成 | ?T 普通表达式=Option 类型→运行时 match 映射；void 映射 `return None;`；尾覆盖只认帧深 1 的顶层 return；oracle 加 void 三态+Option 变量双臂 | 全关 |

- stage: work（r3 修复轮） | plan_id: 742 | plan_revision: 1 | outcome: pass |
  worktree_path: D:/autostack/.wt/lang-742/auto-lang | branch: plan-742-dev |
  base_commit: 6d93cfdb7（=r2 审核面）| code_commit: 8c9b2d3d9 |
  dependency_revisions: auto-down=组内兄弟 detached c39ac7f9（同 r1/r2）|
  task_ids: T-1/T-2/T-3（r3 语义全勾）|
  evidence: §3/§9.1 + [work-r3 证据包](attachments/742/work-r3/README.md)
  ——矩阵 oracle 11 值编译+运行全绿；r2 replay 提交态 13/13 逐项对拍；
  api_gen 47/47×3；tt 5649/5649（满池轮红=预存 flake 家族定向复跑绿）；
  Notes 提交态绑定（CLI SHA 5e5b146c…→再生成→build→电池 fails=0→
  实驱 S10 存活）|
  blockers: 无在案未决发现；Vue/VM 联动面环境阻塞如实记录（pnpm 镜像
  网络/有头桌面 MCP）；Upload 实编反例与 db.rs 遗留正则 pass 列为
  债务候选随复审处置 | next: review（/auto-plan:review r3，独立上下文；
  审面=8c9b2d3d9，基面 6d93cfdb7，工作树冻结待审；已知延后 D-742-1 另案）

- stage: review（第三次） | review_scope: final |
  reviewer_identity: Codex /root（前两轮审核上下文，未参与本轮实施，无子agent）|
  readiness_limitations: 入口已承认Vue/VM oracle未完成，按技能作定向代码调查/重放，不能final pass |
  plan_id: 742 | plan_revision: 1 | outcome: needs_fix |
  worktree_path: D:/autostack/.wt/lang-742/auto-lang | branch: plan-742-dev |
  reviewed_commit: 8c9b2d3d96d648e3a0ffa7c40b4c25694a6da85d |
  base_commit: 6d93cfdb73f5178b624846d46f88614ed999c158 |
  dependency_revisions: auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，
  a2r-std=同提交源码，Notes=1391bd7eb2f6b3b08e9841d30b25125e47ced058 |
  spec_inputs: [baseline.json](attachments/742/review-r3/baseline.json) 五canonical SHA256 |
  acceptance_results: §4.4（fail:01/02/04；pass:03可考边界；partial:05/06）|
  findings: F-742-R3-01(P2 marker恢复改字面量)、F-742-R3-02(P2嵌套借用漏遍历)、
  F-742-R3-03(P2就绪/T3必要oracle与S10证据未闭合) |
  prior_findings: r2十三旧例全部通过；R2-01完整不变量仍partial，其余已验形态关闭；R1-04仍closed |
  pending_closeout_gates: 修复提交上的永久oracle、完整scoped gates、必要Notes双端/legacy和真实S10过程；
  后续merge沉淀/归档/批量到期判定 |
  evidence: [报告](attachments/742/review-r3/REVIEW.md)、
  [冻结规范增量](attachments/742/review-r3/spec-delta.md) |
  next: /auto-plan:work 742，同树修复并补齐work-owned oracle后重新交接。
  状态退回executing、0/3；累计三次请求均未通过，历史记录和已关闭具体输入保留。

### 9.2 r3 轮诊断扩展（回应 R3 报告"诊断从 mask 存在扩至 placeholder 身份与嵌套遍历"）

| 剩余不变量 | 前轮修复为何漏掉 | 调整 | 当前结果 |
|---|---|---|---|
| placeholder 身份恢复 | r2 掩码化只覆盖"插入"侧；恢复侧 plain replace 无法区分自插入 marker 与同拼写合法字符串——注释以"不可能拼写"豁免属错误判断 | 撤销占位符方案：qualify_a2r_std 改单趟优先级扫描（已限定>sys/sqlite 直连>通用限定>use 行），无恢复步即无碰撞；fn 级单测钉代码正例+marker 字面量负例 | 全关（replay marker_literal 逐字节保持） |
| 嵌套调用借用遍历 | r2 的 mask 修的是"字面量身份"；借用 pass 处理外层调用后 cursor 跳过整个 close，内层模块调用从未进入扫描 | borrow_pass worker 化，参数递归重扫（任意深度），外层 & 在递归结果之上插入；矩阵 oracle 加 outer(measure(s)) 双层组合（外不需借/内需借） | 全关（replay nested_borrow ok） |
| T-3 证据完整性 | r3 前归档了 S10 转录的错误片段（仅末段验证），完整转录留在 /tmp；且在 Vue/VM 未验时勾满 T-3 | 本轮：Vue 阻塞实际解除（离线安装+子模块）→ 双端 + smoke 13/13 采证归档；S10 全生命周期单转录（含双二进制身份、存储哈希前后一致、新旧 PID）；VM autotest 以"启动已验 + 套件前提写明"为边界如实记录，不冒称 | §4 原验证清单（auto run + v1 电池 + legacy 冒烟）全项达成；VM autotest 为 NOTES-001 交付面复验项，前提=有头桌面 MCP 会话 |

- stage: work（r4 修复轮，第三次自动修复轮） | plan_id: 742 | plan_revision: 1 |
  outcome: pass | worktree_path: D:/autostack/.wt/lang-742/auto-lang |
  branch: plan-742-dev | base_commit: 8c9b2d3d9（=r3 审核面）|
  code_commit: 2c40e1fe4 | dependency_revisions: auto-down=组内兄弟 detached
  c39ac7f9（notes vendor 子模块同钉此提交）| task_ids: T-1/T-2/T-3 |
  evidence: §3/§9.2 + [work-r4 证据包](attachments/742/work-r4/README.md)
  ——api_gen 48/48×2（新增 fn 级单测+矩阵 13 值全绿）；tt 5649/5649；
  r3 replay 提交态 17/17 逐项对拍（marker_literal/nested_borrow 翻转）；
  T-3：双端 auto run + smoke 13/13 + v1 电池 fails=0 + S10 全生命周期转录
  （双身份+存储哈希前后一致）| blockers: 无在案未决代码发现。边界（如实，
  非阻塞项）：VM autotest 19 场景未运行——前提为有头桌面 MCP 会话，
  VM 启动+后端联动已验证；Upload 入口限定化为源码级接线共享 helper；
  db.rs 遗留正则 pass 债务候选 | next: review（/auto-plan:review r4；
  审面=2c40e1fe4，基面 8c9b2d3d9，工作树冻结待审）

- stage: review（第四次） | review_scope: final |
  reviewer_identity: Codex /root（前轮审核上下文，未参与本轮实施，无子agent）|
  plan_id: 742 | plan_revision: 1 | outcome: needs_fix |
  worktree_path: D:/autostack/.wt/lang-742/auto-lang | branch: plan-742-dev |
  reviewed_commit: 2c40e1fe4673c2041ae3c366037d7eab38017d81 |
  base_commit: 8c9b2d3d96d648e3a0ffa7c40b4c25694a6da85d |
  dependency_revisions: auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，
  Notes=1391bd7eb2f6b3b08e9841d30b25125e47ced058，制品/runtime路径见baseline |
  spec_inputs: [baseline.json](attachments/742/review-r4/baseline.json) 五canonical SHA256 |
  acceptance_results: AC01已验形态pass、AC02fail、AC03原归因范围pass、AC04fail、
  AC05/06partial（Vue/S10当前独立pass，VM交互未验；不新增19场景硬门）|
  findings: F-742-R4-01；R3-01/02旧例已关，R3-03的S10/Vue证据已补齐，VM边界保留 |
  pending_closeout_gates: 修复模块身份边界并提交、同修复版本的依赖验证/未验边界；
  后续merge沉淀/到期批量档/归档 |
  evidence: [报告](attachments/742/review-r4/REVIEW.md)、
  [冻结规范增量](attachments/742/review-r4/spec-delta.md) |
  next: /auto-plan:work 742，同树修复F-742-R4-01、永久实编反例/控制和诊断后重新交接。
  当前executing 0/3；累计四次未通过，历史发现及已通过具体证据保留。

### 9.3 r4 轮诊断（R4 报告要求："代码区身份与完整模块/token 身份不同"）

| 剩余不变量 | 前轮为何漏掉 | 调整 | 当前结果 |
|---|---|---|---|
| 文本匹配的 token 完整性 | r3 单趟扫描修了 placeholder 身份，但 starts_with 匹配无左边界——代码区命中≠完整标识符/token 身份；`custom_a2r_std` 以 stdlib token 结尾即被误命中 | qualify_a2r_std 全臂加标识符左边界（i==0 或前字节非 [A-Za-z0-9_]）；fn 级单测 + 矩阵 nsfx 端点（custom_a2r_std.at companion，实编 nsfx=7 + 精确形态/反向断言） | 全关（replay namespace_suffix ok） |
| 系统性教训（r2→r4 连续同类） | 每轮新改的文本扫描缺其他 pass 已有的纪律之一：r2 缺恢复身份、r3 缺嵌套遍历、r4 缺 token 边界 | 全链扫描器纪律清单固化为四轴：字面量/注释掩码、标识符边界、完整 token 身份（非子串）、遍历完备（参数递归）；本计划触及的全部 pass（split/lower/qualify/borrow/qualify_a2r_std/strip）已逐轴核对 | 记录于 work-r5 README；db.rs 遗留正则 pass 债务候选仍待另案 |

- stage: work（r5 修复轮，用户逐轮转发审核结论驱动） | plan_id: 742 |
  plan_revision: 1 | outcome: pass |
  worktree_path: D:/autostack/.wt/lang-742/auto-lang | branch: plan-742-dev |
  base_commit: 2c40e1fe4（=r4 审核面）| code_commit: 2174f0de2 |
  dependency_revisions: auto-down=组内兄弟 detached c39ac7f9 |
  task_ids: T-1/T-2/T-3（r5 语义：T-1/T-2 于本修复，T-3 电池重绑 + 已立
  Vue/S10 证据沿用——审核 r4 已确认"已运行的 Vue/S10 不因记账被抹去"）|
  evidence: §9.3 + [work-r5 证据包](attachments/742/work-r5/README.md)
  ——api_gen 48/48×2；日常 5278/5278；tt 5649/5649 干净收据；r4 replay
  20/20（namespace_suffix/namespace_control/nested_twice 通过）；Notes 电池
  fails=0（CLI SHA 4af3e449…，generation 08:50:48；首轮 S0 单红=预热竞争
  复跑绿）| blockers: 无在案未决代码发现。边界沿用：VM autotest 前提
  （有头桌面 MCP 会话）未变；Upload 入口源码级接线；db.rs 遗留正则债务
  候选 | next: review（/auto-plan:review r5；审面=2174f0de2，基面
  2c40e1fe4，工作树冻结待审）

- stage: review（第五次） | review_scope: final |
  reviewer_identity: Codex /root（前轮审核上下文，未参与本轮实施，无子agent）|
  plan_id: 742 | plan_revision: 1 | outcome: needs_fix |
  worktree_path: D:/autostack/.wt/lang-742/auto-lang | branch: plan-742-dev |
  reviewed_commit: 2174f0de28dd786369b2e44e94bfa1a1ac806499 |
  base_commit: 2c40e1fe4673c2041ae3c366037d7eab38017d81 |
  dependency_revisions: auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，
  Notes=1391bd7eb2f6b3b08e9841d30b25125e47ced058，当前制品见baseline |
  spec_inputs: [baseline.json](attachments/742/review-r5/baseline.json) 五canonical SHA256 |
  acceptance_results: AC01已验形态pass、AC02fail、AC03原可考范围pass、AC04fail、
  AC05/06partial（S10/Vue当前独立pass，VM交互未验）|
  findings: F-742-R5-01(P2比较实参与字符串借用组合)；R4-01旧例闭合，旧20输入全关 |
  pending_closeout_gates: 修复实参拆分、提交永久oracle并重绑依赖验证；
  merge规范沉淀/批量到期判定/归档清理 |
  evidence: [报告](attachments/742/review-r5/REVIEW.md)、
  [冻结规范增量](attachments/742/review-r5/spec-delta.md) |
  next: /auto-plan:work 742，同树修复R5-01、更新表达式/类型token诊断与永久实编控制后交接。
  当前executing 0/3；累计五次final未通过，历史及已通过证据保留。

### 9.4 r5 轮诊断（F-742-R5-01）

| 剩余不变量 | 前轮为何漏掉 | 调整 | 当前结果 |
|---|---|---|---|
| 实参切分的 token 语义 | split_top_level_args 的角深度把一切 `<` 当泛型定界——表达式里的比较操作数吞掉了顶层逗号，后续参数错位漏借用 | 判别依据=发射器形态规律：泛型 `<` 紧贴标识符、比较 `<` 两侧带空格；仅 glued-ident 开角，闭合侧饱和递减；fn 单测 + 矩阵 cmp 双端点（失败形+同形控制）实编 | 全关（replay compare_less ok；choose(1 < 2, &s) 精确形态断言） |

- stage: work（r6 修复轮，用户逐轮转发审核结论驱动） | plan_id: 742 |
  plan_revision: 1 | outcome: pass |
  worktree_path: D:/autostack/.wt/lang-742/auto-lang | branch: plan-742-dev |
  base_commit: 2174f0de2（=r5 审核面）| code_commit: 8d59f5e88 |
  dependency_revisions: auto-down=组内兄弟 detached c39ac7f9 |
  task_ids: T-1/T-2（本轮修复）；T-3 电池重绑（提交态脚本 S0–S19 fails=0，
  CLI SHA cc57d2c9…；Vue/S10 制品验证经 r5 审核确认）|
  evidence: §9.4 + [work-r6 证据包](attachments/742/work-r6/README.md)
  ——api_gen 49/49×2；tt 5649/5649；r5 replay 全 case 通过（compare_less/
  compare_greater/compare_control/multiline_literal）| blockers: 无在案
  未决代码发现。边界沿用：VM autotest 前提；Upload 源码级接线；db.rs 遗留
  正则债务候选；另见本轮发现的 notes 仓另一会话 S20 WIP 路径缺陷（非
  742 面，work-r6 README 注记）| next: review（/auto-plan:review r6；
  审面=8d59f5e88，基面 2174f0de2，工作树冻结待审）

- stage: review（第六次 final） | review_scope: final |
  reviewer_identity: Kimi Code（独立会话，未参与 742 任何一轮实施；
  验证经两个只读子 agent + 本会话直接检查）| plan_id: 742 |
  plan_revision: 1 | outcome: pass |
  worktree_path: D:/autostack/.wt/lang-742/auto-lang | branch: plan-742-dev |
  reviewed_commit: 8d59f5e88962a51a7e608b5ffe9bd8d03ff22086 |
  base_commit: 2174f0de28dd786369b2e44e94bfa1a1ac806499 |
  dependency_revisions: auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，
  Notes=1391bd7eb2f6b3b08e9841d30b25125e47ced058，主检出收口 HEAD
  720cef365（PLAN-773 已落地，trans/rust.rs 改动与 742 面不相交，
  merge-tree 实测干净）|
  spec_inputs: [baseline.json](attachments/742/review-r6/baseline.json)
  五 canonical SHA256（a2r-std/project.md 已被 773 沉淀改写，merge 须
  rebase 式合入）|
  acceptance_results: §4.7（pass:01/02/04/05/06；03 可考边界 pass）|
  findings: 零新 finding；R5-01 关闭（重放+两枚审核方新探针实证），
  R1–R5 历史 finding 全部保持关闭 |
  prior_findings: 全部 closed |
  pending_closeout_gates: merge 规范沉淀（基于当前 canonical）、
  .autoos/specs.json 沉淀与 spec-index、批量回归到期判定、归档与
  worktree 清理（含 worktree 内未跟踪 attachments/742 scratch 副本，
  先跑 wt-guard.sh）|
  evidence: [报告](attachments/742/review-r6/REVIEW.md)、
  [基线](attachments/742/review-r6/baseline.json)、
  [冻结规范增量](attachments/742/review-r6/spec-delta.md) SHA256
  04AA21D108255F07A1F55D65EEB38A7B7D6F9F78506539E5EBD3C70013DA8803 |
  next: /auto-plan:merge 742。状态翻 reviewed、3/3；累计第六次 final
  通过，历史记录与已通过证据全部保留。

## 10. 合入沉淀收据（PLAN-742:r1，2026-10-14）

| Checkpoint | Evidence |
|---|---|
| prepared | reviewed baseline：r6 final pass（reviewed_commit 8d59f5e88，基面 2174f0de2，零新 finding）；冻结增量 SHA256 04AA21D1…；canonical 五目标 + 账本投影目标 |
| landed | rebase master（旧→新：6d93cfdb7→b01d90001、8c9b2d3d9→e60b4f023、2c40e1fe4→f3567c2f7、2174f0de2→c9846c76b、8d59f5e88→e6eb873b2；range-diff 等价 `=`）；delivery=15c12883c（e6eb873b2 的 docs/projection-only 后代，实现零改动）；`git merge --ff-only plan-742-dev` 无合并提交，master tip=15c12883c；cargo check -p auto-man 过 |
| ledger_refreshed | 工作区 D:/autostack/auto-lang（.autoos/specs.json 跟踪文件）；Regime B 守卫离线投影（HTTP store 探测无响应）：备份 specs.json.p742-merge.bak → 程序化 RMW → v12→v13 → 回读校验；新增 P742-1..7（designs×4/tests/reviews/reports，file+tags 全带，diff 107+/2- 仅目标范围）；spec-index.py 重建 INDEX.md |
| archived | docs/plans/archive/742-api-handler-body-parity.md（git mv），status: archived，completion_kind: delivered；本收据随归档提交入默认分支 |
| cleaned | wt-guard 双树 clean（auto-lang/auto-down 均 exit 0）；worktree 提交态 15c12883c 全落地后移除；branch plan-742-dev 删除（was 15c12883c）；auto-down 兄弟移除；组目录 .wt/lang-742 删除；worktree list 复核仅余主检出与他组 |

- 沉淀 canonical：api-generation-integrity.md（§体路径不变量）、auto-man/project.md、
  trans/overview.md、a2r-std/project.md（773 现版合入式）、ui/overview.md。
- 实际状态核对：任务 3/3、§0–§9 记录完整、五 canonical 均在 master（15c12883c）、
  账本 P742-1..7 回读在册、附件 review-r1..r6 + work-r2..r6 均已入库（2a5107512
  consolidation + 3efcde4da force-add）。
- 产物观察项（landing ≠ deployment）：auto-lang 为工具链仓——`target/release`
  未构建（本仓无守护进程消费 release 面）；兄弟仓消费的 dev CLI
  （target/debug/auto.exe）由消费方会话自行重建；无 web bundle 面。
- 批量回归到期判定：收据 last_covered_plan_id=766（2026-10-12），其后有
  773/774/775/776 合并且 >48h → **到期**，交接 /auto-plan:regress 主检出
  单实例执行（不在本收据内冒称已跑）。

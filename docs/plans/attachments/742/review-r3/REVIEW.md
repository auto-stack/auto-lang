# PLAN-742 rev 1：第三次复审（2026-10-14）

结论：**needs_fix**。r2 的 13 个源输入及对照全部转绿；新增矩阵 oracle 通过。
本轮仍复现字面量恢复和嵌套借用两处原合同缺口，T-3 联动/崩溃恢复证据未闭合。
保持同一工作树，退回 work，累计三次 final 请求均未通过。

## 审核基线与就绪检查

- review_scope=final；reviewer=Codex `/root`（做过前两次审核，未参与本轮实施，无子 agent）。
  plan_revision=1；[输入 Plan](plan-input.md) 来源为主检出，不读取实施树旧 Plan。
- 实施树 `D:/autostack/.wt/lang-742/auto-lang`，branch `plan-742-dev`，
  HEAD=`8c9b2d3d96d648e3a0ffa7c40b4c25694a6da85d`，
  base=`6d93cfdb73f5178b624846d46f88614ed999c158`。
  git worktree 路径/分支/提交已确认，Rust/Cargo/config/stdlib 实现干净；
  预存未跟踪 review 附件不作为未提交实现。最后检查 HEAD 和输入指纹无漂移。
- main Plan 簿记入口=`5207f0e83de515d145e42c2423a5fcddaef7deb3`；
  auto-down sibling=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`、干净；
  Notes=`1391bd7eb2f6b3b08e9841d30b25125e47ced058`；a2r-std=同工作树源码。
- [baseline.json](baseline.json) 保存源码、配置、依赖、Spec、Plan 与制品 SHA256。
  probe 使用新 Cargo JSON 的 auto_man compiler-artifact 唯一 rlib、native linked_paths，
  不复用 work 侧 probe/ALL-PASS 输出。实际 CLI SHA256 与 work 申报一致：
  `5e5b146ca7f25914411844362defb40a4cf19d002f79173751a4d2f8ae5a8d20`。
- 两连败诊断 §9.1 已阅读并对照实际修改：borrow mask、scope frame、三个体入口、
  Option match 和 void 顶层出口均有实现；“全链闭合”仍被下述反例证伪。
- **入口即承认未完成的 work-owned oracle**：共享 Plan/work-handoff 写明 Vue/VM
  未验证，却将 T-3 勾满且设 execution_done。按
  [auto-plan-review 技能](C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md)：
  “Known unfinished work-owned criteria or required oracles return to work”，
  且不再花一个完整 final-review 周期重复确认已承认未完成的项目。
  因用户显式要求继续复审，本轮做独立代码调查与旧例重放；不签发 pass，也不重跑全日常/
  trans/Notes 双端完整流程。未把缺失 oracle 改成可忽略债务。

## 前轮闭合与当轮 checks

| 检查 | 结果 / 边界 |
|---|---|
| `cargo test -p auto-man --lib api_gen` | **47 passed / 1 ignored**，3.08s；含两个实际生成→实编→运行 oracle；[api-gen.log](api-gen.log) |
| `cargo build -p auto-man --message-format=json` | exit 0；[build JSON](auto-man-build.jsonl) |
| 两触面文件 rustfmt --check；修复 diff 的 crates whitespace | exit 0；[format.log](format.log)；work-r3 README 另有一行尾空白，仅记录为文档整理项 |
| 精确 r2 13 case 独立重放 | **13/13 符合预期**；所有 api.at/repository.at SHA256 与 r2 相同；[generated-results.json](generated-results.json) |
| 本轮四个有效补充控制 | qualified_literal、直接借用正例通过；marker 字符串值错误；嵌套借用实编失败 |
| t/tt/check/tu | 未当作本轮新门禁通过；r2 的 t 5278/tt 5649 是历史证据。本轮入口已有缺失的必要 oracle，完整验收门待下一次 work 就绪后复验 |

17 个有效观察 case = 13 旧例 + 4 新控制：**15 符合预期、1 返回值错误、1 编译失败**。
所有生成入口 exit 0。每个 case 的输入/生成物指纹和 actual 在 JSON 中；
runner 保留真实生成的函数体/签名，用类型一致的 stub 隔离实编和返回值，
没有修改生成调用、没有冒称全服务器编译或 HTTP wire 运行。

r2 旧例闭合：borrow marker 字符串保持；嵌套块的外层 module 调用恢复且局部方法不误改；
void 的早返回和 fallthrough 均编译；Option 变量实际运行 `1:ok / -1:not_found`；
File 模块调用实编、metadata stub 返回 readme.txt。R1-04 dead code 已清偿，保留该事实。
R2-01 完整字面量/借用不变量仍 partial（见 R3-01/02）；R2-02/03/04 旧例及此次已验
形态可关闭，未验的复杂 pattern/Upload 实编不冒称已通过。

另调查了 indexed for 的绑定候选（probe.log 的 pair_scope/pair_control），
对照输入也未得到有效 Rust（循环缺失、字符串 Vec 初始化类型不符）。
未把这个调查当作 scope 回归证据，不要求新增 indexed-for 支持；不计入上述有效 17 case。
`original_fixture` 亦再次生成，实际实编依已运行的永久 oracle 覆盖，不重复计数。

## Findings

### F-742-R3-01 / P2：内部 marker 恢复仍修改合法业务字符串

关联：R2-01 / R1-01 完整字面量保持；AC-742-01/02，T-1/T-2。
位置：`crates/auto-man/src/api_gen.rs:502～507`，`qualify_a2r_std` 的三个 plain replace。
性质：既有字面量保持缺口未闭合，不误称所有 marker 恢复都是本轮新引入。

API 字符串：
`"__AUTO_A2R_STD_QUAL__ __AUTO_A2R_STD_SQLITE__ __AUTO_A2R_STD_SYS__"`。
generate_api 成功；实际 handler 编译运行返回
`"auto_lang::a2r_std:: ::a2r_std::sqlite a2r_std::sys::"`。
对应 [源](cases/marker_literal/api.at)、[生成物](cases/marker_literal/api.rs)、
[运行结果](generated-results.json)。普通 `"a2r_std::sqlite"` 的对照现已正确保持。

代码注释称这种预存字面量“already impossible to express”，但输入是合法 str，
公开生成入口及实际编译都接受，不能据该注释豁免。代码区插入的 placeholder 与用户
字面量同拼写不是同一身份；恢复必须限制到实际插入的代码区，或采用无碰撞的处理。
保留 standalone sys/sqlite 和已限定路径的代码正例，不仅增加一种字符串 denylist。

### F-742-R3-02 / P2：借用修复跳过嵌套模块调用

关联：R2-01 的完整借用覆盖、AC-742-02/04，T-1/T-2。
位置：`borrow_string_module_call_args`（api_gen.rs:2962），末尾 `cursor = close + 1`（:3022）。
性质：原模块调用/嵌套实参要求的剩余缺口；不是要求新函数类型或新增语法。

API：`var s str = "hi"; var n int = repository.outer(repository.measure(s))`。
companion 的实际生成签名是 `measure(value: &str) -> i64`、`outer(value: i64) -> i64`，
生成调用却为 `crate::repository::outer(crate::repository::measure(s))`。
独立实编报 E0308：内层 expected &str, found String。
同签名的直接 `repository.measure(s)` 对照生成 `measure(&s)`，实际运行返回 ok。

证据：[nested_borrow](cases/nested_borrow/api.at)、[实际 companion 签名](cases/nested_borrow/repository.rs)、
[compile.log](cases/nested_borrow/compile.log)，对照 [borrow_positive](cases/borrow_positive/api.at)。
helper 识别外层已知函数后整体复制参数，再越过整个 close，未访问内层调用；mask
修复了字面量身份，没有修复遍历覆盖。对真实调用语法逐层处理，保留求值顺序、
已借用参数和实际签名；用内层需借用/外层不需借用的组合锁 oracle。

### F-742-R3-03 / P2：T-3 未完成且 S10 证据不足，不能申报 3/3

关联：AC-742-05/06，T-3；验证/就绪缺口，不判作 Notes 产品新故障。

[work-handoff.md](work-handoff.md) 明示 Vue 的 pnpm 网络失败、VM 需要有头 MCP，
没有对应双端结果；共享 Plan 却勾 T-3、execution_done。环境原因是 work 的报告，
本轮未独立确认具体网络/桌面根因，也未将其声明为“已通过”。

[s10-driven-transcript.log](s10-driven-transcript.log) 的全部内容只有 setup 隔离路径、
GET 列表及 `COUNT 1 HAS True STATE active`：**未包含创建请求、强杀命令/PID、
确认下线、重启命令/新 PID、同一存储身份或二进制身份**。
因此只能证明一次读回，不证明题名/README 声称的崩溃恢复。CLI SHA 与当前制品
相同是有效信息，不能代替该生命周期证据。电池中的 S10 本来就是外部驱动提醒。

修正就绪/任务计数；代码修复后的同一提交上完成 Vue/VM legacy/主题联动，
并保存实际 S10 创建→强杀→下线→重启→同目录读回的可重放记录与制品身份。
若仍是环境阻塞，保持未完成并说明具体前提，不把它变成 merge-owned 门或延期完成项。

## AC 与知识增量

| AC | 结果 |
|---|---|
| AC-742-01 | fail：原构造/拆分样例通过；整条 API 体发射的字面量保持仍被 R3-01 证伪 |
| AC-742-02 | fail：旧 qualifier/作用域例通过；恢复字面量/嵌套借用仍 R3-01/02 |
| AC-742-03 | pass 于前轮接受的可考材料与当前实编 oracle；不声称遗失 WIP 可重放 |
| AC-742-04 | fail：Option/void/File 旧样例关闭，但合法嵌套 body 调用仍不可编译（R3-02） |
| AC-742-05 | partial：work 电池摘要存在；双端/legacy 未完成，S10 原始记录未闭合（R3-03） |
| AC-742-06 | partial：宿主 D7/D9、Vue 种子源码未变；已有 Rust 控制可重用，当前主题/VM 联动待验 |

冻结 [spec-delta.md](spec-delta.md)，SHA256
`9DF237808934A93E5D5DCB21A8790EFC1925C4093F18AAF3EAA89A8E076AA4FF`。
SD-742-01/02 仍不能批准为完整 parity；SD-742-03 的实测 Rust 范围不变，SD-742-04
联动缺口保留。目标仍原五个 docs/specs 路径，new_spec_components=[]，GOAL-003/007。
canonical、ledger 未修改；D-742-1 继续另案。work 的 db.rs 遗留 regex 候选仅作源码观察，
本轮未复现，不作为新阻塞或已确认完成规则。

## next

`/auto-plan:work 742`：同树修复 R3-01/02，补相应永久实编/值 oracle；诊断从“mask
存在”扩至 placeholder 身份恢复与真实嵌套调用遍历。T-3 完成必要 oracle 后再交接，
不能继续以 3/3 掩盖承认未完成的项。T-1/T-2/T-3 重开，executing、0/3。
历史修复与通过样例保留；本轮没有合并、归档或改实现。

独立重放：同目录 `replay.ps1 -Build`，随后 `execute-generated.ps1`。
工具退出 0 是观察完成；逐项比 expected/actual，不能把全 case 有输出称 ALL-PASS。
下一次修复重新生成和绑定新提交，禁止用本轮 probe 或生成二进制冒充修复证据。

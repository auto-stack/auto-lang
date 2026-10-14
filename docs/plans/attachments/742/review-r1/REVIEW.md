# PLAN-742 rev 1：独立 final review（2026-10-13）

结论：**needs_fix**。基础修复及历史合并存在，但当前发射后处理会破坏合法业务字符串，
计划自己的回归样例仍产出不可编译 handler；现有绿测试不足以满足 handler body parity。
退回 work，重新闭合 T-1/T-2/T-3。没有修改实现或批准新的 canonical Spec。

## 基线与范围

- reviewer：Codex `/root`，本次独立审核上下文，未参与 742 实施；没有子 agent。
- review_scope：final；plan_revision：1；输入 Plan 冻结为 [plan-input.md](plan-input.md)。
  本次仅规范化验收 IDs、知识增量元数据、进度与审核记录，不改变原任务语义。
- 实际检出 `D:/autostack/auto-lang`，`master`，入口 HEAD `c394969b50f0c51e5005931b897440c3b92c42a8`。
  期间 HEAD 变为 `c151935f1b50a9aefb7164771bcc70ccf4b66c9d`，仅 774 Plan 簿记更新；
  运行源码、Cargo 输入与验证配置无 WIP/commit drift，故证据有效绑定最终 HEAD。
- 历史 742 worktree 已退役；`git merge-base --is-ancestor a6e108f60 HEAD` exit 0。
  历史 merge `a6e108f60b0c36a1ffdfea4410a3b11d81eda865`，diff base
  `59f2fbc2a0e6f216f6bfa5b4cd3436ba068927dc`；Plan 原始 base
  `a6c0f9691dbb8e176b028291de758bbffa499fe3`。按历史落地审核例外在主检出验证，
  未创建 reviewer checkout。D7/D9 的逐提交 diff 也已核查，不能只依 merge net diff 漏掉并行已纳入的改动。
- auto-down 当前 `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`、干净；a2r-std 为本仓同 HEAD 源码。
  Notes `D:/autostack/auto-notes` HEAD `1391bd7eb2f6b3b08e9841d30b25125e47ced058`，
  与 auto-os gitlink 相同。auto-os/apps/015-notes 未初始化，不能把那里当运行 app。
- 已核对 auto-os `c4c848d` → Notes `3fecc99` 的历史交付记录，以及 Notes 后续复审/修订。
  当前 NOTES-001 已到 rev 8、execution_done；历史 r4 宣告不等于 rev 8 当前独立验收。
- 指纹和 Spec 输入见 [baseline.json](baseline.json)。构建 public probe 从 Cargo JSON
  的 `auto_man` compiler-artifact 唯一绑定 rlib，带 build-script native 库路径，非随机 rlib glob。

## 验收映射

| AC（原任务语义归一） | Task / 实现 | 本次结果 / 证据 |
|---|---|---|
| AC-742-01：List 构造在 handler 体正确降级，产物可编译并保持业务值 | T-1/T-2；rust.rs `lower_list_constructs` | **fail**；真正 List 构造已降为 vec，但字符串 `List<int>.new([])` 被改成 `vec![]`；F-742-R1-01 |
| AC-742-02：crate companion 模块调用、导入及借用适配 | T-1/T-2；api_gen `generate_rust_server`/`borrow_string_module_call_args` | **fail**；companion 先序、路径与借用 pass 存在，但字符串 `repository.filename` 被改写；F-742-R1-02 |
| AC-742-03：D3 比较发射正常，失败回归能检测三缺陷 | T-1/T-2；742 api_gen test | **partial**；当前样例 `docs[0].title == title` 发射正常；`!contains("== ==")` 不能覆盖 `a == b == c`，原 D3 错误行仍未录入。需补原错误/归因证据，不杜撰一种新支持要求；F-742-R1-03 涵盖回归 oracle 缺口 |
| AC-742-04：追加 D5/D6 的返回包装、语句分界保证 handler 可编译 | T-1/T-2；`wrap_inline_return_mode`/`split_glued_statements` | **fail**；Some/None+path 样例正例存在；粘连 pass 拆开字符串，且 test 的普通值+path 返回生成 E0308；F-742-R1-01/03 |
| AC-742-05：015-notes auto run、电池、legacy 冒烟联动 | T-3；Notes 历史证据/依赖 | **partial**；历史落地及后续电池记录可追溯，当前复审未启动 Notes 双端；框架缺陷修复后必须绑定新提交重跑关联联动面 |
| AC-742-06：追加 D7～D9 的克隆事务、mkdir_all、主题同源和目录枚举 | T-2/T-3；sqlite.rs/fs.rs/vue.rs/runtime shim | **partial**；SQLite clone 事务可见性/rollback/commit、owned String 输入，mkdir_all、dir_names 排序/忽略点名/缺目录返回空的独立 Rust 控制 **pass**；tu 862 pass。当前 Notes 主题/VM 联动待修复后复验，不冒称本次运行过 |

## Findings

### F-742-R1-01 / P1：体后处理把字符串当语句/构造改写

位置：`crates/auto-lang/src/trans/rust.rs:775`、`:813`、`:921`。
归属：742 新回归；AC-742-01/04，T-1/T-2，连带 T-3。

公开 `generate_api(root, "rust")`，非薄 GET handler 返回 `Probe { text: msg }`。
合法输入 `var msg str = "List<int>.new([])"` 生成 `"vec![]"`，生成 handler
实编执行返回 `vec![]`。输入 `"} return secret"` 被拆成多行，下一行又被 return
包装 pass 处理为 `return JsonResponse(secret".to_string());`，Rust 报 unexpected
closing delimiter。两个例子均 generation Ok；没有 fail closed。

修复：让集合降级和语句分界按 AST/语法 token 工作，或至少正确跳过字符串/字符/注释及转义，
不能按原始文本任意命中。加入实际生成、编译和返回值对拍，保留真正 List 和多出口正例。
源与生成物在 [cases/list_literal](cases/list_literal/api.at)、
[cases/brace_literal](cases/brace_literal/api.at)，结果 [generated-results.json](generated-results.json)。

### F-742-R1-02 / P1：companion 点调用改写污染业务文本

位置：`crates/auto-man/src/api_gen.rs:995`。
归属：742 新回归；AC-742-02，T-1/T-2，连带 T-3。

同项目存在有效 `repository.at`，API 业务字符串 `"repository.filename"` 生成
`"crate::repository::filename"`。生成成功，handler 实编运行确认返回内容改变。
当前判断仅做名称前字符边界，未区分字符串或模块绑定，甚至不要求是调用。

修复：在模块调用节点做限定化，或用识别 Rust 字面量和完整调用语法的处理，
同时验证嵌套参数、相同拼写的业务字符串与局部绑定。可重放源见
[cases/module_literal/api.at](cases/module_literal/api.at)。普通字符串
[control](cases/control/api.at) 编译与返回值对照通过。

### F-742-R1-03 / P2：742 的“可编译 Rust”样例实际仍报 E0308

位置：`crates/auto-man/src/api_gen.rs:5449`（测试），`:4148`/`:2577`（返回包装）。
归属：原验收缺口，不把它谎称为 742 新引入的所有返回类型错误；AC-742-03/04，T-1/T-2/T-3。

重跑现有测试为绿，但其 `PUT /api/notes/:id` 的 `update_note -> []NoteDocument`
生成签名 `Result<JsonResponse<Vec<NoteDocument>>, StatusCode>`，尾部仍为
`return JsonResponse(docs);`。保留原函数签名和 handler 体，以等价 wrapper/type stub
实编确定报 E0308。证据 [original-fixture/api.rs](original-fixture/api.rs)、
[compile.log](original-fixture/compile.log)。这是隔离 handler 编译，未冒称全服务器构建。

修复：协调 `needs_result` 的签名与普通值返回包装，保留 ?T 的 Some/None→200/404。
让该回归测试实际编译生成 handler/服务或增加真实实编 oracle，不能仅匹配几个 substring。
D3 也需核实原始错误行或可审查归因；`== ==` 字符串断言不能单独证明链式比较问题已关闭。

### F-742-R1-04 / P3：本计划引入的未使用代码告警未清

`rust.rs:768` 的 `NL`；`api_gen.rs:2782` 的 `borrow_string_db_call_args_old`；
替换调用后遗留 `wrap_inline_return`（`:2568`）。差异与编译日志确认这些遗留来自 742。
对应健康门/T-2；删除不用的旧 pass/常量或恢复必要调用并验证，不能把新增告警混入存量告警。
本仓其他大量告警另属当前基线，不扩大为 742 修复范围。

## Checks 与边界

| 命令 / 方法 | 原始结果 |
|---|---|
| `cargo test -p auto-man --lib api_gen` | 45 passed，1 ignored；[api-gen.log](api-gen.log) |
| `cargo t` | 5278 run，5277 passed，plan707 wait test 120s timeout，1544 skipped；[daily.log](daily.log) |
| `cargo t plan707_wait_generator_park_drive_count_static` | 1/1 pass，1.070s；独立复跑不能把首次整档改称全绿；[plan707-retry.log](plan707-retry.log) |
| `cargo tt` 首跑 | fail-fast：434/435 passed，plan730_commit_target_matrix 409≠403，5214 未运行；[trans.log](trans.log) |
| `cargo tt --no-fail-fast` | **5649/5649 pass**，119.390s，1 slow；[trans-complete.log](trans-complete.log) |
| `cargo tt plan730_commit_target_matrix` | 1/1 pass；与完整复跑一致，记录为本轮不稳定观察，无证据归因为 742；[plan730-retry.log](plan730-retry.log) |
| `cargo tu` | **862 passed，2 ignored**，测试段 0.88s；[ui-gen.log](ui-gen.log) |
| `cargo check -p auto-lang` / `cargo build -p auto-man` | exit 0，仍有存量告警及 F-742-R1-04；[check.log](check.log)、[auto-man-build.jsonl](auto-man-build.jsonl) |
| 实际生成 + handler 实编运行 | control pass；两个字符串值错误；brace 字符串编译失败；[probe.log](probe.log)、[generated-results.json](generated-results.json) |
| D7/D9 实际源码运行控制 | clone共享同一事务/回滚/提交、owned String、目录枚举控制 pass；[runtime-controls.log](runtime-controls.log) |
| 触面五文件 `rustfmt --check --edition 2021` | exit 1，仅 fs.rs 145/197 的 PLAN-687 存量格式；742 发射器/API/Vue/sqlite 无格式 diff；[format.log](format.log) |

未触 aavm 或 docs/schema，未运行 taa/docs_gen 单独档。没有 per-plan cargo tf 要求；
批量回归属于 merge closeout（历史收据只作导航）。没有当轮 HTTP/Vue/VM Notes 实跑，
AC-742-05/06 联动部分 pending，不能用 r4 提交标题或 rev 8 执行摘要代替独立验收。

复现：主检出执行 `& docs/plans/attachments/742/review-r1/replay.ps1 -Build`。
脚本的 exit 0 只表示生成/观察流程完成，必须比较记录的 expected/actual；不宣称 ALL-PASS。
原 fixture 编译命令：`rustc --edition=2021 --crate-type=lib docs/plans/attachments/742/review-r1/original-fixture/runner.rs`。
修复后需绑定新的工作树/提交重放，不得拿本轮预修复 probe 二进制作为修复证据。

## 知识与下一步

[冻结规范增量](spec-delta.md) SHA256
`0B6E59EA24DA9131ACF7A29B591FBBE9DBEFF371EAE66A525BEA00AFD9C03EB0`。
e35f9e99b 的 API 知识已先行落地；SD-742-01/02 尚有反例，本轮不批准沉淀。
SD-742-03 的克隆共享和目录名称边界已运行核查，SD-742-04 仅源码/tu 证据，Notes 当前联动待验。
D-742-1 在历史 Notes 中明确属于另案，补录债务；不能声称本计划已解决。

next：`/auto-plan:work 742`，恢复历史实现树所有权或从当前 master 建回 742 工作树，
按发现修复并提交；T-1 必须有实际实编/值对拍 oracle，T-3 重验关联 Notes 联动。
新 HEAD 上再独立 review。当前 Plan 为 executing、0/3；历史交付事实和旧证据保留。
merge/archive/ledger 与工作树清理仅在后续通过后收口。

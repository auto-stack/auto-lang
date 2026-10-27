# PLAN-828 r2 — r21a 受影响面独立复审

2026-10-21。**affected_outcome: PASS**；F11-S01/function-id、F10-S02/instruction-id 两项 P2 闭合，本轮无新增 finding。**整体仍 executing 0/11**：其他已登记 work-owned OPEN 未完成，本次不授整份计划 reviewed 或交付结论。

## 冻结基线

- 用户指定受影响面复审；读取主检出当前 Plan，revision 2、最新 work 为 in_progress。原实施树 `D:/autostack/.wt/lang-828/auto-lang`，分支 `plan-828-dev`。
- reviewed_commit `3d14a15d613dbd79724c7dd0ac24917dc5169a28`，repair base `e083f2a28ec4d8df05caf5013d502b0c16c0ce0d`；原合同基面 `f79b1b83591310b45b3828dd300e6d368987d6c0`。修复只改 cache.rs 的完整来源字段比较，以及永久回归文件（新增六个实际测试）；session 的选择/写回、执行器和映射算法未改。
- reviewer 为本聊天 Codex `/root`，独立于实施 r21a 的上下文，沿用上轮审查的原始输入与预期；未创建 agent/聊天，不虚构模型或会话身份。代码只读、测试在原树进行。
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` 未消费。入口 status 为空；[冻结基线](baseline.json.txt)记录当前提交、分支、库/锁/配置、Spec/技能与输入哈希，[Plan 输入](plan-input.md)保留复审前内容。结束状态另见 [final-state.json.txt](final-state.json.txt)。

## 闭合判断

| Finding / AC / V / task | 独立验证与结论 |
|---|---|
| F11-S01/function-id，P2；AC-05、V-02/04/07、T-02/07 | `cache.rs` 的 v2 来源比较现在要求 `sp.func == c.func`；同一谓词实际供 restore primary→repair 选择和 persist survivor/repair 复核。上轮 wrong-func 无 repair 原例现在 admitted0，执行准确为 func0/39..44；已有好 repair 时 admitted1 且同样正确。合法 v2 同源/位移和 legacy 错编号拒绝控制保持。**CLOSED** |
| F10-S02/instruction-id，P2；AC-05、V-04/07、T-07 | 现在要求 `i == ci`，不再丢弃指令编号。上轮 `[0,1,2]→[1,1,2]` 真实 mutation 在候选选择前拒绝；有 repair 时 admitted1；无 repair 首次 admitted0，冷恢复写回后第二次 admitted1。完整来源表与当前冷编表对齐，后置 verifier 不再承担候选选择/写回的证明。**CLOSED** |

不仅重放旧例：独立新增三函数矩阵，main 的合法 func=1，修改为同一 snapshot 内另一合法索引0或越界 MAX；分别与重复指令编号及两轴同时损坏组合，覆盖同源/位移、好 primary+坏 repair、坏 primary+好 repair、坏 primary 无 repair。**24/24**：可用候选首轮恢复3单元；无 repair 时首轮正确回冷、第二轮恢复3单元；session 与公开 execute_program 的 trap 均与当前源冷编完整 SpanInfo 相等。合法非零函数编号未误拒。

该变更在已有当前源证据下比对整个消费元组（instr、func、映射后 start/end），与 legacy 分支的完整表比对保持一致。形状、UTF-8、frame、重锚与 trap 覆盖原检查仍保留；无来源校验绕过、签名/授权缩面或平行判定路径。

## 实际执行证据

cwd 均为原实施树，单线程定向测试；本目录脚本直接链接当前 cargo 产物，库路径/哈希绑定于 baseline。

| 命令 / 方法 | 本轮结果 / durable raw |
|---|---|
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_r1 -- --test-threads=1` | **82/82**，exit0，6.32s；[raw](permanent-output.txt)，六个新回归实际执行，非仅列名 |
| `run-controls.ps1 -Source binding-controls.rs -Filter binding_review_`，上轮源码原样重放 | **6/6**，四个失败全部转绿；[raw](binding-output.txt) |
| `run-controls.ps1 -Source adjacent-controls.rs -Filter adjacent_review_` | **1/1、24格全过**；[源码](adjacent-controls.rs)、[raw](adjacent-output.txt)；含实际 mutation、公开执行与二次恢复断言 |
| `run-controls.ps1 -Source applicability-controls.rs -Filter app_review_` | **5/5**，原16格选择矩阵/错误端点/legacy控制全过；[raw](applicability-output.txt) |
| `run-controls.ps1 -Source sidecar-generations.rs -Filter gen_review_` | **8/8**，多代来源、反复 restore/edit/pin、失效 mint 修复/选择均过；[raw](generation-output.txt) |
| `run-controls.ps1 -Source restore-sidecar.rs -Filter sidecar_review_` | **6/6**，零函数/缺失/损坏/非覆盖/错身份及恢复修复保持；[raw](sidecar-output.txt) |
| `run-controls.ps1 -Source original-replay.rs` | **62/62**，186/528/12/4/9/72各矩阵全过；[raw](original-replay-output.txt) |
| `cargo test --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828 -- --test-threads=1` | **17/17**，exit0，13.18s；[raw](original-suite-output.txt) |
| `rustfmt --check --edition 2021 --config skip_children=true` 两个修改文件；`git diff --check e083f2a28 HEAD` | 均exit0；[fmt](fmt-output.txt)、[diff](diff-check-output.txt) |

集合相互重叠，不相加宣称独立验收 case 总数。82、62、17以及各过滤集的原始总数均可核对。core22/798兼容档未重复跑：本轮完整 diff 只改 session 专用来源谓词和回归；core、798旧消费实现、依赖锁和验证配置未变，相关上轮记录保留为历史，不冒称本轮重新执行。实际受影响 session/restore/persist/执行路径均使用新提交重新验证。

probe 的四条既有 warning（SpanInfo import、new_len、context_revision、history_entry）未变，仍归 F-14；依赖 shim-metadata 的旧 doc comment warning 单列。没有本轮新增 warning，不能据此宣称整个计划已零告警。无实施改动或修复混入审查。

## 规范、状态与边界

- [SD-01..06冻结文本](spec-delta-input.md) SHA256 `ed0077a2a9bdf622db527d59bf4e12bbfaefb1df4f51df212768cb041d8a206e`，与 r20a 相同；相关 canonical Specs、锁及配置逐项哈希比较变化0。frontmatter 的精确 Spec 路径及 GOAL-002 保留，plan_revision仍2，没有新的语义合同或缩面。
- SD-04/05 中本轮来源适用性/修复选择子不变量通过；其他受前端、存储、正式入口和具体文档交付阻断的 SD 内容保持未通过，不发布 canonical/ledger，也不以旧 coverage/contract 声明代替规范验收。
- AC-05/V-02/04/07 的本次受影响子集 PASS；不等于整个 AC-05、T-02/T-07 完成。F10-S01 保持 CLOSED；F10-S02/F11-S01 现已验证的 sidecar 来源适用性/选择/修复不变量 CLOSED。F-10 的 A07/实际依赖/GC、F-11 的 changed-only持久化/manifest 驱动恢复另有 OPEN，未被本次关闭涵盖。
- 整体 F-01/02/05b/09/10其余/11其余/12/14 仍 OPEN；任务表保持 executing0/11，T-10整份最终验收 pending。下一责任角色为同树 work，继续完成这些既有必需项。
- 最终代码、验收和规范审查通过后，merge-owned 冻结根裸 `cargo t` + `cargo tv` 及必要触面/独立workspace/consumer门、Spec/ledger/归档/guard仍必需；tf为主检出批量档。本轮未运行整档/未合入/归档/清树，不宣称计划交付完成。

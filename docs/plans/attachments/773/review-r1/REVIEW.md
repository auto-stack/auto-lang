# PLAN-773 独立 final review R1

- 日期：2026-10-14；stage: review；review_scope: final；outcome: **needs_fix**。
- reviewer_identity：本次 Codex 独立复审会话，未参与原实现；未声称另有模型或子 agent 复审。
- plan_revision: 1；reviewed_commit: `284acbfb4340c01ed4f5464257d9b8235bf8c223`；
  worktree: `D:/autostack/.wt/lang-773/auto-lang`；branch: `plan-773-dev`。
- 历史交付：`677e84771d2b3bfce6065260b2e1cc5bcf34483a` 已合入，后续
  `129c4474f99eb3f1d45077e17b030b1a2ec5a7a5` 亦是 master 祖先。
  当前核心/adapter/CLI/vendor 相对后一提交没有差异，不能沿用原工作侧较早提交的验收。
- 原 lang-768 工作树及分支不在 inventory；这是有 Git 祖先证据的历史合入，非
  “树不存在所以已交付”的推断。依据 merge 技能建立 lang-773 收口树，当前保留供修复。
  建组脚本在本环境走 WSL Git，随后已用 Windows `git worktree repair` 修复路径注册；
  两仓工作树当前干净，未创建 junction/symlink。
- 外仓只读依赖：`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；协议裁判仍是
  vendored `fba6563ed2148ce85e68208863159b4ccccac710` 的三文件快照，指纹实测通过。
- 完整源/配置/依赖锁/CLI 指纹见 [baseline.json](baseline.json)。共享缓存仅用于产物，
  构建和执行 cwd 都在 lang-773；CLI 构建后复制到该树，按冻结 SHA 使用。
  主检出其他会话的 WIP 与并行合入没有纳入本轮实现基线。

## 结论与门禁

试点 21 case / 24 check 全部 PASS，协议测试 57/57 PASS；独立反例证明全绿
不能覆盖已承诺的判定、只读、选择、身份、资源和产物交接边界。下列发现均是
原合同缺口，不是新需求，也不能登记成允许延期的完成项。

| 检查 | 本轮实际结果 | 原始证据 |
|---|---|---|
| `cargo test -p auto-test-case --offline --locked --target-dir D:/autostack/auto-lang/target` | 57 PASS，0 ignored；包含三文件指纹守卫 | [protocol-tests.log](protocol-tests.log) |
| `cargo build -p auto-lang --no-default-features --features test-cases --bin auto-case --offline --locked --target-dir D:/autostack/auto-lang/target` | exit 0；auto-lang 218 warnings，shim-metadata 1 warning，未宣称 warning-clean；没有扩修无关警告 | [headless-build.log](headless-build.log) |
| `cargo tree -p auto-test-case --edges normal` / headless auto-lang tree | 核心仅依赖 vendor；无 iced/winit/wgpu/auto-man | [core-tree.txt](core-tree.txt)、[headless-tree.txt](headless-tree.txt) |
| `cargo fmt --all -- --check` | exit 101；外仓生成物、examples 等格式差异；本计划 core/adapter/CLI 未出现在差异表。不是本轮格式通过 | [format-check.log](format-check.log) |
| 无窗口 CLI 真实 pilot | 24 PASS / exit 0；3 个双后端 case、多产物、.as 均在报告 | [pilot-report.json](pilot-report.json)、[pilot-console.txt](pilot-console.txt) |
| 独立只读正常路径控制 | test 树 3,573 文件前后 SHA 清单聚合相等、0 文件变化 | [pilot-readonly.json](pilot-readonly.json) |
| 独立反例和合法/失败对照 | 真实 CLI 16 次调用；核心/分类器另有直接 API 探针，见下表 | [results.json](results.json)、[replay.py](replay.py) |
| `cargo check -p auto-lang`、裸 `cargo t`、`cargo tv`、`cargo tt`、nextest listing | **本轮未跑完整最终门禁**。已稳定复现合同内阻断问题，修复后的提交必须重跑 §6 门禁；旧工作侧红项归因不被本轮追认 | pending on repair |

重放（在保留的 lang-773 cwd；重新构建 CLI 后可换绝对路径）：

```text
python D:/autostack/auto-lang/docs/plans/attachments/773/review-r1/replay.py --binary target/review773/auto-case.exe --scratch target/review773/replay-next --evidence target/review773/replay-next-evidence
```

反例只覆盖脚本自行创建的 disposable fixture；覆盖输入的检查未指向仓库原始
source/oracle。每次调用的原输入在 `invocation-inputs/<Case ID>/`，完整命令/exit 在
results.json；stdout/stderr 和 JSON 报告独立保存。生成的 actual 只用于同一发射器的
比较/身份控制，不作为新增语义金样或实际 Rust 编译正确的证明。

## 阻断发现

| Finding | 级别 / AC / task | 实际反例与违反规则 | 修复和永久控制 |
|---|---|---|---|
| F-773-R1-01 | high；AC-03/04；T-04/05 | `R01-success-satisfies-error`：合法 `fn main(){print(1)}` 成功发射，却以 `{stage:"run",message:{exact:"emit succeeded"}}` **PASS / exit 0**。a2r.rs 成功分支把成功编成 Failure 再走错误 matcher；语言负例没有观察到失败。 | 完成状态与断言分开，成功永远不能满足 Error；加入成功/真实失败/错阶段三侧控制。 |
| F-773-R1-02 | high；AC-01/03/04；T-03/04/05 | `R03-a2r-ignored-observations` 的 stdout/stderr/result 全为 IMPOSSIBLE，仍 PASS；`R03-avm-ignored-artifacts` 引用不存在的 expected artifact，仍 PASS；`R08-error-stream-ignored` parse error 下声明 IMPOSSIBLE stdout 仍 PASS。decode_expect 不区分 adapter，adapter 跳过已接受字段；空 contains 也可充当“强”辨识。 | 冻结真实支持表，unsupported 在 admission ERROR；支持的观测逐项比较，错误出口也不得忽略流断言；拒绝空 code/空 message 这类没有辨识能力的强 oracle。 |
| F-773-R1-03 | high；AC-03/05；T-05 | `R04-snapshot-fail-blocks-product`：emit 成功有实际文件但金样 FAIL，消费者 BLOCKED，违反 §5.2/V-04；换正确快照后消费者 PASS，却只重新执行原 source，`product_from` 传的仅是 logical path 列表用于 detail，无 actual bytes 消费。原测试显式断言错误的 FAIL→BLOCKED。 | 区分可用产物与比较状态，以真实 actual bytes/manifest 驱动注册测试消费者；未实现的公开下游消费明确 ERROR；真发射失败才 BLOCKED，金样 FAIL 不得切断交接。 |
| F-773-R1-04 | high；AC-01/07；T-07 | `R05-selector` 声称 other=NOT_RUN，但其 `other/result.txt` 已写出。CLI 在 run_suite 全部执行后才 mark_unselected；`--adapter typo` 也执行全部检查再返回 exit 1，而不是坏 selector ERROR。不能据报告认为未选程序没有运行。 | 调用 adapter 前做选择/校验，保留 declared/selected/required 状态；未选项不可执行；添加产物/副作用证明和 typo/零匹配对照。 |
| F-773-R1-05 | high；AC-05；T-02/05/07 | `R06-out-overlap` 用 fixture 根作 out，exit 0 且 `report.json` sentinel 被覆盖；`R06-json-out-overwrites-input` 把 json-out 指向 case 文件，exit 0 且合同变成报告。run_suite、persist_reports、proposal 写入均无输出与输入/oracle 根重叠/祖先 reparse 检查；admission 的 check_no_reparse 只看末端，不检查父目录，a2r oracle 读取连该检查也未调用。 | 统一 canonical 路径域/祖先 reparse 审查，所有 out/json-out/proposal 写口在执行前验证；检查虚拟 Windows 路径等价和 module_name 生成的产物路径；加入 run/proposal/已有文件失败出口的只读控制。reparse 的链路缺口本轮为源审查，未制造链接复现。 |
| F-773-R1-06 | high；AC-07；T-03/08 | `R02-oracle-before/after` 真实修改 sidecar bytes，状态从 FAIL 变 PASS，**整个 identity 相等**。oracle 指纹只哈希 ExpectSpec 的 Debug（路径+声明），不哈希 expected bytes；report 无 parser/compiler/config 版本，is_stale 只有未接入 CLI 的值相等 helper，未交付旧 report 当前身份核查。 | expected/oracle 内容、parser/compiler/config、原/提交输入身份进入报告；旧 report 核查走实际入口，逐轴漂移失效；不能用任意手造 o1/o2 helper 测试代替 sidecar 改变。补空结果 reducer/JSON 控制：core-probe 显示零结果 exit 0，summary 尾逗号 JSON 非法。 |
| F-773-R1-07 | high；AC-01/08；T-02/03/08 | core-probe 对有效 2 check suite 设置 max_expanded_checks_per_run=1，仍 load success/errors=0；`R09-5120-expanded-checks-lint` 全部单文件/单 case 上限内，5,120 checks 超过冻结默认 4,096 仍 lint exit 0。该字段仅定义/初始化，无执行/发现使用；read_file_limited 等先整文件读完再检查，a2r oracle 无 size 限制。 | 在所有对应入口执行总量和字节准入，先检查/有界读取；持久化实际生效限制；两侧控制用真实加载/runner，不只 check_limit 函数。 |
| F-773-R1-08 | high；AC-04；T-04/05 | public classify 探针：AutoError::Io 被分类 `run/code=None`；两层 MultipleErrors 内 auto_gate 也成为 run，未递归。两 adapter 将所有 AutoError 的 Err 送语言错误判断，I/O message 可被匹配成语言负例；parse/run code 又从 Msg 文本提取，非承诺的独立类型阶段通道。 | 从真实生产阶段边界取得 Observation，基础设施保持 ERROR；完整递归错误容器；用实际 I/O 注入、panic、资源、错 stage/code/message 反例检验，不能只测 catch_unwind 标准库函数。 |
| F-773-R1-09 | medium；AC-01；T-03/07 | `R07-missing-oracle-lint` 声明不存在 expected.rs，lint exit 0；run 才 ERROR。lint 只 load/decode，不 resolve refs、不验 oracle。结构校验成功不能宣传完整严格合同 lint。缺 source file、坏 anchor 等同类 resolve 缺口按源审查未全量穷举。 | lint/inspect 用共同完整 admission/resolve，文件引用与 expected 在可执行检查前验证；合法引用、坏 anchor、缺 source/expected 各有定位 ERROR 控制。 |
| F-773-R1-10 | medium；AC-08；T-07 | pilot 测试只有 `tests::test_cases_tests::pilot_suite_full_run`，没有计划规定的 vm_file_tests/a2r_tests 双聚合入口；tv profile 只包含 `vm_file_tests:: / cookbook_vm_tests:: / conformance_tests::`，因此加 test-cases feature 仍不会把 pilot 纳入 tv。work 的“旧 filter 可触达”没有 listing 证据。 | 在约定聚合路径接线并核查实际 t/tv/tt listing，harness 与 24 动态检查分别报告；或批准等价合同调整，不能仅改 feature 声称过滤命中。 |
| F-773-R1-11 | medium；AC-05/06；T-05/06/08 | legacy-map.tsv 实际 12 列没有旧 source/oracle hash、source_dir/deps identity，缺 §5.4 承诺的追溯；proposal 只有 candidate SHA，没有旧新 hash/diff/共享消费者，collect_proposals 对实际文件读失败静默忽略，proposal_cmd 有候选时未持久化 run report。 | 补旧新来源指纹与工程上下文对账；proposal 保留原/候选 hash、真实 diff、共享消费者和同源报告，读写失败传播 ERROR。范围仍为 proposal-only，无 accept。 |

F-01..11 均来自原需求，已有代码合入不减轻其验收等级。基础设施、空消息辨识、
父路径、proposal 等未穷举全组合；表内区分了运行实证和源审查，不声称验证完所有故障。
未实现通用 deadline/进程隔离、Rust compile/run、native 等仍是批准的 A 阶段非目标，
未作为新增阻断要求。vendor MIT 文本缺口沿 feasibility 保留事实，不虚构法律结论。

## 验收追溯和就绪调整

| AC | task / artifacts / evidence | 结果 |
|---|---|---|
| AC-01 | T-02/03/06/07；contract/discovery/CLI；57 测试 + F-02/04/07/09 | fail：严格 admission/完整 lint 未闭合 |
| AC-02 | T-01/02；vendor/document/raw_ranges；指纹+9 fence 测试 | partial：bytes/来源已验；F-05 路径和 F-07 字节 admission 未闭合 |
| AC-03 | T-04/05/06；真实 3 双 check pilot；F-01/02/03 | fail：独立正常判定通过，负例/actual 交接不通过 |
| AC-04 | T-04/05；负例 CLI + stage probe；F-01/02/08 | fail：强错误、基础设施、嵌套阶段边界未闭合 |
| AC-05 | T-02/05/07；正常 pilot 3,573 文件只读控制；F-03/05/11 | fail：正常路径只读不证明覆盖写口/失败出口 |
| AC-06 | T-06；21 case/24 check/map；F-11 | partial：数量、双轨/多产物/.as/诊断强化存在；完整 hash/context 对账待补 |
| AC-07 | T-03/07/08；F-04/06 | fail：实际选择和身份失效未闭合 |
| AC-08 | T-01/02/07/08；core/headless tree/构建、F-07/10 | partial：轻核心/无 GUI 已验；资源门和旧档接线待补 |
| AC-09 | T-09；本报告/冻结基线/独立反例 | 未满足 pass；结论 needs_fix，完整最终门禁待修复提交 |
| AC-10 | T-10；历史合入已证，其余 merge-owned | pending：不得规范沉淀、刷新 ledger 或归档/清理 |

T-01 调查完成保留；T-02..08 全部重开（含其依赖证据），T-09/10 未完成；
`current_step: 1 / total_steps: 10`。原完成证据作为历史保留，不抹掉 57/57 与 24/24 事实。
下一责任阶段：auto-plan-work，在同一 lang-773 树按 F-01..11 修复/补永久控制；
形成闭合表、提交新基线后重新 final review。默认分支历史不回滚，不重写历史 merge。

## 规范增量审查 / 收口边界

冻结本次待审 delta 在 [proposed-spec-delta.md](proposed-spec-delta.md)，SHA 见 baseline。
不是已批准发布的 Spec；plan_revision 不因状态/证据补记变化。

| delta | target / current baseline | 结论 |
|---|---|---|
| SD-01 | 新 auto-test-case/project.md、profile-v1.md（当前缺失） | 不通过：严格 schema/只读/报告声明与 F-02/04/05/06/07/09 冲突 |
| SD-02 | 新 auto-test-case/architecture.md、plans.md（当前缺失） | 部分事实可沉淀，整体不通过：固定来源/轻核心是真实事实，产物和资源验收待修 |
| SD-03 | trans/design/test-convention.md；源 hash 冻结 | 双轨 emit≠compile 的原则正确，F-03/11 未闭合前不发布完成声明 |
| SD-04 | vm/overview.md；源 hash 冻结 | pilot 正常覆盖可证，阶段/weak/旧档触达声明待 F-08/10 修复 |

impact metadata 的四个新增路径、两个修改路径和 GOAL-016 均有效，保持原值。
本轮未写 canonical Specs、ADR、INDEX、.autoos/specs.json，没有完成 ledger 或 archive
checkpoint。历史合入不是 reviewed pass；归档请求应在修复复审后继续。
merge 技能的要求为 “An in-contract defect cannot be downgraded merely because it is pre-existing”
（review），以及 active Plan 必须有当前 revision/commit/delta 的 final pass（merge）。
本轮依据这些门禁暂停沉淀/归档，不需要用户再次确认原范围。

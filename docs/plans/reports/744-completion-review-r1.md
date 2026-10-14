# PLAN-744 完成情况核查 R1（2026-10-14）

结论：`needs_fix`。SQLite 实现已进入 master，应用侧 r2 遗留验证已有通过证据；
必需的 VM 嵌套堆列表 oracle 尚未完成，不能将计划视为已通过最终验收。
计划退回 `executing`，仅 T-01 重开；T-02/T-03 及历史记录保留。

## 核查身份和范围

- 源码基线：master `c171c1a09740fbd61e4bf6235286a3424e2e4a81`。
- 核查路径：`D:/autostack/auto-lang`。当前无 744 专属 worktree；这是已合入历史实现的只读核查。
- `git merge-base --is-ancestor cb7a59ede HEAD` 与 `924273fa5 HEAD` 均 exit 0。
- `crates/`、`stdlib/`、`.cargo/`、`.config/` 无未提交改动；主检出存在其他计划簿记 WIP，未纳入本核查。
- Notes：`D:/autostack/auto-notes` main `1391bd7eb2f6b3b08e9841d30b25125e47ced058`；既存未跟踪 `src/front/tests/` 保留。
- 当前 CLI SHA256 与 r2 收据一致：`6e30c87f5c3eca157973ff6e2d6e8da9ddc850b4a276e7579244f0000f3524c5`。
  哈希吻合仅说明二进制身份一致，不能替代本轮运行或声明源码构建完全一致。
- 本轮为独立于文内执行记录的源码/证据审计。发现必需 oracle 缺失后退回 work；
  未重新运行单测、UI 套件、构建、格式或全日常门禁，不给出未执行的绿灯。

## 验收映射

| 验收 | 任务 | 实现/证据 | 结论和边界 |
|---|---|---|---|
| AC-01 | T-01 | `stdlib/auto/sqlite.vm.at` 五声明；`vm/ffi/stdlib.rs` 五 shim 和八 native 名；`AutoVM::remove_task`；r2 README 的 2 passed | partial：实现和辅助函数/任务寿命验证已存在；实际嵌套堆列表逐格断言及绑定的既有 VM 回归证据欠缺 |
| AC-02 | T-02 | r2 `vm-suite-scenarios.log` + `vm-suite-result.json` | retained evidence pass：19 passed/0 failed/0 skipped，suite_exit=0；VM 前端/Rust 后端 |
| AC-03 | T-02 | r2 `vm-negative-result.json` + notes `run_vm_negative.py` | retained evidence pass：13 个 check，fails=[]；陈旧保存 dirty/conflict、Discard 最新标题和 revision 等观察值吻合 |
| 依赖回填 | T-03 | notes NOTES-001 历史回填、744 r2 收据 | 已有交付记录；不代替 744 最终复审/规范收尾 |

负例脚本包含真实 HTTP 外部写者、MCP UI 操作及故障注入；失败项加入 fails，退出码随之为 1。
脚本首项名称虽为“conflict kept dirty”，实际只断言 conflict；JSON 的 dirty=true 和 R5-VM-2
同时断言 dirty/conflict 可支持原 AC-03，不能把首项名字当成额外断言。
最后“幂等”检查只观察 UI dirty=false，并未直接对比重试前后的数据库 revision；
本计划 AC-03 不要求新的幂等承诺，因此不把这点扩成 744 的新增阻塞项。

## Findings

### F744-R1-01 — 必需 VM 嵌套堆列表验证遗漏（medium，AC-01/T-01）

原测试设计要求“query读回（嵌套列表逐格断言）”。实际
`sqlite_vm_open_exec_query_roundtrip` 在 `stdlib.rs:12460` 只调用
`shim_sqlite_query_impl`，断言 `Vec<Vec<String>>`；没有调用 `shim_sqlite_query`
（`stdlib.rs:9876`）的字符串池、inner heap、outer VmRef 与 VM 栈路径。
第二项测试只检查 VM-local/task-lifetime 句柄。

应用 r2 套件默认 Rust 后端，其通过不能证明 VM query 的堆列表返回路径。
纯 VM 后端 boot 探测补充了 db.exec/db.query 的执行线索，但附件没有逐格结果 oracle。
未发现已登记的专用 sqlite VM 文件用例；cookbook SQLite 用例仍 ignore=needs sqlite。
这是原合同的验证遗漏，尚不等于已证明实现有运行错误。

修复：在 Plan 恢复 worktree 增加真正经过 query shim/VM 调用的多行多列逐格断言，
检查外层/内层列表可消费及字符串内容，覆盖空结果/错误哨兵；保留 helper 和任务寿命测试。
再确认五个公开调用的 VM 路径，而非只用 SQL 引擎的 Rust 返回值作证。

### F744-R1-02 — 既有 VM 无回归缺少绑定证据（medium，AC-01/T-01）

744 work/r2 收据仅有 scoped sqlite 2 passed，没有本计划绑定提交的
`cargo t` + `cargo tv` 结果及失败归因。Plan 775 的合流记录有其他基线/范围的门禁引用，
未建立可供 744 复用的具体提交、测试配置、依赖及失败对应关系，不能自动转为 744 pass。
这表示证据缺失，不表示已经观察到回归。

修复：补测试提交后的 worktree 按仓规执行所需门禁，或在无代码改变的情况下明确证明
可复用收据的提交/依赖/配置同一性及原始结果。后续补 Rust 测试时适用 Category B；
本次纯核查不在主检出启动 cargo t/docs_gen，也不运行批量 cargo tf。

### F744-R1-03 — 规范收尾未就绪（low，review/merge-owned）

入场 Plan 没有规范增量，三项影响元数据全空且无“无影响”理由；无 stage=review 的最终 pass。
但 canonical `docs/specs/a2r-std/project.md` 已由 `6011bc0e4` 说明 VM-local/task 生命周期，
不能误报为“Spec 仍说 VM 不支持 SQLite”。原 Plan 架构仍描述进程级注册表，现已追加勘误。

本次仅在 Plan 提出 SD-01/SD-02 和影响路径，供最终 review 核对、冻结；
不修改 canonical Spec/ledger。最终复审/规范对账/归档还未完成。

## P744-D1 的范围裁决

`auto service --server vm` 的 `SqliteDb.exec` 链接失败是实际阻塞纯 VM service 的缺陷；
`auto run -r vm --server vm` boot 探测的 fs.copy_recursive 500 也是真实失败。
fs.exists 仅为未隔离疑点。它们已登记 P744-D1，本次不弱化或注销。

744 原验收是 `auto run -r vm` 的动态组件 SQLite 和 Notes 的 VM UI 场景；
service 独立入口不在该验收路径。两套 Notes 全绿也不能表述为“纯 VM 后端全绿”。
service 解析与 fs parity 应由相应后续工作处理，不作为绕过 AC-01 漏测的理由。

## Spec 输入和冻结边界

| 输入 | SHA256 |
|---|---|
| 入场 Plan（规范化前） | a34bdac14d08a6cc9f1c1551a094bd98cf9a91b9188b78e8c5262584c4cc816a |
| `docs/specs/a2r-std/project.md` | 6289c9d7b395e3e0d49b33e180a71e2d19785eb6878a776f660151b6b9bf2103 |
| `docs/specs/auto-lang/vm/overview.md` | 0e22704f9a678f775633a706db8c6583a60218589b8b7dc3f87ccd1b30ccf3c3 |
| `docs/specs/README.md` | 6f48ac282672ae5565e9e962eded131a5913e48d18b845542e44e4a4354178584c |
| `AGENTS.md` | b5540a7fe007665981974b6b9edf0f8db8dba4d78403e678ecf32b692648bd3a |
| notes `run_vm_negative.py` | a7915a60ae8f13458276a6ede3fbded14aa73a0dc7501b612df4a656e5a79a98 |

SD-01/02 是本次提出的未通过增量，以 Plan 本节和本报告为持久记录；最终 review 必须另行冻结
修复后认可的 delta。没有可复用的 final-pass delta hash。

下一步：work 补 T-01 → 提交并提供复审门禁证据 → review → merge 收尾。
本次不修实现、不创建其他 Plan、不提交他人 WIP、不合入、不归档。

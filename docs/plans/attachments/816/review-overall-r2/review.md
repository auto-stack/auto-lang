# PLAN-816 整体 B-R2 受影响复审

结论：**needs_fix**。B-R1-01/03/05/06 CLOSED；02/04 为原反例已闭、完整不变量仍 PARTIAL/OPEN；07 原合同余量 OPEN。另新增一项 P2：**F-816-B-R2-01，Unix Rust service 的 Ctrl+C 未送达子进程**。因此当前共四项 P2 待处理，不能确认“01..06 全闭合”，也不授整 B/整 Plan pass。

受审 `bd50b0abaf61077e2f413d39b93766c8d9fb7ae6`，差分基线 `75dc3c058faa99fe4a52284dc790797c404b7089`；依赖 auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。唯一实施树 `D:/autostack/.wt/lang-816/auto-lang`，分支 `codex/plan-816-dev`，前后实现 clean。Plan r2，executing 6/14。当前聊天独立核验实施交接，未实施修复、未建立 agent 或另一审查树；未暴露模型/会话标识，不伪造身份。范围为整体修复受影响复审；已知 work-owned 安装余量未交付，故不重复整份最终验收循环。

基线、依赖、配置及 Spec 指纹见 [inputs.json](inputs.json)，冻结 [Plan](plan-frozen.md)、[handoff](handoff-frozen.md)、[规范增量](spec-delta-reviewed.md)，最终状态见 [final-state.json](final-state.json)。独立 replay 全在实施树执行，脚本/结果存主检出本目录。

## 原项闭合对账

| ID | 状态 | 本轮证据与实际边界 |
|---|---|---|
| F-816-B-R1-01 / P1 | CLOSED | proc_host 全字段补 windows_creation_flags=0；locked 三包 bins 实际构建 exit0，当前真实工具 consumer 30/30、compat 6/6。旧 E0063 消除，不用旧 exe 补绿。 |
| F-816-B-R1-02 / P2 | PARTIAL / OPEN | 原 owned stdout holder 悬挂关闭：CLI exit0、心跳停止，正常与非法预算对照保持。reader 回收错误在失败出口仍被覆盖，详见下方。 |
| F-816-B-R1-03 / P2 | CLOSED | 原 CLI spec 片段逐字提取，实际 facility 原生 Linux spawn_detached=Ok；只加子角色立即退出，防自我递归。Windows flags cfg 门控正确。 |
| F-816-B-R1-04 / P2 | PARTIAL / OPEN | 从当前新测试全文复制，真输入变化+保留旧 SFC 故障现 exit101，真 CLI watch 正控过；但总截止时间、排水回收仍可超预算。 |
| F-816-B-R1-05 / P2 | CLOSED（原例及受影响换流） | 真实 --format json new exit0、整份 stdout 可解析单个 JSON；模板进度移 stderr，return42 脚本对照仍过。info 缺 pac.atom 为无效成功样例，单列、不算产品失败或正控。完整命令机器矩阵仍随 T07/07 验收，不用两例替代。 |
| F-816-B-R1-06 / P2 | CLOSED | 最终 HEAD strict exit0、0 violations、3 grandfathered、141 声明边；原审计正反控 self-test 15/15。基线 b61f9cca 之后未再改清单/规则/锁。 |
| F-816-B-R1-07 / P2 | PARTIAL / OPEN | locked 构建及当前二进制身份已独立核验；work 提供真实 Rust READY/health/api/kill 后复绑收据。但完整 CLI 安装清单、fresh/full/Core/首次启动/升级失败余量明确未交付，未有批准延期的合同修订。 |

## F-816-B-R1-02 残项：失败出口吞掉 reader 错误

规则：AC-02 / V-02、Plan §5.2 的退出/流失败分类，及本轮声明的“reader 读错具名传播”。不是另称原挂住仍存在，也未观察到这些反例发生线程泄漏。

`crates/auto-man-runtime/src/http_service.rs:349–355` 在非零退出分支先匹配 `(_, _, _)`，覆盖 cancel/reader 错误；`:408–409` 明确丢弃 tree_reclaim/reader_reclaim。新 helper 捕获错误不等于调用方传播错误。

从原 service-probe 扩展故障宿主输出/退出组合，仍走真实 CLI→generate_api→owned cargo 路径：[adjacent.py reader-error](adjacent.py)、[宿主源码](service-cargo-adjacent.rs)、[执行脚本](service-adjacent-probe.py)、[实际结果](reader-error-results.json)。三个输入：

- READY 后写非法 UTF-8、exit0：CLI exit1，具名 `backend stdout reader: stream did not contain valid UTF-8`，为合法错误传播对照。
- 同一读错、READY 后 exit7：CLI exit1，只剩 `backend exited with 7 — bind conflict or startup failure`，reader 错误消失。
- READY 前同一读错：CLI exit1，只报 `backend exited with 1 before ready`，实际触发 reader 的错误被取消结果盖掉。

期望：保留后端真实退出信息，同时具名呈现确已发生的 reader/cleanup 错误，不能把读错统一解释为 bind/startup failure。闭合须覆盖 READY 前/后 × 干净/非零退出及无流错合法对照；不要求改变原非零退出优先级，也不把 error aggregation 当成功。

原反例与 clean/bad-budget 三态重放见 [service-results.json.txt](service-results.json.txt)。原生 Linux 原样 service tail 正常退出也为 Ok0，见 [unix-service-result.json](unix-service-result.json)，未把曾怀疑的空组取消当成缺陷。

## F-816-B-R1-04 残项：新 reader 通道仍不是总截止时间，join 仍可无界

规则：原 finding 的 bounded driver / Plan §6 “独立测试进程自带截止、拥有的子进程清理”。原内容身份 oracle 已修好，见 [watch-oracle-proof.json](watch-oracle-proof.json)：当前测试 SHA256 `334fd61a…`，与 private 测试副本相同，实际 input_changed=true / incremental_write=false / artifact_still_old=true，测试 exit101。

但 `plan816_cli_driver.rs:249–250`、`:275` 每次收到行都重新给完整 20 秒，属于空闲超时而非总 deadline。保存相同新测试全文，只把故障 CLI 改为“持续每 100ms 打进度，22 秒后 READY，再真实写入新 SFC”；该测试仍 **exit0 / 22.18s**，审计 ready_after=22.0879s。立即就绪、同样真实改内容对照 exit0 / 0.92s。持续输出可以无限推迟截止，并非只允许某个慢机误差。

`run_bounded` 的 `:179`、`:187–188` 是普通 join；根已退出不意味着管道 EOF。原方法不改，另附一个调用其真实方法的测试：limit=200ms，故障根退出0、自己的后代持 stdout/stderr 3 秒；实际阻塞 **3.08s**，截止断言 exit101。后代自带 3 秒寿命，无共享进程清理。[完整结果/对照](adjacent-watch-results.json)、[可重放脚本与故障宿主](adjacent.py)。这是测试基础设施的缺口，不宣称真实生产 watch 在此反例中失败。

闭合：使用总预算覆盖等行与收尾；进程、持管道后代及 reader 的终止/回收责任需一致。补持续噪声无目标标志、根退出但持管道、正常内容变更等控制，不能把通道 recv_timeout 或普通 join 的名字当作有界证明。当前手动 finish 宏也不等于 RAII，仍有 write.unwrap 等 panic 出口，需一起核对失败收尾。

## F-816-B-R2-01 · P2 · Rust service 的 Unix Ctrl+C 未到达独立进程组

规则：AC-02/04、Plan 保持既有 service 受控关闭；canonical `docs/specs/stdlib/design/http-service-deployment.md` §7，以及 `docs/plans/reports/736-http-decision.md:131–134,204`：生成子进程自身收到 Ctrl+C 后汇入关闭 watch；Linux 面未被撤销。不是新增跨 OS 生产认证，也不是新的负载承诺。

`http_service.rs:250` 使用 ProcessTree；实际 `unix_pg::child_setpgid` 把 cargo/服务子树移出父前台进程组。`:363–376` 却假定“自行收到事件”，父收到 SIGINT 只等待 15 秒，没有向 owned 组发送受控关闭信号，然后 cancel_within 的 kill_group 发送 SIGKILL。

原生 Linux 验证用**当前 service 从 ExecutionSpec 到 return 的整个 tail 原样**、实际 facility，READY 故障子进程安装真正 SIGINT handler 并写 graceful marker；对父会话组 killpg(SIGINT)模拟终端 Ctrl+C。[unix-signal.py](unix-signal.py)、[Linux 运行器](unix-signal-linux.py)、[结果](unix-signal-result.json)：

- 实际 ProcessTree：父捕获信号，**15.27s** 后 Ok0/`stopped (terminate)`；子进程 graceful marker **不存在**，由强杀退出。
- 唯一差异为继承父组的信号送达对照：**0.032s**，同一 handler 写 marker、子进程正常退出，父 `stopped cleanly`。

该对照只定位信号路由，**不建议以 DirectChildOnly 代替树归属**。需要在保持 owned 树的前提下落实 Unix 控制信号/关闭请求送达，再排空，超窗才终止。关闭 oracle 要同时核实 child 真收到、正常退出和强制兜底。未用该 idle 故障宿主声称真实 HTTP 在途 drain 全覆盖。

冻结 handoff §6 把 clean/owned-holder/bad-budget 三态说成 Ctrl+C 排空证据，三态实际不发送任何信号；work Rust 真链收据记录的也是 kill。需更正来源边界，不能替代上述关闭验收。

## 07 范围与规范增量裁定

当前人类消息是修复交接，没有批准减去 r2 原 AC。按原合同，完整 CLI 安装清单、fresh/full/Core、首次启动与升级失败仍为 **work-owned**；记录“交裁定”不能使其成为 merge 门或自动延期 R07。默认下一步是补齐，若用户另行明确修订范围，再按新合同审；本轮不替用户修改合同。

work 的真实 Rust 收据是已完成工作的证据，但未附本轮可重放完整设置/命令，且未测 Ctrl+C；本轮没有重复冷编 288 秒去伪装完整最终验收。它不填平已承认的安装矩阵余量。handoff §3 仍有“14 crate / 本 Plan 不改 792 / 777@7ce08b5cf”的旧表述，与 §6 的26项/当前边界不一致，应在 T08 同步。

SD-01..05 原行及 handoff protocol/tools 候选已冻结；frontmatter 新增两个实际 Spec 路径正确。当前 canonical 主源只读，未改 live ledger；候选不得在服务关闭和安装矩阵未闭时沉淀“全部支持完成”。AC01/03/04仍 partial；AC02因残项及新信号反例 fail；AC05不通过；AC06未来 merge-owned；历史 AC07..09/M816-A 已发布保持。

## 本轮验证、复用与下一步

- 当前源 locked 三包 bins 构建过；Cargo JSON 唯一目标/路径/实际 hash 见 [artifact-bindings.json](artifact-bindings.json)。auto=`3e56e17b…`，tool=`f7e73df5…`。测试采用本轮产物，未复用 handoff 的旧 hash；不要求不同构建上下文得到确定性同 hash。
- 当前独立 consumer **30/30**、runtime seam/非UI service **4/4**、compat 四集成目标 **6/6**、原样真实 CLI driver **4/4**（script argv、serve/req、watch 真内容、真实 tool gen），共 **44** 项当前定向测试通过；不是整档。
- [777 strict](777.json) 0违规/3grandfathered/141边，self-test15/15；client/execution 两 verify-deps Git Bash PASS。初次默认 bash 的环境出口127单列日志，改用仓库 Git Bash 成功，不记产品错误。
- 五个变更 Rust 文件 rustfmt --check、git diff --check 过。构建有依赖/兼容域既有 warnings，不声称全仓零告警；当前修复没有新增对应诊断位置，未做无关清理。
- 从上次输入逐项比对46个文件41个未变；git diff只有基线和五个修复源。client/protocol/execution 源、Cargo.lock、验证配置、auto-down及原泵探针等未变，故明确复用 R1 client17/exec23/protocol20、早晚取消/残留重试与 Linux client check 证据；producer/CLI/service/watch 变化相关结果已重跑，未用历史 pass 替代。
- 原 A/T03/T04 已通过范围保持。T05..T09/当前T13不勾选；修齐02/04/新Unix信号项及07原余量后提交新 SHA，再受影响复审。最终 pass 后才由 merge 在冻结提交执行裸 cargo t 与实际触面档/consumer/workspace 门，tf 仅主检出到期批量；Spec/ledger/合入/归档/guard 清树仍 pending。本轮未运行整档、未修产品、未提交或合入。

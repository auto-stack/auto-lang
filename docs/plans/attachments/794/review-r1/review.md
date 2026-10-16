# PLAN-794 独立 final review R1

2026-10-16；stage=review；review_scope=final；**outcome=needs_fix**。

## 基线

- 当前 Codex 审查上下文未参与实施；从主检出合同和原工作树实际代码重建结论，未另建 agent 或工作树。
- plan_revision=1；worktree=`D:/autostack/.wt/lang-794/auto-lang`；branch=`codex/plan-794-dev`。
- reviewed_commit=`91974d8f8d826a9cdb3ea6a50038a518bc02c4e9`；base_commit=`8b97d3c9bc9c86cb4d8ebde4a5d7856554e83b9c`。
- auto-down 组内兄弟=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，clean；开始/结束受审 HEAD 相同、实现 clean。审查临时测试已移除；未修实现、未合入。
- 实读依赖锁、测试配置、探针及 CLI SHA256 见 [implementation-inputs.json.txt](implementation-inputs.json.txt)；实读主检出 Specs/原 Plan hash 见 [spec-inputs.json.txt](spec-inputs.json.txt)。Cargo.lock 为本机忽略文件，不声称属于提交。
- 主检出入场已有其他计划、设计文档和 ledger 的未提交工作，本轮不修改这些内容；只新增794复审附件、更新794计划和追加债务记录。

## Findings

### F-794-R1-01 / P2：模块边界冻结基线过期

类型=收口缺口；AC-04/05，T-05/06。`Cargo.toml:96` 新增 consumer exclude，触及777锚定面，但 `.autoos/module-boundaries-baseline.json` 仍绑定 `3e1a9742c9748de2020abce91cbd3bcb79227270`。

复现（原工作树）：

```text
python scripts/modularity-audit.py --mode strict --output <report>
exit 2；verdict=fail；violations=[stale_baseline]
锚定输入自 base_revision 起已变更: ['Cargo.toml']
```

[完整报告](modularity.json.txt)。声明依赖未新增禁向边，ownership coverage 完整、原有2条窄 grandfather 保持；不是新增坏依赖。observe 的删边计数不能代替现行 `docs/specs/design/repository-layout.md` 要求的有效冻结基线。先核对实际变化，再在干净提交上显式重采样并复跑 strict，或消除不必要的锚定面变更；不能放宽规则或扩大豁免，也不能只推迟到merge。

### F-794-R1-02 / P2：指定 VM 验收探针的状态查询可误绿，也会粘住旧 pending

类型=原合同验收 oracle 缺口（探针本身继承自781，不据此断言生产状态机回归）；AC-02/04/05，T-05。`examples/capability-tests/p781-scroll-child-anchor/tests/vm_probe.py:158–170` 的 `expect_reply` 只 press 一次，之后反复读取保存于 `.receipt` 的显示字符串；查询动作失败也不阻止 `got == want` 返回成功。

- 原探针本轮实际 **56/57，exit2**：`v03_old_key_not_found_on_ch2` 为 `pressed=True got=pending want=invalid:not_found`，15秒后仍读同一标签。[日志](ui-vm.log)、[原始报告](vm-original.json.txt)。`app.at` 的 Query handler 将 native 当前值赋给 receipt；轮询标签不会再次调用 native。
- 确定性负控：[query-oracle-control.py](query-oracle-control.py) 用 `press=False`、旧标签 `applied` 调用真实 `Driver.expect_reply`，实际返回 **`(True, 'pressed=False got=applied')`**，预期拒绝断言 exit1。[原始结果](query-oracle-control.log)。这只隔离验证器的失败传播，不冒充真实UI用例。
- 保留全部原 case/断言，补充“每轮重新执行只读 Query、press失败不得通过”的控制组：[ui-query-control.py](ui-query-control.py)，真实 VM **57/57，exit0**。[日志](ui-query-control.log)、[原始报告](vm-query-control.json.txt)。本次QC50首个新查询即返回 invalid:not_found；这证明当前控制组通过，不把它写成首轮 pending 的唯一根因或原探针已修复。

修复要求：让指定验收探针真正重新查询状态并传播点击失败；加入失败点击+旧成功标签的永久负控，以及pending到终态的真实重放。不能靠延长等待、删除case或只补ALL-PASS收据关闭。若新鲜查询仍pending，继续调查实际状态机/测量链。

### F-794-R1-03 / P3：新增定向测试引入 unused_mut

类型=新增健康检查缺口；AC-05，T-06。`crates/auto-lang/src/tests/plan794_scroll_host_provider.rs:126` 的 `let mut replies` 只读查询；`cargo t scroll_host_provider` 明确报告 `variable does not need to be mutable`。[targeted.log](targeted.log)。该文件为本计划新增，不是存量告警；与交接“零新增警告”不符。去掉无用mut并提交，定向复核即可，不要求清理本批未触及的存量告警。

## 验收映射与实际验证

| AC | tasks | 本轮结果 | 证据与边界 |
|---|---|---|---|
| AC-01 | T01..03/T05 | pass | 八shim feature中性且无UI实现直引；外部no-ui独立manifest 5/5，独立源码补控覆盖八native在位路径、未知轴、Unicode、progress_x、各锚点回供 |
| AC-02 | T01/T03..05 | partial | ID/arity/缺席与参数投影通过；原VM 56/57，重新查询控制组57/57；指定oracle仍有F02 |
| AC-03 | T02/T03/T05 | pass | 两VM快照/实例覆盖/默认优先/Clone与merge保留；独立补控覆盖全部八shim在位/缺席、空句柄、两种to形态、拒绝/各状态，32轮后栈、heap、pool live_shares回基线，underflow/phantom不增加 |
| AC-04 | T04..06 | partial | 薄adapter委托唯一原registry/renderer；Vue59/59、VM控制组57/57；剩余宿主族SD清楚；F01/F02未闭 |
| AC-05 | T07 | fail | 独立审计完成，F01..03 OPEN，不能给final pass |
| AC-06 | T08 | pending | merge-owned裸cargo t+tv、知识沉淀/ledger/归档/guard清理尚未执行 |

命令均在原工作树执行，`SCCACHE_SERVER_PORT=42917`沿用本机端口旁路：

```text
cargo t plan794                         6/6 pass（包含在下项9例中，不累加）
cargo t scroll_host_provider            9/9 pass，含新增unused_mut告警
cargo test --manifest-path test/fixtures/modularity/scroll-provider-consumer/Cargo.toml
                                        5/5 pass
cargo test --manifest-path ... --test review_r1
                                        独立补控2/2 pass
cargo build -p auto                     exit0
rustfmt --edition 2021 --config skip_children=true --check <11个触面Rust文件>
                                        exit0
git diff --check <base> HEAD             exit0
```

补控源码=[probes.rs](probes.rs)，结果=[probes.log](probes.log)；临时复制到consumer/tests/review_r1.rs运行后只移除此文件；不更改受审生产代码。最初使用库内cfg(test)专用构造器导致补控编译失败，已改为串行清默认槽+公共AutoVM::new；最终2/2对应已保存源码，不计初稿为产品失败。

UI：使用autoui-verifier技能和原p781双端探针。Vue默认Rust后端复跑因原有PID19128占用生成backend.exe无法重链而未启动（[日志](ui-vue.log)、[直接启动诊断](vue-start.log)）；未停止该原有服务。用临时wrapper将同一CLI的run附加`--server vm`，在14835/14836隔离复跑**同一Vue探针59/59**（[日志](ui-vue-vmback.log)、[原始报告](vue.json.txt)）。这是Vue前端+VM API后端，未声称重验Rust backend。wrapper仅一行：

```text
@"D:\autostack\.wt\lang-794\auto-lang\target\debug\auto.exe" %* --server vm
```

Vue报告中的auto_bin hash是wrapper的hash；真实CLI hash单列于implementation-inputs，与VM报告相同，SHA256=`f9690405a4b66c727f4001db2d2647713f9b44ba852c500d94ffd69230cb90c9`。本轮自启VM与隔离Vue进程已收尾；原实施服务保留。两端实际offset变化、目标正面积交集、双pane/嵌套隔离、换绑后新key、managed拒绝与用户滚动不回拉均有原始数值；不以queued/progress代替完成。截图已目视核对；本轮验证功能迁移，不宣称两后端原有排版像素一致。没有改golden或移除case；656专项未重复运行，781旧offset子矩阵及本轮参数补控覆盖本批触面。

## 规范增量与遗漏扫描

SD01..04与当前四份目标Spec逐项核对，冻结正文见[spec-delta.md](spec-delta.md)，hash见[spec-delta.sha256.txt](spec-delta.sha256.txt)。前置目标hash见spec-inputs。frontmatter四个supersedes路径均真实，new为空合理，GOAL-003/007存在。owned DTO/五方法、构造期快照/实例覆盖/兼容if-absent、四装配点、原测量/renderer状态机单源与缺席值均对应实际代码；未发布canonical Spec或ledger。

可选轴弹序调整符合已有函数签名与codegen左到右压序，实测X/未知轴Y/坐标X先Y后均通过；未批准新增算法或降低781合同。T01旧表9903弹序文字仍写delta→handle→axis，和后续勘定、代码及SD01的delta→axis→handle矛盾，T06更新材料时应同步该旧行，保留勘定史。

未发现漏迁八native、第二core registry、provider回调持能力槽锁、UI类型/堆id泄漏或未经批准延期。测试与基线缺口按F01..03登记，不以债务名义放行。

## 收尾与下一步

工作树链接扫描另有实际红项：[wt-guard.log](wt-guard.log)，exit1，至少列出50个reparse point（脚本显示上限），包括p656/p781生成node_modules中的junction。抽查vue目标均指向各自树内`.pnpm/.../vue`；未声称所有目标均如此。AGENTS禁止worktree链接；本轮不删除生成目录/链接或工作树。原树负责人须按实际target只摘链接并重跑guard至clean，未clean禁止worktree remove。T08清理仍pending，不能把代码树clean等同guard clean。

Plan回`executing 4/8`：保留T01..04，重开T05/06；T07/08继续pending。原树修复F01..03、同步材料并提交后复审受影响项。review阶段未运行整档t/tv/tu/tt/tb/taa或docs_gen；merge-owned裸t+tv仍pending，ui_gen零触面不触发tu，tf只归主检出批量回归。未合入/归档/交付。

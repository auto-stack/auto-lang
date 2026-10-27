# PLAN-828 第一批 R3 受影响面复审

2026-10-22，Plan r3，review_scope=phase（第一批 T-02/T-03/T-04 修复）；**needs_fix，仅剩 1 项 P2**。

实施树 `D:/autostack/.wt/lang-828/auto-lang`，`plan-828-dev@599cc15afee9fb964a8a88a243c2210ea0597aa2`，修复基面 `14c929a5d9d54a90106b8e62b058e98e75a28b9f`；入口/结束 clean、HEAD 未漂移。主检出交接簿记 `717e290be75cf86da65402a8866c414ff3547bf3`。sibling auto-down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`，未消费/未改。

Reviewer=本 Codex 独立审核上下文，未实施该修复、未启代理，不编造底层未暴露身份。技能/共享合同、当前 Specs、[Plan（含 SD-01..06）](plan-input.md)、[接口决策](interface-input.md)与 514 个源码配置输入哈希见 [baseline.json](baseline.json)，结束一致性见 final-state.json；[修复 diff](repair.diff)。未修改实现、canonical Specs 或 ledger。

## R2 三项及既闭机制

| Finding | 本轮独立核验 | 结论 |
|---|---|---|
| B1-R2-01 P1 写目标误命中 | 原 `x=3→y=3` 编辑增量与冷编均为1，不再旧 HIR/code 命中；同根 Place.local/entry 恢复控制拒绝不同内容 | 原 P1 CLOSED；完整 can3 字段政策尚有下述新 P2，不能称全内容身份已完成 |
| B1-C03 P2 依赖归属 | 原 reread、历史 A/B 控制通过；history 自带依赖，目标条目检查，首读冻结、重读不盖章；新增取消候选/迟到提交两段组合保持原依赖，stale 分别正确强制重验 | CLOSED（现有 scalar + 实际 body-fact query core 范围；不声称实现 inline/CTE） |
| B1-R2-02 P2 closed query | 原 close→query 拒绝，零工作计数；open 与旧关闭入口保持 | CLOSED |
| 原 B1 owner/index/intrinsic/span | 原10控制重放全过（can3 允许更早拒于 canonical，仍核 verifier 原拒绝能力） | 既闭保持 |
| 原 mixed intrinsic/来源/新增 owner | 原 roundtrip2全过，r1 86、close6+3 与节点释放 oracle 通过 | 既闭保持 |

原 R2 **5 adjacent + 2 semantic 控制 7/7** 通过。semantic 控制仅调整“can3 应区分原变体”的前提，原输入、拒绝采纳/冷回退断言不变，原件与适配脚本保留。永久 b1=9/9、session=18/18（跳过两个后批512）、contract=22/22、t01r=4/4、r1=86/86。轮内 r1 曾红12项是执行者的开发历史，本轮冻结根实测86/86，不冒称本上下文复现过其轮内红。

## B1-R3-01 / P2：can3 仍漏 CallBinding.moved，语义不同的 HIR 被恢复当成同内容

**既有规则**：AC-02/05、T-03/T-04、§5.2/5.3 的当前内容与受检对身份；R2 关闭合同及本轮接口决策明确承诺以 `hir::PartialEq` 语义字段为内容依据，排除 span/`mutable_explicit` 等 cosmetic。`hir.rs:389–396` 的 `CallBinding.moved` 明确为语义字段，参与 `PartialEq`，v2 codec 也真实保存它。这里不要求本计划增加 owned Str/.move 语言支持；要求当前承载的工件字段符合内容身份或受控拒绝，不能用更宽的恢复结构悄然采纳域外/不同 HIR。

**可重放反例**：[canonical-controls.rs](canonical-controls.rs) 的 `canonical_call_binding_move_marker_distinguished_at_restore`：

```text
fn helper(x int) int { return x + 1 }
fn main() int { return helper(5) }
fn tail() int { return 9 }
```

冷编出当前正确 HIR，仅将 main 调用绑定的 `moved=false` 改为 true；保留 param/arg/owner、span 与当前上下文，构造合法编码 v2 工件，写入 fresh store 的当前 main key/envelope。随后真实 `warmup_persisted → restore → resident("main")`。

| 实际观察 | 结果 |
|---|---|
| 原 body 与变体 HIR PartialEq | 不等（已 assert mutation 发生） |
| can3 内容输出/版本 | **相同** |
| 当前真实 verify_function | 接受（故 verifier 不能补这个内容身份遗漏） |
| restore 采纳/验证数 | **3/3、3/3**；没有跳过被改 main |
| resident main 的 moved | **true**，当前冷源为 **false** |

[原始日志](canonical-controls.log)，实际退出码101。正确行为是在采纳前区分/拒绝这个不同字段，resident 保持冷源 false；合法 false 的原始 call artifact 则继续正常采纳，前两轮正控已证明这一半。

**已证根因**：`cache.rs:817–822` 仍只渲染每个 CallBinding 的 param 和 arg，没有渲染 moved；`canonical_hir_function` 注释与 interface-decision 的“全部语义字段”声明超出实现。恢复 canonical 比较因此通过，后续 verifier 也接收，污染 HIR 进入 resident/history。不是 header、span、重绑或旧 can3 前行迁移失效。

**严重度/边界**：P2。R2 的真实 i32 错误结果已关闭；本次没有声称复现新的 i32 执行结果错误，更没有把 Str 所有权支持纳入本计划。失败是当前工件的语义身份/恢复合同和所声明字段覆盖仍不完整，不能以既有9项/86项绿替代这个明确反控。

**关闭 oracle**：同一输入和 mutation 不再将 true 变体作为当前 false 内容采纳；合法 false call 正控、所有 R2 及前两轮已闭控制保持。按既定政策把 moved 纳入规范化内容编码，或在当前 profile 工件准入对该不支持的标记受控拒绝；不要仅把这个语义字段改称 cosmetic 来通过。若编码/内容算法改变，隔离已有 can3 语义，确保相关 HIR/ABC/sidecar 键与读写同点使用新内容规则、旧行不覆盖。核对其余已支持节点字段后冻结新 SHA，仅做此窄修复与受影响重放。

## 其余验证与限制

| 检查 | 本轮实际结果 |
|---|---|
| 原 R2 adjacent / semantic | 5/5、2/2；[raw](adjacent-controls.log)、[raw](semantic-artifact-controls.log) |
| 原 artifact / plain+mixed roundtrip | 10/10、2/2 |
| 新 candidate cancel/late dependency | 1/1（同一测试两段状态链）；与新 moved 反例同文件，整体1过1败 |
| 永久 b1 / session / contract / t01r / r1 | 9/9、18/18（2个后批512未跑）、22/22、4/4、86/86 |
| 旧 close/pin/PausedRun / 独立session组合 | 6/6+3/3、6/6，精确filter不重计include展开96项 |
| can3 共享调用方 798 cache_tier | 20/20；[raw](cache-tier.log) |
| 健康/冻结 | 精确3改动Rust文件 rustfmt --check=0、diff --check 14c..HEAD=0；源码/依赖/config结束指纹与入口相等 |

五验收目标由 `cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test function_incremental_828_b1 --test function_incremental_828 --test function_incremental_828_contract --test function_incremental_828_r1 --test function_incremental_828_contract_t01r --no-run --message-format=json` 构建，再运行 Cargo 返回的明确 executable；[目标/哈希/退出码](target-results.json)、cargo-artifacts.jsonl、linked-libraries.json 保存。独立控制经 [run-controls.ps1](run-controls.ps1)，cache_tier 用同树 `cargo test --locked --manifest-path experimental/ac-av-probe/Cargo.toml --test cache_tier -- --test-threads=1`。无根整档或 docs_gen。

SD-01/02/05 的内容身份完整性目前仍受 R3-01 阻断，SD 均保留 proposed，canonical/ledger 不沉淀。后批 T-05..T-09 完整验收、T-10 最终验收、最终 PASS 后 merge-owned 冻结裸 t + 必要触面档仍 pending；不据此 phase 给后批派发/worker/资源/P95/GC 授 PASS。Plan 保持 executing0/11，下一角色原树 work 做 R3-01 窄修复，[GLM任务单](glm-next-round.md)。未合入/归档/清树。

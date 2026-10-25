# PLAN-832 T-09 交接与 readiness（R2 修复后）

状态：**R2 残余 F03/F04 的 work 修复完成，可交 T-10 受影响独立复审；merge 门 pending**。F01/F02 已在 R2 独立关闭并保持；r3 的 jet/ark 延期继续有效。

## 1. 当前身份

- Man：`f52cd93b09401f358a89a19058fbf1fa7673fd16`。
- lang：生产源自 `87f82d637ecd586c17a01c1c975e1456bc346991` 起无 diff；本轮后续仅证据/Spec/交接提交。
- UI=`aee499e87093ff727f68ac8467c506e9ecf6ecdc`；down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- [连续失败诊断](repair-r2/diagnosis.md)、[R2 修复闭环](repair-r2/report.md)、[机器结果](repair-r2/results.json.txt) 绑定实际输入与输出。

## 2. AC / finding 结果

| 项目 | work 结果 | 当前证据 |
|---|---|---|
| AC-01 / F01 | pass 保持 | 独立 Core workspace、无 UI/down fresh/install/rebuild/relocate 与真实管理命令；R2 已独立关闭，本轮未改 |
| AC-02/03/06 / F03 | pass | 直接开发 CLI 清 Cargo/AUTO 环境真实生成；七格生产对照、永久无注入 CI 1/1；显式缺件/缺 root 具名失败，安装邻接通过 |
| AC-04/06 / F04 | pass | 真实 Core 升级 backup 按文档恢复后 640 verify=0、旧脚本输出；陌生 extra exit2；full 永久恢复 driver 通过 |
| AC-05 | pass 保持 | legacy 保全/恢复事实未改 |
| F02 / owner staging | pass 保持 | R2 已核验 actual AccessDenied/live/dead/unknown，本轮未改 |
| AC-07 | pending | 等待 T-10 受影响独立复审 |
| AC-08 | pending | merge-owned 统一门、落地、ledger、归档、guard 未执行 |

当前无已知 open in-contract finding、必需 oracle gap 或未决合同问题。R1/R2 的完整证据复用仅覆盖未变代码、依赖、配置与输入；变更过的直接发现与恢复路径使用本轮新证据，不继承旧示例总 pass。

## 3. 二次失败诊断调整已验证

- 原漏检：Cargo 父测试提供 manifest 并先解析后注入工具。调整：`current_exe` 开发布局解析 + 子进程环境移除 + 不注入的永久测试。直接 CLI 七格与 CI 1/1 均通过。
- 原漏检：未从真实带 marker backup 执行新恢复文档。调整：文档明确保存并移出 owned marker，严格校验不变；真实 Core/full 往返及陌生 extra 负控均通过。

详见[诊断](repair-r2/diagnosis.md)与[闭环](repair-r2/report.md)。

## 4. 本轮验证与未执行

实际执行：full build；直接生产 discovery 七格；无注入 CLI gen 1/1；真实 Core backup 恢复六步（含负控）；full 永久恢复 driver；rustfmt、Python AST、diff-check。未运行语言/Man 整档、六 profile、完整 14 项或无关 Core/owner/legacy 重放。

下一责任角色为 **T-10 受影响独立复审**。final pass 后由 merge 在新冻结输入执行语言裸 `cargo t` 与实际触面、Man `t`/`tcore`/`ti` 及必要 consumer/UI 门，再完成合入、来源读回、Spec/ledger/索引、归档与 guard 清理。`tf` 仍仅主检出按到期规则单实例批量。

## 5. Merge 门禁新增修复

R3 后首次 merge 门发现 Man 占位 `packages/blueprints` 阻断语言真实库，且原 cmd_bp 测试绕过生产 registry。Man `75a81b4f` 已统一生产/测试入口，保留显式 env 和项目库优先，仅占位根回退安装 SDK 或语言 owner。`cmd_bp` 4/4、开发/安装 `bp list` 与显式根负控通过，详见 [merge 修复报告](merge/repair-report.md)。该代码变化使 R3 旧提交审查不能替代新受影响复审。

lang 首次 `cargo t` 另命中已登记 B845-FEATURE-01（`probe_adapter_tests` 可选 `auto_test_case` 门控缺失），PLAN-847 为独立 owner；832 未修改或豁免该 blocker，lang 合入须等其恢复。当前 Man/lang 均未落地，ledger/归档/清理未执行。

独立 reviewer 随后发现安装回退仍可能被编译期语言库抢先（F832-MERGE-BP-01）。最终 Man `ceb8dfdd` 已把安装 SDK 置于开发回退之前，并保留显式 env/真实项目库优先。永久 driver 的安装 SDK 唯一 marker 已被真实 `bp show` 读取，`cmd_bp` 4/4 与 full build 通过；[证据](merge/repair-report.md)。仍需同一 reviewer 对最终提交做受影响复审。

# PLAN-832 T-09 交接与 readiness（R1 修复后）

状态：**R1 四项 findings 的 work 修复完成，可交 T-10 受影响独立复审；merge 门 pending**。r3 合同不变，jet/ark 真实构建继续按用户裁定转 v0.7 环境门控渠道，未实测不冒称通过。

## 1. 当前身份

- Man：`ebfc88f`（最终 Core/CI/安装/文档与发行 driver）；核心构建安装实现提交=`f601673`。
- lang：`0fd092bdc`（最终 CI/Spec）；后续交接/证据提交不改变生产输入。
- UI：`aee499e87093ff727f68ac8467c506e9ecf6ecdc`；down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。
- Core 安装=640 files/29 packages，仅 Man/lang source；full 安装=6608 files/四仓 source。确定性身份=固定源+各自锁+安装清单，二进制 hash 不替代来源。

[R1 修复闭环报告](repair-r1/report.md) 与 [机器结果](repair-r1/results.json) 绑定上述代码、输入及证据复用边界。

## 2. AC 结果与证据

| AC | work 结果 | 当前证据 |
|---|---|---|
| AC-01 Core/单源 | pass | 无 UI/down 两仓 fresh clone、独立锁/构建、normal/build/dev 闭包、安装 SDK locked check、真实语言命令及 UI 具名失败；Core 重定位删除全部旧 target 后通过 |
| AC-02 命令/API/寿命 | pass | 原清环境真实 gen 1/1、full driver 7/7、`tcore` 23/23、discovery 6/6、client/te-child 正控；原命令/JSON/寿命证据的未变面保留 |
| AC-03 三仓 UI/安装 | pass（r3） | full 6608 verify、安装态真实 gen、原三仓生成/build/run 与 792 证据的未变面保留；六 profile 生成、vue/rust/vscode 构建、VM 运行满足 r3；jet/ark 构建延期有效 |
| AC-04 发行/恢复 | pass | 无 UI Core fresh/install/rebuild/relocate 新增闭环；实际 System PID 4 unknown 保留并具名，live/dead/注入 unknown 控制通过；旧事务/回滚未变面保留 |
| AC-05 历史/事务 | pass | 保全 29 refs/23 tags/tree 与切换顺序证据不变，本轮未改 legacy |
| AC-06 删除/测试/CI/SD | pass | 185 迁移源及删除事实不变；`man-tcore` 纳入跨仓队列，owner bootstrap 与工具解析实测；Man 当前安装/恢复文档及退役 release 已落地；SD-01..06 更新 |
| AC-07 独立 final review | pending | R1 修复已冻结，等待受影响独立复审；pass 仅表示 merge 门 pending |
| AC-08 merge | pending | 统一冻结门、落地、ledger、归档、guard 均未执行 |

## 3. R1 finding 闭环

- **F832-R1-01**：Core 使用独立 `core/Cargo.toml`/lock/target，不解析 full optional UI path；安装只采集 Man/lang。fresh 3/3+重定位 1/1，Core 640 files，full 6608 files 正控通过。
- **F832-R1-02**：owner 状态三分；实际 AccessDenied 不再判死亡，只有确认死亡才删除，unknown 具名保留。
- **F832-R1-03**：生产 discovery、CLI/fixture 和跨仓 bootstrap 全部按 owner 解析；原始无 override 反例及 sibling/helper 场景通过。
- **F832-R1-04**：Man 首页/开发入口/固定来源/Core/full 安装/升级恢复说明已更新，旧 am 明确 legacy，旧 release 脚本 exit=2 退役。

当前无已知 open in-contract finding 或必需 oracle gap；完整统一回归不属于 repair 前置，仍由 merge 执行。

## 4. 验证与复用边界

本轮实际执行：Core fresh clone/fresh lock/locked build/normal/build/dev tree/install/SDK locked check/script/UI denial；Core 删除 target 后重定位；full install/verify/安装态 gen；原始 CI 反例、full driver 7/7、`tcore` 23/23、discovery 6/6、真实 client 和 te-child、owner liveness、文档退役与卫生检查。

复用仅限未变代码、依赖、配置和输入：T-06 的三仓 UI 构建/VM/compat，T-07 的其他事务故障矩阵，792、legacy 保全及 185 源审计。`f601673..ebfc88f` 不改 Core 构建/安装实现，只补发行重定位、`tcore` 和文档；lang 后续改动不改语言源/资源。未机械重跑六 profile、全量 14 项或整档。

## 5. 下一责任角色与 merge 门

下一责任角色为 **T-10 独立复审**，重点重放四项 R1 invariant 和最终提交身份。final pass 后，merge 在新冻结提交执行语言裸 `cargo t` 与实际触面档、Man `t`/`tcore`/`ti` 及必要 consumer/UI 门、完整三仓队列与适用发行门，再完成有序本地合入、来源读回、Spec/ledger/索引、归档和 guard 清理。`tf` 仍仅主检出按到期规则单实例批量。远端 push/registry 发布不属于本轮。

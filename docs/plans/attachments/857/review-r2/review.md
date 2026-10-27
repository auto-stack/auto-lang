# PLAN-857 独立复审 R2

- `stage=review`、`review_scope=final`，独立上下文 `/root/review_857`；Plan revision 1；受审提交 `e20b95e175f9184e13c00c78efa7b63349165276`，R1受审基线 `ecf9939c5b2e369f9ce7d6ea25b0b66439bb7da6`，交付基面 `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`；工作树 `D:/autostack/.wt/lang-857/auto-lang`、分支 `plan-857-dev`。
- **结论：needs_fix。** R1 的三项行为反例与正控均在修复提交上通过；两项原证据缺口的实际正例已补。新增 F-857-R2-01 说明默认/重置比较器会对缺失、真实不完整、身份错误的重置证据返回成功，R1-04 的比较器失败传播不变量尚未闭合。相关任务不得移交merge。
- 规范增量冻结副本和实际输入、锁、SDK、来源、日志SHA见 [spec-delta.md](spec-delta.md)、[baseline.json.txt](baseline.json.txt)。原始记录位于 `D:/autostack/plan-artifacts/plan-857/review-r2/`，具体路径和SHA均在baseline中；未依赖仅有的PASS横幅。

## R1 发现复核

| R1 finding | 独立检查与结果 | 状态 |
|---|---|---|
| F-01 宽视野误拒拾取 | 直接重放原 `visible-pick-replay.rs.txt`：1024、4096两距离的实际渲染25像素与Host选择均通过；修复永久用例和独立数学2例通过。补充有限目标/两投影/角度下144个可投影点的proxy拾取全部成功，并以126个已知世界轴点回投检查误差未超过1逻辑像素（测试范围见 `large-scan.rs.txt`、`axis-roundtrip.rs.txt`）。 | 原反例与所查共享路径关闭；不承诺所有极小物体。 |
| F-02 全裁剪组伪手柄 | 原近/远、正交/透视及可见正控重放通过；永久裁剪测试3项通过，含真实Host跨近平面和远平面的可见组正控。实现使用齐次深度和w裁剪后聚合。 | 原反例与已检查跨裁剪面正控关闭。 |
| F-03 交叉按键release | 原双方向及各自无干扰正控重放通过，永久测试1项通过；检查生产及consumer真实窗口报告34阶段通过、partial拒绝，实际删除来源判断的变异在phase28具名失败。 | 原反例和所查窗口路径关闭。 |
| F-04 默认/重置与856像素 | 856冻结runner `05372dfc...` 与当前producer `bc18c3ae...`、SDK/锁/配置已核；独立核对保存的默认和真实Home后3D合成区域逐RGB像素相等，reset报告为34阶段、runner与当前记录一致；纹理hash亦相同。 | 原缺失正例已补；比较器自身失败传播和身份检验因R2-01仍不完整。 |
| F-05 855性能/工作集/上传/owner | 855隔离来源清单455项逐文件对照提交 `05fdc81c...` 零差异；8条性能记录的哈希、40样本、p50/p95、工作集采样、mesh上传与目标数核对；grid运行前实际有7232字节owner，VM及grid最后owner释放记录存在。性能副本与最终来源仅5个已列文件不同，经过路径检查无选中、拾取、组说明，因此相关测量可复用。 | 所查验收证据关闭；两轮、同机CPU/工作集、额外导航非等价负载，不能扩展成GPU时间或跨机承诺。 |

本轮独立运行 `repair857_visible` 3、`repair857_clip` 3、`repair857_buttons` 1、`viewport3d_decision` 2，9项通过；3份R1原始测试与正控也独立通过。来源475项、SDK 681文件/17 local/479包/29边通过验证；交接证据索引25项及8条性能原始记录SHA匹配。既有SDK146 warnings保留。各项与受审源码的差异和原始输出见baseline。

## F-857-R2-01 [P2] 默认/重置像素比较器对缺失、partial或错误身份重置报告可返回成功

- **既有规则**：AC-02/06、V-01、V-06及共同合同要求默认与真实Home重置均对照856冻结基线；不完整/错误身份结果不得成为成功证据。此项是R1-04的比较器覆盖新发现，不否认本次已保存的真实有效重置正例。
- **实际复现**：对 [工作阶段比较器](../work/repair-r1/compare-default.py.txt) 只替换输出目录与重置报告路径，旧856和当前857程序仍按原方式各实际运行80帧、图片对照和一像素变异控制正常：
  1. 重置报告路径不存在：`exit 0`，summary `reset_equal=false`；见 [实际执行脚本](default-missing-reset.py.txt) 和 `default-missing-reset-driver.log.txt`。
  2. 指向真实生产探针partial报告（`passed=false`、`viewport3d_phase=0`）：仍 `exit 0`、`reset_equal=false`；见 [脚本](default-partial-reset.py.txt)、`default-partial-reset-driver.log.txt`。
  3. 以真实通过的重置报告及截图为底，仅把报告 `runner_sha256` 改成64个0：仍 `exit 0`、`reset_equal=true`；见 [脚本](default-wrong-runner.py.txt)、`default-wrong-runner-driver.log.txt`。篡改样本仅为负控，并非原始报告。
- **原因**：`compare-default.py.txt:21-26` 对`reset.exists() && passed`使用可选分支；没有要求重置必须存在/完整，也未把重置报告中的运行器身份与当前runner绑定。输出字段虽暴露 `reset_equal=false`，进程仍成功，调用方仅检查退出状态时会把未验证的必需路径当通过；错误身份例甚至写 `reset_equal=true`。
- **修复及最小回归范围**：必须要求重置报告及截图存在，明确完整探针、报告runner SHA/当前冻结来源/基线身份一致后才比较并成功；缺失、真实partial、错误runner均须非零且报对应原因，真实默认+Home及逐像素差异负控保持通过。保存明确的基线、runner、报告/截图摘要和命令。只重跑比较器及其正反例、受影响身份证据；不为此重复完整实验门。

## 两次独立复审未通过后的诊断

R1 的3项行为缺陷已用原始及兄弟控制关闭；其余两项原缺口是证据闭环。工作修复了F-04正向比较，并补了单像素颜色变异，但没有将“重置报告缺失/partial/错runner”列为比较器本身的负输入：代码把必需分支当作可选，且未验证其报告身份。问题不是旧正例修得不够，也不是需要降级合同。下一次work须在 [诊断](diagnosis-r2.md) 中逐项实现并验证失败传播和冻结身份，再请求复审；第二次未通过不能只重跑旧的颜色变异绿例。

附带观察：可见组原点的后代完全位于屏幕横向之外时仍有组手柄；当前合同明确禁止近/远裁剪伪影，但没有明确禁止用可见组原点把屏外后代拖回，因此此探索不作为本轮新缺陷或新增验收要求。

## 状态与后续

AC-01/03/04/05的受影响路径在本轮覆盖中通过；AC-02的本次默认和重置正例通过，但AC-06的比较器失败传播/身份未闭合；AC-07不能通过。四项Spec delta目标及GOAL-008存在，冻结hash见baseline；不在review修改canonical Spec/ledger。完整双端、受影响AGP和原新native/owner/content合入门均属merge-owned pending。未修改实现、运行整档、合并或清理工作树。

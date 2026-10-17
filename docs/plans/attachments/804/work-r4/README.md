# work-r4 — F804-R3-01/02 返修证据（R3 返修轮）

探针：`os-probe.py <worktree>`（三场景 normal/stall/nodrag，断言
exit 语义 + 零遗留宿主 + normal 零输入残留双采样；收据绑定
git HEAD / test 二进制 / host 二进制 sha256）。

## 运行清单（按时间序，失败实录保留）

| 收据 | 结果 | 说明 |
|---|---|---|
| os-probe-receipt-232808 | FAIL | 调试轮：normal 未达（触发目标几何误判期） |
| os-probe-receipt-233245 | FAIL | 调试轮：同上（键盘→固定点→网格形态摸索） |
| os-probe-receipt-233537 | FAIL | 调试轮：同上（组件序交换前最后一轮） |
| os-probe-receipt-234746 | PASS | 组件序交换+中心点目标点序后首次三场景全绿（代码未冻结，HEAD 仍=21d5f9c018） |
| **os-probe-receipt-235226** | **PASS** | **canonical**：冻结提交 1ca36a674e9199985f8bc804d1ac0f61d1bcaa97 之上（test_bin=5a4c2c36…、host_bin=cdf30c7b…）——plan §9 R4 引用本收据 |

normal-*.log.txt / stall-*.log.txt / nodrag-*.log.txt 为各次运行的完整
stdout/stderr（含生产 CALL_HANDLER 链与 finish marker 值）；
2351xx/2352xx 前缀三份属 canonical 轮。

根因与修复见 plan §9「R3 诊断/覆盖增补表」：normal 场景为虚拟窗级联
遮挡（component[0] 发起方被后窗盖住按钮）——组件序交换 +
(50%,57%) 中心点目标；child 归属为 ChildGuard RAII + example
STALL_MS/NO_DRAG 反控钩子。

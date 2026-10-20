# PLAN-798:r7 R21 实际主检出依赖最终复核 — pass

review_scope: affected-final；reviewed_commit:216d139aba888760963d12a5f2cbe693489b06c4；dependency root lock SHA256:F1F9855F0ECB56B0F316FF53BE5BDA96465167BF4AE9D933E2223122428AD125，与主检出逐字节一致。auto-down仍c39ac7f9d；独立实验workspace及worker自己的入库锁、core/store/sema/测试输入/配置/阈值未改。

实施会话限制与R19相同，未获得独立上下文/另一个评审人。以R19完整AC/V/K14/SD矩阵为基础，重新检查R20唯一import删除与真实主检出依赖差异，核对root与独立workspace边界、实际消费路径和原件。未把旧E970根锁的通过或失败混成当前依赖结果，也不根据依赖差异直接宣称旧flake因果已证明。

主检出根锁的类型检查通过。新依赖下798正式家族7/7、30样本门exit0；编辑P50/P95/P99=784.1/814.6/859.1ms，签名P95=671.4ms。编辑P95从30条日志独立重算一致；签名逐样本未输出，只核执行门汇总和断言，不虚构其raw序列。详见main-lock-formal-*。先前687.4/234.9和872.5/387.7等均保留原始提交与依赖身份。

本轮无跟踪代码/功能/验收或Spec变更；R19全部正确性/资源/来源/lease/GC结论和R20函数实现逐字节等价证明继续适用。独立实验180样本与资源证据使用其未变的入库锁，不受根锁对齐影响。原R19关于最终新增告警的遗漏已由R20明确修正；依赖旧warnings不冒称清零。

AC01..09、V01..14/K14、Spec增量pass；仅merge-owned四档与发布/归档/guard清理pending。以下四档在本R21通过后以同一代码/实际主检出依赖冻结重跑：完整无过滤cargo t --no-fail-fast --test-threads 8、tv、独立workspace/native、auto-test-case合同。并行度是运行参数，未改过滤/断言/阈值。未通过之前不合入，不以旧不同根锁结果代替。复用的非root证据理由和源码/锁身份均如上公开。

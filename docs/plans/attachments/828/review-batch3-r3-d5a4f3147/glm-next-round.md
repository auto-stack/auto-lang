# PLAN-828 第三批R3 → work：仅C04两处窄残项

同树D:/autostack/.wt/lang-828/auto-lang，基线d5a4f3147e336095e0e9d9e6207b67aea1c51d1c。**C03本轮已闭；C01/C02/C05原具体错误保持。不要重做执行器观测、publication、候选、存储身份或回收模型。**

1. **握手Result出口漏收据**：formal_worker.rs的Init发送`?`与`match link.recv()?`仍提前返回Err。新other分支只处理合法JSON错误类型，覆盖不到EOF/坏JSON。沿本轮handshake-controls保持输入/oracle：ready前EOF与坏JSON应得到可保存的红report（0样本、原因/输入/worker身份/init），当前直接Err；ready类型错与第二编辑坏JSON已正确保存，保持。检查同类握手发送错误，不逐个字符串打补丁。正式entry必须先有该次失败证据再expect/assert/退出。

2. **HIR字节重复累计**：session.rs在A07 put前和Ok后各加了一遍hbytes.len。真实四个成功put载荷173+252+72+406=903，phase_io却1155，多出同一份252B HIR。运行io-controls：旁路记录器委托真实put一次，fresh无Conflict歧义；修后903，no-store正控保持。把attempted/accepted/Conflict/修复/失败的“逻辑payload字节”口径写清，保持已发生计量；不增加物理磁盘监控或改预算。

沿用B3-C04/P2一个finding的两个子关闭行。修好全部后跑上述小反例、原失败落盘控制、C03代表性正反控及必要定向检查。数据流/所有权若不改，不要反复跑旧两个512或整档；最终30轮/raw与长负载证据根据新冻结实际路径变动决定复用或补跑，并明确身份依据。coverage旧表_t08应5而非4，随同簿记校准。

提交含全部修改的新冻结根＋两行关闭证据，交**C04窄复审**；通过后T-10。r3/executing0/11、merge-owned门pending，不代发消息/新建chat/合入。

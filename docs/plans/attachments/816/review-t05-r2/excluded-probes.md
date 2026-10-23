# 排除的初版探针

- `adjacent-initial-narrow.json.txt`：主检出既存二进制 D4E9F00F... 的真实 describe 只有 describe/validate 两项，故 UI 请求被诚实拒绝；它不是所需的旧五操作工具对照，不据此产生产品 finding。
- 初版 legacy proxy 的 std::io::copy 使用 Stdout 缓冲，未逐块 flush，导致 CLI 等 describe 回答超时45秒（scratch `p816-t05-r2-adjacent-5zivo4y2`）。这是审查宿主错误；已改每块 write_all + flush。该超时不计产品问题，只有修正宿主后有效协议控制才用于结论。
- R1 的 replay.py / client-probe.rs 与旧冻结哈希一致；R1 replay-results.json.txt 被工作回放改写，已不等于 R1 final-state 的冻结哈希。本轮复制同字节源码/断言到独立 R2 目录重跑，不将当前 R1 结果文件当旧红色原始记录，也不伪造恢复它。

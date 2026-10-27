# PLAN-824 F03 R2 修复入口与两轮复审诊断

基线 `6a68240e89bdfd141ecfb893f83c7718bf503056`，专用树 `D:/autostack/.wt/lang-824/auto-lang`，分支 `plan-824-dev`；D821/D822/D828/D852 未变。F01/F02 与警告已关闭，F03 仍开放，无新合同决定。

| 不变量 | 两轮漏检原因 | 实际路径及状态 | 调整及关闭条件 |
|---|---|---|---|
| 类型展开在构造前计量实际字节、工作并接受取消 | R1 只看节点上限和落槽后的 deep_size，没覆盖短 DAG + 长重复名称的临时操作数；R2 16,011 字节合法输入在 65,535 节点下展开至少 38,796,208 字节但峰值 240 | MIR 类型表→ `resolve_dyn_type`→pack/is/take 的临时树→槽或比较；成功、预算拒绝、取消及回滚 | 在实际分配前对名称、容器和类型节点计量，工作/取消逐步生效；重放长名与小正控、节点上限、三调用方及紧燃料；检查峰值与保源 |
| portable 类型读取、写入和准入也按工作轮询并在分配前受预算保护 | 旧取消测试只遍历值节点，空列表仅一个值节点；write_tydesc 无上下文，read/admit 虽计工作但不轮询 | freeze 的 descriptor 写入、unfreeze descriptor 构造、admit/ty_ident 及当前 binding 构造；输入/原值只借用，拒绝后可重试 | 用 1026 类型节点且值树很小的复审输入验证 freeze/unfreeze 取消及重试；另覆盖仅准入阶段取消、长名称/宽树的分配前拒绝和 binding 适配 |

原始重放 `review-r2/probes/src/main.rs` 在上述基线上已复现报告：长名 Success/peak 240，freeze/unfreeze 均 Ok、polls=0，小对照 Success；F01/F02 错误及 bool 正控符合已关闭结论。当前为修复入口，不代表完成。完整的构造/适配字节模型、共享预算及有效测试结果仍需核对。

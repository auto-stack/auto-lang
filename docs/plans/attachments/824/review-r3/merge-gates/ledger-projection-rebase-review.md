# PLAN-824 台账来源重放独立复审

- outcome: PASS；reviewer `/root/review824_sync`；v104 SHA256 `6805b6f86ccb25606dce60e1d5f50a23b71a436d692ed9cfa90d8cdcc6b85324`。
- 主干仅增加其他计划文档，22项提交逐项等价重放。修正8个代码来源和2个证据来源引用为重放后的等价祖先；逆向替换后字节完全还原前一已审投影，其他字节无改动。
- 10个来源文件的 Git blob SHA 与台账标签一致；投影报告8个证据链接可从重放后的证据提交恢复。真实 SpecsStore 加载 104/6/1289，schema 0错误。

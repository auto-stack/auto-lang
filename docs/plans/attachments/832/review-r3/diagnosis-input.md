# PLAN-832 连续两次 final needs_fix 后的覆盖诊断

R1/R2 均未获 final pass。以下诊断在 R2 修复实施前建立，针对剩余不变量，不重开已独立关闭的 F01/F02。

| Remaining invariant | Why earlier checks missed it | Affected paths/states | Design or oracle adjustment | Closing condition |
|---|---|---|---|---|
| 开发 CLI 自身能在无 Cargo/AUTO 环境时按 owner 发现 UI tool | Cargo 父测试自带 `CARGO_MANIFEST_DIR`，且 R1 测试先 `resolve_existing` 再把结果作为 `AUTO_UI_TOOL_BIN` 注入 CLI；只证明测试 helper，不证明普通生产进程 | `auto-tool-client::discovery` 的开发候选；`ToolUiOps` 默认 spec；直接 `auto gen`；安装邻接、显式覆盖与缺件负控 | 从实际 `current_exe` 的 `target/debug|release` 开发布局推导 Man 的同组 UI owner；要求 repo marker，避免安装目录/任意主检出。永久 driver 清除 Cargo/AUTO 发现环境并停止工具注入；重放 R2 六格生产矩阵 | 直接开发 CLI 在无环境时真实生成 `App.vue`；显式有效覆盖优先，显式缺件具名失败；安装邻接继续通过；不搜索 `D:/autostack` 主检出 |
| 按当前文档恢复的真实 backup 必须 verify 且旧命令可运行 | R1 只验 fresh/install/retired release，没有从 producer 生成的带 `.man-backup.marker` backup 执行文档步骤 | Core/full 共用 backup marker、严格 `manifest_files` 校验、手工恢复；自动异常回滚与陌生额外文件保护 | 文档在真实 backup 移回 root 后，将本次 owned marker 移到具名保存位置，再 verify；保持严格校验，不忽略陌生文件。发行 driver 用真实升级产生 backup 并复演文档步骤 | upgrade→真实 backup→文档恢复→verify=0→旧脚本实际输出；marker 被保存而非混入交付；自动回滚/额外文件拒绝语义不变 |

统一门仍归 merge；本轮只执行上述反例、合法对照和必要编译/卫生检查。

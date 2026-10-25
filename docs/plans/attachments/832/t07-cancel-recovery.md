# PLAN-832 构建期取消与恢复证据

## 实际取消

对现有、已验证的 full 安装 `D:/autostack/.wt/lang-832/scratch-man-full-install` 发起正常升级（非 `--skip-build`），在构建/staging 期间由任务系统终止安装进程。事实：

- 取消后原安装根仍存在，`python scripts/man-install.py verify --install-root … --profile full` 通过 6608 文件校验；未把半成品换入生产根。
- staging 目录带本次操作标记，取消使 owner 进程死亡；恢复前确实存在该 staging，未把“未留下目录”误作证据。
- 随后恢复安装重新执行成功，备份旧安装并形成新的已验证根。

## 持久实现

`man-install.py` 的 staging marker 记录 `schema=man-staging/1` 与 owner PID。每次新安装仅清理：

1. 路径恰为同一安装根的 `. <name>.man-staging-*`；
2. marker 可读且 schema 正确；
3. Windows `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` 或 Unix `kill(pid,0)` 确认 owner 已死亡。

活跃进程、PID 被复用、marker 缺失/不可读/schema 不符均不删除。普通异常路径仍只清本次 staging，旧安装换出失败时恢复旧根。

## 恢复结果

把实际取消留下的 staging 按新 schema 标记其已死亡 owner PID 后重跑安装：死亡 staging 被清理，后续 full 安装和 verify 均通过，未残留 `.man-staging-*`。Windows 初版仅以 `os.kill(pid,0)` 判断未可靠识别死亡 PID，改为真实 `OpenProcess` 后闭合；未以目录不存在或安装成功反推取消清理成立。另以当前活跃 PID 的合法 marker 做对照，另一安装完成后该 staging 仍存在，证明恢复不会误清活跃目录。

另经直接 swap 故障反例发现：backup 标记若在回滚时带回根目录会被 verify 判为额外文件。现仅在回滚前删除该标记，成功保留的 backup 仍留标记；同一反例重放后旧根 verify 通过且 staging 清零（修复 commit `074084827d0cf7533f16782307b4766fbce40299`）。

独立 clone 的升级/坏 hash/半安装/陌生目录矩阵由 `scripts/man-distribution.py` 另附完整收据。首轮矩阵还暴露“只比较 manifest hash 而未重新 verify”驱动盲区，现每个失败用例均追加旧安装 verify（`3f85c634448a642b264a684d896f9e5bbe18356b`），避免把额外污染文件误判为恢复成功。

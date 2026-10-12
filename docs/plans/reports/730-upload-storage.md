# PLAN-730 T-04/T-08 存储报告：安全目标、create-only 发布与故障矩阵

- 代码基线：@ `8a2fc567e`；单测命令 `cargo t plan730`（http_upload_service::tests + tests/plan730_http_upload_tests::plan730_probe）。
- 原语证据（T-01 探针，Windows 实测）：`std::fs::hard_link` 对已存在目标 `AlreadyExists`
  （原文件字节不变）→ 发布 = hard_link + unlink(staging)；`std::fs::rename` 覆盖已存在目标（反证拒绝）。

## 1. 目标安全矩阵（`plan730_commit_target_matrix` + a2r facade 测试）

| 目标 | 结果 |
|---|---|
| `../escape.bin`、`a/../../b.bin` | 403（词法拒绝，零发布） |
| `C:/abs.bin`、`\\\\srv\\s\\b.bin`（UNC） | 403 |
| `missing_dir/x.bin`（父目录不存在） | 403（只允许已存在的安全父目录，不自动创建） |
| `..`（纯遍历） | 403 |
| 合法已存在父目录 `ok/x.bin` | 201，staging 清零，磁盘字节相同 |

同卷/嵌套配置（`plan730_root_config_rejected_before_body_read`）：staging 嵌套在 root 内、
staging 缺失均在**读取 body 之前**拒绝（InvalidOptions failed 会话）；同卷判定 Windows
canonicalize 卷前缀 / unix st_dev（T-01 P3）；跨卷 hard_link EXDEV 兜底映射配置诊断。

## 2. create-only 并发仲裁

- 同名二次提交 → 409（VM e2e + 生成 e2e + 单测三面）：原文件字节不变；staging 清理。
- 并发双 commit 由 `hard_link` 原子性收敛（第二个 AlreadyExists → 409）。
- commit gate = hard_link syscall：gate 前取消胜出（不发布+清理）；gate 后迟到取消
  等实际结果（`upload_late_cancel_count` 探针计数；成功保留、失败清理）。
- 发布成功后 unlink(staging) 失败 → 成功收据 + 可观测残留诊断（`upload_residual_cleanup_failures`
  计数 + 私有目录路径日志；不宣称零残留）。

## 3. 故障/清理矩阵

| 场景 | 结果 |
|---|---|
| wire 超限（options 下调 512B，发 1KiB） | 413 failed 会话；staging 清零；commit 409；reject(0) 采纳 413 |
| 接收中取消（cancel_session） | staged/接收路径立即终态+清理；后续 commit 409 |
| lease 到期（300ms 旋钮） | 看门狗自清（**自 abort 修复**：先摘自身句柄再 release）+ 过期墓碑 → 迟到 commit 真实 410 |
| writer 通道关闭未 Finish | writer 自清 staging（覆盖调用方 future 被 drop 路径） |
| Content-Encoding: gzip | 415 failed 会话（读取前） |
| legacy 写失败 | 真实 500（`upload dir create failed: ...`）；不返回假路径 |
| legacy 绑定失败/handler Err/取消 | provisional 文件删除（目录回空） |

## 4. 平台注记

- Windows 实测全绿；Linux CI 侧同断言（volume_key 走 st_dev；hard_link EEXIST 语义同源）。
- 中间段父目录替换残余竞态与 729 同豁免边界（拥有 root 写权限的本地竞态者）；
  终段 symlink 目标由 hard_link 的 create-only 拒绝（已存在 → 409，不跟随）。

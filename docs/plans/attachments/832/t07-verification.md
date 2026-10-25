# PLAN-832 T-07 独立发行、重定位与失败恢复验收

最终收据：[t07-distribution-receipt.json.txt](t07-distribution-receipt.json.txt)，**14 pass / 0 fail**。

固定输入：Man `3f85c634448a642b264a684d896f9e5bbe18356b`；lang `ffcf8a59e5a66c3dd154c0511ee2843b533fda78`；UI `aee499e87093ff727f68ac8467c506e9ecf6ecdc`；down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。clone 内实际 HEAD 与之一致。后续文档提交不改变生产输入。

## 通过项

| Case | 实际证据 |
|---|---|
| SRC-1 | 四仓本地独立 clone，非 worktree 复用 |
| DST-1 | Man fresh `cargo generate-lockfile` 成功 |
| DST-2 | clone 内 Core `cargo build --locked` 成功（94.2s） |
| DST-3 | full 安装成功（277.5s），verify 与 locked SDK metadata 通过 |
| DST-4 | 第三方 crate 的管理+SDK+协议真实执行，单一 source、无 UI/compat |
| DST-5 | 安装 CLI 真生成并 build Vue，dist 产物存在 |
| DST-6 | 删除 target 后整组 A→B，Core locked 重建（107.7s）；安装重定位 verify、脚本执行与 locked metadata 全过 |
| DST-7 | 再次升级成功，旧安装保留于具名 backup |
| FLR-1 | 陌生非空目录拒绝，用户字节不变 |
| FLR-2 | prepare 失败具名，旧安装 verify 通过，staging 清零 |
| FLR-3 | swap 半安装失败具名，回滚后 verify 通过，staging 清零 |
| FLR-4 | 等长内容篡改被 verify 拒绝，恢复原字节后复绿 |
| FLR-5 | 显式缺失 UI tool 在生成副作用前拒绝，未创建 gen/ |
| FLR-6 | 故障后 managed install 再次成功并 verify |

## 构建期取消与 oracle

[t07-cancel-recovery.md](t07-cancel-recovery.md) 另记录真实取消：旧安装不被半成品替换；死亡 owner staging 被清理，活跃 staging 不误删。回滚修复=`074084827d0cf7533f16782307b4766fbce40299`；每个失败用例追加旧安装 verify 的驱动强化=`3f85c634448a642b264a684d896f9e5bbe18356b`。

首轮曾遇到 Windows 长路径、重定位后跳过 UI tool 重建、backup 标记污染及驱动未重新 verify 等问题；均已按实际原因修正并在最终版本重放，而非删案例或降低验收。

T-08 现在可以切换语言消费者并删除迁出原件；merge 统一门仍 pending。

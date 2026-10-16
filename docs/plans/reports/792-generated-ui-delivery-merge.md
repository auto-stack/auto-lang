# PLAN-792 合入沉淀收据（生成式 Rust UI 独立构建交付）

- **stage**: merge
- **plan_id**: PLAN-792
- **plan_revision**: 1
- **outcome**: delivered
- **delivery_commit**: 归档回执补记（最终 tip = 报告提交后的 worktree 链尾）
- **reviewed_commit（R5）**: `41d95ea95d6d8209d2373336e758729826bc1fbf`（rebase 前旧链 tip）
- **base_commit**: `ccf66f013b4b6351202f4e47669f8c51f7457c54`
- **worktree**: `D:/autostack/.wt/lang-792/auto-lang` / branch `plan-792-dev`
- **archive path**: `docs/plans/archive/792-generated-ui-independent-build.md`

## Checkpoint 回执

| Checkpoint | 状态 | 证据 |
|---|---|---|
| prepared | done | R5 pass@41d95ea95（五轮复审链闭合）；冻结 SD：SD-01 新文件 + SD-02/SD-03 增量；触面=auto-man rust_ui/automan + auto CLI main + fixture/docs |
| landed | done | rebase over master（并行 789..795 已落地）：range-diff 5/7 全等 + 2 提交仅上下文位移（Cargo.toml exclude 并集/auto-man Cargo.toml hunk 上下文）；旧→新映射 11c3f9ca0→974d93a18、8a83e1f39→7af3d60b4、477e49819→16659a8c3、dc645528f→5697308a1、6650077c7→bf381642f、33d45f708→199bd3b4f、41d95ea95→dbb2b14b5；ff-only 落地 |
| ledger_refreshed | done | guarded offline projection（HTTP store 探测无响应）；`.autoos/specs.json` v36→v37；items P792-D1/D2/D3/T/R5/M；范围守卫=git diff 仅 P792 条目+版本；回读校验通过 |
| archived | done | `docs/plans/archive/792-generated-ui-independent-build.md`（status=archived, 8/8）+ 本收据与 attachments/792 入库 |
| cleaned | done | wt-guard clean → worktree/分支/组目录（含 auto-down 兄弟）移除 |

## Merge 整合适配（R5 复用理由失效的补跑——如实披露）

R5 明示"merge 整合导致源码闭包/依赖/生成 manifest/测试配置变化，复用理由即失效"。
rebase 后 master 已含 PLAN-793（auto-lang 新增**必需** path 依赖 `auto-diagnostics`），
首次整合 driver 复放 V01 红（vendored 闭包缺该 crate → manifest load 失败）。
处置（commit 见链）：

- `DELIVERY_CRATES` 9→10 crate（+auto-diagnostics；其自身依赖 auto-val/auto-lex
  已在闭包；根级编译输入五件与 include 越界面经盘点不变）；
- 整合树全量重放：定向 delivery 9/9 + **driver 25/25**（CLI 重建 + V01 fresh
  解析/真实 build + V02 重定位 locked + V03 十二负控 + V04 全量重跑，非复用）；
- SD-01 闭包清单与生成器披露文案同步（10 crate + 闭包随 auto-lang 必需
  path 依赖演进的登记规则）。

该适配超出 R5 冻结基线（41d95ea95）的字面内容，属 rebase 后语义整合缺口；
除上述机械闭包追踪与文案外，交付机制代码与 R5 受审版本逐字节一致。

## Merge-owned 冻结门禁（最终 tip）

| 门禁 | 结果 |
|---|---|
| 裸 `cargo t` | **5335/5335 passed**（70.2s，--no-fail-fast 全量枚举 @最终 tip）；fail-fast 默认档首两跑在 plan484_024 点位瞬态红（solo 2/2 绿，第三次全量枚举绿——790/791 收据在案负载时序 flake 族，非本计划触面：auto-lang 零 diff） |
| `cargo tu` | 不触发（`ui_gen/**` 零触面——diff 仅 auto-man/auto CLI/docs/fixture） |
| `cargo tv` | 不触发（VM/compiler 零触面；auto-lang 源码无 diff） |
| `cargo tf` | 批量档，见到期检查（792%5≠0；收据 `last_covered_plan_id=795` 已覆盖本计划） |

## 交付概述

- 交付模式：`auto gen --delivery-root <dir>`（显式可搬动受管 bundle；开发模式零变化）。
- 独立消费者验证：fresh 解析+真实 build（不借 workspace lock/rlib）+重定位
  locked 重放+资源 SHA-256 随包 verify.py 正式入口+十二负控。
- 五轮复审链：R1 资源正式入口/失败保旧 → R2 外来目录+Windows 根相对 →
  R3 backup 事务归属+TOML 语义 → R4 GC 生命周期 → R5 pass。
- 边界如实登记：bundle 含 auto-lang 全库等 10 crate（33.7MiB 量级）；已证平台
  仅 Windows/x86_64-pc-windows-msvc；registry 发布/全 renderer 发行不在覆盖内。

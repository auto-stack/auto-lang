# PLAN-831 T-02 旧仓重采与保全收据（2026-10-24）

## 盘点（inventory，只读）

- 仓：`D:/autostack/auto-ui`（remote gitee auto-stack/auto-ui）；单 worktree，stash 0。
- tip：master=`f7d92eef273c7f5592cce9d1612fa28ebe12d45e`（=refs/heads/legacy/pre-v06-extraction
  =origin/HEAD=origin/master=tag legacy-pre-v06-f7d92eef）。
- 其余 refs：origin/gpui2=`1e44f92c…`、origin/gpui3=`233ecff1…`、origin/iced=`7d7fc749…`、
  tags v0.1=`1de8af0c…`、v0.2=`c243b712…`（共 10 refs）。
- WIP：`M Cargo.lock`（唯一）；untracked 0；ignored 保留 0。**主检出 WIP 未触碰**
  （快照已字节级捕获；属其 owner 状态）。

## 保全（状态寻址快照，payload 仓外独立恢复位置）

- payload：`D:/autostack/.recovery/831-legacy-preservation`（沿 784 仓外惯例）。
- snapshot `45044cf9507d5628`：bundle SHA256 `d48d04c1aa1b9618403c88ef8ce39005fbd899855943c84ffaa0ed93378168c7`，
  manifest `…/auto-ui/snapshots/45044cf9507d5628/manifest.json`，WIP 捕获=1。
- 锚定：branch `legacy/pre-v06-extraction` 与 tag `legacy-pre-v06-f7d92eef` **reuse**
  （784 既有锚点仍指向 f7d92eef，语义保持；未重复创建）。
- 验证：`verify` ok=true / failures=[]。
- 恢复演练：`recover-drill` ok=true，refs_recovered=10，old_tags=3，
  演练树 `31ede719e46fedbf942050caaca254a29ccd9c02`（scratch，不动原仓）。

## 本 Plan 执行组（唯一权威树；每仓一个）

- `D:/autostack/.wt/lang-831/auto-lang` @`2dafba909`，分支 `codex/plan-831-dev`。
- `D:/autostack/.wt/lang-831/auto-ui` @`f7d92ee`，分支 `codex/plan-831-dev`（迁移工作分支）。
- 禁 junction/symlink；移除前逐树 `wt-guard clean`；不删 legacy 分支/标签/恢复载荷。

## 恢复程序

`python scripts/legacy-preservation.py recover-drill --repo D:/autostack/auto-ui
--payload-dir D:/autostack/.recovery/831-legacy-preservation`（或 snapshots 子命令选择
snap-id）；真实恢复按 784 报告 §恢复语义（快照不覆旧、陌生目录不删）。

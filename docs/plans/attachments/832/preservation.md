# PLAN-832 T-02 旧 auto-man 保全与恢复记录

日期：2026-10-25。目标旧仓：`D:/autostack/auto-man`；实施仓：`D:/autostack/.wt/lang-832/auto-man`（`codex/plan-832-dev`）。保全工具源：`D:/autostack/.wt/lang-832/auto-lang/scripts/legacy-preservation.py`。

## 1. 当前状态重采

完整机器盘点见 [legacy-inventory.json.txt](legacy-inventory.json.txt)。当前事实：

- master HEAD=`51fafe82e8e0c4e66325ee14088eed8c3270e07e`，tracked/untracked WIP 均为空，stash=0。
- refs 冻结映射共 29 项：master、`codex/plan-832-dev`、`legacy/pre-v06-extraction` 同 tip；origin/master 与 origin/HEAD 同 tip；origin/v0.0.x 独立；legacy 标签及 22 个历史版本标签均保留。
- ignored listing=`.vscode/`、`.vscode/iar-vsc.json`、`Cargo.lock`；两个保留文件均实际存在（Cargo.lock=76711 bytes，iar-vsc.json=107 bytes）。
- worktrees 仅旧主检出与本 Plan 新树；当前 refs 无 784 的 `codex/v06-extraction` 准备分支，也无其工作树可认领。本 Plan 树自 master 新建，未挪动任何旧准备树/分支。
- 远端只读记录=`git@gitee.com:auto-stack/auto-man.git`；未 fetch/push/更名/force。

旧默认代码树未修改；保全载荷在仓外 `D:/autostack/.recovery/832-legacy-preservation`，不在任何 worktree 内，未创建 junction/symlink。

## 2. 完整快照

采用快照 **`58c2185bab58ae63`**：

- manifest：`D:/autostack/.recovery/832-legacy-preservation/auto-man/snapshots/58c2185bab58ae63/manifest.json`
- bundle：448066 bytes，SHA256=`b057f3eef68d6ef9fc78f9287d545b97690e608bc9c0f5ebda069422fc2bf26b`，`verify=ok`，HEAD included，bundle refs=31。
- 保留内容：WIP=0、untracked=0、ignored-keep=2（`Cargo.lock` 与 `.vscode/iar-vsc.json`）；状态含完整 refs→objectID 映射。
- legacy 锚：`legacy/pre-v06-extraction` 与 `legacy-pre-v06-51fafe82` 均保持 `51fafe82…`，同 tip 幂等复用，未移动。

实际命令：

```bash
python scripts/legacy-preservation.py inventory \
  --repo D:/autostack/auto-man \
  --out docs/plans/attachments/832/legacy-inventory.json
python scripts/legacy-preservation.py preserve \
  --repo D:/autostack/auto-man \
  --payload-dir D:/autostack/.recovery/832-legacy-preservation \
  --ignored-keep Cargo.lock .vscode/iar-vsc.json
python scripts/legacy-preservation.py verify \
  --repo D:/autostack/auto-man \
  --payload-dir D:/autostack/.recovery/832-legacy-preservation \
  --tip 51fafe82e8e0c4e66325ee14088eed8c3270e07e \
  --branch legacy/pre-v06-extraction \
  --tag legacy-pre-v06-51fafe82 \
  --snapshot 58c2185bab58ae63
python scripts/legacy-preservation.py recover-drill \
  --repo D:/autostack/auto-man \
  --payload-dir D:/autostack/.recovery/832-legacy-preservation \
  --scratch D:/autostack/.wt/lang-832/scratch-legacy-recover-full \
  --snapshot 58c2185bab58ae63
```

## 3. 验证结果

- `verify`：`ok=true`、`failures=[]`；refs/锚、payload 与原文件 hash 均通过。
- `recover-drill`：`ok=true`、`failures=[]`；独立恢复 29 refs、23 个旧标签，tree=`4017c1c84e5b66370953266765de2e7f6f968437`。
- 保全后旧主检出仍位于同一 master/tip，未改默认代码树；新 Plan 树继续作为唯一实现树。

## 4. 首次不完整快照的处置（诚实记录）

首次命令把 `--ignored-keep` 重复写了两次；该参数实际是单次 `nargs="*"`，后一次覆盖前一次，生成 `0a4c7660e4c2cc6b` 时只含 `.vscode/iar-vsc.json`。其 verify/recover-drill 虽通过，但只证明该子集，**不作为 T-02 完整保全证据**。

随后以一次 `--ignored-keep Cargo.lock .vscode/iar-vsc.json` 同时传入两项，生成上述完整快照 `58c2185bab58ae63` 并重跑全部验证。旧快照按不可覆盖原则保留，不删除、不改写；后续只使用完整快照。

## 5. 交接

T-04 可从本 Plan Man 分支追加迁移提交；legacy refs、完整快照与恢复载荷不得作为清理对象。任何后续 refs/WIP/保留字节变化须新增状态寻址快照，不覆盖本快照。跨仓切换失败时以本快照/bundle及新提交恢复，不 reset 旧仓历史。

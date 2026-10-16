# PLAN-785 合入沉淀收据（文本剪贴板宿主能力依赖倒置）

日期：2026-10-16；stage=merge；plan_revision=1；completion_kind=delivered。

## 交付链

- 受审基线：R3 pass @ `c49ca102a`（reviewed_commit；R1 needs_fix→R2 needs_fix→R3 pass，两连败诊断/覆盖调整表见计划 §9）。
- rebase：`c49ca102a` → `2431eb300`（rebase onto master@9fe050809；`git range-diff` 三提交全等 `=`；`git diff c49ca102a HEAD -- crates/ test/` 零差异；映射 64c72d20c→d03ea0cd8、012c89b99→dea54b85f、cb5fa865b→2431eb300；两枚 master 同步 merge commit 内容已在上游、随 rebase 消解）。
- 规范沉淀：SD-01（vm/overview 文本剪贴板 native 宿主能力节+概览指针）/SD-02（ui/overview 未迁边界收窄+ArboardClipboardProvider 装配节），源 hash 与 R3 冻结 delta 逐字吻合（vm AB378E0B…、ui BB13D724…为改前指纹）→ 沉淀提交 **`c8f697009` = 交付提交**（docs-only 后裔，实现/依赖零变）。
- merge-owned 冻结门禁（交付提交上）：**裸 cargo t 5371/5371 PASS**（1548 skipped，72.2s）+ **cargo tv 163/163 PASS**（5.17s）；未触 ui_gen/aavm/trans——tu/taa/tt 不适用；tf 归批量规约（785%5==0 到期，见下）。

## canonical 沉淀

- `docs/specs/auto-lang/vm/overview.md`：新增"文本剪贴板 native 宿主能力（PLAN-785）"节——2925/2926 经 ClipboardTextProvider 供值、core 零 ui 直引、owned/&str 值域、缺席=原 feature-disabled VMError、在位不可用=空串/false、三层装配优先、快照/实例/Clone-merge、注入不改 revision/proof。
- `docs/specs/auto-lang/ui/overview.md`：未迁边界收窄（clipboard 文本已迁出；余 files/images/editor/scroll/terminal/media）+ 新增 ArboardClipboardProvider 装配节（六宿主接线、no-ui 显式注入、feature 边界与 F-785-P01 注记、UI 锁重放边界明示）。

## ledger 刷新

v28→v29（Regime B 受管离线投影：无 store writer 可达——HTTP 探测不通；备份/脚本改写/作用域 diff/schema 校验/回读五护栏；条目 P785-D1/D2/P785-T/P785-R3/P785-M，designs/tests/reviews/reports 四节最低面满足）。

## 观察项（产物新鲜度）

- `target/debug/auto.exe` 未随本合并重建——本计划对既有程序行为面零变（默认构建下装配回接等价实现，见 R1..R3 优先/编码矩阵）；CLI 重建非必需，如需现值可 `cargo build -p auto`。
- 独立 fixture 全新 UI 锁解析漂移（wgpu-hal）沿 779 R3 先例保留为跨仓发行依赖身份验证事项（F-785-P01 同族）。

## 批量回归到期判定

785 % 5 == 0 → **到期**：合入后由 `/auto-plan:regress` 在主检出单实例执行批量全量门（收据 `docs/plans/.last-batch-regression.json` 届时更新）。

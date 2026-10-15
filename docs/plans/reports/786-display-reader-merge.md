# PLAN-786 合入沉淀收据（展示字段读取收口交付）

- delivery_commit: `240281c96`（master ff-only tip；rebase 后映射：
  bde9d0523→13b6a3462 / db71263b5→5e8a2f833 / 322719893→1c5c90223 /
  079b3b8d4→2a7afd6e7 / bb3cd34b7→18416ac2b / 040b25d1c→240281c96，
  `git range-diff a1d5ed311..040b25d1c a37557764..240281c96` 6/6 全等）
- review: R5 final pass，绑定 `bb3cd34b7`（rebase 后 `18416ac2b`），
  [REVIEW.md](../attachments/786/review-r5/REVIEW.md)，SD 冻结 SHA256 69D8D31B…
- merge-owned 门禁: 裸 `cargo t` **5364/5364 绿**（bb3cd34b7 冻结基线，
  108.4s，1548 skipped，零失败）；触面仅 auto-man+fixtures，按计划 §6
  不触发 tv/tt/tb/tu/taa。
- canonical 沉淀: SD-01 `docs/specs/auto-man/project.md`
  （sha256 f12b0b4098450025）/ SD-02 `docs/specs/project-model/project.md`
  （sha256 27d9fd39ef486a38），措辞按 R5 冻结 spec-delta；spec-index.py
  刷新（INDEX 无差异）。
- ledger: `.autoos/specs.json` v25→v26，**Regime B 受管离线投影**（本
  ZCode 会话无 auto-ai spec 工具，无可达 store 服务，沿 784 前例），
  五护栏全过：字节备份（Temp specs-v25-backup-786.json，1,435,675B）/
  脚本 JSON RMW（单空格缩进逐字节格式保真）/ 作用域 diff（除 5 目标
  条目与 version 外深度相等）/ schema 校验（六节 id 精确、own 条目全
  键集）/ 回读（P786-D1/D2/T/R5/M 就位，file+tags 完整）。最小填充：
  designs×2 / tests×1 / reviews×1 / reports×1。
- 真实去重账目: vue.rs `parse_pac_scalar`/`parse_pac_name` 两行扫描循环
  删除；兼容算法单一冻结副本（reader `legacy_scan`）；非选副本
  （rust_ui/vscode/ark/jet/cmd_vue、shadcn/front 路径）显式保留，
  由 784 迁移清单承接（t04 migration-inventory 已登记 display-reader
  迁移面）。
- 生产工件观察（landing≠deployment）: auto-lang 无已登记的运行态生产
  进程/发布二进制重启协议（对照 auto-musk PLAN-092 教训）；`auto` CLI
  若有分发构建属后续发布流程，本次合入不触发。无 web bundle/daemon 面。
- 批量回归到期判定: 见归档计划 §9 merge 收据（786 % 5 ≠ 0，按
  `.last-batch-regression.json` 判定）。

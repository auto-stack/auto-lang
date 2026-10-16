# PLAN-795 合入沉淀收据（包锁核心拆分交付）

- 交付链：`579f6dec0`（base）→ `11ddc162e`（实现冻结）→ `edf982e52`（777 基线重锚，R2 pass 复审绑定）→ docs-only 沉淀提交（本件+索引+ledger v34 投影；实现/依赖零变动，`git diff edf982e52 <delivery> --name-only` 仅 docs/specs|docs/plans/reports|scripts/spec-index.py|.autoos/specs.json）。
- merge-owned 冻结门禁（绑定 `edf982e52`，docs-only 后代代码恒等）：
  - 裸 `cargo t`：**5326/5326 全绿**（90.9s，1 slow=plan738 p4_rust_emission 67.9s，1550 skipped 属档位排除面）。
  - 定向：`cargo test -p auto-package-lock` 14+11=25 绿；`cargo test -p auto-man --lib lock::` 9 绿（387 filtered）；consumer manifest 4 绿；`verify-deps.sh` PASS（闭包=package-lock-consumer→auto-package-lock→auto-val，零 auto-lang/man/gen/ui/vm）。
  - 健康面：变更 Rust 文件 `rustfmt --check` 0；实现 diff `git diff 579f6dec0 edf982e52 --check` 0；新 crate clippy 0 告警（预存 auto-val 6 告警非本计划）。
  - 777 strict：pass/0 violations（基线重锚提交内闭合，R2 独立复验一致）。
  - 触面档：tv/tt/tb/tu/taa 零触发（diff 未触 auto-lang crates/trans/book/ui_gen/aavm）；tf 批量档到期判定=795%5==0 → 落地后按规则移交 `/auto-plan:regress`（主检出单实例）。
- canonical 沉淀：SD-01 `docs/specs/package-lock/project.md`（新，随实现提交入库）；SD-02 `docs/specs/auto-man/project.md` 分工节+模块行；双模块 plans.md（package-lock 新建 + auto-man 795 行）；overview 工具链行；goals GOAL-003/GOAL-016 回写；INDEX 再生（spec-index GROUPS 增 package-lock）。
- ledger：`.autoos/specs.json` v33→v34 guarded offline projection（无 store writer 可达；byte-exact 备份+外科增量+回读校验，见计划 §9 收据）；条目 P795-D1/P795-T/P795-R2/P795-M（designs/tests/reviews/reports 各 ≥1）。
- 诚实边界：pac.lock ≠ 792 Cargo.lock 发行锁；verify 非 Git 内容/供给链完整性校验；resolver/Target/build/exporter/可执行模板未迁仍携语言栈（auto-man→auto-lang 边 grandfather exit R5）。
- 预存红备注（非本计划）：auto-man rust_ui review738_r5/review751 两测在基线裸复现红（792 域）；`cargo t` 档不含 auto-man，本轮门禁不受影响。

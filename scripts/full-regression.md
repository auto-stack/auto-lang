# 固定组周期回归（PLAN-896 / PLAN-898）

普通计划使用cargo t具名固定完整组及必要集成。无参数cargo t为日常范围，cargo t all显式全语言组。
周期触发和主检出单实例保持：计划号%5、或>48h且有合并（含L0）、缺收据即到期，语言侧在主检出
单实例执行一次 `cargo t all`；UI/Man各保owner入口。全仓迁移发现只在明确审计运行（--discover-only），
不是每次局部运行前置。慢组/CPU/内存与构建成本如实分列。

PLAN-898（2026-10-31）：旧 `cargo tf --batch` 调度器（run-full-regression.py 与
periodic-module-schedule.json）、tier 选择守恒基线（full-regression-*-ids/selected.txt、
tier-selection-minimums.json.txt、check-tier-selection-minimums.py）及旧档别名已删除；
850/867/888 历史语义与冻结身份记录见归档计划 `docs/plans/archive/`（844/846/850/867/888/896）。
批量收据仍写 `docs/plans/.last-batch-regression.json`（command 字段=cargo t all），
由 auto-plan-regress 技能执行并三角化新红。

work/review只做相关反例/组与必要发现，不因提交运行周期全集。最终代码审查通过后由merge
执行冻结固定组门；单实例、根目录/依赖身份、故障停止、原报告保护/恢复、原UI/Man手工条件
均继续生效。

零依赖 Rust driver（scripts/full-regression-driver）只桥接 Cargo 别名到 Python 3；平台默认
Python 命令不可用时设置 `PYTHON`。治理自检：`python -m unittest discover -s scripts/tests`。

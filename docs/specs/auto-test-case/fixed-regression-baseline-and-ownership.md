# 固定回归基线与测试归属

> **Status**: active（PLAN-1014 SD-01）
> 范围：`scripts/test-groups.toml` 固定组、`scripts/fixed-tests.py` 选择/守卫/租约、
> `scripts/test-governance/toolchains.json` 工具声明、`scripts/tests/test_fixed_groups.py`
> （`infra-fixed`）与 pilot 治理组的归属纪律。本文钉住"哪个组拥有哪些测试义务、
> 覆盖如何守恒、基线门如何诚实报告"，防删组/缩 coverage 冒绿。

## 当前 owner 与迁移语义

- **固定组是唯一执行入口**：Cargo package/Rust 模块 namespace 定义组；无叶目录/动态
  影响图。`cargo t --list` 只读静态组名；`--select-only` 出静态计划，非执行通过。
- **迁出 crate 的组注册必须退役**：crate 已迁他仓时（如 `auto-lsp` → auto-ui
  `crates/auto-lsp`），本仓 `-p <crate>` 组是失效注册（发现即败）。退役＝本仓不再
  自称运行该测试；owner 入口=目标仓既有测试装配。迁移落地时必须显式记录该映射，
  不得把"本仓不再运行"读作"测试通过"，也不要求语言 `all` 构建外仓。
- **legacy/ACE 家族边界（PLAN-1014 起）**：`test_cases::(ac|abi::|sema::ace_*)` 是
  ACE 桥接族，其 owner=显式 ACE 职责组（`ace-adapters`：正向选择同族表达式）。
  legacy 广组（`daily`/`integration`/`auto-lang-core`）以**族式排除**划界，与
  `run_rust` 守卫前缀表同构——新 ACE 模块自动划归显式组，不靠手工枚举清单
  （枚举式曾漏 8 模块 61 例致三组守卫长红）。守卫不弱化：legacy 选择含 ACE 前缀
  即在执行前拒绝；未知新前缀仍被抓。
- **转移守恒义务**：从 legacy 排除的每个模块必须出现在显式 ACE 组的实际发现里
  （`--discover-only` 可核），且该组实跑绿才算承接完成。删除/改名组不消除义务。

## 工具准备与 pilot 双形态

- ACE 相关检查的组必须**声明工具**（`toolchains.json`，source-bound
  `cargo build --locked --offline`，产物路径+SHA256 收据）；legacy 组不得携带
  tools/experimental workspace（loader 校验）。
- pilot 双 worker 形态：`batch-av`（feature=test-case-batch、
  `experimental/ace/target-batch`）供 group/combined 模式；`av-regular`
  （无 feature、`target/test-tools/regular`）物化 default-off 语义的原路径。
  `infra-pilot-ace`（ACE 域、all=false、tools=case-parent/batch-av/av-regular）
  是真实 CLI 检查的 owner；纯 Python 回合核算留在 legacy `infra-pilot-cli`。
  测试不得依赖环境残留 exe：缺工具=组失败（准备阶段拒绝），不允许 skipUnless
  静默吞覆盖（曾因 ambient exe 差异在冷树/主检出间漂移出假绿/假红）。
- default-off 语义钉住：regular worker 下 independent 真实通过；group 模式明确
  拒绝（exit 2 + error + 全项 NOT_RUN），不 fallback regular 冒 batch。

## 基线门与诚实报告口径

- 六红基线（covered1005 收据）的闭合=每组根因修复+完整组绿，不以条件排除充 PASS。
- 正式 `cargo t all` 在主检出单实例执行；known-reds-only 不是全绿。发现数/实际
  执行数/原条件 NA/ignore 分别计数；环境归因红须隔离复放绿并具名记录，不据此判
  代码失败，也不静默计入通过。
- 守卫揭示的隐藏存量红按"主检出具名复放同红"三角化归属后处置；不得为全量门
  绿改 NA/收据（正式收据只按真实结果更新，不重写历史收据冒当时全绿）。

## 守卫负控纪律

对"守卫敏感"的复验：伪造 legacy 组选 ACE 前缀（如 `test_cases::ac_mn::refusal_tests::`）
→ 守卫在执行前拒绝；合法 legacy 命名空间对照正常执行。修复前的守卫实跳即天然
正例。`infra-fixed`（60 例）持续钉住 loader/合并/退役/ACE 声明等组级不变量。

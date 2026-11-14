# ACE 普通测试逐模块预算（一至两分钟）

> PLAN-1060 SD-01：ACE 普通（非正式性能）测试的逐模块预算合同。本文钉住判据与边界；
> 判定工具是 [scripts/test-governance/ace_cost_report.py](../../../scripts/test-governance/ace_cost_report.py)
> 的 `--mode regular`，守恒判定是本计划 §6.2 的具名负控（固定组 `infra-ace-cost-report`）。

## 预算判据

- **适用范围**：`scripts/test-groups.toml` 中 21 个 ACE 职责组（`ace-syntax`、`ace-source`、
  `ace-host`、`ace-a2x` … `ace-stdlib`）各自的**完整单组调用**，以及 71 个 `domain=ace` 的
  TestCase 入口组。
- **T_warm 定义**：一次完整组调用的 `results.json.wall_seconds`——包含准备、发现/准入、运行、
  父判定与复制/清理；**冷构建/工具构建单列为 T_cold**（`--cold-run`），不混入日常判据。
- **门限**：**≤120 秒是验收上限，≤60 秒是目标**。60 不是强杀门：达到 60 需要削减真实覆盖时，
  保留覆盖并在台账记录目标差额，不得以删测试/`#[ignore]`/拆固定组冒充。
- **判定档**：`GREEN`（≤60）、`WITHIN_BUDGET`（60–120）、`OVER_BUDGET`（>120，退出码 1）、
  功能失败（完整选择但 TERMINAL 非 PASS）——三档互不覆盖，边界不取整（120.001 仍越界）。

## 执行入口与排除

- 常规正确性集合统一排除 `::performance::`（PLAN-1016）；正式性能/规模/持续压力只用
  `cargo t ace-perf`，其 `[profile.ace-perf] test-threads = 1` 与父截止不变。
- Linux/WSL 腿按 2026-11-14 裁定不在 Windows 执行（PLAN-1061 承接）：既不判红也不充 PASS，
  台账记"裁定排除"。
- 既存功能红保留 FAIL 并具名登记（`docs/plans/KNOWN-DEBT-AND-RISKS.md`），耗时达标不覆盖它。

## 证据与复用

- 单次运行复核优先；证据失效条件是源码/工具/输入/配置漂移（driver 的 `source_state` 校验）。
  纯文档/注释改动不使既有测量失效；机器负载造成的临界波动按"环境冲突"记录并择机复测。
- 计时口径不含他计划并发负载的修正系数——负载必须与数字一并记录，同代码不同窗口的差值保留。

## 当前基线（PLAN-1060 冻结提交）

21 组全部 ≤120s：17 组 ≤60s，`ace-a2x` 75.8s 为唯一 60–120s 组；3 组因既有功能红为 FAIL
（`ace-host` 101.6s、`ace-runtime` 13.4s、`ace-rust-lowering` 5.8s）。71 个 case 入口组全部 ≤60s。
聚合 `cargo t ace` 的 `ace-workspace` 单趟 320.7s（起点 879.0s）。
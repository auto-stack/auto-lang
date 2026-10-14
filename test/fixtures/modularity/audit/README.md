# modularity-audit 自测夹具（PLAN-777）

本目录是 `scripts/modularity-audit.py --self-test` 的专用夹具，验证计划 777 的
V-01..V-04 验收项（稳定只读 / 新增边判红 / 候选诚实 / 基线窄豁免纪律）。

要点：

- 这是一个**假想 workspace**（虚拟 manifest，无锁文件、永不编译、永不参与根
  workspace 解析——根 `Cargo.toml` members 为显式 `crates/*`，本目录不在其中）。
  自测把它复制到临时目录、`git init` 成独立小仓后运行真实审计 CLI。
- `crates/fixture-lang` 扮演语言 core；`fixture-lang-sdk` 是其 SDK；
  `fixture-ui` / `fixture-ui-kit` 扮演 UI；`fixture-test-util` 只作 dev 依赖；
  `fixture-app` 是顶层装配。
- 禁向规则只有一条：`lang-core → ui`（normal/build）。合法反向（UI→SDK）、
  dev-only、顶层装配依赖不得误红。
- `crates/fixture-lang/src/text_cases.rs` 植入七处受管命中
  （code×1 / comment×2 / doc×1 / string×1 / test×1 / macro×1）和一处别名盲区
  （`use crate::fixture_ui as fui;` 后经 `fui::` 的调用不可见，属声明的扫描器盲区）。
- 自测的每个变异场景都先证明夹具被实际修改（读回断言），再采信判定结果。

修改本夹具会改变 `--self-test` 的期望计数；如非修复扫描器缺陷，勿动。

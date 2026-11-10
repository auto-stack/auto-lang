# 生成工程调试策略对照研究（PLAN-1001，2026-11-10）

分类：repository-boundaries / 研究结论+用户裁定（本文件冻结研究实证与采用决定；落地实施属后续独立计划，D-1001-R7）。
来源：[PLAN-1001（归档）](../../plans/archive/1001-incremental-cache-lifecycle-and-generated-debug-policy-validation.md)；
方法基础 [910 生成缓存生命周期](generated-cargo-build-lifecycle.md) 与 [942 共享研究](generated-workspace-sharing-study.md)。

## 1. 问题与裁定

问题：生成 workspace（UI dev / Web release）的调试信息策略（当前=cargo 默认 dev 全量）是否值得调整？

**裁定（2026-11-10 用户）：采用 C** ——成员 `debug = "line-tables-only"` + `[profile.dev.package."*"] debug = false`
（全部非成员含 path 依赖）。理由：大部分 Rust 代码由 AI 编写，一般无需进入调试模式；配置须保留基本行信息使
报错栈仍有用——C 保留**成员行号**（dbghelp oracle 三臂均解析生成调用点 `main.rs:379`）；**依赖帧无行号/类型信息
为裁定已知的接受代价**。产品默认未因本研究改变；落地为生成器/配置默认的变更属后续独立计划。

## 2. 实测数字（002-counter+013-todo 共享逐臂冷 target，wrapper 禁用，冻结工具 auto 6757a12f/auto-ui-tool 1d455b2e；来源五仓/锁/成员名/路由全冻结于 PLAN-1001 §5 T-06 回填）

| 臂 | 闭包（target+生成ws，分配字节去重） | 冷编 | 编辑重编 |
|---|---|---|---|
| A 现行（默认 dev 全量） | 11.71 GiB（incremental 4.34/deps 5.94/pdb 1.80） | 208.2s | 14.5s |
| B 成员 line-tables-only | 11.10 GiB（deps 与 A **逐字节相同**） | 197.0s | 14.1s |
| C B+依赖 debug=false | **5.13 GiB** | **144.0s** | **7.0s** |

- E_delta=43,392B（重复冷臂 A2 全闭包差 0.00035%）；A−B=−5.20%（成员 incremental+pdb）；B−C=−51.0%（deps rlib/incremental/pdb 三类）。
- 机制：package 覆盖**不禁用**增量（三臂会话数 31/31/31）；A vs B 依赖闭包逐字节不变；逐单元 JSON：A 全 2、B 成员 line-tables/依赖 2、C 成员 line-tables/依赖 0。
- Web（auto-chat/auto-database，cargo release）：`[profile.dev.*]` **零影响**（B/C 配置下 0 单元重编实证）；真实 HTTP 路由 200/405 验证。
- 迁移/回退：既有 target 切 C=595 单元重编/共存峰 17.01 GiB（旧 A rlib/pdb 残留至清理）；C 回退完整 debug=595 单元/共存 16.00 GiB；恢复=移除配置重建（覆盖表随配置消失）。
- 缺口如实记录：本机无调试器（断点/局部变量未验证）；UI 窗口应用行为未运行（活动桌面环境边界）；构建期瞬态峰值未采样（稳态为主口径）。

## 3. 落地要点（后续计划输入）

- 注入机制：owner 入口每次构建**重写**生成 ws 根清单——manifest 注入不可行；用 scope/仓级 `.cargo/config.toml`
  （cwd 祖先）逐臂/逐环境切换。**勿用** `[profile.dev] line-tables + [profile.dev.package."*"] debug=true` 钉法：
  显式 debug=true 令 cargo 给 host-only/build 依赖（默认延迟关闭）开调试信息，级联全闭包重编。
- 先例：本仓 `experimental/ace` 自 PLAN-896 起 dev/test 根即"成员 line-tables-only+`package."*"` debug=false"
  （见[测试构建与缓存生命周期](../auto-test-case/build-storage-lifecycle.md)）——生成消费者采用 C 与 ACE 既有形态同构。
- 落地须处置迁移共存（≈+5.3GiB 瞬态）与旧变体非增量残留（cargo clean 或整 target 治理，超出 PLAN-1001 增量工具管辖）。

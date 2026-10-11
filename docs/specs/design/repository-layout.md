# 仓库布局与资产归位

> Current-state：PLAN-761，2026-10-11；根 Git 树 30 项。

## 根目录职责

根目录保留源码/发布入口（auto、blueprints、crates、docs、examples、packages、parity、patches、schema、stdlib、website）、test/scripts、仓库与工具约定配置、Cargo/Node 入口、双语 README 和 LICENSE。AGENTS.md、CLAUDE.md 保持工具自动发现位置。

根 item 数按 **Git 树**统计；本机缓存/配置/未跟踪临时文件不代表 GitHub 上的目录行数。为了少一项不能删除测试覆盖、验收证据或破坏配置自动发现。

## 测试与辅助入口

- `test/runners/{a2c,a2r,a2ts,vm}_tests.at`：从仓根运行的语言级套件入口；golden 仍在 `crates/auto-lang/test/`。
- `test/auto/`：Auto 语言单测；`test/ui/vue-gen-gap/`：Vue 生成夹具，其 pac 相对路径直接指向根 blueprints。
- `test/fixtures/plan446/{b3check,j2check}/`：保留历史回归语料；Rust J1 repro 与脚本引用实际新路径，缺件不可作为通过。
- `scripts/repro/p446/`：MCP 复现脚本；AUTO_BIN 和 P446_CORPUS_ROOT 可覆盖运行对象，`--check` 验证路径而不拉起窗口。
- `scripts/tools/`：维护工具；native-fixture 继续为独立 Cargo workspace，不并入根 workspace。
- `scripts/deploy/`：systemd/nginx 模板与真实代理验收工具；`scripts/deploy-website.ps1` 根据脚本位置计算仓根。搬迁验证不触发远端部署。

PLAN-761 前的 `tests/`、`tmp-corpus/`、`tools/`、`deploy/` 是上述路径的历史名称；旧计划、冻结报告和账本历史描述可保留旧名，执行入口与 current-state Specs 使用新名。

## 文档、本机配置与临时件

- 独立历史风险台账在 `docs/plans/DEBTS.md`，与 KNOWN-DEBT-AND-RISKS 并存时不丢唯一记录；工作流向当前账本登记。
- 文件操作安全规则在 `docs/guides/filesystem-safety.md`，以 AGENTS 的 worktree 禁链接与移除前 guard 为准。
- Plan600 验收 PNG 原字节保存在 `docs/plans/evidence/600/`；历史说明中的同名图对应此件。
- `.agent/`、`.claude/`、个人 `.vscode/iar-vsc.json`、`.clangd` 不跟踪；共享技能使用 `.agents/skills/`。取消跟踪时保留用户的本机配置。
- 可维护脚本进入 scripts/repro，需留存的证据进入 docs；临时输出集中到已忽略的 scratch/ 或 tmp/。根 `.tmp-*`、测试副产物不入库。
- 构建 debug 忽略根锚定为 `/debug/`，避免吞 `crates/auto-lang/src/ui/debug/` 源码；JSON 等过宽规则的命中不等于文件可删。
- 根 package.json 及两个现有锁文件仍保留；包管理器统一另行明确，目录整理不取消 npm 快捷命令。

来源：[PLAN-761](../../plans/archive/761-root-directory-cleanup.md)、[原检查报告](../../reports/root-directory-audit-20261011.md)。

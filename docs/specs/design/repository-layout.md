# 仓库布局与资产归位

> Current-state：PLAN-763 / PLAN-777，2026-10-15；根 Git 树 24 项。

## 根目录职责

根目录保留源码/发布入口（crates、docs、examples、packages、stdlib、website）、test/scripts、仓库与工具约定配置、Cargo/Node 入口、双语 README 和 LICENSE。AGENTS.md、CLAUDE.md 保持工具自动发现位置。

根 item 数按 **Git 树**统计；本机缓存/配置/未跟踪临时文件不代表 GitHub 上的目录行数。为了少一项不能删除测试覆盖、验收证据或破坏配置自动发现。

## 测试与辅助入口

- `test/runners/{a2c,a2r,a2ts,vm}_tests.at`：从仓根运行的语言级套件入口；golden 仍在 `crates/auto-lang/test/`。
- `test/auto/`：Auto 语言单测；`test/ui/vue-gen-gap/`：Vue 生成夹具，其 pac 相对路径直接指向 packages/blueprints。
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

来源：[PLAN-761](../../plans/archive/761-root-directory-cleanup.md)、[原检查报告](../../reports/root-directory-audit-20261011.md)、[PLAN-777](../../plans/archive/777-v06-modular-boundary-baseline.md)（模块边界审计增量）。

## 包、测试、协议与补丁（PLAN-763）

| 现行路径 | 职责 | 原根目录 |
|---|---|---|
| packages/aavm/ | 实验性自举编译器、VM 与封存 v1；逻辑命名空间 auto.lib 不变 | auto/ |
| packages/blueprints/ | 可复用 bps 包库、目录与参考实现；示例是消费方 | blueprints/ |
| test/parity/ | 三后端行为对拍，独立 Cargo workspace | parity/ |
| docs/schema/ | 活跃的机器声明源 aura.at 与投影协议；编译时仍加载 | schema/ |
| crates/patches/ | iced_widget 0.14.2 的本地补丁完整源码；Cargo override 保持启用 | patches/ |

八个旧 GitHub Actions 定义经来源/运行记录审计删除；七个构建/测试流最近各五次均失败，部署流零次运行。当前仓库不提供 Actions 自动部署或 CI。本地 cargo t/tv/tt/tb/tu/taa 与手动发布脚本继续维护。添加新流程需要明确维护责任、依赖检出与实际验证，不能默认启用发布。历史 workflow 和源文件可从 Git 恢复，审计见 docs/reports/763-ci-audit.json。

共享 .agents、.autoos、.cargo、.config 仍在约定位置；本机点目录不计入 GitHub 根树。stdlib/auto 是生产标准库，packages/aavm 是实验包，两者不混合。冻结历史路径无需批量改写。

## 模块边界审计（PLAN-777）

`scripts/modularity-audit.py` 是模块基线与增量检查入口，配置为 `.autoos/module-boundaries.toml`，冻结基线为 `.autoos/module-boundaries-baseline.json`。从仓根使用：

```text
python scripts/modularity-audit.py --mode observe
python scripts/modularity-audit.py --mode strict
python scripts/modularity-audit.py --self-test
```

默认输出标准输出；`--output` 用于报告另存。报告包含 schema、方法、revision、文件 hash、物理行、声明边、文本候选、责任和未知范围。同输入稳定，不含时间戳或检出绝对路径。Rust规模以Git受管 `.rs` 的物理行统计，含空行/注释/测试/生成文件，不代表生产SLOC或可删除量。

声明边主源为workspace成员manifest的path依赖，区分normal/build/dev、optional与target，workspace继承path按workspace根解析。`cargo metadata --no-deps --locked --format-version 1` 作只读一致性对照；缺锁/锁不满足记录gap，不自动生成或更新，不作为编译成功证据。域表按包名登记当前成员及已知外置实现包；未登记目标保持unclassified，不声称完整生态分类。文本扫描只提供启发式候选，注释/字符串/测试/宏分类及别名、re-export、宏等盲区需读报告；不是Rust语义依赖图。

strict以当前工作树声明边对照冻结规则，未提交的新边同样在判定面内；文本候选漂移只告警。规则必须有域方向约束；没有方向约束的scope-only规则作为惰性规则报告，不应把它作为禁向拦截凭证。存量禁向边只能以完整声明身份（from/to/kind/target/optional/scope/dep_key）匹配冻结边，并具备逐边owner/exit_phase才可豁免；optional激活等身份变化不得沿用原豁免。撤销边再现、配置漂移、锚定manifest/配置变更、过期revision和宽目录豁免不得静默通过。ownership登记覆盖调研焊点、资产与consumer，必需责任组缺失在strict判红；根外consumer只登记，不能把路径核查写成完整生态验收。

`--refresh-baseline` 仅在干净树执行，绑定实际revision与配置，不能当作批准新坏依赖的替代步骤；刷新后仍需strict核对逐边责任/退出阶段。保留历史报告和输入指纹，后续迁移按已批准Plan更新边界和存量退出记录。

报告生成必须只读审计输入。输出前保护配置、基线、实际读取的manifest（含外部path目标）、全部受管文件和读取的锁；路径大小写及硬链接文件身份不能绕过保护。合法独立报告可另存；基线刷新有独立目标守卫，不能覆盖配置/manifest/源码等输入。review/merge证据只绑定实际检查的提交和输入，后续修改须重新检查。

该基线不表示AutoUI/AutoMan已拆仓；它只建立当前责任、存量债务和新增边界检查。后续ParserBridge、native provider、项目模型探针和实际迁移分别验收。

# PLAN-842 R1 修复与最终来源重验

复审发现及真实 CLI 原始反例见 [R1](../review-r1/findings.md)。三仓提交：lang `4e84a8895300731eb94e9cde1737b761745cff39`、UI `eab3d85c3e7aa0af5ae58c08cdeb6a4d691c5691`、Man `8215bf635db76c9f3d1e49ef46a56966a0738d1c`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。三树干净。

- F-842-R1-01：工作区和交付共用 UI 来源选择检查清单 TOML、包名及库入口；语言显式根同时检查工作区清单及生成工作区引用的四个语言包。缺失/损坏/错误身份/缺入口/缺依赖具名拒绝；有效 `AUTO_LANG_ROOT` 与 `AUTO_LANG_CRATE` 正控通过。针对真实安装 CLI 验证失效来源不会修改旧工作区清单。日志见 [受控测试](r1-lang-neighbor-tests.log)、[实际 CLI](r1-final-delivery.log) 和 [单项日志](installed-logs/)。初始 UI 回归先红后绿见 `t10-review-ui-source-{red,green}.log`。
- F-842-R1-02：删除生成前的陈残回收，仅成功落位后回收；新建受损根+唯一完整备份，sidecar 失败时确认原样保留，成功重试后回收且 verify 通过。真实 CLI 同样复现失败/成功；未知 schema 及无事务标记的外来目录控制通过。初始回归先红后绿见 `t10-review-recovery-{red,green}.log`，邻接测试见 `t10-ui-gc-neighbors.log`、`t10-ui-redelivery.log`。
- 最终 Full 安装 6799 文件，manifest SHA-256 `b8071b2d8af3a094e55e12ac92e6e2808460b92c66f99c5cea751228f3dfdabe`；安装 verify、真实交付生成、fresh lock、locked build、生成宿主 1/1、依赖树、原 producer/新两类坏根保旧和恢复通过。安装 Vue 示例实际生成与生产构建通过；安装内 blueprint 唯一标记读取及恢复 verify、资源等长篡改和未知 schema 拒绝与恢复通过。
- 最终 Core 安装 636 文件，manifest SHA-256 `8cf23b8c415208750a3e8af467653d607b5d0a3f95caa10433466c4a78cf12f4`；verify、实际 VM 程序、UI 功能拒绝、独立安装 SDK locked check、外部 consumer fresh lock 和 3/3、依赖闭包隔离均通过。初次手工核对时分别误用 `auto run <file>` 和无子命令 `auto ui`，保留错误日志；修正为 `auto <file>` 和 `auto ui list` 后通过。
- 语言 `cargo check`、Man Full/Core 检查、来源定向 2/2、UI 来源及投影 7/7、交付端到端 1/1、恢复 2/2、外来目录 1/1、再交付失败 1/1通过；一次误用 `delivery_swap` 过滤得到 0 测试，未作为通过计数，已换为实际测试名。历史编译 warnings 仍存在；仅运行定向测试，merge-owned 整档未运行。

先前 T-05 安装器/模型迁移证据和未触及的语料、协议、UI 方言/host 行为可按源码、配置和输入复用；本轮变更集中于来源验证和备份回收及对应发行测试。最终复审必须重新核对影响与所有 AC/SD；本实施会话的自查不能称独立复审。

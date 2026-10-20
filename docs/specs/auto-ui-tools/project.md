# auto-ui-tools — UI 工程工具包(M815-A 阶段子集)

> 状态:PLAN-815 M815-A 已发布子集(2026-10-20,R4 阶段审查通过,delivery `e609caf1a`)。
> 本文件只陈述已发布窄工具与来源登记;完整生成/build/run、生产资产迁移与旧 Man 兼容退出属于 M815-B,不在本子集中。

## 当前行为(SD-01-A,AC-07)

`crates/auto-ui-tools` 提供 lib 与 `auto-ui-tool` 可执行入口;M815-A 只公开
`describe` 与 `validate_project` 元数据操作。正常/构建/开发依赖闭包无
auto-man、旧 auto-lang、VM/生成器实现(闭包由
`test/fixtures/modularity/ui-tool-consumer/verify-deps.sh` 钉住)。客户端经轻协议
(auto-tool-protocol)调用独立进程;CLI 不因本阶段自动切换路径。

`describe` 返回 tool 名称、实现版本、协议版本、能力及生命周期。两个工作操作均
non-interactive、non-cancellable;`cancel` 为协议控制消息,对已服务的 A 操作明确
`NotCancellable`,对未知 target 明确 `InvalidField`。

`validate_project` 校验请求元数据、路径和请求约束,报告明确 checked /
not_checked(metadata/paths/request-constraints 与
language-parse/dynamic-evaluation/generation/build)。它不解析/求值语言、不生成、
不构建;静态 unsupported 不冒充已求值。路径根据请求内绝对根解析;不能由请求确定
的相对、Windows 盘符相对(`C:dir`)或无盘符根(`\dir`)路径明确报告不可校验
(`relative-path-without-root` / `cwd-qualified-path`),不探测进程 cwd;绝对路径与
普通可锚定相对路径按角色探测(根/front/dep=目录、input_sources=文件、output_root
存在时须为目录;project_root 缺失/相对/是文件为显式 finding)。存在的输出目录无需
提前创建;未知 UI 生成能力在副作用前拒绝(unknown-operation,零副作用)。

## 资产登记(T-02 交付;物理迁移属 B)

资产当前仅登记 gallery / wm / shadcn-ui 及 drawlist 外部输入的
owner/source/license/目标(`resources.rs` 清单 + 锚定测试);生产 embed 仍在
auto-man 原处,本阶段未复制/搬走资源。完整生成/build/run 及生产资源迁移属于 B
(单次迁移与 embed 切换在 B 的生成主体任务中完成)。

## 测试面

- 单元/进程:`cargo test -p auto-ui-tools`(17 真实进程 consumer 用例含 cwd
  对照/盘符相对/受控坏 payload + 4 资产清单锚定)。
- 独立消费门:`bash test/fixtures/modularity/ui-tool-consumer/verify-deps.sh`
  与 `AUTO_UI_TOOL_BIN=<binary> cargo test --manifest-path
  test/fixtures/modularity/ui-tool-consumer/Cargo.toml`(真实子进程)。

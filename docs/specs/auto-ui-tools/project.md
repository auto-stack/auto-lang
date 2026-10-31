# auto-ui-tools — UI 工程工具包(M815-A 阶段子集)

> 状态:PLAN-815 已完成 B 面全量收口(2026-10-22,T-09 R2 终审 pass,生产冻结位 `0f2858e35`);
> M815-A 已发布子集(2026-10-20,R4 阶段审查通过,delivery `e609caf1a`)事实保留于下文。
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

## M815-B:UI 工程工具唯一 owner 全量规则(PLAN-815 SD-01)

`auto-ui-tools` 是 UI 工程工具唯一 owner——生成/构建/运行/资产全闭包脱 Man
(normal/build/dev 无 auto-man/legacy auto-lang 大包反向依赖);旧 Man 入口为
薄转发门面(语义冻结,六兼容/媒体/请求缝定向验证)。生产入口全部消费已求值
I-PROJECT 请求(`*_with_request` 家族),不得回读主 pac.at 形成第二真源
(请求在场即权威:值或缺席都由请求定,legacy 臂才读盘——external_backend/
exe_name/window/title/api/dep_roots 等 PacSource 请求臂消费链随测试锚定)。
外部命令经统一 owned 子进程设施;构建失败/缺工具/产物缺失为具名失败;
多目标请求完整生成并诚实聚合(输出根=真实布局,不按 output_root 重定向)。
字段×profile×操作消费矩阵与适用边界:
`docs/plans/attachments/815/t07-acceptance/batch3/field-profile-matrix.md`。
shell-pack 公开再生器(`generate_shell_pack_lib(_sources)`)同受 canonical
import 规则约束(生成清单含 auto-ui-runtime+auto-lang-core 双直依赖;
freshness 字节对拍门 `test_shell_pack_lib_freshness`)——F-815-T09-01 修复
后为在案规则。（PLAN-815 M815-B，T-09 R2 pass @ 生产冻结位 0f2858e35；终稿正文与证据：attachments/815/handoff.md §规范增量）

## M816-B:op 级执行选项与真实分发(PLAN-816 增量)

- `generate_report_with` 家族真实分发请求选项:792 delivery 独立分支产
  relocatable manifest;rust/jet/ark/vscode profile 传 `output`+`project`;
  vue+显式 output=具名拒绝(旧静默忽略已诚实化)。报告 `output_root` 为既有
  聚合语义(canonical 布局根),实际产物落指定目录——重定时锚点=重定向目录
  本身(负控:四 profile/delivery/vue 拒绝永久 oracle 在案)。
- StrictGuard:per-op 置位/恢复,strict 解析错误非零拒绝;工具在
  `Capability.options` 按操作广播所支持选项(见 tool-protocol M816-B 节)。
（PLAN-816 M816-B,T-05 R2;终稿证据:attachments/816/handoff.md）

## PLAN-831 仓属与来源（SD-05，as-built）

本 crate 物理属 auto-ui 仓（迁移基线=语言仓 e199caf8a 内容）。真实 auto-ui-tool 二进制与嵌入资产随 owner 迁移；发行 bundle 内为双直依赖生成工程提供 tool 实际二进制与运行资产。构建矩阵判据（r3）：全 profile 真实尝试逐项记录；必需面=vue/rust/vscode completed；jet/ark 缺环境=获准延期（producer 具名缺席形态：jet=9009/gradle wrapper 生成失败原文，ark=Hvigor wrapper not found 句头原文），在场执行后的真实失败传播红——分类读失败类别语义位置（缺席句头 startswith 优先；执行退出码仅从原因区提取），项目路径字节不可翻转任一方向。

## PLAN-890 Web消费

UI tool/VM worker/Rust生成与back-proxy直接消费固定auto-web，内部显式安装provider并传递typed launch。
HTTP生成/运行策略归Web，UI仍拥有界面与非HTTP适配；命令、工具身份与ready/关闭义务不因迁仓降级。
[UI消费指南](../../../../auto-ui/docs/web-integration.md)与[Web指针](../auto-web/project.md)。

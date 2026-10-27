## formal delta

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/project.md, docs/specs/language-sdk/project.md | 过渡根再导出/包装与SDK旧hook→已迁canonical入口和具名保留的稳定API | 退出过渡耦合 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-parser-api/project.md, docs/specs/auto-ui-dialect/project.md | Parser/UI文法旧桥→841终态入口，已无消费者过渡适配实际删除 | 不再背旧模型 | AC-01..05 |
| SD-03 | modify | docs/specs/auto-ui-compiler/project.md, docs/specs/auto-ui-runtime/project.md | UI compiler/runtime旧入口/feature/隐式装配→明确正式组合与源消费 | 保持默认与生命周期 | AC-01..05 |
| SD-04 | modify | docs/specs/auto-man/project.md, docs/specs/auto-cli/project.md, docs/specs/repository-boundaries/generated-ui-distribution.md | Man/CLI/发行旧路径/安装/producer→固定版本canonical消费与迁移或拒绝 | 保真实命令/升级 | AC-01..05 |
| SD-05 | add | docs/specs/repository-boundaries/compatibility-retirement.md | 无统一逐入口退役规格→可退出清单、正式保留、版本迁移与历史恢复 | R11可审退役 | AC-01..05 |
| SD-06 | modify | docs/specs/repository-boundaries/generated-ui-distribution.md, docs/specs/repository-boundaries/man-three-repository-delivery.md | Full 安装 SDK 合法投影可被真实交付生成接纳；坏结构仍拒绝、旧产物按 792 保护，独立包可锁定构建 | r2 用户授权接手安装投影缺口 | AC-08 |

## T-09 delta

| ID | 存放目标 | 应写入的现状、退出和验证边界 |
|---|---|---|
| SD-01 | `docs/specs/auto-lang/project.md`、`docs/specs/language-sdk/project.md` | 0.2.0 退出 `auto_lang::{ast,parser,session,error}` 模块和 `use-bigvm`；分别指向 auto-ast、SDK parser、parser-api、diagnostics。根 `Parser`、正式 UI/构建/resolve 薄接口和非 UI 外部方言保留。SDK 纯 Core 无 UI 时 hook 缺席/no-op；已链接 UI 语言包的直接 SDK 默认需现有 inventory，显式 provider 优先，双实例/线程/退出边界及 T-06 消融证据列明。 |
| SD-02 | `docs/specs/auto-parser-api/project.md`、`docs/specs/auto-ui-dialect/project.md` | UI dialect 0.2.0 移除两处 `UiParseContext` 名称，使用 `DomainParseContext` 显式组合；`ExternalDialect`、Override/Append 非 UI 合同不删。UI grammar 的旧 `UiGrammarSource`/`assemble_ui_grammar` 仍退出；codec schema 1 合法、旧/未来 schema、错误 domain/tag 与尾随非空数据拒绝。 |
| SD-03 | `docs/specs/auto-ui-compiler/project.md`、`docs/specs/auto-ui-runtime/project.md` | runtime 0.2.0 退出 `AuraViewBuilder::with_registry_and_imports`，改为 `with_registry`；`DynamicComponent` 同名不同方法仍处理导入。生成器及 29 示例直连 runtime/compiler/SDK；宿主入口安装 SDK hook 与 runtime provider；保默认、显式、缺席、实例/线程/退出。旧 cache identity 失效并重建；MPV-native 显式含 libloading，真实 DLL/视频播放未在本轮实测。 |
| SD-04 | `docs/specs/auto-man/project.md`、`docs/specs/auto-cli/project.md`、`docs/specs/repository-boundaries/generated-ui-distribution.md` | Man/生成后端使用 canonical owner；指定语言/UI 根权威、错误不回退；非受管语言/UI 源验证本地 Cargo 依赖图、每个包自身版本、包身份及约束，允许合法 dev 环和重命名；受管安装以文件系统身份识别同一 SDK 的路径别名，并读取已验证收据；开发 UI owner 与受管 Full 安装按 owner/path/hash 校验并记录各来源；生成 Rust 应用显式装配宿主、无 auto-lang 直接依赖。`generated-ui-delivery/1` 仍支持，未知版本/坏 producer 拒绝、旧产物及未知版本备份保护；fresh/locked、资源失败和恢复是实测门。 |
| SD-05 | 新增 `docs/specs/repository-boundaries/compatibility-retirement.md` | 用 [冻结矩阵](retirement-matrix.md) 精确记录退出、正式保留、0.2.0 边界、旧固定源与迁移/回退、必要宿主登记；不把历史脚本/负向 probe 当作生产消费，也不声称调查了未知仓外消费者。 |
| SD-06 | `docs/specs/repository-boundaries/generated-ui-distribution.md`、`docs/specs/repository-boundaries/man-three-repository-delivery.md` | `man-install/1` Full SDK 的合法中间投影与源码/已投影态均可交付；按 TOML 表作用域识别、异常混态拒绝。Core 不含 UI；安装器在准备替换之前验证已有 schema/profile/文件，未知或损坏拒绝并保留。0.1 真实旧安装升级、backup 恢复及 prepare/swap 反例已有证据；清单 hash 是一致性而非签名。 |

# project-model

> **Status**: active（pilot）
> 路径：`crates/auto-project-model`  | 技术栈：Rust（依赖仅 auto-atom / auto-val）
>
> PLAN-780 交付：轻静态工程声明模型 pilot——脱离 compiler/VM/UI 表达并读取
> 静态工程声明，与执行型配置（`AutoConfig::from_code` → `Pac::new`）对拍。
> 本 spec 为 R5 冻结增量（reviewed_commit `68fcc3bf7`）的 canonical 沉淀。

## 模块与依赖

`crates/auto-project-model` 为静态工程声明模型 pilot，仅直接依赖
auto-atom/auto-val；normal 闭包不包含 compiler、VM、auto-man、UI 或生成器
（外部 consumer 门：`test/fixtures/modularity/project-model/consumer/` 独立
manifest 构建即 AC-01 证据）。文档层自持扫描器，结构化值层复用 AtomParser；
不实现完整语言求值。公开入口为 `parse_document(text, SourceInfo)` 与只读
`parse_file(path)`。读取不执行网络、项目物化或构建；Builder/IndexStore/
AutoVM 不进入 DTO。

## 模型与边界

`ProjectDescriptor` 提供 identity、render/legacy backend、scene/members/
defines、依赖声明、展示/运行字段、toggle、external_backend 及
source/provenance；冻结字段的归一化对齐当前 `Pac` 语义（对拍探针：
`auto-man/src/project_model_adapter.rs`，corpus 门 170 受管文件计数）。
窗口、port、开关、render 形态等按已实现 DTO 规则解释；展示 locale/env
覆盖及构建效果仍由消费层负责。

普通未知属性/节点保留在 `raw_props`/`raw_nodes`；dep 体保留 props、重复
信息和 children，未知节点参数保留 raw_args。重复 path 按末声明覆盖，仍
记录重复及来源。该保留能力不等于完整源文本往返协议，规范不得额外承诺。

静态支持面为已冻结字面量属性、数组/对象、dep 裸名/引号/参数形态、顶层
对象文档（PLAN-729 minimal project format）及已验证注释/分隔形态。变量/
调用/插值/赋值/f-string、显式不支持的 wrapper（`root { … }` 历史形态）及
类型形态通过 `unsupported` 分类或 stage 错误暴露，非静态不计入等价；不
静默执行回退。轻模型的 static 标志不是对全部语言语法的兼容承诺，pilot 以
冻结域和 oracle 对拍为验收依据。

## 注释、错误与已知限制

块注释内部换行只更新源码位置，不形成语法分隔；注释外真实换行与逗号按
已验证容器规则处理。同一行无分隔的相邻值不能因块注释内部 LF 而获 static
标志。数组与对象的旧 VM 逗号规则不完全相同（数组：值后逗号/换行/注释
任意混排；对象：逗号仅同行紧随值，换行后逗号 Err），不能由一类控制推导
另一类。

顶层文档支持 `#` 行注释；当前容器值位 `#` 形态由旧 VM 拒绝，模型亦拒绝/
非静态，不能宣称容器中任意 `#` 注释均合法。旧 VM 允许而 AtomParser 拒绝
的逗号混排（例如 `["a", /*c*/, "b"]`）仍在冻结子集外：模型返回值层错误，
不宣称等价。这是保留限制，不要求扩张完整 VM 语法。

错误携带 source path、stage、位置；文件读取/编码失败是 Io，不以空配置
代替（required corpus 门必红）。扫描失败出口保持消费/返回/错误进度保证；
终止性控制有外部截止。EOF 块注释行为保持当前 legacy 已验证规则，不额外
宣称更严格语法。

## 后续接线归属

静态模型、compiler 执行 adapter 及构建/物化职责分层。auto-man 内部标量
文本读取可后续逐入口迁移；auto-lang registry/loader 接线及依赖边需另立
计划。npm_deps 和构建 targets 暂不扩为完整语义 DTO，现有生产读取保留。
模板渲染和动态配置留在显式上层 adapter；不切换默认生产入口、不建立外仓、
不宣称 auto-man 已脱离语言。字段 owner 对照与接线顺序见
`test/fixtures/modularity/project-model/T04-OWNER-MAP.md`（T-04 交付）。

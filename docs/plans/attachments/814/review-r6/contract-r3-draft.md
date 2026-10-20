# PLAN-814 拟议 r3 合同增量（待用户批准，未生效）

本文件是复审准备的可审阅文本。当前主计划仍为 r2；本草案不修改其语义、canonical Specs 或 ledger，也不是阶段 pass。仅修订以下两项，其余原 AC、旧 API/类型/单份状态与 A/B 交付要求保持。

## 1. 具名的上层兼容注册例外

拟替换 r2 AC-02 / V-02 / §5.2 中有关“无隐式链接装配”的绝对约束：

> 纯 SDK 的 normal/build/dev 闭包不含 UI 文法实现、UI compiler/runtime/renderer 或 Man；SDK 不提交 UI 注册项，不硬编码 UI factory。SDK 未装配 UI 文法时的缺席/拒绝语义保持。
>
> 允许旧 auto-lang 上层兼容包及 auto-ui-compiler 经源码可见的 inventory::submit! 登记轻量 UI 文法函数表或宿主安装器的函数指针，覆盖不经显式装配入口的旧直达调用。该登记通过 inventory 启动构造器实现（当前锁定 0.3.25；Windows .CRT$XCU），会修改 inventory 自身注册表；不在启动登记阶段调用 UI 文法/宿主工厂、执行 I/O 或初始化 UI/VM 业务状态。
>
> 具名入口继续显式装配；未显式设置时才由 SDK 收集已登记来源。多个当前来源均构造自同一 auto-ui-dialect 实现，不依赖它们的遍历顺序选择不同语义。不新增不同版本同时装配的支持承诺。
>
> 上层消费者允许按其功能依赖并链接 UI 实现；不能把这一点写成“上层无 UI 实现拉入”。纯 SDK 的无 UI 源码消费者仍须成立。旧 Parser/VM API 与类型身份、单份状态、宿主默认/显式覆盖及缺席行为保持。

此例外明确承认链接时注册，不再以“显式 submit 语句”等同“调用者显式装配”，也不声称没有构造器或没有注册表状态变化。仅删除 inventory 不能自动授权砍旧 API、改成硬错误或更改纯 SDK 缺席语义。

## 2. 轻端口的归属修订

允许将 `UiParseContext` 与 `UiGrammar` 的类型/函数表定义放入轻 `auto-parser-api`；UI 文法实现继续由 auto-ui-dialect 单源拥有，SDK 不反向依赖文法实现。auto-ui-dialect 保留原公开路径再导出及类型身份，通用 `ParseContext` 的既有实现方不被强制实现 UI 端口。

这是单源端口与适配器方案之间的设计取舍，不是孤儿规则证明的唯一实现；批准后更新以下现行约束：

- `docs/specs/auto-parser-api/project.md`：原“UI 伴随端口不入本 Core 包”改为“允许 UI opt-in 轻端口定义，不带 UI 实现依赖”。
- `docs/specs/auto-ui-dialect/design/full-grammar.md`：trait 定义由方言 crate 改为 parser-api，方言 crate 保留再导出与实现 owner。
- 主计划的 SD-01/02/04/05 与上述两个 Spec 目标同步记录这一窄例外及其验收，不提前发布 B 完整 runtime 事实。

批准之后，先由 new 将主计划升为 r3 并同步相关 AC/V/SD 文本，再绑定冻结实现重新验收受影响的显式/直达/纯 SDK 正反控；阶段 pass 后仍由 merge 执行冻结门禁与实际 A 发布。未批准之前，r2 的原限制继续有效。

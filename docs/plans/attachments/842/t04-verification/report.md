# PLAN-842 T-04 验证

冻结提交：lang `44d1e7da7d7f0f5d4221403fd83fa48268bd787d`、UI `5c0e0de65f02045f8623f29c3e7bbb18832668b6`、Man `36a73758176815219a3b28bbc4457f21677f23a6`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。版本号升级归 T-05，最终安装/消费者重新验证归 T-07；此项不是最终 review 或 merge。

- 实际移除公开 `auto_lang::{ast,parser,session,error}` 转发，内部用私有同源引用，保 `Parser` 根 API；移除 `UiParseContext` 的根与 grammar 两处别名，及无消费者的 `AuraViewBuilder::with_registry_and_imports`。完整符号退出清单即上述七个可观察入口。真实的 `DynamicComponent::with_registry_and_imports` 承担导入函数供给，保持不变。
- 可复跑的独立外部工程检查：七个旧入口分别被编译器具名拒绝；规范 `auto_ast`、`auto_diagnostics`、`auto_lang_core`、`auto_parser_api` 与 UI builder 一起实际编译通过。探针见语言工作树 `test/fixtures/modularity/plan842-retirement/`；`external-consumer/` 保存各例原始输出。
- 真实轻文法 6/6、完整 UI 文法 21 passed/1 ignored（沿用原 deadline 隔离策略）、已注册和未注册组件图标渲染同一测试 1/1；AST 跨源身份 2/2；外部 UI dialect consumer 48/48、轻量非 UI consumer 3/3。
- auto-lang 默认及 no-default、LSP、实际 Man CLI 与 UI 工具 check 通过；已有 warnings 未宣称清零。首次轻量方言测试因缺独立 fixture lock 而 `--locked` 拒绝，已离线生成并重跑通过；该新增未跟踪锁与之前 6 份锁均保留。中途误用零匹配过滤得到 0 测试，不作为通过证据。

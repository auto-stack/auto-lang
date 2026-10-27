# auto-ui-dialect 完整 UI 文法设计（full-grammar）

> **Status**: active（PLAN-841 终态）
> 路径：auto-ui 仓 `crates/auto-ui-dialect/src/grammar/`

## 1. 范围与调用图

- `widget.rs`：widget 壳、props、use/setup/expose/watch/computed/on/bind、
  内联 handler mint 与 scope 生命周期。
- `view.rs`：view 块/fragment/component、节点族、style 绑定、条件 DSL、
  事件修饰与参数。
- `store.rs`：store 的 msg/model/computed/watch/on/timer。
- `actions.rs`：actions/timer/routes/menubar/toolbar 与 style recipe。
- `helpers.rs`：近失、事件名、await、主属性与 fstr 辅助。
- `declarations.rs`：msg/model 单一字段算法。

生产调用为 `Parser::parse_domain(&UiDialect) -> UiCode`，随后由 dialect
`validate_ui` 与 compiler `lower_ui` 校验/lowering；公开生成、runtime、
SDK、Vue/Rust/Jet/Ark 工程与 incremental 路径均在缓存、消费或写盘前
经过该路径。codec decode 同样执行领域校验，以扩展递归工作栈支持合法
深层视图往返，并完整消费单份 artifact，拒绝非空白后缀或第二份 JSON。
`dialect/ui.rs` 全局 adapter、Parser UI 返回门面、
`with_session` 默认装配和 `UiGrammar` 函数表均已退出。Godot SceneDecl
文法与 AST 归 Core `scene_grammar.rs`，不在 UI 模块复制。

## 2. DomainParseContext 端口

`UiParseContext` 两处过渡别名已在 0.2 退役；直接使用 auto-parser-api 的中立 `DomainParseContext`，trait 定义不含 UI 类型。端口提供：

- 游标/软关键字/owned token 回推、两类 advance-peek 与 probe；
- raw cursor 保存/恢复/释放和 diagnostics mark/truncate；
- on/skip_check 旗标、scope/变量与 handler 参数绑定；
- Core body/stmt/closure/object/fstr/dot/on-events 委托；
- docs 与 suspicious-keyword warning。

raw CSS 由 Core lexer 捕获真实字节；toy fixture 的 token 重建只用于轻
端口覆盖，不替代真 Parser 保真测试。跨文件、codec 与缓存不使用该端口
传输序列化字节，进程内直接移动 owned `UiCode`。

## 3. 支持与失败边界

既有 UI 宽容语法与错误位置保持；fn/on 体内 UI 关键字让行 Core。on-handler
在成功和 Err 路径都恢复 on 旗标并退出参数 scope，源尾裸参数引用确定性
未定义；UI 场景 fn 体内 `store.x = 1` 是合法 Core 接收者表达式。

未闭合 fragment/msg 的无界 EOF 怪癖只在隔离进程和外部截止下探测，不
冒称已修复。合法输入沿用 Parser body_depth 与既有截止。

## 4. 验证矩阵

- auto-ui-dialect 91 单测 + 2 doctest；SDK `plan810` 21 passed/1 ignored；
  scene 门测试归 auto-lang-core。
- `ui-grammar-consumer` 轻端口 6 项；`ui-dialect-consumer` 真 Parser
  组合、快照、裸字段与非 UI 旧方言控制。
- `ui-ast-model-consumer` 重放同一冻结 golden 的 9 个 UI case；
  Core/Scene 段由 auto-ast owner 重放。
- compiler ui_ast 4 项贯通 lower→VM 与四生成器；merge 阶段再执行统一
  裸 cargo t 与 tv/tt/tu 触面档。

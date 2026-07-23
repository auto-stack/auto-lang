# comptime 相关 plan

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 095 | Compile-Time Execution Engine (CTEE) | ✅ Complete | old/ | `#if`/`#for`/`#is`/`#{}` 全链路落地：token→AST→parser→CTEE 变换→七处管线集成；确定性沙箱与资源限额仅完成错误类型，执行侧未落地 |
| 137 | Comptime 示例代码库 | ✅ Complete（2026-03-20） | old/ | `test/comptime/` 三级示例语料（`#[expect_value]` 标记规范）；`#if`/`#for`/`#is` 当时仅解析可用，示例用运行时等价代码顶替，CommentTest 运行器留作后续 |
| 094 | Hybrid FFI Bridge | ✅ Phase 1-5 Complete | old/ | plan-095 的前置依赖：为编译期 native 调用提供 `#[rust_fn]` 桥（43 个 shim） |
| 310 | Auto 所有权逃逸分析与智能指针回退 | ✅ Completed（2026-06-16） | archive/ | 把逃逸分析 pass 锚定在 a2r 管线 `CTEE::transform` 之后，固化了 CTEE 在 trans/rust 中的位置约束 |
| 243 | LSP & VSCode Modernization | ✅ Phase 1（Phase 2-6 待实现） | plans/ | 要求 LSP 能解析 `#{}`/`#if`/`#for`/`#is` 不产生误报——comptime 语法面的工具链消费者 |

注：095/137 的状态以 plan 文件自身头部为准；094/310/243 仅为交叉引用方，其状态同样引自各自 plan 文件。

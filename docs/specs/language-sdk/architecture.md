# language-sdk 内部架构（模块职责 DAG 与结构规则）

> **Status**: active（PLAN-843 结构半部交付 @ codex/plan-843-dev e3c0a53d0；PLAN-884 收口补齐
> 运行桥接/stdlib 装配/Rust 转译职责，见 [structure-closure.md](structure-closure.md)）
> 路径：`crates/auto-lang-core/src/**` | 上游合同：[project.md](project.md)

## 结构总则（PLAN-843 裁定）

- **入口仅承载类型/构造/明确派发与组合**；完整算法按语义职责归主题模块。禁止 include! 机械
  分片、空 re-export 模块、文件切两半式平移、新增 >4000 生产行的 common 大杂包。
- **共享状态唯一 owner**（下表）；不复制第二份 Store/registry/cache。内部跨族引用只扩最小
  `pub(super)`（编译器驱动收敛）；入口 `pub(crate)` 计数不得上升。
- 超大 match 单体（如表达式发射、opcode 派发）用**尾递归派发**分解：连续臂切片 + 兜底臂
  `_ => return self.next(...)` 链转发；臂体逐字保留、首匹配序不变、签名不变；原 `_` 兜底臂
  恒留末段。
- 巨型 if 链单体（如 `trans::rust::call_inner` 特型派发）**先逐块审计 early-return 再切分**
  （843 期 KNOWN-DEBT 候选，844 承接）。PLAN-884 已完成该审计与切分：22 个顶层形状族入
  `trans/rust/call_emit/*`，派发序=原语句序、族 helper 返回显式 `CallShape::{Miss,Handled}`
  （禁用 `Ok(())` 混表"未命中/已发射"），末族无条件终止；首匹配序不可重排。旧形态"保留巨型
  链"不再是合法完结态——新增长链须给审计图与不可达块登记，见
  [structure-closure.md](structure-closure.md) §4。
- 路径恒等：拆分家族经 `mod X;` 文件化声明 + `pub use X::*;` 门面保持 `crate::vm::native::X`
  等既有路径；crate 根（lib.rs）公开 API 经 `pub use entry::*;`/`pub use flags::*;` 保持。
- **源码指纹面随 owner 迁移**：stdlib_assembly manifest/loader 的 include_str 输入清单须覆盖
  拆分后的全部族文件；源码 pin 型测试（include_str/read_to_string 断言源内容）随 owner 迁移
  扫描面（plan705/716/738 先例）。

## 模块 DAG（依赖只准自上而下）

```
lexer/token → parser(+方言表/parser_bridge) → ast
ast+token → types(TypeStore) → infer/* → typeck/trait_checker
parser+types+infer → compile(CompileSession) → vm/codegen → vm/engine
vm/engine → vm/native + vm/ffi（编码/宿主服务桥，host_capabilities provider 端口）
ast+types → trans/*（rust 家族等，Sink 产物） → 各目标
lib.rs = crate 根组合（mod 声明+再导出）+ entry.rs(run* 族) + entry/run_bridge(同构桥接单源) + flags.rs(全局旗标)
```

## 状态 owner 与主题模块表

| 域 | 入口（组合/构造/派发） | 主题模块（算法所在） | 状态 owner |
|---|---|---|---|
| 解析 | `parser.rs` | `parser/{expressions,statements,decls,type_syntax,scenario,tests}` | `Parser`（17 pub 字段合同；方言派发序 pre→builtin→post） |
| 类型 | `types.rs` | `infer/*`（既有多模块） | `TypeStore` 单源 |
| 发射 | `vm/codegen.rs` | `vm/codegen/{stmt_emit,expr_emit,ty_conv,fn_emit,vars,expr_containers..expr_fallback}` | `Codegen`（flash/符号表） |
| 执行 | `vm/engine.rs` | `vm/engine/{heap,values,futures_timers,tasks,ops_construct..ops_flow}` | `AutoVM`（tasks/RAM/帧/预算；Rc/线程亲和/锁外回调不变量） |
| native | `vm/native.rs` | `vm/native/natives_{int,strings,collections,chrono,web,formats,sys,rand,fs,host,io,test}` | `NativeInterface` 单一注册表（`pub use` 门面保 ID 表宏路径） |
| FFI | `vm/ffi/stdlib.rs`（装配壳 2278 行） | `vm/ffi/stdlib/{file_io,http_client,crash_trace,strings_json,math,image,sqlite,sys_env}` + `{request_root,storage,exit_audit,http_async,rust_stdlib_dispatch}`（共 13 域；注册=19 个域 helper 有序调用） | guard 生命周期（RequestFileRootGuard 等）；各域全局表单份 |
| HTTP 服务 | `vm/ffi/http_server.rs` | `http_server/{server_route,server_model,server_dispatch,server_scope,server_argbind}` | `RequestScope`/DispatchCtx |
| 转译 | `trans/rust.rs`（编排 4293 行） | `trans/rust/{expr_emit,stmt_emit,decl_emit,ty,call_emit,project_fixups,post_fixes,ast_scans,merged_fixes}` | 各 Trans 无跨目标共享大杂包 |
| 运行桥接 | `entry.rs`（公开 run 入口） | `entry/run_bridge.rs`（G1/G2/G3 单源 + G5 staged 独立） | 专用执行线程/LocalSet、`LAST_RUN_TRAP_CODE` |
| crate 根 | `lib.rs` | `entry.rs` + `flags.rs` | 进程级全局（GLOBAL_RT/ERROR_LIMIT/VM_DEBUG/SCRIPT_ARGS/trace 族） |

## 规模与稳定接口规则

- 入口生产行上限基准：parser 2175 / codegen 1274 / engine 2131 / native 2367（以上达标）；
  stdlib 5679 / http_server 4156 / rust 5960（PLAN-843 残留口径）。PLAN-884 收口后实测：
  stdlib 壳 2278（域族外移，达标）、rust 编排 4293（主题算法外移，达标）、http_server 4156
  保持——其壳为经审计的合法核心（accept loop/控制面/scope 生命周期/SSE 步进/WS 帧/by-name
  绑定），无仓内调用方引用旧内联路由/派发，**不按总行数判未收口、不迁文件**。
- 公开 API 面（pub fn/类型/字段）与 `Parser` pub 字段、`TypeStore`/`CompileSession`/`AutoVM`/
  `NativeInterface` 类型身份为稳定接口；上层 `auto_lang::*` 兼容门面与 auto-lsp 等真实消费者
  编译面必须保持。
- 拆分守恒审计口径（PLAN-843 复审先例）：基线原文 vs 拆分全集的规范化行多重集对比，仅允许
  记录在案的机械变换残差（包装行/限定符/pub(super)）。
